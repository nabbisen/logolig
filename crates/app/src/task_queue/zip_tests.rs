//! ZIP integrity tests (RFC 009).

use super::*;

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use logolig::services::exporter::run_in_memory;
use logolig::services::ingest::ingest_bytes;
use logolig::{ExportPlan, WebManifestSettings};

struct TempFile(PathBuf);

impl TempFile {
    fn new(label: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "logolig-{label}-{}-{unique}.zip",
            std::process::id()
        )))
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn png_fixture() -> Vec<u8> {
    let mut buf = image::RgbaImage::new(8, 8);
    for px in buf.pixels_mut() {
        *px = image::Rgba([0xCC, 0x33, 0x33, 0xFF]);
    }
    let mut out = Vec::new();
    image::DynamicImage::ImageRgba8(buf)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("PNG encoding for fixture should not fail");
    out
}

fn all_options_plan() -> ExportPlan {
    ExportPlan {
        monochrome: true,
        include_microsoft_app_logos: true,
        web_manifest: Some(WebManifestSettings::default()),
        png_sizes: vec![16, 32, 48, 192, 512],
        ..ExportPlan::default()
    }
}

fn all_options_artifacts() -> Vec<InMemoryArtifact> {
    let asset = ingest_bytes("logo.png", png_fixture()).expect("fixture should ingest");
    run_in_memory(&asset, &all_options_plan()).expect("run_in_memory should succeed")
}

fn read_back(bytes: Vec<u8>) -> zip::ZipArchive<std::io::Cursor<Vec<u8>>> {
    zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip should be readable")
}

fn expected_entries(artifacts: &[InMemoryArtifact]) -> Vec<(String, Vec<u8>)> {
    artifacts
        .iter()
        .map(|a| {
            (
                a.relative_path.to_string_lossy().replace('\\', "/"),
                a.bytes.clone(),
            )
        })
        .collect()
}

fn assert_archive_matches(
    archive: &mut zip::ZipArchive<impl std::io::Read + std::io::Seek>,
    expected: &[(String, Vec<u8>)],
) {
    assert_eq!(archive.len(), expected.len(), "entry count");
    let mut names: Vec<String> = archive.file_names().map(String::from).collect();
    names.sort();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "duplicate entry names: {names:?}");
    for (name, bytes) in expected {
        let mut entry = archive.by_name(name).expect("entry should exist");
        let mut got = Vec::new();
        entry.read_to_end(&mut got).unwrap();
        assert_eq!(&got, bytes, "bytes of {name}");
    }
}

fn item(relative_path: &str, bytes: &[u8]) -> ResultAssetItem {
    ResultAssetItem {
        file_name: relative_path.rsplit('/').next().unwrap().to_string(),
        relative_path: relative_path.to_string(),
        bytes: bytes.to_vec(),
        kind: ResultAssetKind::Svg,
        dimensions: None,
        thumbnail: None,
    }
}

/// Z1: full pipeline with every option on. Entry names are exactly the artifact
/// relative paths, colour and `mono/` twins both present, no duplicates.
/// (Failed on the unmodified code with `Duplicate filename`.)
#[test]
fn z1_full_pipeline_zip_contains_every_artifact_once() {
    let artifacts = all_options_artifacts();
    let expected = expected_entries(&artifacts);
    for must in [
        "favicon-32.png",
        "mono/favicon-32.png",
        "favicon.ico",
        "mono/favicon.ico",
    ] {
        assert!(
            expected.iter().any(|(n, _)| n == must),
            "fixture lacks {must}"
        );
    }
    let items = collect_assets(artifacts).expect("collect_assets should succeed");
    let bytes = build_zip_bytes(&items.items).expect("zip should build");
    assert_archive_matches(&mut read_back(bytes), &expected);
}

/// Z2: classification and relative paths.
#[test]
fn z2_collect_assets_classifies_and_keeps_relative_paths() {
    let items = collect_assets(all_options_artifacts()).expect("collect_assets should succeed");
    for item in &items.items {
        assert!(!item.relative_path.contains('\\'), "{}", item.relative_path);
        assert!(item.relative_path.ends_with(&item.file_name));
        if let Some(rest) = item.relative_path.strip_prefix("mono/") {
            assert_eq!(rest, item.file_name);
            assert!(
                matches!(item.kind, ResultAssetKind::PngMono | ResultAssetKind::Ico),
                "{}: {:?}",
                item.relative_path,
                item.kind
            );
        } else {
            assert_eq!(item.relative_path, item.file_name, "root item");
        }
    }
    assert!(
        items
            .items
            .iter()
            .any(|i| i.relative_path.starts_with("mono/"))
    );
}

/// Z2 (separators): a path built from components, as on Windows, joins with `/`.
#[test]
fn z2_component_built_path_joins_with_forward_slash() {
    let path: PathBuf = ["mono", "favicon-32.png"].iter().collect();
    assert_eq!(relative_path_string(&path).unwrap(), "mono/favicon-32.png");
}

fn artifact(path: PathBuf) -> InMemoryArtifact {
    InMemoryArtifact {
        relative_path: path,
        bytes: vec![0],
    }
}

/// Z3: path hygiene. `..`, absolute and (on Unix) non-UTF-8 components are rejected.
#[test]
fn z3_collect_assets_rejects_unsafe_paths() {
    for bad in ["../evil.png", "mono/../../evil.png", "/etc/passwd"] {
        let err = collect_assets(vec![artifact(PathBuf::from(bad))])
            .expect_err("unsafe path must be rejected");
        assert!(matches!(err, AppError::Export { .. }), "{bad}: {err:?}");
    }
    let err = collect_assets(vec![artifact(PathBuf::new())]).expect_err("empty path rejected");
    assert!(matches!(err, AppError::Export { .. }));

    #[cfg(unix)]
    {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        let bad = PathBuf::from(OsString::from_vec(vec![b'a', 0xFF, b'.', b'p', b'n', b'g']));
        let err = collect_assets(vec![artifact(bad)]).expect_err("non-UTF-8 must be rejected");
        assert!(matches!(err, AppError::Export { .. }), "{err:?}");
    }
}

/// Z4: duplicate relative paths handed straight to `build_zip_bytes` are an error.
#[test]
fn z4_duplicate_relative_paths_are_an_error() {
    let items = vec![item("favicon.ico", b"a"), item("favicon.ico", b"b")];
    let err = build_zip_bytes(&items).expect_err("duplicate must fail");
    assert!(matches!(err, AppError::Export { .. }), "{err:?}");
}

/// Z5: writing into a directory that does not exist fails and creates nothing.
#[test]
fn z5_write_into_missing_directory_fails_and_leaves_nothing() {
    let target = std::env::temp_dir()
        .join(format!("logolig-missing-dir-{}", std::process::id()))
        .join("bundle.zip");
    let err = write_zip_blocking(&target, &[item("a.svg", b"<svg/>")])
        .expect_err("missing directory must fail");
    assert!(matches!(err, AppError::Export { .. }), "{err:?}");
    assert!(!target.exists());
}

/// Z5b (the original defect): a build failure must not leave a file behind.
#[test]
fn z5b_build_failure_leaves_no_file_at_target() {
    let out = TempFile::new("z5b");
    let items = vec![item("favicon.ico", b"a"), item("favicon.ico", b"b")];
    write_zip_blocking(&out.0, &items).expect_err("duplicate must fail");
    assert!(!out.0.exists(), "no partial file may remain");
}

/// Z6: happy path through the real file write.
#[test]
fn z6_write_zip_blocking_happy_path() {
    let artifacts = all_options_artifacts();
    let expected = expected_entries(&artifacts);
    let items = collect_assets(artifacts).expect("collect_assets should succeed");
    let out = TempFile::new("z6");
    write_zip_blocking(&out.0, &items.items).expect("zip write should succeed");
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&out.0).expect("zip should exist"))
        .expect("zip should be readable");
    assert_archive_matches(&mut archive, &expected);
}
