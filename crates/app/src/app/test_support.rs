//! Test helpers shared by the `*_tests` modules: source assets, results and SVG fixtures.

use super::*;
use std::sync::Arc;

use crate::result::{ResultAssetItem, ResultAssetKind, ResultAssets};

pub(super) fn dummy_source_asset() -> SourceAsset {
    SourceAsset {
        path: PathBuf::from("logo.png"),
        kind: logolig::SourceKind::Png,
        raw: Arc::<[u8]>::from([0x89, b'P', b'N', b'G'].as_slice()),
        intrinsic_size: Some((1, 1)),
    }
}

pub(super) fn dummy_result_assets() -> ResultAssets {
    ResultAssets {
        items: vec![ResultAssetItem {
            file_name: "favicon-snippet.html".to_string(),
            relative_path: "favicon-snippet.html".to_string(),
            bytes: b"<link rel=\"icon\" href=\"/favicon.ico\">".to_vec(),
            kind: ResultAssetKind::HtmlSnippet,
            dimensions: None,
            thumbnail: None,
        }],
    }
}

pub(super) fn ingest_ok(state: &mut AppState, size: Option<(u32, u32)>) {
    let asset = SourceAsset {
        intrinsic_size: size,
        ..dummy_source_asset()
    };
    // The result of the current job (RFC 011).
    let job = state.job_id;
    let _ = update(state, Message::IngestCompleted(job, Ok(asset)));
}

pub(super) const SVG_TEXT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><text x="2" y="30" font-size="20">Hi</text></svg>"#;
pub(super) const SVG_IMAGE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="64" height="64"><image width="8" height="8" xlink:href="/some/file.png"/></svg>"#;
pub(super) const SVG_BOTH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="64" height="64"><image width="8" height="8" xlink:href="/some/file.png"/><text x="2" y="30" font-size="20">Hi</text></svg>"#;
pub(super) const SVG_PLAIN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="8" height="8"/></svg>"#;

pub(super) fn svg_asset(path: &str) -> SourceAsset {
    SourceAsset {
        path: PathBuf::from(path),
        kind: logolig::SourceKind::Svg,
        raw: Arc::<[u8]>::from(b"<svg/>".as_slice()),
        intrinsic_size: Some((64, 64)),
    }
}

/// State with `logo.svg` as the current source, as after a successful ingest.
pub(super) fn state_with_svg_source() -> AppState {
    AppState {
        source_asset: Some(svg_asset("logo.svg")),
        ..AppState::default()
    }
}

pub(super) fn analyzed(state: &mut AppState, path: &str, svg: &str) {
    let report = logolig::services::svg::analyze(svg.as_bytes());
    let job = state.job_id;
    let _ = update(
        state,
        Message::SvgAnalyzed(job, PathBuf::from(path), report),
    );
}

/// A preview cache for `path` with fully opaque pixels, as the preview build would deliver
/// for a source without transparency (the audit of the first cache raises an advisory toast).
pub(super) fn opaque_preview_cache(
    path: &str,
    algorithm: logolig::ResizeAlgorithm,
) -> logolig::PreviewCache {
    let raster = |side: u32| {
        logolig::Rgba8::try_from_raw(
            side,
            side,
            Arc::<[u8]>::from(vec![0xFF; (side * side * 4) as usize]),
        )
        .expect("a square of side*side*4 bytes")
    };
    logolig::PreviewCache {
        source_path: PathBuf::from(path),
        algorithm,
        tab_16: raster(16),
        icon_120: raster(120),
    }
}
