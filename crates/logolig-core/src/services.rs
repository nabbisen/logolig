//! サービス層。
//!
//! ファイル I/O、SVG ラスタライズ、リサイズなど副作用を伴う処理。
//! UI 層 (logolig-app) からはここを経由してのみ I/O を行う。
//!
//! Step 2 で実装済み:
//! - `ingest`         — PNG/SVG 受け入れ + 論理サイズ取得 (async)
//! - `decode_png`     — PNG SourceAsset を RGBA8 に展開
//! - `rasterize_svg`  — SVG SourceAsset を任意サイズの RGBA8 に展開 (個別レンダリング)
//! - `resize`         — RGBA8 → 別サイズの RGBA8 (fast_image_resize, Lanczos3 デフォルト)
//!
//! Step 4 で追加予定:
//! - `ico_writer`     — 複数 RGBA8 を 1 つの .ico に結合
//! - `html_snippet`   — `<head>` 用 HTML 文字列の生成

pub mod decode_png;
pub mod ingest;
pub mod rasterize_svg;
pub mod resize;
