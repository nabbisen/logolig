//! The transparency checkerboard view of the preview panel.

use iced::widget::{container, image, stack};
use iced::{Element, Length};

use logolig::Rgba8;

use crate::app::Message;

// ---------------------------------------------------------------------------
// v1.7.0: Transparency checker view
// ---------------------------------------------------------------------------

/// Checkerboard size: 240×240 with 12-px tiles. Per ABDD §12,
/// visually distinguishes transparent areas without relying on colour alone.
const CHECKER_SIDE: u32 = 240;
const CHECKER_TILE: u32 = 12;

/// Generated once and cached for the lifetime of the process.
/// Repeated allocation per frame would be wasteful; use `OnceLock`.
fn checker_handle() -> &'static image::Handle {
    use std::sync::OnceLock;
    static HANDLE: OnceLock<image::Handle> = OnceLock::new();
    HANDLE.get_or_init(|| {
        // Two light-grey shades. The mid-grey makes transparent areas
        // visible whether the logo is light or dark.
        let light: [u8; 4] = [0xE6, 0xE6, 0xE6, 0xFF];
        let dark: [u8; 4] = [0xC0, 0xC0, 0xC0, 0xFF];
        let n = (CHECKER_SIDE as usize) * (CHECKER_SIDE as usize);
        let mut pixels = Vec::with_capacity(n * 4);
        for y in 0..CHECKER_SIDE {
            for x in 0..CHECKER_SIDE {
                let cx = x / CHECKER_TILE;
                let cy = y / CHECKER_TILE;
                let color = if (cx + cy).is_multiple_of(2) {
                    light
                } else {
                    dark
                };
                pixels.extend_from_slice(&color);
            }
        }
        image::Handle::from_rgba(CHECKER_SIDE, CHECKER_SIDE, pixels)
    })
}

/// Transparency-checker view. No framing (no tab or phone border) —
/// the icon is overlaid at actual size on the checkerboard.
///
/// Design decisions:
/// - **No framing**: dedicated mode for inspecting transparency.
///   A tab or phone frame would distract from what to look at.
/// - **120 px display**: icon_120 at actual size — a small icon makes
///   transparent areas less obvious.
/// - **240×240 checkerboard**: leaves a visible margin around the icon
pub(super) fn checker_view<'a>(rgba: &'a Rgba8) -> Element<'a, Message> {
    let icon_bytes: Vec<u8> = rgba.as_bytes().to_vec();
    let icon_handle = image::Handle::from_rgba(rgba.width, rgba.height, icon_bytes);

    // Checkerboard background: 240×240 at actual pixel size
    let checker_layer = container(
        image(checker_handle().clone())
            .width(Length::Fixed(CHECKER_SIDE as f32))
            .height(Length::Fixed(CHECKER_SIDE as f32))
            .filter_method(image::FilterMethod::Nearest),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill);

    // Icon layer: cache.icon_120 at actual size, centred over the checkerboard
    let icon_layer = container(
        image(icon_handle)
            .width(Length::Fixed(rgba.width as f32))
            .height(Length::Fixed(rgba.height as f32))
            .filter_method(image::FilterMethod::Nearest),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill);

    // stack: later elements are drawn on top (iced 0.14).
    container(stack![checker_layer, icon_layer])
        .width(Length::Fixed(CHECKER_SIDE as f32))
        .height(Length::Fixed(CHECKER_SIDE as f32))
        .into()
}
