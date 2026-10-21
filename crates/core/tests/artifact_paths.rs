//! Artifact path identity (RFC 009).
//!
//! Every artifact returned by `run_in_memory` must have a unique relative
//! path, compared case-insensitively (ZIP extraction onto case-insensitive
//! file systems would otherwise silently overwrite entries).

mod fixtures;

use std::path::PathBuf;

use logolig::services::exporter::{ensure_unique_paths, run_in_memory};
use logolig::services::ingest::ingest_bytes;
use logolig::{AppError, ExportPlan, InMemoryArtifact, WebManifestSettings};

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

fn artifact(path: &str) -> InMemoryArtifact {
    InMemoryArtifact {
        relative_path: PathBuf::from(path),
        bytes: vec![1, 2, 3],
    }
}

/// C2: two identical paths are rejected and the error names the path.
#[test]
fn c2_identical_paths_are_rejected_naming_the_path() {
    let err = ensure_unique_paths(&[artifact("favicon.ico"), artifact("favicon.ico")])
        .expect_err("duplicate must be rejected");
    match err {
        AppError::Export { detail } => assert!(detail.contains("favicon.ico"), "{detail}"),
        other => panic!("expected AppError::Export, got {other:?}"),
    }
}

/// C3: paths that differ only by case are rejected too.
#[test]
fn c3_paths_differing_only_by_case_are_rejected() {
    let err = ensure_unique_paths(&[artifact("Favicon.ico"), artifact("favicon.ico")])
        .expect_err("case-insensitive duplicate must be rejected");
    assert!(matches!(err, AppError::Export { .. }), "{err:?}");
}

/// The colour file and its `mono/` twin are distinct paths, not duplicates.
#[test]
fn mono_twin_is_not_a_duplicate() {
    ensure_unique_paths(&[artifact("favicon.ico"), artifact("mono/favicon.ico")])
        .expect("different directories are different paths");
}

/// C5: an empty list is valid.
#[test]
fn c5_empty_slice_is_ok() {
    ensure_unique_paths(&[]).expect("empty list has no duplicates");
}
