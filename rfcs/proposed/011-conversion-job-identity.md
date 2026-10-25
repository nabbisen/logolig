# RFC 011 — Conversion job identity and concurrent-input policy

**Status.** Proposed
**Priority.** P2 (race with low probability and confusing effect; cheap to fix)
**Tracks.** Defect: results of an older conversion can overwrite a newer one.
**Touches.** `crates/app/src/app.rs` (`AppState`, `Message`, `update`), `crates/app/src/task_queue.rs`, app tests.

## Summary

Asynchronous results (`IngestCompleted`, `PreviewBuilt`, `ConvertCompleted`) carry no identity of the request that produced them, and `FileDropped` is accepted in every screen state. A newer file can therefore be displayed with an older file's results.

## Evidence

Verified by source reading; **not executed**.

- The window-level subscription maps `FileDropped` to `Message::FileDropped` regardless of `Screen`; `start_ingest` runs unconditionally.
- `Message::ConvertCompleted(Ok(assets))` unconditionally sets `result_assets` and `Screen::Result`.
- Interleaving: drop A, A is converting, drop B, A's conversion finishes after B's ingest started or after B's conversion finished (large SVG vectorisation vs a small PNG). Final state: `source_asset = B`, `result_assets = A`.
- `PreviewBuilt` is guarded by source path and algorithm only; re-dropping the same path is not distinguished.
- `fail()` chooses the next screen from whether `result_assets` is `Some`, which is also affected by stale results.

## Design

1. `AppState.job_id: u64` (monotonic). `start_ingest` increments it.
2. `ingest_task`, `build_preview_task` and `convert_task` capture the id and return it with their result: `IngestCompleted(job, Result<..>)`, `PreviewBuilt(job, ..)`, `ConvertCompleted(job, ..)`.
3. `update` ignores any result whose `job` differs from `state.job_id` (no state change, no toast).
4. **Input policy while a job is running.** Recommended: *latest intent wins* — a new drop or pick supersedes the running job (it keeps working because of step 3). Alternative: ignore drops while `Screen::Converting`, which is simpler but makes a mis-drop un-correctable until completion.
5. Errors from a superseded job are discarded; only the current job may raise error toasts.
6. Remove the redundant `busy` flag or define it as `job_in_flight`; today it is set but is not used to guard input.

Task cancellation via `Task::abortable` is possible but needs handle storage and does not stop CPU-bound work already running on the executor; the id check is sufficient and stateless.

## Addendum (2026-10-25, RFC 019 phase 2 review)

Two more concurrency observations: (1) pressing Cmd+O twice quickly (or clicking the drop zone twice) may open two file choosers, since nothing records that a chooser is already open; the job-identity design should also cover "a chooser is open". (2) CPU-bound conversion work blocks a tokio worker and starves other events during long conversions: tracked and quantified in RFC 030.

## Open question

- **Q1** CPU-bound conversion runs inside `async move { .. }` on iced's executor. If a worker thread is blocked by a long vectorisation, UI responsiveness depends on the runtime's thread count. Measure with a 4000 × 4000 PNG before deciding whether to move the work to a blocking pool. Not part of this RFC's acceptance.

## Tests (design-derived)

All as `update()` tests with hand-built messages, no GUI needed:

- `IngestCompleted(job 1, A)`, `FileDropped(B)`, `ConvertCompleted(job 1, A)` ⇒ no change to `result_assets`.
- Two results arriving in reverse order ⇒ only the latest job is applied.
- Stale `PreviewBuilt` after a new drop of the same path ⇒ ignored.
- A stale error does not push a toast.

## Acceptance criteria

The tests above pass; behaviour for the single-job case is unchanged (existing app tests pass).

## Risk

Low. Message signatures change (internal only). Best done after RFC 017 splits `app.rs`, to avoid a large conflicting diff.
