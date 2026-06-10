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

    // v1.2.0 のデフォルトは 7 ファイル: favicon.svg (vectorized) を追加。
    let expected_names = [
        "favicon.svg",
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
    // SVG ソースのときは入力 SVG をそのまま `favicon.svg` としてコピー (v1.2.0)
    assert!(dir.join("favicon.svg").is_file(), "SVG output expected");
    let svg_content = std::fs::read(dir.join("favicon.svg")).unwrap();
    assert_eq!(
        svg_content, fixtures::SVG_16.as_bytes(),
        "SVG source must be copied byte-for-byte (non-destructive)"
    );

    assert!(dir.join("favicon.ico").is_file());
    assert!(dir.join("apple-touch-icon.png").is_file());
    assert!(dir.join("favicon-snippet.html").is_file());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn vectorize_off_omits_svg_file_for_raster_source() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("vectorize-off");
    let mut plan = ExportPlan::default();
    plan.vectorize_on_raster = false;

    let report = run(&asset, &plan, &dir).expect("export should succeed");

    // ラスタソース + vectorize オフ → SVG は出力されない
    assert!(!dir.join("favicon.svg").exists());
    // HTML スニペットも `<link type="image/svg+xml">` を含まない
    let html = std::fs::read_to_string(dir.join("favicon-snippet.html")).unwrap();
    assert!(!html.contains(r#"type="image/svg+xml""#));
    assert!(!html.contains("favicon.svg"));

    // 報告される artifact count から SVG 1 件が引かれている
    assert_eq!(report.artifacts.len(), 6); // 7 - 1 (favicon.svg)

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn include_svg_off_omits_svg_for_svg_source_too() {
    // SVG 入力でも `include_svg=false` なら出力しない
    let asset = ingest_bytes("tile.svg", fixtures::SVG_16.as_bytes().to_vec()).unwrap();
    let dir = fresh_tmp_dir("include-svg-off");
    let mut plan = ExportPlan::default();
    plan.include_svg = false;

    run(&asset, &plan, &dir).unwrap();
    assert!(!dir.join("favicon.svg").exists());
    let html = std::fs::read_to_string(dir.join("favicon-snippet.html")).unwrap();
    assert!(!html.contains("favicon.svg"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn exports_default_artifact_set_from_webp_source() {
    let asset = ingest_bytes("tile.webp", fixtures::webp_8x8_blue()).unwrap();
    let dir = fresh_tmp_dir("webp-default");
    let plan = ExportPlan::default();

    run(&asset, &plan, &dir).expect("WebP export should succeed");
    assert!(dir.join("favicon.ico").is_file());
    assert!(dir.join("apple-touch-icon.png").is_file());
    assert!(dir.join("favicon-32.png").is_file());
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
    // v1.2.0 default では SVG が先頭に来る
    assert!(html.contains(r#"<link rel="icon" type="image/svg+xml" href="/favicon.svg">"#));
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
    // エラー内容は AppError::Export { .. }
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
    // SVG (vectorized) と PNG と ICO は残る (v1.2.0 default で SVG が増えた)
    assert!(dir.join("favicon.svg").is_file());
    assert!(dir.join("favicon.ico").is_file());
    assert!(dir.join("favicon-32.png").is_file());
    assert_eq!(
        report.artifacts.len(),
        // svg (1) + ico (1) + png_sizes (3) = 5
        2 + ExportPlan::default().png_sizes.len()
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// v1.9.0: モノクローム出力セット (mono/ サブディレクトリ)
// ---------------------------------------------------------------------------

#[test]
fn monochrome_emits_mono_subdir_with_png_and_ico() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("mono-png");
    let plan = ExportPlan {
        monochrome: true,
        ..ExportPlan::default()
    };

    let report = run(&asset, &plan, &dir).expect("export should succeed");

    // 通常出力 + mono/ サブディレクトリの全ファイル
    let expected_color = [
        "favicon.svg",
        "favicon.ico",
        "favicon-32.png",
        "favicon-192.png",
        "favicon-512.png",
        "apple-touch-icon.png",
        "favicon-snippet.html",
    ];
    let expected_mono = [
        "mono/favicon.ico",
        "mono/favicon-32.png",
        "mono/favicon-192.png",
        "mono/favicon-512.png",
    ];

    for name in expected_color {
        let p = dir.join(name);
        assert!(p.is_file(), "missing color artifact: {}", p.display());
    }
    for name in expected_mono {
        let p = dir.join(name);
        assert!(p.is_file(), "missing mono artifact: {}", p.display());
    }
    // mono/ 自体がディレクトリとして存在する
    assert!(dir.join("mono").is_dir());

    // artifacts の合計は color 7 + mono 4 = 11 (v1.9.0 の SVG mono は無し)
    assert_eq!(report.artifacts.len(), expected_color.len() + expected_mono.len());

    // mono/favicon-32.png は通常 favicon-32.png と異なるバイト列 (グレースケール化)
    let color_bytes = std::fs::read(dir.join("favicon-32.png")).unwrap();
    let mono_bytes = std::fs::read(dir.join("mono/favicon-32.png")).unwrap();
    assert_ne!(
        color_bytes, mono_bytes,
        "mono PNG should differ from color PNG"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn monochrome_off_does_not_create_mono_dir() {
    // monochrome=false のとき (デフォルト) は mono/ ディレクトリが
    // 一切作られないことを確認 — 既存ユーザの出力が破壊されないことの保証。
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("mono-off");
    let plan = ExportPlan::default(); // monochrome = false

    run(&asset, &plan, &dir).expect("export should succeed");
    assert!(
        !dir.join("mono").exists(),
        "mono/ should not exist when monochrome=false"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn monochrome_with_ico_off_skips_mono_ico() {
    // include_ico=false なら mono/favicon.ico も出ないこと。
    // ユーザが「ICO は要らない」 と決めたなら mono/ICO も要らないはず。
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("mono-no-ico");
    let plan = ExportPlan {
        monochrome: true,
        include_ico: false,
        ..ExportPlan::default()
    };

    run(&asset, &plan, &dir).expect("export should succeed");
    assert!(dir.join("mono").is_dir());
    assert!(!dir.join("mono/favicon.ico").exists());
    // PNG は出ているはず
    assert!(dir.join("mono/favicon-32.png").is_file());

    let _ = std::fs::remove_dir_all(&dir);
}
