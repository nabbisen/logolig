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

use crate::app::{is_mobile, resolve_theme, AppState, Message, Screen};
use crate::ui::{advanced_drawer, bottom_nav, colors, drop_zone, picker_overlay, sidebar};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let mobile = is_mobile(state);

    // v1.20.0: モバイル/デスクトップで sidebar/footer を排他切替。
    // - デスクトップ (>= 768px): `side_bar` slot に縦並びサイドバー
    // - モバイル (< 768px): `footer` slot に横並び下部ナビ
    //
    // 同じ Message を発火するので、 user-facing な機能は全く変わらない。
    // 「同じ機能の場所が変わるだけ」 のメンタルモデル (PNG モック明示)。
    let mut layout = AppLayout::new(body(state))
        .header(header(state))
        .direction(LayoutDirection::Ltr)
        .toasts(state.toasts.clone())
        .on_close_menus(Message::SidebarPickerClosed)
        .on_close_modals(Message::CloseModals);

    if mobile {
        layout = layout.footer(bottom_nav::view(state));
    } else {
        layout = layout.side_bar(sidebar::view(state));
    }

    // 言語 / テーマピッカーが開いていれば context_menu slot に乗せる。
    if let Some(picker) = state.active_picker {
        layout = layout.context_menu(picker_overlay::view(state, picker));
    }

    // 詳細設定ドロワー (v1.17.0 の Right Sheet)。 サイドバー / 下部ナビの
    // 設定アイコン押下 → AdvancedToggled で開閉。
    if state.advanced_open {
        let sheet_pixels = drawer_pixel_width(state.window_size.width, mobile);
        let sheet = Sheet::new(advanced_drawer::view(state))
            .at(SheetEdge::End)
            .with_size(SheetSize::Pixels(sheet_pixels));
        layout = layout.sheet(sheet);
    }

    render(layout)
}

/// v1.17.0: Right Sheet の幅を画面幅から算出する。
///
/// - **デスクトップ**: `window_width / 3` を `[280px, 480px]` で clamp。
///   小さい画面ではラベルが読める下限を確保し、 大きい画面では中央コンテンツ
///   を圧迫しない上限を設ける。
/// - **モバイル (v1.20.0)**: `[280px, window_width - 16px]` で clamp。
///   画面幅をほぼ占有 (16px の margin だけ残してコンテンツが背後に少し
///   見える程度)。 これにより 375px の iPhone でも 359px のドロワーで
///   設定項目が読めるようになる。 280px の下限は、 280px 未満の画面 (= 殆ど
///   存在しない) で破綻させないための保険。
fn drawer_pixel_width(window_width: f32, mobile: bool) -> f32 {
    if mobile {
        (window_width - 16.0).clamp(280.0, window_width)
    } else {
        (window_width / 3.0).clamp(280.0, 480.0)
    }
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

    // v1.20.0: モバイル時はヘッダーの左右 padding を縮小 (画面幅の節約)。
    // - デスクトップ: 12 vertical / 20 horizontal
    // - モバイル: 12 vertical / 8 horizontal
    let h_pad: f32 = if is_mobile(state) { 8.0 } else { 20.0 };
    container(title_block)
        .padding([12.0, h_pad])
        .width(Length::Fill)
        .into()
}
