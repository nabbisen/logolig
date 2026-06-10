//! トップレベルレイアウトの組み立て。
//!
//! ## v1.18.0: 左サイドバー化
//!
//! 旧 v1.17 までは `column[header, body]` の縦並びで、 ヘッダー右側に
//! アイコンボタン群 (言語 / テーマ / 詳細 / 閉じる) を置いていた。 v1.18 で
//! PNG モック (新外部設計) に従い:
//!
//! - **左サイドバー** に 設定 / 言語 / テーマ の 3 アイコンを縦並び
//! - **ヘッダー** はロゴ + アプリ名 (or ファイル名) のみに簡素化
//! - **閉じるボタン廃止** — OS のネイティブウィンドウチャートに任せる
//!   (ブラウザ移行を視野に入れた決定、 Q2-a)
//! - **言語/テーマピッカー** は `context_menu` slot のオーバーレイで
//!   (cycle UI 廃止、 Q3 ポップアップ方式)
//!
//! AppLayout の slot 構成:
//!
//! ```text
//!                ┌────── header ──────┐  ロゴ + アプリ名/ファイル名
//!                ├──┬─────────────────┤
//!                │  │                 │
//!   side_bar →  │  │       body      │  body は変換中/結果カードグリッド等
//!                │  │                 │
//!                └──┴─────────────────┘
//!                          ↑
//!                  context_menu (任意): 言語/テーマピッカーオーバーレイ
//!                  sheet (任意): 設定ドロワー (右側、 v1.17 既存)
//! ```
//!
//! 各スロットの中身は `crate::ui::*` の純粋関数に委譲する。

use iced::widget::{container, row, text};
use iced::{Element, Length};
use snora::{AppLayout, LayoutDirection, Sheet, SheetEdge, SheetSize, render};

use logolig_core::MessageKey;

use crate::app::{resolve_theme, AppState, Message, Screen};
use crate::ui::{advanced_drawer, colors, drop_zone, picker_overlay, sidebar};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let mut layout = AppLayout::new(body(state))
        .header(header(state))
        .side_bar(sidebar::view(state))
        .direction(LayoutDirection::Ltr)
        .toasts(state.toasts.clone())
        // v1.18.0: 言語/テーマピッカー外側クリックで閉じる
        .on_close_menus(Message::SidebarPickerClosed)
        .on_close_modals(Message::CloseModals);

    // 言語 / テーマピッカーが開いていれば context_menu slot に乗せる。
    // PNG モックの「言語メニュー」 / 「テーマメニュー」 オーバーレイに対応。
    if let Some(picker) = state.active_picker {
        layout = layout.context_menu(picker_overlay::view(state, picker));
    }

    // 詳細設定ドロワー (v1.17.0 の Right Sheet)。 サイドバー設定アイコンの
    // 押下 → AdvancedToggled で開閉。
    if state.advanced_open {
        let sheet_pixels = drawer_pixel_width(state.window_size.width);
        let sheet = Sheet::new(advanced_drawer::view(state))
            .at(SheetEdge::End)
            .with_size(SheetSize::Pixels(sheet_pixels));
        layout = layout.sheet(sheet);
    }

    render(layout)
}

/// v1.17.0: Right Sheet の幅を画面幅から算出する (`window_width / 3` を
/// `[280px, 480px]` で clamp)。
fn drawer_pixel_width(window_width: f32) -> f32 {
    (window_width / 3.0).clamp(280.0, 480.0)
}

fn body(state: &AppState) -> Element<'_, Message> {
    match state.screen {
        Screen::Empty => drop_zone::view(state),
        Screen::Converting => crate::ui::converting::view(state),
        Screen::Result => crate::ui::result_view::view(state),
    }
}

/// v1.18.0 簡素化ヘッダ。
///
/// 旧の右側アイコン群 (言語 / テーマ / 詳細 / 閉じる) は **完全撤去**:
/// - 設定 / 言語 / テーマ → 左サイドバーに移動
/// - 閉じる → OS ネイティブウィンドウチャートに任せる (Q2-a)
///
/// 残るのはタイトル領域 (アプリ名 or ファイル名) のみ。 PNG モックの
/// シンプルなヘッダー (ロゴ + アプリ名のみ) に対応。
fn header(state: &AppState) -> Element<'_, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    // 画面状態によって表示内容を切替:
    // - Empty / Converting: アプリ名 + tagline (= アプリの自己紹介が主目的)
    // - Result: 現在処理中のファイル名 (= 主役は今扱っているファイル、 v1.16.0)
    let title_block: Element<'_, Message> = match state.screen {
        Screen::Result => {
            let file_name = state
                .source_asset
                .as_ref()
                .map(|a| a.display_name())
                .unwrap_or_default();
            text(file_name)
                .size(16)
                .color(colors::file_name(&theme))
                .into()
        }
        Screen::Empty | Screen::Converting => {
            let app_name = text(t.t(MessageKey::AppTitle))
                .size(20)
                .color(colors::app_name(&theme));
            let tagline_text = format!("— {}", t.t(MessageKey::AppTagline));
            let tagline = text(tagline_text).size(13).color(colors::tagline(&theme));
            row![app_name, tagline]
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .into()
        }
    };

    container(title_block)
        .padding([12, 20])
        .width(Length::Fill)
        .into()
}
