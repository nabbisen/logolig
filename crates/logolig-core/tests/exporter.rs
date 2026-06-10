//! エクスポートオーケストレータの end-to-end テスト。
//!
//! 一時ディレクトリに対して export を走らせ、 期待される全ファイルが
//! 「正しい中身」で存在することを確認する。

mod fixtures;

use std::path::PathBuf;

use logolig_core::services::exporter::run;
use logolig_core::services::ingest::ingest_bytes;
use logolig_core::ExportPlan;

/// 一意の一時ディレクトリを作る。 std::env::temp_dir + nanos.
fn fresh_tmp_dir(label: &str) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("logolig-test-{label}-{pid}-{nanos}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn exports_default_artifact_set_from_png_source() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("png-default");
    let plan = ExportPlan::default();

    let report = run(&asset, &plan, &dir).expect("export should succeed");

    // 期待されるアーティファクト群
    let expected_names = [
        "favicon.ico",
        "favicon-32.png",
        "favicon-192.png",
        "favicon-512.png",
        "apple-touch-icon.png",
        "favicon-snippet.html",
    ];
    for name in expected_names {
        let p = dir.join(name);
        assert!(p.is_file(), "missing artifact: {}", p.display());
    }
    assert_eq!(report.artifacts.len(), expected_names.len());

    // staging 残骸が無いこと (transactional rollback / cleanup の確認)
    let leftovers: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with(".logolig-"))
        .collect();
    assert!(leftovers.is_empty(), "staging directory should be cleaned up");

    // 後始末
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn exports_default_artifact_set_from_svg_source() {
    let asset = ingest_bytes("tile.svg", fixtures::SVG_16.as_bytes().to_vec()).unwrap();
    let dir = fresh_tmp_dir("svg-default");
    let plan = ExportPlan::default();

    run(&asset, &plan, &dir).expect("svg export should succeed");
    assert!(dir.join("favicon.ico").is_file());
    assert!(dir.join("apple-touch-icon.png").is_file());
    assert!(dir.join("favicon-snippet.html").is_file());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn ico_can_be_read_back_with_correct_frames() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("ico-roundtrip");
    let plan = ExportPlan::default();

    run(&asset, &plan, &dir).unwrap();

    // 書き戻した ICO を ico crate で読み直して、 16/32/48 のフレームが揃っていること
    let f = std::fs::File::open(dir.join("favicon.ico")).unwrap();
    let icondir = ico::IconDir::read(std::io::BufReader::new(f)).unwrap();
    let mut sizes: Vec<u32> = icondir
        .entries()
        .iter()
        .map(|e| e.width())
        .collect();
    sizes.sort_unstable();
    assert_eq!(sizes, vec![16, 32, 48]);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn html_snippet_file_contains_link_tags() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("html-content");
    run(&asset, &ExportPlan::default(), &dir).unwrap();

    let html = std::fs::read_to_string(dir.join("favicon-snippet.html")).unwrap();
    assert!(html.contains(r#"<link rel="icon" href="/favicon.ico""#));
    assert!(html.contains(r#"rel="apple-touch-icon""#));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fails_cleanly_when_output_dir_does_not_exist() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let bad = std::env::temp_dir().join("logolig-this-path-must-not-exist-yet-xyz");
    let _ = std::fs::remove_dir_all(&bad);
    let err = run(&asset, &ExportPlan::default(), &bad).expect_err("should fail");
    // エラー内容は AppError::Export(_)
    let s = err.to_string();
    assert!(s.contains("output directory"));
}

#[test]
fn no_apple_touch_omits_that_file() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("opt-out");
    let mut plan = ExportPlan::default();
    plan.include_apple_touch = false;
    plan.include_html_snippet = false;

    let report = run(&asset, &plan, &dir).unwrap();
    assert!(!dir.join("apple-touch-icon.png").exists());
    assert!(!dir.join("favicon-snippet.html").exists());
    // PNG と ICO は残る
    assert!(dir.join("favicon.ico").is_file());
    assert!(dir.join("favicon-32.png").is_file());
    assert_eq!(
        report.artifacts.len(),
        1 + ExportPlan::default().png_sizes.len()
    );

    let _ = std::fs::remove_dir_all(&dir);
}
