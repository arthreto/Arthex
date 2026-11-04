//! Icons travel to the interface as small PNG data URLs, so the webview never loads a file or
//! a remote image itself.

use std::{
    collections::HashMap,
    io::Cursor,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock, PoisonError},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use image::{DynamicImage, ImageFormat, ImageReader, Limits, imageops::FilterType};

/// Side of the stored icons, twice the largest size shown.
pub const SIZE: u32 = 96;
const MAX_SOURCE_SIDE: u32 = 2048;

/// Decodes an image of any supported format and returns it as a PNG data URL, or `None` if the
/// bytes are not a usable image.
pub fn png_data_url(bytes: &[u8]) -> Option<String> {
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SOURCE_SIDE);
    limits.max_image_height = Some(MAX_SOURCE_SIDE);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().ok()?;
    if image.width() < 8 || image.height() < 8 {
        return None;
    }
    encode(image)
}

/// An SVG icon chosen by the user, shown through `<img>` where it cannot run scripts.
pub fn svg_data_url(bytes: &[u8]) -> Option<String> {
    const MAX_SVG: usize = 200 * 1024;
    let text = std::str::from_utf8(bytes).ok()?;
    let looks_like_svg = text.trim_start().starts_with('<') && text.contains("<svg");
    (bytes.len() <= MAX_SVG && looks_like_svg)
        .then(|| format!("data:image/svg+xml;base64,{}", STANDARD.encode(bytes)))
}

/// Accepts only the data URLs this module produces, so the interface cannot store anything else.
pub fn is_icon_data_url(value: &str) -> bool {
    const MAX_URL: usize = 300 * 1024;
    value.len() <= MAX_URL
        && (value.starts_with("data:image/png;base64,")
            || value.starts_with("data:image/svg+xml;base64,"))
}

/// The icon Windows shows for a file or program, cached for the session.
pub fn file_icon(path: &Path) -> Option<String> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Option<String>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Mutex::default);
    if let Some(icon) = cache
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(path)
    {
        return icon.clone();
    }
    let icon = extract(path);
    cache
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(path.to_path_buf(), icon.clone());
    icon
}

#[cfg(windows)]
fn extract(path: &Path) -> Option<String> {
    #[allow(clippy::cast_possible_wrap)]
    let (width, height, pixels) = super::shell::icon(path, SIZE as i32)?;
    rgba_data_url(width, height, pixels)
}

/// Raw RGBA pixels, as the Windows shell hands them out, to a PNG data URL.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn rgba_data_url(width: u32, height: u32, pixels: Vec<u8>) -> Option<String> {
    encode(DynamicImage::ImageRgba8(image::RgbaImage::from_raw(
        width, height, pixels,
    )?))
}

#[cfg(not(windows))]
fn extract(_path: &Path) -> Option<String> {
    None
}

fn encode(image: DynamicImage) -> Option<String> {
    let image = if image.width() > SIZE || image.height() > SIZE {
        image.resize(SIZE, SIZE, FilterType::Lanczos3)
    } else {
        image
    };
    let mut png = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .ok()?;
    Some(format!("data:image/png;base64,{}", STANDARD.encode(png)))
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};

    use super::*;

    fn png(side: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        RgbaImage::from_pixel(side, side, Rgba([20, 120, 220, 255]))
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        bytes
    }

    #[test]
    fn shrinks_large_images_and_keeps_small_ones() {
        let url = png_data_url(&png(512)).unwrap();
        let decoded = STANDARD
            .decode(url.strip_prefix("data:image/png;base64,").unwrap())
            .unwrap();
        assert_eq!(image::load_from_memory(&decoded).unwrap().width(), SIZE);
        assert!(png_data_url(&png(32)).is_some());
    }

    #[test]
    fn rejects_what_is_not_an_icon() {
        assert!(png_data_url(b"<html>not an image</html>").is_none());
        assert!(png_data_url(&png(4)).is_none());
        assert!(svg_data_url(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>").is_some());
        assert!(svg_data_url(b"plain text").is_none());
        assert!(is_icon_data_url("data:image/png;base64,AA"));
        assert!(!is_icon_data_url("https://example.org/icon.png"));
    }
}
