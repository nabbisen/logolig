# Developer Handoff — RFC 031 follow-up: the notice does not stack (job J12c, small)

Role and rules as in `implementation-handoff.md` of this folder (same workflow: `git pull --ff-only`, `git log origin/main..` before the push, gates chained, CI waited for, no tags/force-push). Review of J12b: `.git-exclude/reviewed/rfc-031-dialogs-review-2026-11-15.md`.

## Decision (architect)

Pressing Ctrl+O (or Save) repeatedly with no dialog available stacked identical persistent error toasts (J12b §3). A growing pile of the same message is clutter and confuses; one notice is enough. **While a notice of the same kind (same title and body) is still showing, `Message::DialogUnavailable` pushes no further toast** (it still clears `dialog_open`, as now). Scope: **only this notice**; do not change how any other toast behaves.

## Work (one commit)

1. In `push_dialog_unavailable_toast` (`app/toast.rs`) return early if `state.toasts` already holds a toast with the same title **and** body (compare what the toast stores; if `snora::Toast` fields are not readable, keep the ids of the notice in a small `AppState` field cleared on `DismissToast`/sweep, and say why). Keep the function's doc comment truthful.
2. Tests (logic): two requests give one toast; after `DismissToast` a request gives a toast again; the chooser kind and the Save kind are different notices (both may show); `dialog_open` is cleared in every case. Existing `an_unavailable_dialog_clears_the_flag_and_pushes_exactly_one_error_toast` keeps its name.
3. No real-window run (the logic test is the evidence; say so). Gates and CI as usual. Review request: `.git-exclude/review-request/rfc-031-notice-no-stack/README.md`; report only its path.

## Acceptance / prohibited

Acceptance: the above tests pass, nothing else changes. Prohibited: deduping other toasts, timers, new dependencies, new `#[allow]`.
