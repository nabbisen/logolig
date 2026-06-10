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
//! Step 3 で追加:
//! - `preview`        — Source → 16×16 + 120×120 のプレビューキャッシュ
//!
//! Step 4 で追加:
//! - `encode_png`     — Rgba8 → PNG バイト列
//! - `ico_writer`     — 複数 RGBA8 を 1 つの ICO にまとめる
//! - `html_snippet`   — `<head>` 用 HTML 文字列の生成
//! - `exporter`       — オーケストレータ。 SourceAsset + ExportPlan + dir → 全成果物

pub mod decode_png;
pub mod encode_png;
pub mod exporter;
pub mod html_snippet;
pub mod ico_writer;
pub mod ingest;
pub mod preview;
pub mod rasterize_svg;
pub mod resize;
