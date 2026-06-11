// Keep main.rs thin (§10). All app logic lives in `app::run`.
mod app;
mod native_store;
mod result;
mod shell;
mod task_queue;
mod ui;

fn main() -> iced::Result {
    app::run()
}
