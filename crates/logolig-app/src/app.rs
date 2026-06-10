//! アプリケーションの中心 (§11)。
//!
//! - `AppState` を保持する
//! - `Message` を受けて状態遷移する
//! - `view` / `update` / `theme` / `subscription` を iced に接続する
//! - エラーは画面遷移ではなく snora の `Toast` (persistent) として表現する
//!   (§12「色だけに依存しない状態表現」「ABDD」に整合)
//! - 重い処理は `iced::Task` で逃がし UI スレッドをブロックしない (§2.4)

use std::path::PathBuf;
use std::time::Instant;

use iced::{Element, Subscription, Task, Theme};
use snora::{Toast, ToastIntent, ToastLifetime};

use logolig_core::{
    services::transparency_audit::{audit as audit_transparency, TransparencyReport},
    AppError, ExportPlan, MessageKey, PreviewCache, PreviewContext, PreviewProfile,
    ResizeAlgorithm, SettingsStore, SourceAsset, ThemeMode,
};
use logolig_i18n::{detect_system_locale, Locale, Translator};

// ----------------------------------------------------------------------
// 状態モデル
// ----------------------------------------------------------------------

/// 画面遷移ステート (§4.2)。
///
/// 旧設計の `Failed` は削除した。エラーは Toast で表現するため、
/// 画面状態は v1.16.0 で **3 状態に簡素化**:
///
/// - `Empty`: ファイル投入待ち。 中央にドロップゾーン。
/// - `Converting`: ファイル投入直後の自動変換中。 円形プログレス + メッセージ。
///   v1.15 までは `Importing` (decode + preview) と `Exporting` (write to disk)
///   の 2 状態に分かれていたが、 v1.16 は「投入したらメモリ上で全部変換」 の
///   一段モデルなので統合。
/// - `Result`: 変換完了。 アセットカード一覧 + 個別 DL + ZIP 一括 DL。
///
/// 旧 `Preview` 状態 (View as / Surface ピッカーで見え方を確認する画面) は
/// v1.16.0 で **廃止**。 新設計では「投入 → 即変換 → 結果確認」 の流れで、
/// プレビューは Result 画面の任意展開セクションとして残る (Q1 b)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Empty,
    Converting,
    Result,
}

/// アプリ全体の状態 (§4.1)。
#[derive(Debug)]
pub struct AppState {
    pub screen: Screen,
    pub theme: ThemeMode,
    pub advanced_open: bool,
    pub source_path: Option<PathBuf>,
    pub source_asset: Option<SourceAsset>,
    pub preview: Option<PreviewProfile>,
    /// プレビュー表示用にリサイズ済みのラスタキャッシュ (§5.2 コンテキストプレビュー用)。
    /// ソースを読み込んだ時、 algorithm が変わった時に再生成される。
    pub preview_cache: Option<PreviewCache>,
    pub export_plan: ExportPlan,
    pub busy: bool,
    /// snora の Toast キュー。エラー・成功通知はすべてここを経由する。
    pub toasts: Vec<Toast<Message>>,
    /// Toast の id を発行するためのカウンタ。
    pub next_toast_id: u64,

    // v1.3.0: 詳細設定でサイズを追加するためのテキストバッファ。
    // 「入力中」 のローカル状態は core ではなく UI 層が持つ責務。
    pub png_size_input: String,
    pub ico_size_input: String,

    // v1.4.0: 設定永続化ストア。 起動時に load_or_default() で AppState を初期化し、
    // ユーザ操作で設定が変わるたびに store.update() を呼ぶ即時保存戦略。
    //
    // - `Option<>` で持つのは初期化失敗時に「アプリは普通に動かしつつ、 永続化
    //   だけ無効」 を成立させるため。 例えば config dir が読み取り専用の場合
    //   などにアプリ自体が落ちないようにする。
    // - `locale` は v1.5 の i18n で使う伏線として PersistedSettings に含めて
    //   いる。 v1.4 では読み書きするだけで何にも反映しない。
    pub store: Option<crate::native_store::NativeStore>,

    // v1.5.0: i18n
    /// 現在の Translator。 ロケール切替時はこれを `Translator::for_locale(new)` で
    /// 入れ替えるだけで再描画 1 回で UI 全体が新言語になる。
    pub translator: Translator,
    /// ユーザによるロケール上書き。 `None` なら OS ロケール検出値を使う。
    /// 詳細設定の Language pick_list で変更され、 `PersistedSettings.locale`
    /// として保存される。
    pub locale_override: Option<Locale>,

    // v1.7.0: 透過チェッカー
    /// 直近に読み込んだ画像の透過状態。 `None` はまだ画像が読み込まれていないか、
    /// audit を実行していない状態。 ingest 完了時に audit を走らせて埋める。
    /// 警告 Toast の重複発行を防ぐため、 同じ画像で再度 audit しないように使う。
    pub transparency: Option<logolig_core::TransparencyReport>,
    // v1.10.0: `preview_checker: bool` は廃止 (PreviewContext::TransparencyChecker
    // バリアントに昇格)。 これにより「タブ風 + チェッカー」 のような無意味な
    // 同時 ON 状態を型レベルで排除。

    // v1.17.0: 旧 `advanced_groups` フィールド (AdvancedGroupExpansion) は削除。
    // 詳細設定ドロワーが flat 構造になったため、 アコーディオン展開状態の
    // 管理は不要になった。 唯一の折りたたみ「上級設定」 は
    // `advanced_extras_open` で管理する。

    // ---------------------------------------------------------------
    // v1.16.0: 新画面構造 (Empty / Converting / Result) 用の状態
    // ---------------------------------------------------------------
    /// 変換結果のアセット一式 (メモリ上保持)。
    ///
    /// `Screen::Result` の間だけ `Some(...)`。 ファイル投入時にクリアされ、
    /// 変換完了時に埋まる。 個別 DL ボタンや「ZIP 一括」 ボタンはこの値から
    /// バイト列を取り出して `rfd` でユーザに保存先を聞いて書き出す。
    ///
    /// v1.15 までは「変換 = ディスク書出」 だったので状態は不要だったが、
    /// v1.16 は「変換はメモリ完結、 保存はユーザ任意」 のため、 アプリ状態
    /// にアセット束を持つ必要が出た。
    pub result_assets: Option<crate::result::ResultAssets>,

    /// Result 画面のプレビューパネル (Browser tab / Phone home / Checker) の
    /// 開閉状態。 Q1 (b) の方針で「結果画面に小さく残す + 任意に開く」 を
    /// 実現するためのトグル。 デフォルト false (折りたたみ)。
    /// セッション内のみ保持、 永続化対象外。
    pub result_preview_open: bool,

    // ---------------------------------------------------------------
    // v1.17.0: 設定ドロワー Right Sheet 化用の状態
    // ---------------------------------------------------------------
    /// 現在のウィンドウサイズ (logical pixels)。
    ///
    /// Right Sheet の幅を `画面幅 / 3` をベースに clamp で `[280, 480]` に
    /// 抑える計算に使う。 これによりウィンドウサイズによらずドロワーが
    /// 「狭すぎてラベルが読めない」 「広すぎて中央コンテンツを圧迫する」 の
    /// 両極端を避けられる。
    ///
    /// 起動時の値は仮 (1280x720)、 `Message::WindowResized(size)` で更新。
    /// セッション内のみ保持、 永続化対象外。
    pub window_size: iced::Size<f32>,

    /// 詳細設定ドロワーの「上級設定」 折りたたみセクションの開閉状態。
    /// PNG モックに無い旧設定 (Apple touch / HTML snippet / Web manifest /
    /// Monochrome / リサイズアルゴリズム / vectorize_on_raster) をここに
    /// 集約する。 デフォルト false (折りたたみ)。
    /// セッション内のみ保持、 永続化対象外。
    pub advanced_extras_open: bool,

    // ---------------------------------------------------------------
    // v1.18.0: 左サイドバー + ピッカーポップアップ
    // ---------------------------------------------------------------
    /// 左サイドバーから出るピッカーポップアップの状態。
    ///
    /// PNG モック準拠で、 言語アイコン / テーマアイコンをクリックすると
    /// 選択肢のオーバーレイポップアップが出る (旧 cycle UI を廃止)。 同時に
    /// 1 種類しか開けない (snora の `context_menu` slot が単数のため)。
    pub active_picker: Option<SidebarPicker>,
}

/// v1.18.0: 左サイドバーから出るピッカーの種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarPicker {
    /// 言語選択ポップアップ (English / 日本語)。
    Locale,
    /// テーマ選択ポップアップ (System / Light / Dark)。
    Theme,
}

/// 詳細設定の 3 グループそれぞれの展開状態。
///
/// `Default` は「What to export のみ展開」 — 詳細ドロワーを初めて開いた時に
/// 必須項目だけが見えていて、 他は折りたたまれている状態。 ユーザは興味の
// v1.17.0: 旧 `AdvancedGroupExpansion` / `AdvancedGroup` は削除。 詳細設定
// ドロワーが flat 構造になり、 アコーディオン展開状態を管理する必要がなく
// なったため。 「上級設定」 折りたたみ 1 個のみ AppState の
// `advanced_extras_open` で管理する (v1.16.0 phase B で追加)。


impl AppState {
    /// boot 関数。 NativeStore で設定を load_or_default() し、 内容を AppState に反映。
    ///
    /// 失敗時はエラー Toast を出した上で default の AppState を返す。 永続化が
    /// 効かない状態でもアプリ自体は動くようにする (§ABDD: 機能が縮退しても止まらない)。
    ///
    /// ## v1.5.0: i18n 初期化
    ///
    /// 1. `PersistedSettings.locale` (BCP-47 風タグ "en" 等) があればそれを使う
    /// 2. なければ OS ロケールを `sys-locale` で検出
    /// 3. それも未対応なら英語 (Locale::default())
    ///
    /// 検出後の Translator が `state.translator` に入り、 各 view が
    /// `state.translator.t(MessageKey::AppTitle)` のように使う。
    pub fn boot() -> Self {
        let mut state = Self::default();
        let store = crate::native_store::NativeStore::new();
        let mut persisted_locale_tag: Option<String> = None;
        match store.load_or_default() {
            Ok(persisted) => {
                state.export_plan = persisted.export_plan;
                state.theme = persisted.theme;
                persisted_locale_tag = persisted.locale.clone();
                state.store = Some(store);
            }
            Err(err) => {
                // 永続化の初期化に失敗してもアプリは続行する。
                // ここではまだ Translator が default (英語) なので英語の Toast を出す。
                // 後で Translator が確定したら locale 反映の再描画で UI 全体が
                // 揃った言語になる (この Toast は次の dismiss/expire まで英語のまま)。
                let title = state
                    .translator
                    .t(MessageKey::ToastSettingsLoadFailedTitle);
                let body = state.translator.t_args(
                    MessageKey::ToastSettingsLoadFailedBody,
                    &[("error", &err.to_string())],
                );
                push_warning_toast(&mut state, &title, &body);
                // store は None のまま。 後続の persist_settings() は no-op になる。
            }
        }

        // ロケール解決:
        //   1. PersistedSettings.locale があれば優先 (B2: ユーザ上書き)
        //   2. なければ OS 検出
        //   3. どちらもダメなら英語 (Locale::default())
        let resolved_locale = persisted_locale_tag
            .as_deref()
            .and_then(Locale::from_bcp47)
            .map(|loc| (Some(loc), loc))
            .unwrap_or_else(|| (None, detect_system_locale()));
        state.locale_override = resolved_locale.0;
        state.translator = Translator::for_locale(resolved_locale.1);

        state
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::default(),
            theme: ThemeMode::default(),
            advanced_open: false,
            source_path: None,
            source_asset: None,
            preview: None,
            preview_cache: None,
            export_plan: ExportPlan::default(),
            busy: false,
            toasts: Vec::new(),
            next_toast_id: 0,
            png_size_input: String::new(),
            ico_size_input: String::new(),
            store: None,
            translator: Translator::default(),
            locale_override: None,
            transparency: None,
            // v1.16.0
            result_assets: None,
            result_preview_open: false,
            // v1.17.0
            window_size: iced::Size::new(1280.0, 720.0),
            advanced_extras_open: false,
            // v1.18.0
            active_picker: None,
        }
    }
}

// ----------------------------------------------------------------------
// メッセージ
// ----------------------------------------------------------------------

/// UI / サービス層が発火する全イベント (§4.3)。
///
/// `snora::AppLayout<Element, Message>` は `Message: Clone` を要求するため
/// 全バリアントが Clone 可能であること。
///
/// # 段階的開発について
/// Step 1 ではスケルトンのため、いくつかのバリアントはまだ
/// **構築箇所が無い** (`ExportCompleted` は出力完了)。
/// Step 4 で生成側を実装する。
/// 完成形のメッセージ集合を最初から見せるためここで宣言しておく。
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Message {
    // 入力
    FileDropped(PathBuf),
    PickFileRequested,
    FilePicked(Option<PathBuf>),

    // 読み込み
    IngestCompleted(Result<SourceAsset, AppError>),

    // プレビュー (Step 3)
    PreviewBuilt(Result<PreviewCache, AppError>),
    PreviewContextSelected(PreviewContext),
    PreviewBackgroundSelected(ThemeMode),

    // v1.16.0: 自動変換完了 (メモリ完結)。
    /// ファイル投入後の自動変換が終わり、 ResultAssets が組み上がった通知。
    /// 旧 `Message::ExportCompleted` (= ディスク書出し完了) とは別物で、
    /// こちらはディスクには触らずアプリ状態にアセット束を載せるだけ。
    /// ユーザがあとで個別 DL / ZIP DL を押したときに初めて書出が走る。
    ConvertCompleted(Result<crate::result::ResultAssets, logolig_core::AppError>),

    /// v1.16.0: 個別 DL ボタン押下。 アセット index を指定。
    DownloadOneRequested(usize),
    /// v1.16.0: ZIP 一括 DL ボタン押下。
    DownloadAllRequested,
    /// v1.16.0: 個別 DL のファイル保存先選択結果 (None = キャンセル)。
    DownloadOneTargetPicked(usize, Option<std::path::PathBuf>),
    /// v1.16.0: ZIP 一括 DL のファイル保存先選択結果 (None = キャンセル)。
    DownloadAllTargetPicked(Option<std::path::PathBuf>),
    /// v1.16.0: 個別 DL の書出完了通知。
    DownloadOneCompleted(Result<std::path::PathBuf, logolig_core::AppError>),
    /// v1.16.0: ZIP 一括 DL の書出完了通知。
    DownloadAllCompleted(Result<std::path::PathBuf, logolig_core::AppError>),

    /// v1.16.0: Result 画面の「プレビューを見る」 折りたたみセクションのトグル。
    ResultPreviewToggled,

    // v1.17.0: 詳細設定ドロワー Right Sheet 化
    /// ウィンドウサイズ変更通知。 Right Sheet の幅を再計算するために使う。
    WindowResized(iced::Size<f32>),
    /// 「上級設定」 折りたたみセクションのトグル (詳細設定ドロワー内)。
    AdvancedExtrasToggled,
    /// PNG プリセットサイズチェックボックスを ON にしたときの追加。
    /// 値は重複も範囲も検証済の前提 (16/32/48/96/192/512 のいずれか)。
    PngPresetSizeAdded(u32),
    /// no-op (UI 上で実装途中の placeholder トグル等で使う)。
    NoOp,

    // v1.18.0: 左サイドバー + ピッカーポップアップ
    /// サイドバーのピッカーアイコンをクリック → 該当ピッカーを開く。
    SidebarPickerOpened(SidebarPicker),
    /// ピッカー外をクリックなどで閉じる (snora の `on_close_menus` から発火)。
    SidebarPickerClosed,
    /// 言語ピッカーで選択肢を確定。 None は「OS のロケールに従う」 (auto)。
    LocalePicked(Option<Locale>),
    /// テーマピッカーで選択肢を確定。
    ThemePicked(ThemeMode),

    // テーマ・UI
    // v1.19.0: 旧 `ThemeToggled` (cycle UI 用) は削除済。 v1.18.0 で left
    // sidebar + ピッカーポップアップに移行した際に `ThemePicked(ThemeMode)`
    // が後継として導入された。
    AdvancedToggled,
    AlgorithmChanged(ResizeAlgorithm),
    /// v1.2.0: SVG 出力をオン/オフ
    IncludeSvgToggled(bool),
    /// v1.2.0: ラスタソースのベクトル化をオン/オフ
    VectorizeOnRasterToggled(bool),

    // v1.3.0: 詳細設定の編集 UI
    /// favicon.ico を出力するか
    IncludeIcoToggled(bool),
    /// apple-touch-icon.png を出力するか
    IncludeAppleTouchToggled(bool),
    /// favicon-snippet.html を出力するか
    IncludeHtmlSnippetToggled(bool),
    /// PNG サイズ集合の入力テキスト変更
    PngSizeInputChanged(String),
    /// PNG サイズ集合への追加実行 (Add ボタン or Enter)
    PngSizeAddRequested,
    /// PNG サイズ集合からの削除 (チップの × ボタン)
    PngSizeRemoveRequested(u32),
    /// ICO サイズ集合の入力テキスト変更
    IcoSizeInputChanged(String),
    /// ICO サイズ集合への追加実行
    IcoSizeAddRequested,
    /// ICO サイズ集合からの削除
    IcoSizeRemoveRequested(u32),

    // v1.4.1: vtracer プリセット切り替え + ExportPlan リセット
    /// vtracer プリセット選択の変更 (Sharp / Default / PhotoRich)。
    VtracerPresetChanged(logolig_core::VtracerPreset),
    /// ExportPlan を default に戻す。 theme / locale など他の設定には影響しない。
    ExportPlanResetRequested,

    // v1.5.0: i18n
    /// 言語選択。 `None` で OS デフォルトに戻す。
    LocaleChanged(Option<Locale>),

    // v1.19.0: 旧 `LocaleCycled` (ヘッダ言語アイコン cycle UI 用) と
    // `AppCloseRequested` (ヘッダ ✕ ボタン用) は削除済。
    // - `LocaleCycled` は v1.18.0 で left sidebar + 言語ピッカーポップアップ
    //   に置き換わり、 `LocalePicked(Option<Locale>)` が後継。
    // - `AppCloseRequested` は v1.18.0 で ✕ ボタン自体が完全撤去 (= OS の
    //   ネイティブウィンドウチャートに任せる方針) されたため不要。

    // v1.17.0: 旧 `AdvancedGroupToggled` Message は削除。 アコーディオン構造
    // 廃止に伴い不要 — 「上級設定」 折りたたみのトグルは
    // `AdvancedExtrasToggled` で代替。

    // v1.12.0: 編集画面の戻り動線
    /// 編集画面の「戻る」 / 「キャンセル」 ボタン。 startup 画面 (drop zone)
    /// に戻る。 ロード済みのソース・プレビューキャッシュを破棄する。
    /// ESC キーバインドは将来追加予定 (subscription 経由)。
    EditCancelled,

    // v1.10.0: PreviewCheckerToggled は削除 (Checker は
    // PreviewContextSelected(PreviewContext::TransparencyChecker) で
    // 表現される。 既存の context picker ロジックがそのまま使える)。

    // v1.8.0: Web manifest
    /// `manifest.webmanifest` 出力の有効/無効 toggle。 ON で
    /// `WebManifestSettings::default()` が `state.export_plan.web_manifest`
    /// に挿入される。
    IncludeWebManifestToggled(bool),
    /// `name` フィールドの編集。
    WebManifestNameChanged(String),
    /// `short_name` フィールドの編集。
    WebManifestShortNameChanged(String),
    /// `theme_color` フィールドの編集 (リアルタイムで state に反映、
    /// 検証は値確定時 — text_input::on_submit またはフォーカス外し時に行う)。
    WebManifestThemeColorChanged(String),
    /// `background_color` フィールドの編集。
    WebManifestBackgroundColorChanged(String),

    // v1.9.0: モノクローム出力
    /// `mono/` グレースケール出力セットの有効/無効 toggle。
    IncludeMonochromeToggled(bool),

    // v1.19.0: 旧 Export* Message 系は削除済。 v1.16.0 で
    // `ConvertCompleted` 経路に移行 (ファイル投入 → 自動変換 → Result 画面 →
    // 個別 DL or ZIP DL)、 旧の「Export ボタン → ディレクトリ選択 → 一括書出」
    // 動線は廃止された。 削除済 Message:
    // - `ExportRequested` (Export ボタン押下、 ボタン自体が無くなった)
    // - `ExportDirPicked(Option<PathBuf>)` (出力先ディレクトリ選択結果)
    // - `ExportCompleted(Result<ExportReport, AppError>)` (一括書出完了通知)
    // 個別 DL / ZIP DL の完了通知は `DownloadOneCompleted` /
    // `DownloadAllCompleted` で別途実装済 (v1.16.0)。

    // Toast ライフサイクル
    ToastTick,
    DismissToast(u64),

    // snora モーダル外クリック (BottomSheet/Dialog)
    CloseModals,
}

// ----------------------------------------------------------------------
// エントリポイント
// ----------------------------------------------------------------------

/// `main` から呼ばれる起動関数。
///
/// iced 0.14 の `application` は第 1 引数を **boot 関数** (`Fn() -> State`) として取る。
/// タイトルは builder メソッド `.title(...)` で渡す。
/// v1.5.0: タイトルもキー化 — `state.translator.t(MessageKey::AppTitle)` 経由で
/// ロケール変更にも追従する。
pub fn run() -> iced::Result {
    iced::application(AppState::boot, update, view)
        .title(window_title)
        .theme(theme)
        .subscription(subscription)
        .run()
}

fn window_title(state: &AppState) -> String {
    state.translator.t(MessageKey::AppTitle)
}

/// 現在の状態から iced の `Theme` を解決する。
///
/// v1.14.0: ui モジュールが theme palette ベースの色解決をするため、
/// `pub(crate)` で公開する (元は private fn だった)。
pub(crate) fn resolve_theme(state: &AppState) -> Theme {
    match state.theme {
        // System は Step 3 で OS テーマを覗くようにする。Step 1 では Light を採用。
        ThemeMode::System | ThemeMode::Light => Theme::Light,
        ThemeMode::Dark => Theme::Dark,
    }
}

/// iced の `application().theme()` から呼び出される旧シグネチャの薄いラッパ。
fn theme(state: &AppState) -> Theme {
    resolve_theme(state)
}

/// v1.20.0: モバイル/デスクトップ判定。
///
/// ウィンドウ幅が `MOBILE_BREAKPOINT_PX` (768px) 未満ならモバイル扱いとする。
/// 768px は Bootstrap の `md` ブレークポイントや旧 iPad mini portrait の
/// 横幅 (768×1024) など、 web アクセシビリティ業界で広く採用されている
/// 「タブレットの境界」 と一致するため、 ユーザの一般的なメンタルモデルに
/// 馴染む。
///
/// この判定値は UI の各所 (sidebar / bottom_nav の出し分け、 result_view の
/// グリッド列数、 advanced_drawer の幅、 ヘッダーパディング) で使う。
///
/// なお、 Window 起動直後 (まだ resize_events が来ていない時点) では
/// `AppState::window_size` がデフォルト 1280×720 になっているので、
/// 起動直後の判定はデスクトップ (false) になる。 リサイズイベントで遅延
/// 訂正される (実害なし — 表示が一瞬デスクトップで描画されてからモバイル
/// に切り替わるだけ)。
pub(crate) fn is_mobile(state: &AppState) -> bool {
    state.window_size.width < MOBILE_BREAKPOINT_PX
}

/// v1.20.0: モバイル/デスクトップ境界 (px)。 詳細は [`is_mobile`] のドキュメント
/// 参照。
pub(crate) const MOBILE_BREAKPOINT_PX: f32 = 768.0;

fn subscription(state: &AppState) -> Subscription<Message> {
    // v1.17.0: 3 つのサブスクリプションを結合する:
    //   (a) snora の Toast tick — transient toast の自動消滅
    //   (b) iced のウィンドウイベント — ファイルドロップを Message::FileDropped に変換
    //   (c) ウィンドウリサイズ — window_size を更新して Right Sheet 幅を再計算
    let toasts = snora::toast::subscription(&state.toasts, || Message::ToastTick);

    // (b) と (c) を 1 つの events stream から派生させる。 iced::window::events は
    // 全ウィンドウイベントを返すので、 ここで 2 種類に振り分ける。
    let window_evts = iced::window::events().filter_map(|(_id, ev)| match ev {
        iced::window::Event::FileDropped(path) => Some(Message::FileDropped(path)),
        iced::window::Event::Resized(size) => Some(Message::WindowResized(size)),
        // v1.17.0: 起動時にも window_size を取得したいので Opened にも反応する。
        // Opened は window::Size を含むため、 そのまま WindowResized として扱う。
        iced::window::Event::Opened { size, .. } => Some(Message::WindowResized(size)),
        _ => None,
    });

    Subscription::batch([toasts, window_evts])
}

fn view(state: &AppState) -> Element<'_, Message> {
    crate::shell::view(state)
}

// ----------------------------------------------------------------------
// update（状態遷移）
// ----------------------------------------------------------------------

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::FileDropped(path) | Message::FilePicked(Some(path)) => {
            start_ingest(state, path)
        }
        Message::FilePicked(None) => {
            // ピッカーをキャンセルされただけ。状態は変えない。
            Task::none()
        }
        Message::PickFileRequested => {
            // ネイティブファイルピッカーを Task として起動する (§12 代替経路)。
            // 結果は `Message::FilePicked(Option<PathBuf>)` として返る。
            // v1.12.0: 「再選択」 もこの Message を再利用する (現画面を保ったまま
            // ファイルピッカーを開き、 キャンセル時は元の編集画面のまま)。
            crate::task_queue::pick_file_task()
        }

        // v1.12.0: 編集画面の戻り動線。
        //
        // v1.16.0: 旧 Preview/ExportReady → 新 Result/Converting の状態どこから
        // でも Empty に戻す統一動線。 「← Back」 ボタンの実装。
        //
        // 永続化される設定 (export_plan / theme / locale 等) は保持。
        // 詳細ドロワーが開いていたら閉じる (UI 状態をニュートラルに揃える)。
        Message::EditCancelled => {
            state.source_asset = None;
            state.preview = None;
            state.preview_cache = None;
            state.transparency = None;
            state.result_assets = None;
            state.result_preview_open = false;
            state.screen = Screen::Empty;
            state.advanced_open = false;
            Task::none()
        }
        Message::IngestCompleted(Ok(asset)) => {
            // プレビュー生成タスクを起動するために asset を Arc で保持する。
            // SourceAsset 自体は Clone (Arc<[u8]> ベース) なので軽量。
            let arc = std::sync::Arc::new(asset.clone());
            state.source_asset = Some(asset);
            state.preview = Some(PreviewProfile::default());
            state.preview_cache = None;
            // v1.16.0: ingest 完了 → 自動変換へ進む。 旧 v1.15 では Preview 状態
            // でユーザの確認を待っていたが、 v1.16 は「投入したら自動変換」 の
            // モデル。 Converting 状態を維持したまま、 並行で:
            //   (a) preview build (Result 画面の折りたたみプレビュー用)
            //   (b) convert (メモリ完結変換、 ResultAssets を組む)
            // を走らせる。 (b) の完了で Screen::Result に遷移する。
            state.screen = Screen::Converting;
            state.busy = false;
            // v1.7.0: 新しい画像 → 透過状態は未確定に戻す。
            // PreviewBuilt で audit を走らせて確定する。 これにより:
            // - 再読み込み時に過去の警告が引き継がれない
            // - 同じ画像で警告 Toast が複数回出ないように制御できる
            state.transparency = None;
            // (a) と (b) を並行実行
            Task::batch([
                crate::task_queue::build_preview_task(arc.clone(), state.export_plan.algorithm),
                crate::task_queue::convert_task(arc, state.export_plan.clone()),
            ])
        }
        Message::IngestCompleted(Err(err)) => fail(state, err),

        Message::PreviewBuilt(Ok(cache)) => {
            // 受け取った cache が「現在の状態」 と整合しているか確認する。
            // 古いソース・古い algorithm のキャッシュなら破棄。
            let asset_path = state.source_asset.as_ref().map(|a| a.path.clone());
            let still_valid = asset_path.as_ref() == Some(&cache.source_path)
                && cache.algorithm == state.export_plan.algorithm;
            if still_valid {
                // v1.7.0: 透過チェッカー
                // この cache が「この画像で初めて」 構築されたなら audit を走らせる。
                // 既に audit 済 (state.transparency が Some) なら警告を再表示しない
                // — 例えば algorithm 変更で preview が再構築された場合に Toast が
                // 何度も出ないように。 audit 自体は idempotent なのでバグはないが、
                // UX として 1 画像 1 警告にする。
                let needs_audit = state.transparency.is_none();
                if needs_audit {
                    let report = audit_transparency(&cache.icon_120);
                    state.transparency = Some(report);
                    if report.needs_warning() {
                        // v1.11.0: JPEG 入力は形式上 alpha を持てないので
                        // `FullyOpaque` 判定が必ず出るが、 ユーザの「やり方が
                        // 間違っている」 のではなく「JPEG という形式の制約」 で
                        // ある。 通常の透過警告ではなく JPEG 専用の教育的警告
                        // に振り替える。 source_kind を見て分岐:
                        let is_jpeg = state
                            .source_asset
                            .as_ref()
                            .map(|a| a.kind == logolig_core::SourceKind::Jpeg)
                            .unwrap_or(false);
                        if is_jpeg {
                            push_jpeg_input_warning(state);
                        } else {
                            push_transparency_warning(state, report);
                        }
                    }
                }
                state.preview_cache = Some(cache);
            }
            Task::none()
        }
        Message::PreviewBuilt(Err(err)) => {
            // プレビュー生成失敗は致命的ではない (元データは健在)。
            // 画面遷移はせず、 Toast でだけ知らせる。
            push_error_toast(state, err);
            Task::none()
        }
        Message::PreviewContextSelected(ctx) => {
            if let Some(p) = state.preview.as_mut() {
                p.context = ctx;
            }
            Task::none()
        }
        Message::PreviewBackgroundSelected(theme) => {
            if let Some(p) = state.preview.as_mut() {
                p.background = theme;
            }
            Task::none()
        }

        // v1.16.0: 自動変換完了
        Message::ConvertCompleted(Ok(assets)) => {
            state.result_assets = Some(assets);
            state.screen = Screen::Result;
            state.busy = false;
            Task::none()
        }
        Message::ConvertCompleted(Err(err)) => fail(state, err),

        // v1.16.0: DL ボタン群
        Message::DownloadOneRequested(idx) => {
            // ファイル名を取り出して、 デフォルト拡張子付きの保存ダイアログを開く
            let Some(assets) = state.result_assets.as_ref() else {
                return Task::none();
            };
            let Some(item) = assets.items.get(idx) else {
                return Task::none();
            };
            crate::task_queue::pick_save_one_task(idx, item.file_name.clone())
        }
        Message::DownloadAllRequested => {
            crate::task_queue::pick_save_all_task()
        }
        Message::DownloadOneTargetPicked(_, None) => Task::none(),
        Message::DownloadOneTargetPicked(idx, Some(path)) => {
            let Some(assets) = state.result_assets.as_ref() else {
                return Task::none();
            };
            let Some(item) = assets.items.get(idx) else {
                return Task::none();
            };
            let bytes = item.bytes.clone();
            crate::task_queue::write_one_task(path, bytes)
        }
        Message::DownloadAllTargetPicked(None) => Task::none(),
        Message::DownloadAllTargetPicked(Some(path)) => {
            let Some(assets) = state.result_assets.as_ref() else {
                return Task::none();
            };
            // ZIP 化はメインスレッドの邪魔にならないよう task に逃がす。
            let items_clone = assets.items.clone();
            crate::task_queue::write_zip_task(path, items_clone)
        }
        Message::DownloadOneCompleted(Ok(path)) => {
            let title = state.translator.t(MessageKey::ToastExportTitle);
            let body = format!("{}", path.display());
            push_long_success_toast(state, &title, &body);
            Task::none()
        }
        Message::DownloadOneCompleted(Err(err)) => {
            push_error_toast(state, err);
            Task::none()
        }
        Message::DownloadAllCompleted(Ok(path)) => {
            let title = state.translator.t(MessageKey::ToastExportTitle);
            let body = format!("{}", path.display());
            push_long_success_toast(state, &title, &body);
            Task::none()
        }
        Message::DownloadAllCompleted(Err(err)) => {
            push_error_toast(state, err);
            Task::none()
        }

        // v1.16.0: Result 画面のプレビュー折りたたみトグル
        Message::ResultPreviewToggled => {
            state.result_preview_open = !state.result_preview_open;
            Task::none()
        }

        // v1.17.0: ウィンドウリサイズ通知 → window_size を更新
        Message::WindowResized(size) => {
            state.window_size = size;
            Task::none()
        }
        // v1.17.0: 上級設定セクションの開閉
        Message::AdvancedExtrasToggled => {
            state.advanced_extras_open = !state.advanced_extras_open;
            Task::none()
        }
        // v1.17.0: PNG プリセットサイズの ON/OFF (チェックボックス)
        Message::PngPresetSizeAdded(size) => {
            if state.export_plan.add_png_size(size) {
                persist_settings(state);
            }
            Task::none()
        }
        // v1.17.0: placeholder トグル (現状は内部状態を持たないので何もしない)
        Message::NoOp => Task::none(),

        // v1.18.0: 左サイドバーの言語/テーマピッカー
        //
        // クリックで `active_picker` をセット。 既に同じピッカーが開いていれば
        // トグルで閉じる (連打挙動)。 別のピッカーが開いていれば即座に切替
        // (= 1 種類しか開けない、 snora の `context_menu` slot 単数性に対応)。
        Message::SidebarPickerOpened(picker) => {
            state.active_picker = if state.active_picker == Some(picker) {
                None
            } else {
                Some(picker)
            };
            Task::none()
        }
        Message::SidebarPickerClosed => {
            state.active_picker = None;
            Task::none()
        }
        Message::LocalePicked(opt) => {
            state.locale_override = opt;
            let resolved = opt.unwrap_or_else(detect_system_locale);
            state.translator = Translator::for_locale(resolved);
            state.active_picker = None;
            persist_settings(state);
            Task::none()
        }
        Message::ThemePicked(theme) => {
            state.theme = theme;
            state.active_picker = None;
            persist_settings(state);
            Task::none()
        }

        // v1.19.0: 旧 `Message::ThemeToggled` ハンドラは削除済 (上記 Message
        // 列挙宣言箇所のコメント参照)。 後継は `ThemePicked(ThemeMode)`。

        Message::AdvancedToggled => {
            // advanced_open は永続化対象外 (UI 状態)。 保存しない。
            state.advanced_open = !state.advanced_open;
            Task::none()
        }
        Message::AlgorithmChanged(alg) => {
            state.export_plan.algorithm = alg;
            persist_settings(state);
            // algorithm 変更を反映するためプレビュー再生成。
            // 既存キャッシュは古いので破棄する (UI は cache=None のあいだ
            // ローディング表示にフォールバックする)。
            state.preview_cache = None;
            if let Some(asset) = state.source_asset.as_ref() {
                let arc = std::sync::Arc::new(asset.clone());
                crate::task_queue::build_preview_task(arc, alg)
            } else {
                Task::none()
            }
        }
        Message::IncludeSvgToggled(on) => {
            // 出力プランの変更だけ。 プレビュー (16×16 / 120×120) には影響しない。
            state.export_plan.include_svg = on;
            persist_settings(state);
            Task::none()
        }
        Message::VectorizeOnRasterToggled(on) => {
            state.export_plan.vectorize_on_raster = on;
            persist_settings(state);
            Task::none()
        }

        // -----------------------------------------------------------------
        // v1.3.0: 詳細設定の編集 UI ハンドラ
        // -----------------------------------------------------------------
        Message::IncludeIcoToggled(on) => {
            state.export_plan.include_ico = on;
            persist_settings(state);
            Task::none()
        }
        Message::IncludeAppleTouchToggled(on) => {
            state.export_plan.include_apple_touch = on;
            persist_settings(state);
            Task::none()
        }
        Message::IncludeHtmlSnippetToggled(on) => {
            state.export_plan.include_html_snippet = on;
            persist_settings(state);
            Task::none()
        }
        Message::PngSizeInputChanged(s) => {
            // 数字のみに簡易フィルタ。 全文置換は許容するが、 数字以外は無視
            // (ペースト時のスペースなどを許容するため文字種フィルタは後で trim 時)。
            state.png_size_input = s;
            Task::none()
        }
        Message::PngSizeAddRequested => {
            let raw = state.png_size_input.trim().to_string();
            match parse_size(&raw, logolig_core::PNG_SIZE_MIN, logolig_core::PNG_SIZE_MAX) {
                Ok(size) => {
                    if state.export_plan.add_png_size(size) {
                        state.png_size_input.clear();
                        persist_settings(state);
                    } else {
                        // 範囲外は parse_size で捕捉済みなので、 ここに来るのは重複のみ
                        let title = state.translator.t(MessageKey::ToastSizeAlreadyInSetTitle);
                        let body = state.translator.t_args(
                            MessageKey::ToastPngSizeAlreadyInSetBody,
                            &[("size", &size.to_string())],
                        );
                        push_warning_toast(state, &title, &body);
                    }
                }
                Err(SizeParseError::Empty) => {
                    // 空入力での Add は無視 (UX: Enter 連打で迷子にしない)
                }
                Err(SizeParseError::NotANumber) => {
                    let title = state.translator.t(MessageKey::ToastInvalidSizeTitle);
                    let body = state
                        .translator
                        .t_args(MessageKey::ToastInvalidSizeBody, &[("input", &raw)]);
                    push_warning_toast(state, &title, &body);
                }
                Err(SizeParseError::OutOfRange { min, max }) => {
                    let title = state.translator.t(MessageKey::ToastSizeOutOfRangeTitle);
                    let body = state.translator.t_args(
                        MessageKey::ToastPngSizeOutOfRangeBody,
                        &[("min", &min.to_string()), ("max", &max.to_string())],
                    );
                    push_warning_toast(state, &title, &body);
                }
            }
            Task::none()
        }
        Message::PngSizeRemoveRequested(size) => {
            if state.export_plan.remove_png_size(size) {
                persist_settings(state);
            }
            Task::none()
        }
        Message::IcoSizeInputChanged(s) => {
            state.ico_size_input = s;
            Task::none()
        }
        Message::IcoSizeAddRequested => {
            let raw = state.ico_size_input.trim().to_string();
            match parse_size(&raw, logolig_core::ICO_SIZE_MIN, logolig_core::ICO_SIZE_MAX) {
                Ok(size) => {
                    if state.export_plan.add_ico_size(size) {
                        state.ico_size_input.clear();
                        persist_settings(state);
                    } else {
                        let title = state.translator.t(MessageKey::ToastSizeAlreadyInSetTitle);
                        let body = state.translator.t_args(
                            MessageKey::ToastIcoSizeAlreadyInSetBody,
                            &[("size", &size.to_string())],
                        );
                        push_warning_toast(state, &title, &body);
                    }
                }
                Err(SizeParseError::Empty) => {}
                Err(SizeParseError::NotANumber) => {
                    let title = state.translator.t(MessageKey::ToastInvalidSizeTitle);
                    let body = state
                        .translator
                        .t_args(MessageKey::ToastInvalidSizeBody, &[("input", &raw)]);
                    push_warning_toast(state, &title, &body);
                }
                Err(SizeParseError::OutOfRange { min, max }) => {
                    let title = state.translator.t(MessageKey::ToastSizeOutOfRangeTitle);
                    let body = state.translator.t_args(
                        MessageKey::ToastIcoSizeOutOfRangeBody,
                        &[("min", &min.to_string()), ("max", &max.to_string())],
                    );
                    push_warning_toast(state, &title, &body);
                }
            }
            Task::none()
        }
        Message::IcoSizeRemoveRequested(size) => {
            if state.export_plan.remove_ico_size(size) {
                persist_settings(state);
            }
            Task::none()
        }

        // -----------------------------------------------------------------
        // v1.4.1: vtracer プリセット + ExportPlan リセット
        // -----------------------------------------------------------------
        Message::VtracerPresetChanged(preset) => {
            state.export_plan.vtracer_preset = preset;
            persist_settings(state);
            Task::none()
        }
        Message::ExportPlanResetRequested => {
            // ExportPlan のみ default に戻す。 theme / locale / advanced_open など
            // 他の状態は触らない (§v1.4.1 Reset スコープ判断)。
            state.export_plan = ExportPlan::default();
            // 入力中の文字列バッファもクリア (リセット直後に変な値が残らないように)
            state.png_size_input.clear();
            state.ico_size_input.clear();
            // プレビューのアルゴリズムが変わった可能性 (Reset 時に Lanczos3 に戻る)。
            // キャッシュは破棄して再生成しないと色合わせが古いまま。
            state.preview_cache = None;
            persist_settings(state);
            // 完了通知 (transient): UX として「ボタン押した → 何が起きた」 を明示
            let title = state.translator.t(MessageKey::ToastResetTitle);
            let body = state.translator.t(MessageKey::ToastResetBody);
            push_success_toast(state, &title, &body);
            // プレビューがあるなら algorithm 変更後と同じ要領で再生成
            if let Some(asset) = state.source_asset.as_ref() {
                let arc = std::sync::Arc::new(asset.clone());
                crate::task_queue::build_preview_task(arc, state.export_plan.algorithm)
            } else {
                Task::none()
            }
        }

        // -----------------------------------------------------------------
        // v1.5.0: ロケール変更
        // -----------------------------------------------------------------
        Message::LocaleChanged(opt) => {
            // None ならシステムロケールに戻す。 Some(loc) ならそれを採用。
            state.locale_override = opt;
            let resolved = opt.unwrap_or_else(detect_system_locale);
            state.translator = Translator::for_locale(resolved);
            persist_settings(state);
            Task::none()
        }

        // v1.19.0: 旧 `Message::LocaleCycled` / `Message::AppCloseRequested`
        // ハンドラは削除済 (上記 Message 列挙宣言箇所のコメント参照)。
        // 後継:
        // - LocaleCycled → `LocalePicked(Option<Locale>)` (left sidebar の
        //   言語ピッカーポップアップで直接選択)
        // - AppCloseRequested → ✕ ボタン廃止により不要 (OS ネイティブに任せる)

        // -----------------------------------------------------------------
        // v1.10.3 → v1.17.0: 旧アコーディオンハンドラは削除済 (上記参照)。
        // -----------------------------------------------------------------

        // -----------------------------------------------------------------
        // v1.7.0 → v1.10.0: 透過チェッカーの実装は PreviewContextSelected に統合。
        // 専用 Message は廃止。
        // -----------------------------------------------------------------

        // -----------------------------------------------------------------
        // v1.8.0: Web manifest 設定
        // -----------------------------------------------------------------
        Message::IncludeWebManifestToggled(on) => {
            // ON → デフォルト値で構造体を挿入。 OFF → None に戻す。
            // ユーザが入力した値があっても OFF 時は破棄する (シンプルな状態遷移)。
            // 必要なら v1.8.x で「OFF にしてもメモリ上の値は保持」 を検討。
            state.export_plan.web_manifest = if on {
                Some(logolig_core::WebManifestSettings::default())
            } else {
                None
            };
            persist_settings(state);
            Task::none()
        }
        Message::WebManifestNameChanged(s) => {
            if let Some(m) = state.export_plan.web_manifest.as_mut() {
                m.name = s;
                persist_settings(state);
            }
            Task::none()
        }
        Message::WebManifestShortNameChanged(s) => {
            if let Some(m) = state.export_plan.web_manifest.as_mut() {
                m.short_name = s;
                persist_settings(state);
            }
            Task::none()
        }
        Message::WebManifestThemeColorChanged(s) => {
            // 入力中の検証は行わない (ユーザがタイプしている途中で警告を出すと UX が悪い)。
            // `is_valid_color` 検証は実行時 export まで遅延する — この時点では
            // ユーザが #FF や #FFFFF のような途中状態でも自由に編集できる。
            if let Some(m) = state.export_plan.web_manifest.as_mut() {
                m.theme_color = s;
                persist_settings(state);
            }
            Task::none()
        }
        Message::WebManifestBackgroundColorChanged(s) => {
            if let Some(m) = state.export_plan.web_manifest.as_mut() {
                m.background_color = s;
                persist_settings(state);
            }
            Task::none()
        }

        // -----------------------------------------------------------------
        // v1.9.0: モノクローム出力セット
        // -----------------------------------------------------------------
        Message::IncludeMonochromeToggled(on) => {
            // 単純な bool フラグ操作。 既存の include_ico / include_apple_touch
            // などと同じ流儀で persist_settings まで行う。
            //
            // 既存のプレビューや preview_cache に副作用なし — モノクロームは
            // 出力時にのみ生成され、 プレビュー画面の表示には影響しない
            // (プレビューでの mono 表示は v1.11 IA 刷新と合わせて検討)。
            state.export_plan.monochrome = on;
            persist_settings(state);
            Task::none()
        }

        // v1.19.0: 旧 `Message::ExportRequested` / `ExportDirPicked` /
        // `ExportCompleted` ハンドラは削除済 (上記 Message 列挙宣言箇所の
        // コメント参照)。 v1.16.0 の `convert_task` 経路 + 個別 DL / ZIP DL に
        // 完全移行された。

        Message::ToastTick => {
            snora::toast::sweep_expired(&mut state.toasts, Instant::now());
            Task::none()
        }
        Message::DismissToast(id) => {
            state.toasts.retain(|t| t.id != id);
            Task::none()
        }
        Message::CloseModals => {
            state.advanced_open = false;
            Task::none()
        }
    }
}

/// ファイルパスを受け取って ingest タスクを起動する共通処理。
///
/// v1.16.0: 旧 `Screen::Importing` は v1.16 で `Screen::Converting` に統合。
/// ingest → preview build → asset bundle build までの全フェーズが Converting
/// 状態にまとまる。
fn start_ingest(state: &mut AppState, path: PathBuf) -> Task<Message> {
    state.source_path = Some(path.clone());
    state.screen = Screen::Converting;
    state.busy = true;
    // 古いソースのプレビューキャッシュとアセット束は破棄。 新しい ingest
    // 完了で再生成される。
    state.preview_cache = None;
    state.result_assets = None;
    state.result_preview_open = false;
    crate::task_queue::ingest_task(path)
}

/// エラー発生時の共通遷移。
///
/// - busy フラグを下ろす
/// - 画面は「ソースがあれば Result、なければ Empty」に戻す
///   (v1.16: 旧 Preview 状態は廃止されたため、 ソース有 → Result に直す)
/// - Persistent な Error toast を積む（読まないと消えない）
fn fail(state: &mut AppState, err: AppError) -> Task<Message> {
    state.busy = false;
    state.screen = if state.result_assets.is_some() {
        Screen::Result
    } else {
        Screen::Empty
    };
    push_error_toast(state, err);
    Task::none()
}

/// エラーを persistent な Toast として通知。ユーザーが閉じるまで残る。
/// v1.5.0: AppError の `key()` + `args()` を Translator で翻訳する。
fn push_error_toast(state: &mut AppState, err: AppError) {
    let id = next_id(state);
    let body = state.translator.translate_error(&err);
    // タイトルは「操作失敗」 のような汎用文言。 これも翻訳キー化したいが、
    // v1.5.0 では「Operation failed」 を英語で固定しておく。 v1.6 以降で
    // 必要なら ToastOperationFailedTitle のような MessageKey を増やす。
    state.toasts.push(
        Toast::new(
            id,
            ToastIntent::Error,
            "Operation failed",
            body,
            Message::DismissToast(id),
        )
        .persistent(),
    );
}

/// 成功通知。デフォルトの transient lifetime で自動消滅する。
fn push_success_toast(state: &mut AppState, title: &str, body: &str) {
    let id = next_id(state);
    state.toasts.push(Toast::new(
        id,
        ToastIntent::Success,
        title.to_string(),
        body.to_string(),
        Message::DismissToast(id),
    ));
}

/// 長め (7 秒) の transient 成功通知 (v1.4.2)。
///
/// v1.4.1 では Export 完了を `persistent()` にしてユーザの dismiss 待ちにして
/// いたが、 ユーザフィードバックで「persistent は不要、 ただし default の 4 秒
/// では短い」 と判明。 v1.4.2 では 7 秒の transient に変更。
///
/// 7 秒の根拠: snora の default 4 秒は短いメッセージ向け。 Export 通知は
/// 「N files written to /長い/path/to/dir」 のように読み切るのに時間が要るので、
/// その分長めにする。 ただし 10 秒以上は「画面に居座っている」 印象になるため
/// 避ける。 7 秒は「読んで頷いて目を逸らす」 までの時間として妥当な落とし所。
///
/// Toast 表示位置の問題 (snora が右下固定) は別途 ROADMAP で snora 拡張依頼として
/// 扱う。 位置改善が入れば 7 秒でも見落としリスクはさらに下がる見込み。
fn push_long_success_toast(state: &mut AppState, title: &str, body: &str) {
    let id = next_id(state);
    state.toasts.push(
        Toast::new(
            id,
            ToastIntent::Success,
            title.to_string(),
            body.to_string(),
            Message::DismissToast(id),
        )
        .with_lifetime(ToastLifetime::seconds(7)),
    );
}

fn next_id(state: &mut AppState) -> u64 {
    let id = state.next_toast_id;
    state.next_toast_id += 1;
    id
}

// ----------------------------------------------------------------------
// v1.3.0: サイズ入力のパース + Warning トースト
// ----------------------------------------------------------------------

#[derive(Debug)]
enum SizeParseError {
    Empty,
    NotANumber,
    OutOfRange { min: u32, max: u32 },
}

/// 数字文字列を u32 に変換し、 範囲チェックも行う。
/// `add_*_size` も内部で範囲を弾くが、 ここで弾くことでユーザに具体的な
/// エラー文言 (range / not a number / 重複) を出し分けられる。
fn parse_size(raw: &str, min: u32, max: u32) -> Result<u32, SizeParseError> {
    if raw.is_empty() {
        return Err(SizeParseError::Empty);
    }
    let n: u32 = raw.parse().map_err(|_| SizeParseError::NotANumber)?;
    if n < min || n > max {
        return Err(SizeParseError::OutOfRange { min, max });
    }
    Ok(n)
}

/// Warning 用 Toast (transient)。 入力検証失敗を伝える。
/// 既存の push_error_toast (persistent) と push_success_toast の中間。
fn push_warning_toast(state: &mut AppState, title: &str, body: &str) {
    let id = next_id(state);
    state.toasts.push(Toast::new(
        id,
        ToastIntent::Warning,
        title.to_string(),
        body.to_string(),
        Message::DismissToast(id),
    ));
}

/// v1.7.0: 透過チェッカー警告。 完全不透明 / 完全透明な画像が読み込まれた時に
/// 表示する Warning Toast。 通常の入力検証 Toast (transient 4 秒) より少し長め
/// にしたいところだが、 v1.7 では既存の transient ライフタイムをそのまま使う
/// (ユーザに「重大ではない注意」 と伝える意図)。
fn push_transparency_warning(state: &mut AppState, report: TransparencyReport) {
    let (title_key, body_key) = match report {
        TransparencyReport::FullyOpaque => (
            MessageKey::ToastFullyOpaqueTitle,
            MessageKey::ToastFullyOpaqueBody,
        ),
        TransparencyReport::FullyTransparent => (
            MessageKey::ToastFullyTransparentTitle,
            MessageKey::ToastFullyTransparentBody,
        ),
        TransparencyReport::HasTransparency => {
            // needs_warning() == false なので呼ばれないはずだが、 防御的に no-op
            return;
        }
    };
    let title = state.translator.t(title_key);
    let body = state.translator.t(body_key);
    push_warning_toast(state, &title, &body);
}

/// v1.11.0: JPEG 入力時の教育的警告。
///
/// JPEG 形式は alpha チャネルを持てないため、 v1.7 の transparency audit は
/// 必ず `FullyOpaque` を返す。 一般の `FullyOpaque` 警告 (PNG で背景を切り
/// 抜き忘れた等) と異なり、 JPEG では「形式の制約」 が原因なので、 専用の
/// 教育的トーンの文言で「PNG にすると favicon に適する」 と伝える。
fn push_jpeg_input_warning(state: &mut AppState) {
    let title = state.translator.t(MessageKey::ToastJpegInputTitle);
    let body = state.translator.t(MessageKey::ToastJpegInputBody);
    push_warning_toast(state, &title, &body);
}

// ----------------------------------------------------------------------
// v1.4.0: 設定永続化
// ----------------------------------------------------------------------

/// 現在の AppState に対応する `PersistedSettings` を組み立てる。
fn snapshot_persisted(state: &AppState) -> logolig_core::PersistedSettings {
    logolig_core::PersistedSettings {
        export_plan: state.export_plan.clone(),
        theme: state.theme,
        // v1.5.0: ユーザによるロケール上書きがあれば BCP-47 タグとして保存。
        // None なら次回起動時も OS ロケール検出にフォールバックする。
        locale: state.locale_override.map(|loc| loc.as_bcp47().to_string()),
    }
}

/// 設定を即時保存する (即時保存戦略, §1.4.0)。
///
/// `state.store` が `None` の場合 (= 起動時に永続化初期化に失敗) は no-op。
/// 保存失敗時はエラー Toast を出すが、 アプリ自体は続行する。
///
/// 注意: 即時保存はユーザ操作のたびに `update()` を呼ぶ。 現状の
/// `PersistedSettings` は数 KB 以下で I/O コストが無視できるが、 将来データが
/// 肥大化した時は debounce / lazy save に切り替える必要がある。
fn persist_settings(state: &mut AppState) {
    let Some(store) = state.store.as_ref() else {
        return;
    };
    let snapshot = snapshot_persisted(state);
    if let Err(err) = store.save(&snapshot) {
        // 保存失敗を transient warning として通知 (毎操作 persistent では UI が埋まる)。
        // v1.5.0: i18n 対応。
        let title = state
            .translator
            .t(MessageKey::ToastSettingsSaveFailedTitle);
        let body = state.translator.t_args(
            MessageKey::ToastSettingsSaveFailedBody,
            &[("error", &err.to_string())],
        );
        push_warning_toast(state, &title, &body);
    }
}
