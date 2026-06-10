// `main.rs` は薄く保つ (§10)。アプリ本体は `app::run` に集約。
mod app;
mod native_store;
mod result;
mod shell;
mod task_queue;
mod ui;

fn main() -> iced::Result {
    app::run()
}
