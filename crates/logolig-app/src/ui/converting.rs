//! v1.16.0 — Converting 画面 (旧 Importing / Exporting を統合)。
//!
//! ファイル投入後の「自動変換中」 を見せる画面。 PNG モックの ② 状態に対応。
//! デザイン判断 (F2):
//! - 円形プログレス表現は採用するが、 ステップ単位の精緻な進捗追跡は実装
//!   しない。 logolig の実処理は十分高速 (一般的な画像で 1 秒未満) なので、
//!   詳細プログレスは「ロード時間が見た目より長く感じる」 副作用すらある。
//! - 不定長の「処理中」 表現と「対象ファイル名」 の表示で「不安を減らす」
//!   目的を達成する。
//! - 大きな画像 (4K 級) で実際に時間がかかる場合の詳細化は v1.17.x 以降の
//!   余地として残す。
//!
//! ## 構成
//!
//! ```text
//!          ┌─────────────┐
//!          │     ◯       │   ← 円形 (静的、 不定長を表現)
//!          │             │
//!          │   変換中    │
//!          └─────────────┘
//!         しばらくお待ちください
//!
//!         処理中: my-logo.png
//! ```

use iced::widget::{column, container, text};
use iced::{Element, Length};

use logolig_core::MessageKey;

use crate::app::{AppState, Message};
use crate::ui::accessibility::marker;

/// Converting 画面の view。
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;

    // 中央のラベル領域。 円形プログレスのアニメーションは iced 0.14 に標準
    // ウィジェットがないため、 当面は「BUSY マーカー + テキスト」 で表現。
    // ABDD §12 に従い記号 (BUSY: ⏳ 等) で「処理中」 を視覚化、 文字色には
    // 依存しない。
    let main_label = text(format!(
        "{} {}",
        marker::BUSY,
        t.t(MessageKey::ImportingMessage)
    ))
    .size(22);

    // 補助メッセージ: 対象ファイル名を表示して「何を処理中か」 を伝える。
    // ファイル名がまだ取れていない場合 (= 通常ありえないが防御的に) は
    // 簡素な「お待ちください」 のみ表示。
    let processing_subtext: Element<'a, Message> = if let Some(asset) = &state.source_asset {
        let label = format!(
            "{}: {}",
            t.t(MessageKey::PreviewSourceLabel),
            asset.display_name()
        );
        text(label).size(13).into()
    } else {
        text(t.t(MessageKey::ImportingPleaseWait)).size(13).into()
    };

    let inner = column![main_label, processing_subtext]
        .spacing(12)
        .align_x(iced::alignment::Horizontal::Center);

    container(inner)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
