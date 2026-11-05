//! The program of an installed module: its system side. It runs apart from the app, which only
//! relays JSON lines on its standard input and output:
//!
//! - to it: `{"type":"init","dataDir","locale"}` first, then `{"type":"call","id","method","params"}`;
//! - from it: `{"type":"result","id","value"}`, `{"type":"error","id","message"}` and
//!   `{"type":"event","name","payload"}`, emitted to the interface as `<module id>://<name>`.
//!
//! It starts on its first call and stops after a minute without one, or runs as long as its
//! module is active when it declares `background`. It must exit when its standard input closes.

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use serde::Deserialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager};

use super::{Installed, ModuleHost};
use crate::app::{self, AppError, Result};

const CALL_TIMEOUT: Duration = Duration::from_secs(30);
const IDLE: Duration = Duration::from_secs(60);
const MAX_LINE: u64 = 4 * 1024 * 1024;
/// A background program that keeps failing is left stopped until the module is turned back on.
const MAX_RESTARTS: u32 = 3;

type Reply = std::result::Result<Value, String>;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Message {
    Result { id: u64, value: Value },
    Error { id: u64, message: String },
    Event { name: String, payload: Value },
}

struct Program {
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    pending: Mutex<HashMap<u64, mpsc::Sender<Reply>>>,
    next_call: AtomicU64,
    last_used: Mutex<Instant>,
}

impl Program {
    fn send(&self, message: &Value) -> Result<()> {
        let mut stdin = self.stdin.lock().unwrap_or_else(PoisonError::into_inner);
        let mut line = serde_json::to_vec(message)?;
        line.push(b'\n');
        stdin.write_all(&line)?;
        stdin.flush()?;
        Ok(())
    }

    fn kill(&self) {
        let mut child = self.child.lock().unwrap_or_else(PoisonError::into_inner);
        let _ = child.kill();
        let _ = child.wait();
    }
}

#[derive(Default)]
pub struct Programs {
    running: Mutex<HashMap<String, Arc<Program>>>,
    restarts: Mutex<HashMap<String, u32>>,
    /// Two calls arriving together start one program, not two.
    starting: Mutex<()>,
}

impl Programs {
    fn get(&self, id: &str) -> Option<Arc<Program>> {
        self.running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            .cloned()
    }

    /// Stops a module's program, before an update, an uninstall or when it is turned off.
    pub fn stop(&self, id: &str) {
        let program = self
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(id);
        if let Some(program) = program {
            program.kill();
        }
    }
}

fn is_event_name(name: &str) -> bool {
    (1..=32).contains(&name.len())
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn start(app: &AppHandle, module: &Installed) -> Result<Arc<Program>> {
    let host = app.state::<ModuleHost>();
    let _starting = host
        .programs
        .starting
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some(program) = host.programs.get(&module.id) {
        return Ok(program);
    }
    let native = module
        .info
        .native
        .as_deref()
        .ok_or_else(|| AppError::Invalid(format!("{} has no program", module.id)))?;
    let data_dir = host.data.join(&module.id);
    std::fs::create_dir_all(&data_dir)?;

    let mut command = Command::new(module.dir.join(native));
    command
        .current_dir(&data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // A program without a window of its own, not even a console.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn()?;
    let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
        let _ = child.kill();
        return Err(AppError::Failed("the module's program has no pipes".into()));
    };
    let program = Arc::new(Program {
        child: Mutex::new(child),
        stdin: Mutex::new(stdin),
        pending: Mutex::new(HashMap::new()),
        next_call: AtomicU64::new(1),
        last_used: Mutex::new(Instant::now()),
    });
    let settings = app::settings(app);
    let locale = settings
        .language
        .or_else(app::i18n::system_locale)
        .unwrap_or_else(|| "en".into());
    program.send(&json!({ "type": "init", "dataDir": data_dir, "locale": locale }))?;

    host.programs
        .running
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(module.id.clone(), Arc::clone(&program));
    let (app, id, reader) = (app.clone(), module.id.clone(), Arc::clone(&program));
    thread::spawn(move || {
        read(&app, &id, &reader, stdout);
        ended(&app, &id, &reader);
    });
    tracing::info!(module = %module.id, "module program started");
    Ok(program)
}

fn read(app: &AppHandle, id: &str, program: &Program, stdout: impl Read) {
    let mut lines = BufReader::new(stdout);
    loop {
        let mut line = String::new();
        match Read::by_ref(&mut lines).take(MAX_LINE).read_line(&mut line) {
            Ok(0) | Err(_) => return,
            Ok(_) if !line.ends_with('\n') => {
                tracing::warn!(module = %id, "module program sent an oversized line");
                return;
            }
            Ok(_) => {}
        }
        match serde_json::from_str::<Message>(&line) {
            Ok(Message::Result { id: call, value }) => answer(program, call, Ok(value)),
            Ok(Message::Error { id: call, message }) => answer(program, call, Err(message)),
            Ok(Message::Event { name, payload }) if is_event_name(&name) => {
                app.emit(&format!("{id}://{name}"), payload).ok();
            }
            _ => tracing::warn!(module = %id, "module program sent an unknown message"),
        }
    }
}

fn answer(program: &Program, call: u64, reply: Reply) {
    let waiting = program
        .pending
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .remove(&call);
    if let Some(waiting) = waiting {
        let _ = waiting.send(reply);
    }
}

/// The program stopped: calls still waiting fail, and a background program comes back.
fn ended(app: &AppHandle, id: &str, program: &Arc<Program>) {
    program.kill();
    program
        .pending
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clear();
    let host = app.state::<ModuleHost>();
    let mut running = host
        .programs
        .running
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let current = running
        .get(id)
        .is_some_and(|other| Arc::ptr_eq(other, program));
    if !current {
        return;
    }
    running.remove(id);
    drop(running);
    tracing::warn!(module = %id, "module program stopped by itself");

    let mut restarts = host
        .programs
        .restarts
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let count = restarts.entry(id.to_string()).or_default();
    *count += 1;
    if *count <= MAX_RESTARTS {
        drop(restarts);
        thread::sleep(Duration::from_secs(2));
        sync(app, &app::settings(app));
    }
}

/// Relays a call from the module's interface to its program.
pub fn call(app: &AppHandle, id: &str, method: &str, params: Value) -> Result<Value> {
    let host = app.state::<ModuleHost>();
    let module = host
        .get(id)
        .ok_or_else(|| AppError::Invalid(format!("{id} is not installed")))?;
    if !app::settings(app).is_module_active(id, &module.info.elements()) {
        return Err(AppError::Invalid(format!("{id} is turned off")));
    }
    let program = start(app, &module)?;
    let call = program.next_call.fetch_add(1, Ordering::Relaxed);
    let (sender, receiver) = mpsc::channel();
    program
        .pending
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(call, sender);
    if let Err(error) =
        program.send(&json!({ "type": "call", "id": call, "method": method, "params": params }))
    {
        answer(&program, call, Err(String::new()));
        return Err(error);
    }
    let reply = receiver.recv_timeout(CALL_TIMEOUT);
    *program
        .last_used
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = Instant::now();
    if !module.info.background {
        stop_when_idle(app, id, &program);
    }
    match reply {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(message)) => Err(AppError::Failed(message)),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            answer(&program, call, Err(String::new()));
            Err(AppError::Failed(format!("{id} did not answer")))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(AppError::Failed(format!("{id} stopped"))),
    }
}

fn stop_when_idle(app: &AppHandle, id: &str, program: &Arc<Program>) {
    let (app, id, program) = (app.clone(), id.to_string(), Arc::clone(program));
    thread::spawn(move || {
        thread::sleep(IDLE);
        let idle = program
            .last_used
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .elapsed()
            >= IDLE;
        let busy = !program
            .pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_empty();
        let programs = &app.state::<ModuleHost>().programs;
        if idle
            && !busy
            && programs
                .get(&id)
                .is_some_and(|current| Arc::ptr_eq(&current, &program))
        {
            programs.stop(&id);
        }
    });
}

/// Starts the background programs of active modules and stops the programs of inactive ones.
pub fn sync(app: &AppHandle, settings: &app::Settings) {
    let host = app.state::<ModuleHost>();
    for module in host.list() {
        if module.info.native.is_none() {
            continue;
        }
        let active = settings.is_module_active(&module.id, &module.info.elements());
        if !active {
            host.programs.stop(&module.id);
            host.programs
                .restarts
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .remove(&module.id);
        } else if module.info.background
            && host.programs.get(&module.id).is_none()
            && let Err(error) = start(app, &module)
        {
            tracing::warn!(module = %module.id, %error, "could not start a module program");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_messages_of_a_program() {
        let result: Message =
            serde_json::from_str(r#"{"type":"result","id":4,"value":[1]}"#).unwrap();
        assert!(matches!(result, Message::Result { id: 4, .. }));
        let event: Message =
            serde_json::from_str(r#"{"type":"event","name":"changed","payload":null}"#).unwrap();
        assert!(matches!(event, Message::Event { .. }));
        assert!(serde_json::from_str::<Message>(r#"{"type":"exec","command":"x"}"#).is_err());
    }

    #[test]
    fn accepts_simple_event_names_only() {
        assert!(is_event_name("color-picked"));
        assert!(!is_event_name(""));
        assert!(!is_event_name("../settings"));
        assert!(!is_event_name("Changed"));
    }
}
