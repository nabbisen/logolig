use super::*;

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempZip(PathBuf);

impl TempZip {
    fn new(name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "logolig-{name}-{}-{unique}.zip",
            std::process::id()
        ));
        Self(path)
    }
}

impl Drop for TempZip {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn asset(file_name: &str, bytes: &[u8]) -> ResultAssetItem {
    ResultAssetItem {
        file_name: file_name.to_string(),
        relative_path: file_name.to_string(),
        bytes: bytes.to_vec(),
        kind: ResultAssetKind::Svg,
        dimensions: None,
        thumbnail: None,
    }
}

#[test]
fn write_zip_blocking_writes_expected_contents_with_current_modified_time() {
    let output = TempZip::new("bundle-metadata");
    let items = vec![
        asset("favicon.svg", br#"<svg viewBox="0 0 1 1"/>"#),
        asset("site.webmanifest", br#"{"name":"Logolig"}"#),
    ];

    write_zip_blocking(&output.0, &items).expect("zip write should succeed");

    let file = std::fs::File::open(&output.0).expect("zip should be created");
    let mut archive = zip::ZipArchive::new(file).expect("zip should be readable");
    assert_eq!(archive.len(), 2);

    let mut favicon = archive
        .by_name("favicon.svg")
        .expect("favicon.svg should be in the zip");
    let modified = favicon
        .last_modified()
        .expect("zip entry should have a modified timestamp");
    assert!(
        modified.year() >= 2024,
        "zip entry timestamp should not fall back to the 1980 ZIP epoch: {modified}"
    );

    let mut bytes = Vec::new();
    favicon
        .read_to_end(&mut bytes)
        .expect("zip entry should be readable");
    assert_eq!(bytes, br#"<svg viewBox="0 0 1 1"/>"#);
}

#[test]
fn current_zip_modified_time_uses_local_wall_clock_fields() {
    let before = Local::now();
    let modified = current_zip_modified_time();
    let after = Local::now();

    assert!(
        same_local_minute(modified, before) || same_local_minute(modified, after),
        "zip timestamp should use local wall-clock time; got {modified}, before={before}, after={after}"
    );
}

fn same_local_minute(modified: zip::DateTime, local: chrono::DateTime<Local>) -> bool {
    modified.year() == local.year() as u16
        && modified.month() == local.month() as u8
        && modified.day() == local.day() as u8
        && modified.hour() == local.hour() as u8
        && modified.minute() == local.minute() as u8
}
