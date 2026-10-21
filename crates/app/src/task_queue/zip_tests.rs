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

/// Z1 (pre-change form): full pipeline with every option on, written through
/// the existing `write_zip_blocking`. Fails on the unmodified code because the
/// `mono/` prefix is lost and the ZIP library rejects the duplicate names.
#[test]
fn z1_full_pipeline_zip_contains_every_artifact_once() {
    let artifacts = all_options_artifacts();
    let expected: Vec<(String, Vec<u8>)> = artifacts
        .iter()
        .map(|a| {
            (
                a.relative_path.to_string_lossy().replace('\\', "/"),
                a.bytes.clone(),
            )
        })
        .collect();
    let items = collect_assets(artifacts);

    let out = TempFile::new("z1");
    write_zip_blocking(&out.0, &items.items).expect("zip write should succeed");

    let mut archive =
        zip::ZipArchive::new(std::fs::File::open(&out.0).expect("zip should exist"))
            .expect("zip should be readable");
    assert_eq!(archive.len(), expected.len());
    for (name, bytes) in &expected {
        let mut entry = archive.by_name(name).expect("entry should exist");
        let mut got = Vec::new();
        entry.read_to_end(&mut got).unwrap();
        assert_eq!(&got, bytes, "bytes of {name}");
    }
}
