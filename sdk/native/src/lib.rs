//! The program of an installable Arthex module: its system side. Arthex starts it, sends it the
//! calls of the module's interface as JSON lines on its standard input and relays what it writes
//! on its standard output. This crate speaks that protocol, so a program is only its handlers:
//!
//! ```no_run
//! use arthex_module::Module;
//! use serde_json::json;
//!
//! fn main() -> std::io::Result<()> {
//!     Module::new()
//!         .on("hello", |context, params| Ok(json!({ "locale": context.locale(), "echo": params })))
//!         .run()
//! }
//! ```
//!
//! `run` returns when Arthex closes the standard input, which ends the program.

use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, PoisonError},
};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

type Handler = Box<dyn Fn(&Context, Value) -> Result<Value, String>>;
type Output = Arc<Mutex<Box<dyn Write + Send>>>;
type Start = Box<dyn FnOnce(&Context)>;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Incoming {
    #[serde(rename_all = "camelCase")]
    Init {
        data_dir: PathBuf,
        locale: String,
        #[serde(default)]
        shared: HashMap<String, PathBuf>,
    },
    Call {
        id: u64,
        method: String,
        #[serde(default)]
        params: Value,
    },
}

/// Sends events to the module's interface, from any thread. They arrive as `<module id>://<name>`.
#[derive(Clone)]
pub struct Events {
    output: Output,
}

impl Events {
    /// `name` is lowercase letters, digits and `-`, at most 32 characters, or Arthex drops it.
    pub fn emit(&self, name: &str, payload: impl Serialize) -> io::Result<()> {
        send(
            &self.output,
            &json!({ "type": "event", "name": name, "payload": payload }),
        )
    }

    /// Shows or hides the overlay of Arthex: needs `api: 3` in the module's manifest.
    pub fn set_overlay_visible(&self, visible: bool) -> io::Result<()> {
        send(
            &self.output,
            &json!({ "type": "overlay", "visible": visible }),
        )
    }
}

/// What Arthex tells the program when it starts.
pub struct Context {
    data_dir: PathBuf,
    locale: String,
    shared: HashMap<String, PathBuf>,
    events: Events,
}

impl Context {
    /// The folder of a shared part the module's manifest names in `uses`, such as `ffmpeg`.
    pub fn shared(&self, id: &str) -> Option<&Path> {
        self.shared.get(id).map(PathBuf::as_path)
    }

    /// The module's own folder, kept across updates. It is also the working directory.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// The language of the interface (`fr`), for texts the program writes itself.
    pub fn locale(&self) -> &str {
        &self.locale
    }

    pub fn events(&self) -> &Events {
        &self.events
    }
}

#[derive(Default)]
pub struct Module {
    handlers: HashMap<String, Handler>,
    start: Option<Start>,
}

impl Module {
    pub fn new() -> Self {
        Self::default()
    }

    /// Answers `window.__ARTHEX__.call(id, method, params)`. Calls run one after the other.
    #[must_use]
    pub fn on(
        mut self,
        method: &str,
        handler: impl Fn(&Context, Value) -> Result<Value, String> + 'static,
    ) -> Self {
        self.handlers.insert(method.to_string(), Box::new(handler));
        self
    }

    /// Answers a click on one of the module's actions in the right-click menu of Explorer, with
    /// the action's id and the path clicked. Each selected item comes as one call. Answer quickly:
    /// long work belongs on a thread.
    #[must_use]
    pub fn on_context_menu(
        self,
        handler: impl Fn(&Context, &str, &Path) -> Result<Value, String> + 'static,
    ) -> Self {
        self.on("contextMenu", move |context, params| {
            let action = params["action"].as_str().ok_or("no action")?;
            let path = params["path"].as_str().ok_or("no path")?;
            handler(context, action, Path::new(path))
        })
    }

    /// Runs once the program knows its context, to start background work that emits events.
    #[must_use]
    pub fn on_start(mut self, start: impl FnOnce(&Context) + 'static) -> Self {
        self.start = Some(Box::new(start));
        self
    }

    /// Serves Arthex until it closes the standard input.
    pub fn run(self) -> io::Result<()> {
        self.serve(io::stdin().lock(), Box::new(io::stdout()))
    }

    fn serve(mut self, input: impl BufRead, output: Box<dyn Write + Send>) -> io::Result<()> {
        let output: Output = Arc::new(Mutex::new(output));
        let mut context = None;
        for line in input.lines() {
            match serde_json::from_str::<Incoming>(&line?) {
                Ok(Incoming::Init {
                    data_dir,
                    locale,
                    shared,
                }) => {
                    let ready = context.insert(Context {
                        data_dir,
                        locale,
                        shared,
                        events: Events {
                            output: Arc::clone(&output),
                        },
                    });
                    if let Some(start) = self.start.take() {
                        start(ready);
                    }
                }
                Ok(Incoming::Call { id, method, params }) => {
                    let reply = match (&context, self.handlers.get(&method)) {
                        (None, _) => Err("the program was not initialised".to_string()),
                        (_, None) => Err(format!("unknown method {method}")),
                        (Some(context), Some(handler)) => handler(context, params),
                    };
                    let message = match reply {
                        Ok(value) => json!({ "type": "result", "id": id, "value": value }),
                        Err(message) => json!({ "type": "error", "id": id, "message": message }),
                    };
                    send(&output, &message)?;
                }
                // Messages of a newer Arthex are left aside.
                Err(_) => {}
            }
        }
        Ok(())
    }
}

fn send(output: &Output, message: &Value) -> io::Result<()> {
    let mut line = serde_json::to_vec(message)?;
    line.push(b'\n');
    let mut output = output.lock().unwrap_or_else(PoisonError::into_inner);
    output.write_all(&line)?;
    output.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Default)]
    struct Captured(Arc<Mutex<Vec<u8>>>);

    impl Write for Captured {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn serve(module: Module, input: &str) -> Vec<Value> {
        let captured = Captured::default();
        module
            .serve(input.as_bytes(), Box::new(captured.clone()))
            .unwrap();
        let output = String::from_utf8(captured.0.lock().unwrap().clone()).unwrap();
        output
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    const INIT: &str = r#"{"type":"init","dataDir":"/tmp/data","locale":"fr"}"#;

    #[test]
    fn answers_calls_in_order() {
        let module = Module::new()
            .on("double", |_, params| {
                Ok(json!(params.as_i64().unwrap_or(0) * 2))
            })
            .on("locale", |context, _| Ok(json!(context.locale())));
        let input = format!(
            "{INIT}\n{}\n{}\n{}\n",
            r#"{"type":"call","id":1,"method":"double","params":21}"#,
            r#"{"type":"call","id":2,"method":"locale"}"#,
            r#"{"type":"call","id":3,"method":"nope"}"#,
        );
        assert_eq!(
            serve(module, &input),
            [
                json!({ "type": "result", "id": 1, "value": 42 }),
                json!({ "type": "result", "id": 2, "value": "fr" }),
                json!({ "type": "error", "id": 3, "message": "unknown method nope" }),
            ]
        );
    }

    #[test]
    fn hands_over_right_click_actions() {
        let module = Module::new().on_context_menu(|_, action, path| {
            Ok(json!({ "action": action, "path": path.display().to_string() }))
        });
        let input = format!(
            "{INIT}\n{}\n",
            r#"{"type":"call","id":1,"method":"contextMenu","params":{"action":"zip","path":"/a b"}}"#,
        );
        assert_eq!(
            serve(module, &input),
            [json!({ "type": "result", "id": 1, "value": { "action": "zip", "path": "/a b" } })]
        );
    }

    #[test]
    fn starts_once_initialised_and_sends_events() {
        let module = Module::new().on_start(|context| {
            assert_eq!(context.data_dir(), Path::new("/tmp/data"));
            context
                .events()
                .emit("ready", json!({ "ok": true }))
                .unwrap();
            context.events().set_overlay_visible(true).unwrap();
        });
        assert_eq!(
            serve(module, &format!("{INIT}\n")),
            [
                json!({ "type": "event", "name": "ready", "payload": { "ok": true } }),
                json!({ "type": "overlay", "visible": true }),
            ]
        );
    }

    #[test]
    fn refuses_calls_before_init_and_skips_unknown_messages() {
        let module = Module::new().on("ping", |_, _| Ok(Value::Null));
        let input =
            "{\"type\":\"call\",\"id\":7,\"method\":\"ping\"}\n{\"type\":\"later\"}\nnot json\n";
        assert_eq!(
            serve(module, input),
            [json!({ "type": "error", "id": 7, "message": "the program was not initialised" })]
        );
    }
}
