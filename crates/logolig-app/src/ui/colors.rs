//! テーマ連動の色ヘルパ (v1.14.0)。
//!
//! ## なぜ関数で提供するか
//!
//! v1.13 までは UI 各所で `const FOO_COLOR: Color = Color::from_rgb(0.55, ...)`
//! のように **light テーマ向けの固定値** を直書きしていた。 これを ダーク
//! テーマで描画すると:
//!
//! - 「アプリ名 = 控えめな濃いグレー」 → 暗い背景に同化して読めない
//! - 「タグライン = 薄いグレー」 → 暗い背景の上で逆に「白に近すぎる」
//! - 「カード境界 = 薄いグレー」 → 背景にめり込む
//!
//! いずれも「役割としては合っている (主役を立てる、 補助情報、 区切り線)」 が、
//! **実際の色が役割を果たすかは背景に依存** する。 役割は theme palette を
//! 通して解決する。
//!
//! ## API スタイル
//!
//! 各役割を関数で提供する。 view 関数は呼び出し時の `&Theme` を渡す。 これで
//! テーマ切替時に全色が自動的に更新される (iced 側が再描画してくれるため、
//! 追加の subscribe は不要)。
//!
//! ## 「テーマで変える色」 vs 「意図的に hardcoded な色」
//!
//! このモジュールは **テーマで変える色** だけを提供する。 以下は意図的に
//! hardcoded として残してある (このモジュールでは扱わない):
//!
//! - **透過チェッカーの市松グレー** (`#E6E6E6` / `#C0C0C0`):
//!   透明度の有無を示す指標そのもの。 テーマで変えると「テーマ変えただけで
//!   透明度の見え方が変わる」 という誤解を招く。
//! - **ブラウザタブ / スマホ枠 / 壁紙の色** (`chrome_bg_for` 他、 `preview_panel.rs` 内):
//!   `PreviewProfile::background` (Surface ピッカー) で切り替える対象であり、
//!   アプリのテーマと独立した軸。 アプリテーマと連動すると「アプリは Dark、
//!   Surface も Dark」 のとき二重支配になり混乱する。
//!
//! 上記 2 軸は v1.14.0 のリファクタ対象外。

use iced::{Color, Theme};

// ---------------------------------------------------------------------------
// 主要テキスト色 (役割別)
// ---------------------------------------------------------------------------

/// アプリ名 (startup ヘッダ) の文字色。
///
/// 「主役を引き立てる」 (=画像が主役) ため、 通常テキストよりやや弱め。 ただし
/// 「アプリ名であることが分かる」 程度の視認性は維持する。 役割としては
/// 「セカンダリテキスト」 で、 palette の `background.weak.text` を採用する
/// (背景と程よくコントラストし、 主役ではない)。
pub fn app_name(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    p.background.weak.text
}

/// タグライン (アプリ名の隣に添える説明文) の色。
///
/// アプリ名よりさらに弱い「ヒントレベル」 のテキスト。 palette には
/// 「もっと弱いテキスト色」 のスロットがないため、 `app_name` と同じ
/// `background.weak.text` を起点に **alpha=0.65** で薄める。 light でも dark
/// でもベース色からの相対的な弱化が保たれる。
pub fn tagline(theme: &Theme) -> Color {
    with_alpha(app_name(theme), 0.65)
}

/// 編集画面でヘッダ左に表示するファイル名の色。
///
/// 「画面の主役 = 今扱っているファイル」 のため、 アプリ名より目立つ。
/// 通常の本文テキスト相当 (`background.base.text`) で十分強調できる。
pub fn file_name(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    p.background.base.text
}

/// 画面ページタイトル (例:「プレビュー確認・Favicon ファイル作成」)。
///
/// 編集画面の最上位見出し。 通常本文と同等の濃さで、 サイズで目立たせる
/// (色で目立たせると重くなりすぎる)。
pub fn page_title(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    p.background.base.text
}

/// セクションタイトル (プレビューカードの「Preview」 等)。
///
/// 「カードの中の見出し」 としてセカンダリ位置。 base と weak の中間が望ましい
/// が、 palette には中間スロットがないため weak を採用 (アプリ名と同じ階層)。
pub fn section_label(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    p.background.weak.text
}

/// 詳細ドロワーの大グループ見出し (アコーディオンヘッダ)。
///
/// クリック可能なボタンとして見せたいので、 通常テキストに近い濃さ。
pub fn group_heading(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    p.background.base.text
}

/// blurb / 補足説明 / 「at defaults: ...」 のような controlled 注釈。
///
/// 主情報の後ろに添えるレベル。 タグラインと同じくらい弱く、 でも本文として
/// 読める程度。
pub fn muted_text(theme: &Theme) -> Color {
    with_alpha(theme.extended_palette().background.base.text, 0.6)
}

/// startup 画面 (drop zone) のヘッドライン文字色。
///
/// 「Drop PNG, SVG, WebP, or JPEG」 のメインメッセージ。 主役なので濃い。
pub fn drop_zone_headline(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    p.background.base.text
}

// ---------------------------------------------------------------------------
// 背景・枠線色
// ---------------------------------------------------------------------------
//
// プレビューカードとドロップゾーンカードの背景/枠線は、 description が長く
// なる container::style クロージャ内で `theme.extended_palette()` を直接参照
// している (closure 自体が theme-aware なので二重ラップは不要)。
//
// v1.17.0: 旧 `badge_muted_bg` ヘルパは削除 (size_subsection の「at defaults」
// バッジが廃止されて未使用になったため)。 必要になれば 3 行で復活可能。


// ---------------------------------------------------------------------------
// ヘルパ
// ---------------------------------------------------------------------------

/// 既存の Color に alpha を上書きしたコピーを返す。 v1.14.0 で「弱化」 を表す
/// ために多用するヘルパ。
fn with_alpha(c: Color, alpha: f32) -> Color {
    Color { a: alpha, ..c }
}
