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
    AppError, ExportPlan, PreviewProfile, ResizeAlgorithm, SourceAsset, ThemeMode,
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
    pub export_plan: ExportPlan,
    pub busy: bool,
    /// snora の Toast キュー。エラー・成功通知はすべてここを経由する。
    pub toasts: Vec<Toast<Message>>,
    /// Toast の id を発行するためのカウンタ。
    pub next_toast_id: u64,
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
            export_plan: ExportPlan::default(),
            busy: false,
            toasts: Vec::new(),
            next_toast_id: 0,
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
/// **構築箇所が無い** (`FileDropped` は subscription, `FilePicked` は rfd ピッカー、
/// `AlgorithmChanged` は詳細設定 UI, `ExportCompleted` は出力完了)。
/// それぞれ Step 2 / Step 3 / Step 4 で生成側を実装する。
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

    // テーマ・UI
    ThemeToggled,
    AdvancedToggled,
    AlgorithmChanged(ResizeAlgorithm),

    // 書き出し
    ExportRequested,
    ExportCompleted(Result<(), AppError>),

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
            state.source_asset = Some(asset);
            state.preview = Some(PreviewProfile::default());
            state.screen = Screen::Preview;
            state.busy = false;
            Task::none()
        }
        Message::IngestCompleted(Err(err)) => fail(state, err),

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
            Task::none()
        }

        Message::ExportRequested => {
            // Step 4 で実装。今は遷移だけ進める。
            state.screen = Screen::Exporting;
            state.busy = true;
            Task::none()
        }
        Message::ExportCompleted(Ok(())) => {
            state.screen = Screen::ExportReady;
            state.busy = false;
            push_success_toast(state, "Exported", "Files written to disk.");
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
