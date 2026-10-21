//! Artifact path identity (RFC 009).
//!
//! Every artifact returned by `run_in_memory` must have a unique relative
//! path, compared case-insensitively (ZIP extraction onto case-insensitive
//! file systems would otherwise silently overwrite entries).

mod fixtures;

use logolig::services::exporter::run_in_memory;
use logolig::services::ingest::ingest_bytes;
use logolig::{ExportPlan, WebManifestSettings};

fn paths_of(plan: &ExportPlan) -> Vec<String> {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    run_in_memory(&asset, plan)
        .expect("run_in_memory should succeed")
        .into_iter()
        .map(|a| a.relative_path.to_string_lossy().replace('\\', "/"))
        .collect()
}

/// C1: every optional output enabled at once still yields unique paths.
#[test]
fn c1_all_options_enabled_yield_unique_paths_case_insensitively() {
    let plan = ExportPlan {
        monochrome: true,
        include_microsoft_app_logos: true,
        web_manifest: Some(WebManifestSettings::default()),
        png_sizes: vec![16, 32, 48, 192, 512],
        ..ExportPlan::default()
    };
    let paths = paths_of(&plan);
    let mut lowered: Vec<String> = paths.iter().map(|p| p.to_lowercase()).collect();
    lowered.sort();
    let before = lowered.len();
    lowered.dedup();
    assert_eq!(before, lowered.len(), "duplicate artifact paths: {paths:?}");
    // Both layouts are present: colour at the root and monochrome under mono/.
    assert!(paths.contains(&"favicon-32.png".to_string()));
    assert!(paths.contains(&"mono/favicon-32.png".to_string()));
    assert!(paths.contains(&"favicon.ico".to_string()));
    assert!(paths.contains(&"mono/favicon.ico".to_string()));
}

/// C4: regression guard. The default-plan path list and order for a PNG
/// source is the list recorded before RFC 009 was implemented.
#[test]
fn c4_default_plan_path_list_and_order_are_unchanged() {
    let expected: Vec<&str> = vec![
        "favicon.svg",
        "favicon.ico",
        "favicon-32.png",
        "favicon-192.png",
        "favicon-512.png",
        "apple-touch-icon.png",
        "favicon-snippet.html",
    ];
    assert_eq!(paths_of(&ExportPlan::default()), expected);
}
