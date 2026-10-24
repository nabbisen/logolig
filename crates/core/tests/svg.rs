//! SVG safety and honesty (RFC 015 phase 2a).
//!
//! Every file an SVG refers to is created by the test in a unique temporary
//! directory; no test reads or names a file it did not create.

mod fixtures;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use logolig::services::{ingest, rasterize_svg};

/// Serialises the tests that change the process working directory.
static CWD_LOCK: Mutex<()> = Mutex::new(());

/// Unique temporary directory, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "logolig-svg-{label}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        Self(dir)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const MAGENTA: [u8; 4] = [255, 0, 255, 255];

fn magenta_png() -> Vec<u8> {
    fixtures::png_rgba(16, 16, MAGENTA)
}

fn render(svg: &str, size: u32) -> logolig::Rgba8 {
    let asset = ingest::ingest_bytes("fixture.svg", svg.as_bytes().to_vec())
        .expect("fixture SVG should ingest");
    rasterize_svg::rasterize(&asset, size).expect("fixture SVG should render")
}

/// Pixels that are (nearly) the given opaque colour.
fn count_colour(img: &logolig::Rgba8, rgb: [u8; 3]) -> usize {
    img.pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| {
            p[3] > 200
                && p[0].abs_diff(rgb[0]) < 12
                && p[1].abs_diff(rgb[1]) < 12
                && p[2].abs_diff(rgb[2]) < 12
        })
        .count()
}

fn count_visible(img: &logolig::Rgba8) -> usize {
    img.pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] > 0)
        .count()
}

/// T2: an absolute file path in `<image href>` is not read, so the secret's
/// colour never reaches the output.
#[test]
fn t2_absolute_file_reference_is_not_rendered() {
    let dir = TempDir::new("t2");
    let secret = dir.path().join("secret.png");
    std::fs::write(&secret, magenta_png()).unwrap();
    let svg = fixtures::svg_with_image(&secret.to_string_lossy());

    let img = render(&svg, 64);
    assert_eq!(
        count_colour(&img, [255, 0, 255]),
        0,
        "the referenced file's pixels must not appear in the output"
    );
}

/// T3: a relative path is resolved against the process working directory by
/// default. It must not be read, whether the working directory holds the file or not.
#[test]
fn t3_relative_file_reference_is_not_rendered_from_any_working_directory() {
    let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = std::env::current_dir().expect("cwd");
    let with_file = TempDir::new("t3-with");
    let without_file = TempDir::new("t3-without");
    std::fs::write(with_file.path().join("secret.png"), magenta_png()).unwrap();
    let svg = fixtures::svg_with_image("secret.png");

    let outcome = std::panic::catch_unwind(|| {
        std::env::set_current_dir(with_file.path()).unwrap();
        let a = count_colour(&render(&svg, 64), [255, 0, 255]);
        std::env::set_current_dir(without_file.path()).unwrap();
        let b = count_colour(&render(&svg, 64), [255, 0, 255]);
        (a, b)
    });
    std::env::set_current_dir(&original).expect("restore cwd");
    let (a, b) = outcome.expect("render must not panic");
    assert_eq!(
        a, 0,
        "working directory containing the file: file must not be read"
    );
    assert_eq!(b, 0, "working directory without the file: nothing to read");
}

/// T4: a nested SVG file is not loaded either.
#[test]
fn t4_nested_svg_file_reference_is_not_rendered() {
    let dir = TempDir::new("t4");
    let inner = dir.path().join("inner.svg");
    std::fs::write(&inner, fixtures::SVG_GREEN_64).unwrap();
    let svg = fixtures::svg_with_image(&inner.to_string_lossy());

    let img = render(&svg, 64);
    assert_eq!(
        count_colour(&img, [0, 200, 0]),
        0,
        "inner SVG must not be drawn"
    );
}

/// T5: an embedded `data:` image still renders.
#[test]
fn t5_data_url_image_still_renders() {
    use std::fmt::Write as _;
    // Minimal base64 encoder (no new dependency): standard alphabet with padding.
    fn b64(data: &[u8]) -> String {
        const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in data.chunks(3) {
            let n = (u32::from(chunk[0]) << 16)
                | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
                | u32::from(*chunk.get(2).unwrap_or(&0));
            let _ = write!(out, "{}", T[(n >> 18) as usize & 63] as char);
            let _ = write!(out, "{}", T[(n >> 12) as usize & 63] as char);
            out.push(if chunk.len() > 1 {
                T[(n >> 6) as usize & 63] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                T[n as usize & 63] as char
            } else {
                '='
            });
        }
        out
    }
    let svg = fixtures::svg_with_image(&format!("data:image/png;base64,{}", b64(&magenta_png())));
    let img = render(&svg, 64);
    assert!(
        count_colour(&img, [255, 0, 255]) > 64 * 64 * 9 / 10,
        "the embedded image must fill the canvas"
    );
}

/// T9: ingesting an SVG must not open the file it refers to. A named pipe stands in for
/// the "file": opening a FIFO for reading blocks until a writer appears, and a
/// non-blocking open of its write end succeeds only while some process has it open for reading.
#[cfg(unix)]
#[test]
fn t9_ingest_does_not_open_referenced_files() {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let dir = TempDir::new("t9");
    let fifo = dir.path().join("fifo.png");
    match std::process::Command::new("mkfifo").arg(&fifo).status() {
        Ok(s) if s.success() => {}
        other => {
            eprintln!(
                "SKIPPED t9_ingest_does_not_open_referenced_files: mkfifo unavailable ({other:?})"
            );
            return;
        }
    }
    let svg = fixtures::svg_with_image(&fifo.to_string_lossy());

    let opened_for_reading = Arc::new(AtomicBool::new(false));
    let ingest_done = Arc::new(AtomicBool::new(false));
    let flag = opened_for_reading.clone();
    let done = ingest_done.clone();
    let fifo2 = fifo.clone();
    let helper = std::thread::spawn(move || {
        // Poll until the ingest has returned (the 10 s limit is only a safety net).
        let start = std::time::Instant::now();
        while !done.load(Ordering::SeqCst) && start.elapsed() < std::time::Duration::from_secs(10) {
            // O_NONBLOCK: fails with ENXIO while nobody has the pipe open for reading.
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .write(true)
                .custom_flags(0o4000)
                .open(&fifo2)
            {
                flag.store(true, Ordering::SeqCst);
                let _ = f.write_all(&magenta_png());
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });
    let asset = ingest::ingest_bytes("fixture.svg", svg.into_bytes());
    ingest_done.store(true, Ordering::SeqCst);
    helper.join().unwrap();
    assert!(asset.is_ok(), "ingest of the SVG itself must still succeed");
    assert!(
        !opened_for_reading.load(Ordering::SeqCst),
        "ingest opened a file named inside the SVG"
    );
}

/// T1: a plain SVG renders exactly as before the change. The hashes (FNV-1a 64 over the RGBA
/// bytes) were captured on the base commit, before any production change.
#[test]
fn t1_plain_svg_renders_unchanged() {
    fn fnv(data: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in data {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h
    }
    let expected: [(u32, u64, usize); 3] = [
        (16, 0x74b4_be21_dd12_a8a5, 144),
        (32, 0xbd3a_2309_97f5_cd25, 576),
        (64, 0x619b_74a4_8988_cb25, 2304),
    ];
    for (size, hash, visible) in expected {
        let img = render(fixtures::SVG_16, size);
        assert_eq!(
            fnv(&img.pixels),
            hash,
            "pixels of the plain SVG at {size} px changed"
        );
        assert_eq!(
            count_visible(&img),
            visible,
            "visible pixel count at {size} px"
        );
    }
}

/// T6: an SVG with text is reported as such. Fonts are not loaded in this phase, so the text
/// still renders as nothing; this test documents that limit (phase 2b changes the assertion).
#[test]
fn t6_text_is_reported_and_not_drawn_yet() {
    let report = logolig::services::svg::analyze(fixtures::SVG_TEXT_ONLY.as_bytes());
    assert!(report.has_text, "a text element must be detected");
    assert_eq!(report.ignored_external_images, 0);
    let img = render(fixtures::SVG_TEXT_ONLY, 128);
    assert_eq!(count_visible(&img), 0, "text is not drawn yet (phase 2b)");
}

/// T6 (more shapes): text inside nested groups and text drawn through `<use>` are reported.
#[test]
fn t6_text_in_groups_and_through_use_is_reported() {
    let analyze = |svg: &str| logolig::services::svg::analyze(svg.as_bytes());
    assert!(
        analyze(fixtures::SVG_TEXT_IN_GROUPS).has_text,
        "nested groups"
    );
    assert!(
        analyze(fixtures::SVG_TEXT_VIA_USE).has_text,
        "drawn through <use>"
    );
}

/// T6b: text that is not drawn is not a loss, so it must not be reported: text only inside
/// `<defs>`, and an empty `<text>` element. This pins the meaning of `has_text`.
#[test]
fn t6b_text_that_is_never_drawn_or_empty_is_not_reported() {
    let analyze = |svg: &str| logolig::services::svg::analyze(svg.as_bytes());
    assert!(
        !analyze(fixtures::SVG_TEXT_ONLY_IN_DEFS).has_text,
        "unused <defs> text"
    );
    assert!(!analyze(fixtures::SVG_EMPTY_TEXT).has_text, "empty <text>");
}

/// The plain fixture reports nothing (T1's report half).
#[test]
fn plain_svg_report_is_empty() {
    let report = logolig::services::svg::analyze(fixtures::SVG_16.as_bytes());
    assert!(!report.has_text);
    assert_eq!(report.ignored_external_images, 0);
}

/// T2/T3/T4 (report half): each refused external reference is counted, `data:` images are not.
#[test]
fn external_references_are_counted_and_data_urls_are_not() {
    let dir = TempDir::new("count");
    let secret = dir.path().join("secret.png");
    std::fs::write(&secret, magenta_png()).unwrap();
    let inner = dir.path().join("inner.svg");
    std::fs::write(&inner, fixtures::SVG_GREEN_64).unwrap();
    let analyze = |svg: &str| logolig::services::svg::analyze(svg.as_bytes());

    assert_eq!(
        analyze(&fixtures::svg_with_image(&secret.to_string_lossy())).ignored_external_images,
        1,
        "absolute path"
    );
    assert_eq!(
        analyze(&fixtures::svg_with_image("secret.png")).ignored_external_images,
        1,
        "relative path"
    );
    assert_eq!(
        analyze(&fixtures::svg_with_image(&inner.to_string_lossy())).ignored_external_images,
        1,
        "nested SVG"
    );
    assert_eq!(
        analyze(&fixtures::svg_with_image(
            "data:image/png;base64,iVBORw0KGgo="
        ))
        .ignored_external_images,
        0,
        "a data: URL is not an external reference"
    );
}

/// T7: text and two external images together.
#[test]
fn t7_text_and_two_external_images() {
    let dir = TempDir::new("t7");
    let a = dir.path().join("a.png");
    let b = dir.path().join("b.png");
    std::fs::write(&a, magenta_png()).unwrap();
    std::fs::write(&b, magenta_png()).unwrap();
    let svg = fixtures::svg_text_and_two_images(&a.to_string_lossy(), &b.to_string_lossy());
    let report = logolig::services::svg::analyze(svg.as_bytes());
    assert!(report.has_text);
    assert_eq!(report.ignored_external_images, 2);
}

/// T8: garbage and empty input give the default report and never panic.
#[test]
fn t8_garbage_and_empty_input_give_the_default_report() {
    let analyze = logolig::services::svg::analyze;
    assert_eq!(analyze(b""), logolig::SvgReport::default());
    assert_eq!(
        analyze(b"this is not xml at all \x00\xff"),
        logolig::SvgReport::default()
    );
    assert_eq!(
        analyze(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00]),
        logolig::SvgReport::default()
    );
    assert_eq!(analyze(b"<svg"), logolig::SvgReport::default());
}
