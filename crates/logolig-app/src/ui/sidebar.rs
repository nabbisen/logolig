//! v1.18.0 — 左サイドバー (`ui::sidebar`)。
//!
//! PNG モック (新外部設計) 準拠の左サイドバー。 旧来は右上に横並びで置いて
//! いた 設定 / 言語 / テーマ アイコンを、 左端に縦並びで配置する。
//!
//! ## なぜ snora の `app_side_bar` をそのまま使わないか
//!
//! snora-widgets の `app_side_bar` 関数は「64px 幅 + アイコンのみ + tooltip
//! のみ」 のレールを描画する。 これは Material Design / iOS Settings などで
//! 見られる標準的な「icon-only rail」 だが、 PNG モックは:
//!
//! - 幅約 90px
//! - アイコンの**下**にラベル文字 (「設定」 / 「言語」 / 「テーマ」)
//!
//! という形式で、 「初見ユーザにアイコンの意味が伝わる」 学習コスト最小化を
//! 重視している (= ABDD §12 の「視覚的に冗長な手がかり」 にも合致)。 そのため
//! snora の `Icon` データ型 + `icon_element()` (アイコン → iced Element 変換)
//! は流用しつつ、 レール本体は logolig 側で組み立てる。
//!
//! ## アイコン選定 (lucide-icons 経由)
//!
//! - **設定**: `lucide::Icon::Settings` (歯車)
//! - **言語**: `lucide::Icon::Languages` (言語切替の汎用アイコン;
//!   PNG モックの 🌐 グローブよりも「言語」 という意図が明確)
//! - **テーマ**: `lucide::Icon::Moon` (PNG モック準拠; ダークモードを
//!   連想させる「テーマ全般」 のメンタルモデル)
//!
//! Q4 の決定で、 言語ピッカーには「英語 / 日本語」 のみ含む (中国語は別 ver
//! で対応)。
//!
//! ## レイアウト
//!
//! ```text
//!  ┌────┐
//!  │    │
//!  │ ⚙  │   ← 設定 (Icon::Settings, lucide)
//!  │設定 │   ← ラベル
//!  │    │
//!  │ 🅰 │   ← 言語 (Icon::Languages, lucide)
//!  │言語 │
//!  │    │
//!  │ 🌙 │   ← テーマ (Icon::Moon, lucide)
//!  │テーマ│
//!  │    │
//!  └────┘
//! ```

use iced::widget::{button, column, container, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Theme};

use logolig_core::MessageKey;
use snora::lucide;
use snora::widget::icon_element_sized;
use snora::Icon;

use crate::app::{resolve_theme, AppState, Message, SidebarPicker};
use crate::ui::colors;

/// サイドバー全体の固定幅。 PNG モック準拠。
const SIDEBAR_WIDTH: f32 = 90.0;

/// 各アイコンボタンの正方形サイズ。
const ICON_BUTTON_SIZE: f32 = 44.0;

/// サイドバー描画。 戻り値は AppLayout::side_bar() スロットに渡す Element。
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    // 上から: 設定 → 言語 → テーマ の 3 ボタン。
    // 各ボタンは「アイコン + 下のラベル」 の縦並び。 ボタン全体が共通の
    // background style で描画され、 active (= 関連するピッカーが開いている)
    // 場合のみ強調表示される。

    let settings_btn = sidebar_button(
        Icon::Lucide(lucide::Settings),
        t.t(MessageKey::SidebarLabelSettings),
        // 設定ドロワーの開閉トグル (右側 Sheet)
        Some(Message::AdvancedToggled),
        state.advanced_open,
        &theme,
    );

    let locale_btn = sidebar_button(
        Icon::Lucide(lucide::Languages),
        t.t(MessageKey::SidebarLabelLocale),
        Some(Message::SidebarPickerOpened(SidebarPicker::Locale)),
        state.active_picker == Some(SidebarPicker::Locale),
        &theme,
    );

    let theme_btn = sidebar_button(
        Icon::Lucide(lucide::Moon),
        t.t(MessageKey::SidebarLabelTheme),
        Some(Message::SidebarPickerOpened(SidebarPicker::Theme)),
        state.active_picker == Some(SidebarPicker::Theme),
        &theme,
    );

    let col = column![
        settings_btn,
        locale_btn,
        theme_btn,
        // 残り空間を Space で埋めることで、 ボタン群を上寄せにする。
        Space::new().height(Length::Fill),
    ]
    .spacing(20)
    .align_x(Alignment::Center)
    .padding(Padding::default().top(20).bottom(20));

    container(col)
        .width(Length::Fixed(SIDEBAR_WIDTH))
        .height(Length::Fill)
        .style(|theme: &Theme| {
            // サイドバー全体の背景は、 中央コンテンツ (背景色 base) と区別する
            // ために `background.weak` を使う。 縦のサブトル divider を作る感覚。
            let palette = theme.extended_palette();
            container::Style {
                background: Some(Background::Color(palette.background.weak.color)),
                border: Border {
                    color: palette.background.strong.color,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// 1 個のサイドバーボタン (アイコン + ラベル 2 段)。
fn sidebar_button<'a>(
    icon: Icon,
    label: String,
    on_press: Option<Message>,
    is_active: bool,
    theme: &Theme,
) -> Element<'a, Message> {
    // アイコン要素は snora 経由で取得 (lucide-icons feature が有効なら
    // Icon::Lucide を SVG として描画する)。 サイズは PNG モック寄りの 22px。
    let icon_el: Element<'a, Message> = icon_element_sized(&icon, 22.0);

    // アイコン + ラベル。 ABDD §12 「視覚的に冗長な手がかり」 でアイコン だけ
    // ではなくラベルも表示。 ラベルは小さめのフォントで、 active 状態で色を
    // 切り替える。
    let label_color = if is_active {
        colors::page_title(theme)
    } else {
        colors::muted_text(theme)
    };

    let inner = column![
        container(icon_el)
            .center_x(Length::Fixed(ICON_BUTTON_SIZE))
            .center_y(Length::Fixed(ICON_BUTTON_SIZE)),
        text(label).size(11).color(label_color),
    ]
    .spacing(4)
    .align_x(Alignment::Center);

    let mut btn = button(inner).style(move |theme: &Theme, status| {
        sidebar_button_style(theme, status, is_active)
    });
    if let Some(msg) = on_press {
        btn = btn.on_press(msg);
    }

    btn.into()
}

/// サイドバーボタンのスタイル: 透明 / hover 時に background.weak.color を
/// 薄く塗る / active 時にやや強い background.weak で強調。
fn sidebar_button_style(
    theme: &Theme,
    status: iced::widget::button::Status,
    is_active: bool,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = if is_active {
        // アクティブは強めの薄塗り (= 「今このピッカー / ドロワーが開いている」
        // をユーザに伝える)。 background.strong.color を alpha 0.4 程度で。
        Some(Background::Color(Color {
            a: 0.35,
            ..palette.background.strong.color
        }))
    } else {
        match status {
            iced::widget::button::Status::Hovered => {
                Some(Background::Color(palette.background.strong.color))
            }
            _ => None,
        }
    };
    iced::widget::button::Style {
        background: bg,
        text_color: palette.background.base.text,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}
