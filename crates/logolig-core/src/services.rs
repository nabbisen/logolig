//! サービス層。
//!
//! ファイル I/O、SVG ラスタライズ、リサイズなど副作用を伴う処理。
//! UI 層 (logolig-app) からはここを経由してのみ I/O を行う。
//!
//! Step 1 ではすべて未実装 (`AppError::NotImplemented`) を返すスタブ。
//! Step 2 で `ingest` と画像処理が、Step 4 で出力系 (`ico_writer` /
//! `html_snippet`) が埋まる予定。

pub mod ingest;
pub mod rasterize_svg;
