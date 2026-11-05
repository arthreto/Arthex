//! `arthex-module://localhost/<id>/<file>` (`http://arthex-module.localhost/…` on Windows):
//! the interface files of installed modules. Nothing else on disk can be reached through it.

use std::{borrow::Cow, fs};

use tauri::{
    Manager, Runtime, UriSchemeContext,
    http::{Request, Response, StatusCode, header},
};

use super::{ModuleHost, SCHEME, package::is_safe_path};

/// What a module's interface may load. Its program is never served.
const TYPES: &[(&str, &str)] = &[
    ("js", "text/javascript"),
    ("css", "text/css"),
    ("json", "application/json"),
    ("png", "image/png"),
    ("svg", "image/svg+xml"),
    ("woff2", "font/woff2"),
];

pub fn base_url(id: &str) -> String {
    if cfg!(windows) {
        format!("http://{SCHEME}.localhost/{id}/")
    } else {
        format!("{SCHEME}://localhost/{id}/")
    }
}

pub fn handle<R: Runtime>(
    context: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let path = request.uri().path().trim_start_matches('/');
    let served = path.split_once('/').and_then(|(id, file)| {
        let host = context.app_handle().try_state::<ModuleHost>()?;
        let module = host.get(id)?;
        let kind = TYPES
            .iter()
            .find(|(extension, _)| {
                file.rsplit_once('.')
                    .is_some_and(|(_, ext)| ext == *extension)
            })?
            .1;
        if !is_safe_path(file) || module.info.native.as_deref() == Some(file) {
            return None;
        }
        Some((fs::read(module.dir.join(file)).ok()?, kind))
    });
    let response = Response::builder()
        // Module scripts load from another origin than the interface.
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    match served {
        Some((bytes, kind)) => response
            .header(header::CONTENT_TYPE, kind)
            .body(Cow::Owned(bytes)),
        None => response
            .status(StatusCode::NOT_FOUND)
            .body(Cow::Borrowed(&[][..])),
    }
    .unwrap_or_else(|_| Response::new(Cow::Borrowed(&[][..])))
}
