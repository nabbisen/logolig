//! `run_blocking` (RFC 030): CPU-bound work runs off the async executor, and a failed or
//! panicking job becomes an `AppError`, never a silent loss or a panic on the UI side.

use super::*;

use std::sync::mpsc;
use std::time::Duration;

fn runtime() -> tokio::runtime::Runtime {
    // One executor thread on purpose: if the job blocked it, the other task could not run.
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("a current-thread runtime should build")
}

#[test]
fn the_value_of_the_job_is_returned() {
    let out = runtime().block_on(run_blocking("test", || Ok(41 + 1)));
    assert_eq!(out.expect("job succeeds"), 42);
}

#[test]
fn an_error_of_the_job_is_passed_on_unchanged() {
    let out: Result<(), AppError> = runtime().block_on(run_blocking("test", || {
        Err(AppError::export("the job said no"))
    }));
    let err = out.expect_err("job fails");
    assert!(err.to_string().contains("the job said no"), "{err}");
}

#[test]
fn a_panic_in_the_job_becomes_an_error() {
    let out: Result<(), AppError> =
        runtime().block_on(run_blocking("conversion", || panic!("boom in the job")));
    let err = out.expect_err("a panic must not be swallowed or propagated");
    let text = err.to_string();
    assert!(text.contains("conversion task failed"), "{text}");
}

#[test]
fn the_job_runs_on_another_thread_than_the_caller() {
    let caller = std::thread::current().id();
    let job_thread = runtime()
        .block_on(run_blocking("test", || Ok(std::thread::current().id())))
        .expect("job succeeds");
    assert_ne!(job_thread, caller);
}

#[test]
fn a_blocked_job_does_not_stop_the_executor() {
    // The job waits for a message that only another *task on the same single-threaded
    // executor* sends. If the job blocked the executor, that task could never run and the
    // job would time out (the 10 s limit only keeps a failing run from hanging).
    let (tx, rx) = mpsc::channel::<()>();
    let out = runtime().block_on(async move {
        let job = tokio::spawn(run_blocking("test", move || {
            Ok(rx.recv_timeout(Duration::from_secs(10)).is_ok())
        }));
        tokio::task::yield_now().await;
        tx.send(()).expect("the job is still waiting");
        job.await.expect("the job task joins")
    });
    assert!(
        out.expect("job succeeds"),
        "the executor was blocked by the job"
    );
}
