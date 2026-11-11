// Keep main.rs thin (§10). All app logic lives in `app::run`.
mod app;
mod native_store;
mod result;
mod shell;
mod shortcuts;
mod task_queue;
mod toast_time;
mod ui;
#[cfg(test)]
mod ui_layout_tests;
mod window;

fn main() -> iced::Result {
    app::run()
}
