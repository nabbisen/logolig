//! Source-image ingestion.
//!
//! Accepts raw file bytes, detects the format by magic signature (not by
//! file extension), and returns a `SourceAsset`.
//!
//! ## Format detection
//!
//! | Magic bytes               | Format |
//! |---------------------------|--------|
//! | `89 50 4E 47 0D 0A 1A 0A` | PNG    |
//! | `3C 3F 78 6D 6C` / `3C 73 76 67` | SVG (UTF-8 XML / direct `<svg`) |
//! | `FF D8 FF`                | JPEG   |
//! | `52 49 46 46 … 57 45 42 50` | WebP (RIFF container) |
//!
//! Files without a recognised magic signature are rejected regardless of
//! their file extension (prevents extension spoofing).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::{SourceAsset, SourceKind};
use crate::error::AppError;

/// Load a file and return a `SourceAsset`.
///
/// Intended to be called via `iced::Task::perform` so the UI thread
/// is not blocked (§2.4).
pub async fn ingest(path: PathBuf) -> Result<SourceAsset, AppError> {
    // 1. Narrow down by file extension (unknown extensions are rejected)
    let ext_kind = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(SourceKind::from_extension);

    // 2. Read the file asynchronously
    let raw = tokio::fs::read(&path)
        .await
        .map_err(|e| AppError::io(path.display().to_string(), e.to_string()))?;

    // 3. Confirm format with magic bytes; reject on mismatch (UnsupportedFile)
    let kind = detect_kind(&raw, ext_kind)
        .ok_or_else(|| AppError::unsupported_file(path.display().to_string()))?;

    // 4. Extract intrinsic dimensions
    let intrinsic_size = match kind {
        SourceKind::Png => parse_png_size(&raw),
        SourceKind::Svg => parse_svg_size(&raw),
        SourceKind::Webp => parse_webp_size(&raw),
        SourceKind::Jpeg => parse_jpeg_size(&raw),
    };

    Ok(SourceAsset {
        path,
        kind,
        raw: Arc::<[u8]>::from(raw),
        intrinsic_size,
    })
}

/// Synchronous version for tests (inject hand-crafted bytes directly).
/// Not used in the production I/O path (always go through the async version).
pub fn ingest_bytes(path: impl AsRef<Path>, bytes: Vec<u8>) -> Result<SourceAsset, AppError> {
    let path = path.as_ref().to_path_buf();
    let ext_kind = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(SourceKind::from_extension);

    let kind = detect_kind(&bytes, ext_kind)
        .ok_or_else(|| AppError::unsupported_file(path.display().to_string()))?;

    let intrinsic_size = match kind {
        SourceKind::Png => parse_png_size(&bytes),
        SourceKind::Svg => parse_svg_size(&bytes),
        SourceKind::Webp => parse_webp_size(&bytes),
        SourceKind::Jpeg => parse_jpeg_size(&bytes),
    };

    Ok(SourceAsset {
        path,
        kind,
        raw: Arc::<[u8]>::from(bytes),
        intrinsic_size,
    })
}

// ---------------------------------------------------------------------------
// Internal: magic byte detection and header parsing
// ---------------------------------------------------------------------------

const PNG_MAGIC: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// JPEG magic bytes (SOI marker). Added in v1.11.0.
/// JPEG starts with `FF D8 FF` (third byte varies: `FF E0` JFIF / `FF E1` Exif
/// etc., so some implementations check only 2 bytes, but we check 3 to
/// avoid accidental matches).
const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8, 0xFF];

/// WebP is a RIFF container. The first 12 bytes are:
///   "RIFF" (4) + file-size LE (4) + "WEBP" (4)
/// We do not distinguish VP8 / VP8L / VP8X here (image-webp handles that).
fn looks_like_webp(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
}

fn detect_kind(bytes: &[u8], hint: Option<SourceKind>) -> Option<SourceKind> {
    if bytes.starts_with(PNG_MAGIC) {
        return Some(SourceKind::Png);
    }
    if bytes.starts_with(JPEG_MAGIC) {
        return Some(SourceKind::Jpeg);
    }
    if looks_like_webp(bytes) {
        return Some(SourceKind::Webp);
    }
    if looks_like_svg(bytes) {
        return Some(SourceKind::Svg);
    }
    // Files without a recognised magic signature are rejected regardless of extension.
    // For example: a file claiming to be ".png" via its extension is rejected.
    let _ = hint;
    None
}

/// Lightweight SVG detection. Comments or BOMs may precede the XML declaration,
/// so treat the file as SVG if `<svg` appears anywhere in the first 1 KB.
fn looks_like_svg(bytes: &[u8]) -> bool {
    let head_len = bytes.len().min(1024);
    let head = &bytes[..head_len];
    if let Ok(s) = std::str::from_utf8(head) {
        let low = s.to_ascii_lowercase();
        low.contains("<svg")
    } else {
        false
    }
}

/// Read width and height from the PNG IHDR chunk.
/// PNG layout: 8-byte magic followed immediately by the IHDR chunk:
///   8 (magic) + 4 (length) + 4 ("IHDR") + 4 (width BE) + 4 (height BE) ...
fn parse_png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    // Offset where the IHDR width field starts
    const W_OFF: usize = 16;
    if bytes.len() < W_OFF + 8 {
        return None;
    }
    if !bytes.starts_with(PNG_MAGIC) {
        return None;
    }
    if &bytes[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes(bytes[W_OFF..W_OFF + 4].try_into().ok()?);
    let h = u32::from_be_bytes(bytes[W_OFF + 4..W_OFF + 8].try_into().ok()?);
    if w == 0 || h == 0 { None } else { Some((w, h)) }
}

/// Delegate SVG dimension extraction to usvg.
/// usvg fills in width/height even for SVGs that only have a viewBox.
/// On parse failure, return `None` and let ingest succeed.
/// (The error is more contextual if it appears at rasterise time.)
fn parse_svg_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(bytes, &opt).ok()?;
    let size = tree.size();
    Some((size.width().ceil() as u32, size.height().ceil() as u32))
}

/// Extract WebP dimensions from the first chunk of the RIFF container.
///
/// WebP has three sub-formats:
/// - **VP8** (Lossy): width/height as 14-bit values starting 6 bytes after the chunk header
/// - **VP8L** (Lossless): 14-bit values, 1-based, starting 1 byte after the chunk header
/// - **VP8X** (Extended): 24-bit values, 1-based, starting 4 bytes after the chunk header
///
/// Returns `None` on failure (the proper decoder provides detailed errors at decode time).
/// For favicons, VP8 / VP8L dominate. VP8X (animation/alpha extension)
/// gets minimal support only.
fn parse_webp_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if !looks_like_webp(bytes) {
        return None;
    }
    // RIFF header (12 bytes) is followed immediately by chunks.
    // Each chunk header: FourCC (4 bytes) + size LE (4 bytes).
    if bytes.len() < 12 + 8 {
        return None;
    }
    let chunk_fourcc = &bytes[12..16];
    let chunk_data = &bytes[20..];

    match chunk_fourcc {
        b"VP8 " => {
            // VP8 lossy: width/height near bytes 7–10 of the keyframe header.
            // 3-byte start code + 3 bytes of version/flags = skip 6 bytes.
            if chunk_data.len() < 10 {
                return None;
            }
            // 0x3FFF mask extracts 14 bits (top 2 bits are scale flags)
            let w_raw = u16::from_le_bytes([chunk_data[6], chunk_data[7]]);
            let h_raw = u16::from_le_bytes([chunk_data[8], chunk_data[9]]);
            let w = (w_raw & 0x3FFF) as u32;
            let h = (h_raw & 0x3FFF) as u32;
            (w > 0 && h > 0).then_some((w, h))
        }
        b"VP8L" => {
            // VP8L lossless: after 1-byte signature, width-1 and height-1 are packed
            // as 14-bit values each (little-endian).
            if chunk_data.len() < 5 {
                return None;
            }
            // Verify the 0x2F signature byte
            if chunk_data[0] != 0x2F {
                return None;
            }
            let b1 = chunk_data[1] as u32;
            let b2 = chunk_data[2] as u32;
            let b3 = chunk_data[3] as u32;
            let b4 = chunk_data[4] as u32;
            let w = (b1 | ((b2 & 0x3F) << 8)) + 1;
            let h = ((b2 >> 6) | (b3 << 2) | ((b4 & 0x0F) << 10)) + 1;
            (w > 0 && h > 0).then_some((w, h))
        }
        b"VP8X" => {
            // VP8X: 1-byte flags + 3-byte reserved + width-1 (24-bit LE)
            // + height-1 (24-bit LE).
            if chunk_data.len() < 10 {
                return None;
            }
            let w = (chunk_data[4] as u32
                | ((chunk_data[5] as u32) << 8)
                | ((chunk_data[6] as u32) << 16))
                + 1;
            let h = (chunk_data[7] as u32
                | ((chunk_data[8] as u32) << 8)
                | ((chunk_data[9] as u32) << 16))
                + 1;
            (w > 0 && h > 0).then_some((w, h))
        }
        _ => None,
    }
}

/// Read width and height from a JPEG file (v1.11.0).
///
/// JPEG structure: SOI (FF D8) followed by marker segments.
/// Each segment starts with `FF XX` (XX = marker ID); the next 2 bytes are
/// the segment length (BE, self-inclusive). Dimensions are in the SOF marker:
///
/// - SOF0 (0xC0): Baseline DCT
/// - SOF1 (0xC1): Extended sequential DCT
/// - SOF2 (0xC2): Progressive DCT
/// - SOF3 (0xC3): Lossless
/// - 0xC4 is DHT (not SOF). SOF range: C0-C3 / C5-C7 / C9-CB / CD-CF.
///
/// We extract dimensions from the first SOF marker found.
/// SOF segment payload: [precision(1), height(2 BE), width(2 BE), ...]
///
/// Returns `None` on failure (corrupt/truncated file). The image crate can often
/// still decode successfully even without accurate dimensions here,
/// so this is best-effort.
fn parse_jpeg_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || !bytes.starts_with(JPEG_MAGIC) {
        return None;
    }
    // Start after the SOI (FF D8).
    let mut i = 2usize;
    while i + 4 <= bytes.len() {
        // Markers always start with FF. Multiple consecutive 0xFF bytes are allowed
        // (fill bytes); skip them and treat the next non-FF byte as the marker ID.
        if bytes[i] != 0xFF {
            return None;
        }
        let mut j = i + 1;
        while j < bytes.len() && bytes[j] == 0xFF {
            j += 1;
        }
        if j >= bytes.len() {
            return None;
        }
        let marker = bytes[j];
        i = j + 1;

        // Standalone marker (no segment length field):
        // - SOI (D8), EOI (D9), TEM (01), RST0..RST7 (D0-D7)
        if marker == 0xD8 || marker == 0xD9 || marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            continue;
        }

        // Read segment length (2-byte BE, self-inclusive).
        if i + 2 > bytes.len() {
            return None;
        }
        let seg_len = u16::from_be_bytes([bytes[i], bytes[i + 1]]) as usize;
        if seg_len < 2 {
            return None;
        }

        // SOF range: C0-C3, C5-C7, C9-CB, CD-CF (DHT=C4, JPG=C8, DAC=CC excluded)
        let is_sof = matches!(
            marker,
            0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF
        );
        if is_sof {
            // SOF segment data follows for seg_len bytes (self-inclusive).
            // Content: precision(1) + height(2 BE) + width(2 BE) + ...
            // i points to seg_len. Data starts at i+2; skip precision (1 byte),
            // so height is at i+3 and width at i+5.
            if i + 7 > bytes.len() {
                return None;
            }
            let h = u16::from_be_bytes([bytes[i + 3], bytes[i + 4]]) as u32;
            let w = u16::from_be_bytes([bytes[i + 5], bytes[i + 6]]) as u32;
            return (w > 0 && h > 0).then_some((w, h));
        }

        // Not a SOF marker: skip the entire segment.
        i += seg_len;
    }
    None
}
