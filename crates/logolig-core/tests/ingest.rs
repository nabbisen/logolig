//! ingest スケルトンのプレースホルダ確認。
//!
//! Step 1 では `ingest` は async なため、ランタイムを引かずに
//! 「型シグネチャが想定どおりであること」だけを確認する。
//! 本格的な振る舞いテスト (実ファイル PNG/SVG での種別判定など) は
//! Step 2 で `tokio` を dev-dependency に追加してから書く。

use std::future::Future;
use std::path::PathBuf;

use logolig_core::AppError;
use logolig_core::SourceAsset;
use logolig_core::services::ingest::ingest;

#[test]
fn ingest_signature_matches_expectations() {
    // コンパイルできることが本質。`ingest(path)` が
    // `impl Future<Output = Result<SourceAsset, AppError>>` を返すことを型で表現する。
    fn _assert_signature<F>(_: F)
    where
        F: Future<Output = Result<SourceAsset, AppError>>,
    {
    }

    let fut = ingest(PathBuf::from("nonexistent.png"));
    _assert_signature(fut);
}
