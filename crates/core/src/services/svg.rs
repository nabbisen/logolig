//! SVG parsing options and source analysis (RFC 015 phase 2a).
//!
//! ## Why this module exists
//!
//! `usvg::Options::default()` lets an SVG name a local file in `<image href="...">`
//! (an absolute path, a path relative to the process working directory, or a nested
//! SVG) and reads it, already when the file is ingested and again when it is rendered.
//! The pixels of that file would then end up in the generated icons. [`options`] is the
//! single place where parse options are built; it accepts `data:` images only, so no
//! file outside the SVG is ever read.
//!
//! It also keeps the two parse sites (`ingest::parse_svg_size` and
//! `rasterize_svg::rasterize_to_canvas`) from diverging.
//!
//! This phase does **not** load fonts: SVG `<text>` still renders as nothing.
//! [`analyze`] lets the application say so, plainly, once per loaded SVG.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// What an SVG contains that Logolig does not show.
///
/// `#[non_exhaustive]` so that later phases can add fields (for example substituted fonts)
/// without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct SvgReport {
    /// The SVG contains text that would be drawn (not drawn yet): text that usvg would try to
    /// draw and that has content. Text that is never drawn (for example inside an unused
    /// `<defs>`) and empty text elements do not count: nothing is lost there.
    pub has_text: bool,
    /// Number of `<image>` references to something other than a `data:` URL
    /// (absolute path, relative path, nested SVG file) that were refused.
    pub ignored_external_images: usize,
}

/// The one parse-options builder for SVG sources.
///
/// Only `data:` images are resolved; every other reference resolves to nothing. The font
/// resolver stays the default one (no font is loaded in this phase), so rendering is unchanged.
pub(crate) fn options() -> usvg::Options<'static> {
    usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
            resolve_string: Box::new(|_href, _opts| None),
        },
        ..usvg::Options::default()
    }
}

/// Inspect an SVG for parts that are not shown.
///
/// The parse is analysis-only and uses two counting hooks:
///
/// * `resolve_string` is called for every image reference that is not a `data:` URL;
/// * `font_resolver.select_font` is called when usvg converts text that it would draw. With no
///   font database usvg drops such text from the tree, so the tree itself cannot tell; the
///   hook (a documented usvg extension point) can.
///
/// Input that cannot be parsed yields `SvgReport::default()`; the ingest and render paths
/// report their own errors for such files.
pub fn analyze(bytes: &[u8]) -> SvgReport {
    let images = Arc::new(AtomicUsize::new(0));
    let texts = Arc::new(AtomicUsize::new(0));
    let image_counter = Arc::clone(&images);
    let text_counter = Arc::clone(&texts);

    let mut opt = options();
    opt.image_href_resolver.resolve_string = Box::new(move |_href, _opts| {
        image_counter.fetch_add(1, Ordering::SeqCst);
        None
    });
    opt.font_resolver = usvg::FontResolver {
        select_font: Box::new(move |_font, _db| {
            text_counter.fetch_add(1, Ordering::SeqCst);
            None
        }),
        select_fallback: usvg::FontResolver::default_fallback_selector(),
    };

    if usvg::Tree::from_data(bytes, &opt).is_err() {
        return SvgReport::default();
    }
    SvgReport {
        has_text: texts.load(Ordering::SeqCst) > 0,
        ignored_external_images: images.load(Ordering::SeqCst),
    }
}
