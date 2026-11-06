//! `arthex-module://localhost/<id>/<file>` (`http://arthex-module.localhost/…` on Windows):
//! the interface files of installed modules, and under `user-media/` the images and videos in
//! `<data>/module-data/<id>/media/`. Nothing else on disk can be reached through it.

use std::{
    borrow::Cow,
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
};

use tauri::{
    Manager, Runtime, UriSchemeContext,
    http::{Request, Response, StatusCode, header, response::Builder},
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

const MEDIA_FOLDER: &str = "media";
const MEDIA_PREFIX: &str = "user-media/";
const MEDIA_TYPES: &[(&str, &str)] = &[
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("mp4", "video/mp4"),
    ("m4v", "video/mp4"),
    ("mov", "video/mp4"),
    ("webm", "video/webm"),
];
/// A range is answered with at most this much: a video is read as it plays, never held in memory.
const LONGEST_RANGE: u64 = 1024 * 1024;

fn type_of<'a>(types: &'a [(&str, &str)], file: &str) -> Option<&'a str> {
    let (_, extension) = file.rsplit_once('.')?;
    types
        .iter()
        .find(|(known, _)| extension.eq_ignore_ascii_case(known))
        .map(|(_, kind)| *kind)
}

/// The bytes a `Range: bytes=<start>-<end>` header asks of a file of this size, both included.
fn parse_range(header: &str, size: u64) -> Option<(u64, u64)> {
    let (start, end) = header.strip_prefix("bytes=")?.split_once('-')?;
    let start: u64 = start.parse().ok()?;
    if start >= size {
        return None;
    }
    let last = size - 1;
    let end = if end.is_empty() {
        last
    } else {
        end.parse::<u64>().ok()?.min(last)
    };
    (start <= end).then(|| (start, end.min(start + LONGEST_RANGE - 1)))
}

fn read_range(file: &mut File, start: u64, end: u64) -> Option<Vec<u8>> {
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut bytes = Vec::new();
    file.take(end - start + 1).read_to_end(&mut bytes).ok()?;
    Some(bytes)
}

fn media_response(
    host: &ModuleHost,
    id: &str,
    file: &str,
    range: Option<&str>,
    response: Builder,
) -> Option<Response<Cow<'static, [u8]>>> {
    let kind = type_of(MEDIA_TYPES, file)?;
    if !is_safe_path(file) || file.contains('/') || host.get(id).is_none() {
        return None;
    }
    let mut opened = File::open(host.data.join(id).join(MEDIA_FOLDER).join(file)).ok()?;
    let size = opened.metadata().ok()?.len();
    let response = response
        .header(header::CONTENT_TYPE, kind)
        .header(header::ACCEPT_RANGES, "bytes");
    let built = if let Some(range) = range {
        let (start, end) = parse_range(range, size)?;
        response
            .status(StatusCode::PARTIAL_CONTENT)
            .header(header::CONTENT_RANGE, format!("bytes {start}-{end}/{size}"))
            .body(Cow::Owned(read_range(&mut opened, start, end)?))
    } else {
        let mut bytes = Vec::new();
        opened.read_to_end(&mut bytes).ok()?;
        response.body(Cow::Owned(bytes))
    };
    built.ok()
}

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
    let response = || {
        Response::builder()
            // Module scripts load from another origin than the interface.
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(header::CACHE_CONTROL, "no-store")
            .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
    };
    if let Some((id, name)) = path
        .split_once('/')
        .and_then(|(id, file)| Some((id, file.strip_prefix(MEDIA_PREFIX)?)))
    {
        let range = request
            .headers()
            .get(header::RANGE)
            .and_then(|value| value.to_str().ok());
        let media = context
            .app_handle()
            .try_state::<ModuleHost>()
            .and_then(|host| media_response(&host, id, name, range, response()));
        return media.unwrap_or_else(|| {
            response()
                .status(StatusCode::NOT_FOUND)
                .body(Cow::Borrowed(&[][..]))
                .unwrap_or_else(|_| Response::new(Cow::Borrowed(&[][..])))
        });
    }
    let served = path.split_once('/').and_then(|(id, file)| {
        let host = context.app_handle().try_state::<ModuleHost>()?;
        let module = host.get(id)?;
        let kind = type_of(TYPES, file)?;
        if !is_safe_path(file) || module.info.native.as_deref() == Some(file) {
            return None;
        }
        Some((fs::read(module.dir.join(file)).ok()?, kind))
    });
    let response = response();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_range_is_read_from_its_header() {
        assert_eq!(parse_range("bytes=0-99", 1000), Some((0, 99)));
        assert_eq!(parse_range("bytes=900-", 1000), Some((900, 999)));
        assert_eq!(parse_range("bytes=900-5000", 1000), Some((900, 999)));
    }

    #[test]
    fn a_long_range_is_cut_so_that_a_video_stays_out_of_memory() {
        let huge = 10 * LONGEST_RANGE;
        assert_eq!(parse_range("bytes=0-", huge), Some((0, LONGEST_RANGE - 1)));
    }

    #[test]
    fn a_range_outside_the_file_or_broken_is_refused() {
        assert_eq!(parse_range("bytes=1000-", 1000), None);
        assert_eq!(parse_range("bytes=50-10", 1000), None);
        assert_eq!(parse_range("bytes=-500", 1000), None);
        assert_eq!(parse_range("items=0-1", 1000), None);
        assert_eq!(parse_range("bytes=0-1", 0), None);
    }

    #[test]
    fn only_images_and_videos_count_as_media() {
        assert_eq!(type_of(MEDIA_TYPES, "cours.MP4"), Some("video/mp4"));
        assert_eq!(type_of(MEDIA_TYPES, "photo.png"), Some("image/png"));
        assert_eq!(type_of(MEDIA_TYPES, "page.html"), None);
        assert_eq!(type_of(MEDIA_TYPES, "dessin.svg"), None);
        assert_eq!(type_of(MEDIA_TYPES, "sans-extension"), None);
    }
}
