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
use snora::{Toast, ToastIntent};

use logolig_core::{
    AppError, ExportPlan, ExportReport, PreviewCache, PreviewContext, PreviewProfile,
    ResizeAlgorithm, SourceAsset, ThemeMode,
};

// ----------------------------------------------------------------------
// 状態モデル
// ----------------------------------------------------------------------

/// 画面遷移ステート (§4.2)。
///
/// 旧設計の `Failed` は削除した。エラーは Toast で表現するため、
/// 画面状態は「いまソースがあるか／処理中か」のみを表す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Empty,
    Importing,
    Preview,
    ExportReady,
    Exporting,
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

    // テーマ・UI
    ThemeToggled,
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

    // 書き出し
    ExportRequested,
    ExportDirPicked(Option<PathBuf>),
    ExportCompleted(Result<ExportReport, AppError>),

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
pub fn run() -> iced::Result {
    iced::application(AppState::default, update, view)
        .title("Logolig")
        .theme(theme)
        .subscription(subscription)
        .run()
}

fn theme(state: &AppState) -> Theme {
    match state.theme {
        // System は Step 3 で OS テーマを覗くようにする。Step 1 では Light を採用。
        ThemeMode::System | ThemeMode::Light => Theme::Light,
        ThemeMode::Dark => Theme::Dark,
    }
}

fn subscription(state: &AppState) -> Subscription<Message> {
    // 2 つのサブスクリプションを結合する:
    //   (a) snora の Toast tick — transient toast の自動消滅
    //   (b) iced のウィンドウイベント — ファイルドロップを Message::FileDropped に変換
    let toasts = snora::toast::subscription(&state.toasts, || Message::ToastTick);

    let drops = iced::window::events().filter_map(|(_id, ev)| match ev {
        iced::window::Event::FileDropped(path) => Some(Message::FileDropped(path)),
        _ => None,
    });

    Subscription::batch([toasts, drops])
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
            crate::task_queue::pick_file_task()
        }
        Message::IngestCompleted(Ok(asset)) => {
            // プレビュー生成タスクを起動するために asset を Arc で保持する。
            // SourceAsset 自体は Clone (Arc<[u8]> ベース) なので軽量。
            let arc = std::sync::Arc::new(asset.clone());
            state.source_asset = Some(asset);
            state.preview = Some(PreviewProfile::default());
            state.preview_cache = None;
            state.screen = Screen::Preview;
            state.busy = false;
            // プレビュー画像を非同期に作る (§2.4)。
            crate::task_queue::build_preview_task(arc, state.export_plan.algorithm)
        }
        Message::IngestCompleted(Err(err)) => fail(state, err),

        Message::PreviewBuilt(Ok(cache)) => {
            // 受け取った cache が「現在の状態」 と整合しているか確認する。
            // 古いソース・古い algorithm のキャッシュなら破棄。
            let asset_path = state.source_asset.as_ref().map(|a| a.path.clone());
            let still_valid = asset_path.as_ref() == Some(&cache.source_path)
                && cache.algorithm == state.export_plan.algorithm;
            if still_valid {
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

        Message::ThemeToggled => {
            state.theme = state.theme.next();
            Task::none()
        }
        Message::AdvancedToggled => {
            state.advanced_open = !state.advanced_open;
            Task::none()
        }
        Message::AlgorithmChanged(alg) => {
            state.export_plan.algorithm = alg;
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
            Task::none()
        }
        Message::VectorizeOnRasterToggled(on) => {
            state.export_plan.vectorize_on_raster = on;
            Task::none()
        }

        // -----------------------------------------------------------------
        // v1.3.0: 詳細設定の編集 UI ハンドラ
        // -----------------------------------------------------------------
        Message::IncludeIcoToggled(on) => {
            state.export_plan.include_ico = on;
            Task::none()
        }
        Message::IncludeAppleTouchToggled(on) => {
            state.export_plan.include_apple_touch = on;
            Task::none()
        }
        Message::IncludeHtmlSnippetToggled(on) => {
            state.export_plan.include_html_snippet = on;
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
                    } else {
                        // 範囲外は parse_size で捕捉済みなので、 ここに来るのは重複のみ
                        push_warning_toast(
                            state,
                            "Already in set",
                            &format!("PNG size {size} px is already configured."),
                        );
                    }
                }
                Err(SizeParseError::Empty) => {
                    // 空入力での Add は無視 (UX: Enter 連打で迷子にしない)
                }
                Err(SizeParseError::NotANumber) => {
                    push_warning_toast(
                        state,
                        "Invalid size",
                        &format!("'{raw}' is not a valid pixel size."),
                    );
                }
                Err(SizeParseError::OutOfRange { min, max }) => {
                    push_warning_toast(
                        state,
                        "Size out of range",
                        &format!("PNG sizes must be between {min} and {max} px."),
                    );
                }
            }
            Task::none()
        }
        Message::PngSizeRemoveRequested(size) => {
            state.export_plan.remove_png_size(size);
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
                    } else {
                        push_warning_toast(
                            state,
                            "Already in set",
                            &format!("ICO size {size} px is already configured."),
                        );
                    }
                }
                Err(SizeParseError::Empty) => {}
                Err(SizeParseError::NotANumber) => {
                    push_warning_toast(
                        state,
                        "Invalid size",
                        &format!("'{raw}' is not a valid pixel size."),
                    );
                }
                Err(SizeParseError::OutOfRange { min, max }) => {
                    push_warning_toast(
                        state,
                        "Size out of range",
                        &format!("ICO sizes must be between {min} and {max} px (ICO format limit)."),
                    );
                }
            }
            Task::none()
        }
        Message::IcoSizeRemoveRequested(size) => {
            state.export_plan.remove_ico_size(size);
            Task::none()
        }

        Message::ExportRequested => {
            // ソースが無ければ何もしない (UI 側でボタン無効化済みだが念のため)。
            if state.source_asset.is_none() {
                return Task::none();
            }
            // 出力先選択ダイアログを開く。 結果は `ExportDirPicked` で戻る。
            crate::task_queue::pick_export_dir_task()
        }
        Message::ExportDirPicked(None) => {
            // ユーザがキャンセル。 状態は変えない。
            Task::none()
        }
        Message::ExportDirPicked(Some(dir)) => {
            let Some(asset) = state.source_asset.as_ref() else {
                return Task::none();
            };
            state.screen = Screen::Exporting;
            state.busy = true;
            let asset_arc = std::sync::Arc::new(asset.clone());
            let plan = state.export_plan.clone();
            crate::task_queue::export_task(asset_arc, plan, dir)
        }
        Message::ExportCompleted(Ok(report)) => {
            let count = report.artifacts.len();
            let dir_display = report.output_dir.display().to_string();
            state.screen = Screen::ExportReady;
            state.busy = false;
            push_success_toast(
                state,
                "Exported",
                &format!("{count} files written to {dir_display}"),
            );
            Task::none()
        }
        Message::ExportCompleted(Err(err)) => fail(state, err),

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
fn start_ingest(state: &mut AppState, path: PathBuf) -> Task<Message> {
    state.source_path = Some(path.clone());
    state.screen = Screen::Importing;
    state.busy = true;
    // 古いソースのプレビューキャッシュは破棄。新しい ingest 完了で再生成される。
    state.preview_cache = None;
    crate::task_queue::ingest_task(path)
}

/// エラー発生時の共通遷移。
///
/// - busy フラグを下ろす
/// - 画面は「ソースがあれば Preview、なければ Empty」に戻す
/// - Persistent な Error toast を積む（読まないと消えない）
fn fail(state: &mut AppState, err: AppError) -> Task<Message> {
    state.busy = false;
    state.screen = if state.source_asset.is_some() {
        Screen::Preview
    } else {
        Screen::Empty
    };
    push_error_toast(state, err);
    Task::none()
}

/// エラーを persistent な Toast として通知。ユーザーが閉じるまで残る。
fn push_error_toast(state: &mut AppState, err: AppError) {
    let id = next_id(state);
    state.toasts.push(
        Toast::new(
            id,
            ToastIntent::Error,
            "Operation failed",
            err.to_string(),
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
