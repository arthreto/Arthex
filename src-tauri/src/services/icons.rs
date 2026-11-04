//! Icons travel to the interface as small PNG data URLs, so the webview never loads a file or
//! a remote image itself.

use std::io::Cursor;

use base64::{Engine, engine::general_purpose::STANDARD};
use image::{ImageFormat, ImageReader, Limits, imageops::FilterType};

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
    }
}
