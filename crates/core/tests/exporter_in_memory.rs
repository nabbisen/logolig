//! End-to-end tests for the export orchestrator: `run_in_memory`.

#![allow(clippy::field_reassign_with_default)]

mod fixtures;

use fixtures::fresh_tmp_dir;

use logolig::ExportPlan;
use logolig::services::exporter::run;
use logolig::services::exporter::run_in_memory;
use logolig::services::ingest::ingest_bytes;

// ---------------------------------------------------------------------------
// v1.19.0: run_in_memory unit tests
// ---------------------------------------------------------------------------
//
// `run_in_memory` is the in-memory variant of `run`, introduced in v1.16.0
// for the "drop file → auto-convert → Result screen" flow.
// Zero disk I/O; no temp directory needed.
//
// The existing 12 tests continue to cover `run` (disk version) — since `run`
// is now a thin wrapper around `run_in_memory`, passing tests for `run`
// also covers most of the in-memory logic. These additional tests verify:
// - Artifact count / order / relative paths
// - Byte-for-byte match with `run` output
// - Subdirectories (mono/) expressed in relative_path
// - Failure (empty ico_sizes) → nothing written (zero disk side-effects)

#[test]
fn run_in_memory_returns_default_artifact_set_from_png_source() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let plan = ExportPlan::default();

    let artifacts = run_in_memory(&asset, &plan).expect("in-memory run should succeed");

    // v1.2.0 default: 7 files (same set as run).
    let expected_names = [
        "favicon.svg",
        "favicon.ico",
        "favicon-32.png",
        "favicon-192.png",
        "favicon-512.png",
        "apple-touch-icon.png",
        "favicon-snippet.html",
    ];
    let actual_names: Vec<String> = artifacts
        .iter()
        .map(|a| a.relative_path.to_string_lossy().to_string())
        .collect();
    for name in expected_names {
        assert!(
            actual_names.iter().any(|n| n == name),
            "missing artifact: {} (have: {:?})",
            name,
            actual_names
        );
    }
    assert_eq!(artifacts.len(), expected_names.len());
}

#[test]
fn run_in_memory_bytes_match_disk_run_byte_for_byte() {
    // Run both run_in_memory and run with the same asset + plan;
    // verify every artifact matches byte-for-byte. Confirms `run`
    // is a faithful wrapper (in-memory content survives the disk round-trip).
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let plan = ExportPlan::default();

    let in_memory = run_in_memory(&asset, &plan).expect("in-memory should succeed");

    let dir = fresh_tmp_dir("byte-match");
    let report = run(&asset, &plan, &dir).expect("disk should succeed");

    assert_eq!(in_memory.len(), report.artifacts.len());

    for art in &in_memory {
        let on_disk = dir.join(&art.relative_path);
        assert!(on_disk.is_file(), "expected on disk: {}", on_disk.display());
        let disk_bytes = std::fs::read(&on_disk).unwrap();
        assert_eq!(
            art.bytes,
            disk_bytes,
            "byte mismatch for {}",
            art.relative_path.display()
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_in_memory_encodes_mono_subdirectory_in_relative_path() {
    // When monochrome is enabled, verify artifacts have subdirectory-prefixed
    // relative paths like `mono/favicon-{size}.png`.
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let mut plan = ExportPlan::default();
    plan.monochrome = true;

    let artifacts = run_in_memory(&asset, &plan).expect("should succeed");

    let mono_paths: Vec<String> = artifacts
        .iter()
        .map(|a| a.relative_path.to_string_lossy().to_string())
        .filter(|p| p.starts_with("mono/") || p.starts_with("mono\\"))
        .collect();
    assert!(
        !mono_paths.is_empty(),
        "expected mono/ artifacts but found none. all: {:?}",
        artifacts
            .iter()
            .map(|a| a.relative_path.display().to_string())
            .collect::<Vec<_>>()
    );

    // At least one of mono/favicon.ico or mono/favicon-NN.png must be present
    // (ico is always present since include_ico=true).
    assert!(mono_paths.iter().any(|p| p.ends_with("favicon.ico")));
}

#[test]
fn run_in_memory_returns_err_on_empty_ico_sizes() {
    // An invalid plan (empty ico_sizes + include_ico=true) should return Err.
    // Being in-memory, zero disk side-effects are guaranteed
    // (no files are written).
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let mut plan = ExportPlan::default();
    plan.ico_sizes.clear();

    let result = run_in_memory(&asset, &plan);
    assert!(result.is_err(), "expected Err, got {:?}", result.is_ok());
}

#[test]
fn run_in_memory_skips_optional_artifacts_when_disabled() {
    // include_apple_touch=false / include_html_snippet=false / include_svg=false
    // include_ico=false → the ICO relative_path is absent.
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let mut plan = ExportPlan::default();
    plan.include_apple_touch = false;
    plan.include_html_snippet = false;
    plan.include_svg = false;
    plan.include_ico = false;

    let artifacts = run_in_memory(&asset, &plan).expect("should succeed");

    let names: Vec<String> = artifacts
        .iter()
        .map(|a| a.relative_path.to_string_lossy().to_string())
        .collect();

    assert!(
        !names.iter().any(|n| n == "apple-touch-icon.png"),
        "apple-touch should be omitted, got: {:?}",
        names
    );
    assert!(
        !names.iter().any(|n| n == "favicon-snippet.html"),
        "html snippet should be omitted, got: {:?}",
        names
    );
    assert!(
        !names.iter().any(|n| n == "favicon.svg"),
        "svg should be omitted, got: {:?}",
        names
    );
    assert!(
        !names.iter().any(|n| n == "favicon.ico"),
        "ico should be omitted, got: {:?}",
        names
    );

    // PNG remains (default 32/192/512).
    assert!(names.iter().any(|n| n == "favicon-32.png"));
    assert!(names.iter().any(|n| n == "favicon-192.png"));
    assert!(names.iter().any(|n| n == "favicon-512.png"));
}
