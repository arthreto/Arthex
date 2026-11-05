use std::sync::OnceLock;

use image::{
    RgbaImage,
    imageops::{self, FilterType},
};

const LOGO: &[u8] = include_bytes!("../../../icons/128x128.png");
static DECODED: OnceLock<Option<RgbaImage>> = OnceLock::new();

const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
const BLACK: [f32; 3] = [0.0, 0.0, 0.0];

/// Barely there, but not nothing: a layered window lets clicks through where its alpha is zero,
/// and the gaps of the logo must stay clickable.
const FILL_AT_REST: f32 = 1.0 / 255.0;
const FILL_HOVERED: f32 = 0.12;
const BORDER_HOVERED: f32 = 0.2;

/// The logo of Arthex on a tile that only shows under the pointer, in the format of a layered
/// window: premultiplied blue, green, red and alpha, from the top row down.
pub fn icon(size: u16, dark: bool, hovered: bool) -> Vec<u8> {
    let decoded = DECODED.get_or_init(|| {
        image::load_from_memory(LOGO)
            .inspect_err(|error| tracing::warn!(%error, "could not read the logo"))
            .ok()
            .map(|logo| logo.to_rgba8())
    });
    let width = u32::from(size) * 3 / 4;
    let logo = decoded
        .as_ref()
        .map(|logo| imageops::resize(logo, width, width, FilterType::Lanczos3));
    compose(size, dark, hovered, logo.as_ref())
}

fn compose(size: u16, dark: bool, hovered: bool, logo: Option<&RgbaImage>) -> Vec<u8> {
    let extent = f32::from(size);
    let ink = if dark { WHITE } else { BLACK };
    let offset = logo.map_or(0, |logo| (u32::from(size) - logo.width()) / 2);
    let mut pixels = Vec::with_capacity(usize::from(size) * usize::from(size) * 4);
    for row in 0..size {
        for column in 0..size {
            let (x, y) = (f32::from(column) + 0.5, f32::from(row) + 0.5);
            let tile = rounded_box(x, y, extent, extent * 0.28);
            let mut rgba = [0.0; 4];
            if hovered {
                blend(&mut rgba, ink, coverage(tile) * FILL_HOVERED);
                let border = coverage((tile + 0.5).abs() - 0.5) * BORDER_HOVERED;
                blend(&mut rgba, ink, border);
            } else if tile < 0.0 {
                blend(&mut rgba, ink, FILL_AT_REST);
            }
            if let Some([r, g, b, a]) = logo.and_then(|logo| {
                let x = u32::from(column).checked_sub(offset)?;
                let y = u32::from(row).checked_sub(offset)?;
                logo.get_pixel_checked(x, y).map(|pixel| pixel.0)
            }) {
                let color = [r, g, b].map(|channel| f32::from(channel) / 255.0);
                blend(&mut rgba, color, f32::from(a) / 255.0);
            }
            pixels.extend([byte(rgba[2]), byte(rgba[1]), byte(rgba[0]), byte(rgba[3])]);
        }
    }
    pixels
}

/// Signed distance to a rounded square filling the side: negative inside.
fn rounded_box(x: f32, y: f32, side: f32, radius: f32) -> f32 {
    let half = side / 2.0;
    let dx = (x - half).abs() - (half - radius);
    let dy = (y - half).abs() - (half - radius);
    dx.max(0.0).hypot(dy.max(0.0)) + dx.max(dy).min(0.0) - radius
}

fn coverage(distance: f32) -> f32 {
    (0.5 - distance).clamp(0.0, 1.0)
}

fn blend(pixel: &mut [f32; 4], color: [f32; 3], alpha: f32) {
    for channel in 0..3 {
        pixel[channel] = color[channel] * alpha + pixel[channel] * (1.0 - alpha);
    }
    pixel[3] = alpha + pixel[3] * (1.0 - alpha);
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use image::Rgba;

    use super::*;

    const SIZE: u16 = 32;

    fn alpha(pixels: &[u8], x: u16, y: u16) -> u8 {
        pixels[(usize::from(y) * usize::from(SIZE) + usize::from(x)) * 4 + 3]
    }

    fn logo() -> RgbaImage {
        RgbaImage::from_pixel(24, 24, Rgba([255, 0, 0, 255]))
    }

    #[test]
    fn the_real_logo_can_be_read_and_drawn() {
        let pixels = icon(SIZE, true, false);
        assert_eq!(pixels.len(), usize::from(SIZE) * usize::from(SIZE) * 4);
        assert!(alpha(&pixels, SIZE / 2, SIZE / 2) > 200);
    }

    #[test]
    fn the_logo_is_centred() {
        let pixels = compose(SIZE, true, false, Some(&logo()));
        assert_eq!(alpha(&pixels, 16, 16), 255);
        assert_eq!(alpha(&pixels, 4, 16), 255);
        assert!(alpha(&pixels, 3, 16) < 255);
        assert_eq!(alpha(&pixels, 27, 16), 255);
        assert!(alpha(&pixels, 28, 16) < 255);
    }

    #[test]
    fn stays_clickable_around_the_logo_but_not_in_the_corners() {
        let pixels = compose(SIZE, true, false, None);
        assert_eq!(alpha(&pixels, 0, 0), 0);
        assert_eq!(alpha(&pixels, 16, 1), 1);
    }

    #[test]
    fn the_tile_only_shows_under_the_pointer() {
        let (rest, hovered) = (
            compose(SIZE, true, false, None),
            compose(SIZE, true, true, None),
        );
        assert!(alpha(&hovered, 16, 8) > 10 * alpha(&rest, 16, 8));
    }

    #[test]
    fn premultiplied_colours_never_exceed_their_alpha() {
        for hovered in [false, true] {
            for pixel in compose(SIZE, true, hovered, Some(&logo()))
                .as_chunks::<4>()
                .0
            {
                assert!(pixel[0] <= pixel[3] && pixel[1] <= pixel[3] && pixel[2] <= pixel[3]);
            }
        }
    }

    #[test]
    fn the_logo_keeps_its_colours() {
        let pixels = compose(SIZE, true, false, Some(&logo()));
        let centre = (16 * usize::from(SIZE) + 16) * 4;
        assert_eq!(pixels[centre..centre + 4], [0, 0, 255, 255]);
    }
}
