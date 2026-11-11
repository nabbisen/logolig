# Developer Handoff — RFC 031, phase 0: spike on file dialogs (job J12, evidence only)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (read the **GUI Evidence and Automation Rules** and **Test Optimization Rules** again; relative paths with `cd`; English only).

Governing RFC: [RFC 031](../../accepted/031-silent-file-dialog-failure.md), including its **amendment of 2026-11-12**. Previous reviews: `.git-exclude/reviewed/rfc-011-job-identity-review-2026-11-13.md` and the JH review (`.git-exclude/reviewed/rfc-035-headless-ui-test-review-2026-11-12.md`). **Start only after JH is pushed and its CI is green.** On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused. **This job changes no product code and makes no commit of code.** It produces evidence and a recommendation; the architect decides, then a second handoff (implementation) follows. (Adding a spike script under `.git-exclude/review-request/` is not a commit.)

## 1. Questions (the evidence plan, with the level each is answered at)

| # | Question | Level | How |
|---|---|---|---|
| Q1 | Do the two **Save dialogs** (download one, download all) really open twice on a double request? (`update/download.rs`, `task_queue.rs` `pick_save_one_task`, `pick_save_all_task`) | logic and state first, then **one real run** | Logic: show with a hand-built `update()` test (uncommitted, in the scratch folder or as a patch file) that two `DownloadAllRequested` produce two tasks. Real: Ctrl+S-equivalent twice (use the shortcut that sends `DownloadAllRequested`; find it in `shortcuts.rs`) on a Result screen, count new windows by `niri msg -j windows`, as in the J11 run (`.git-exclude/review-request/rfc-011-job-identity/evidence/chooser-run.sh`; reuse its functions, do not rewrite them). Report 1 or 2 windows. |
| Q2 | Can the application tell, cheaply and without a new package, that the portal is **missing** (so "Choose file…" is dead)? (RFC 031 design A) | recorded state / code | Write the probe as a patch file (not committed): ask the session bus whether `org.freedesktop.portal.Desktop` has an owner, using only crates already in the tree (`zbus`/`ashpd` via `rfd`; list which crate and which call). Report: lines of code, whether it needs a **direct** dependency edge (`Cargo.toml` change and the resulting `git diff --stat Cargo.lock`: the target is **no new package**), whether it can block the UI thread (it must be an async call inside the dialog task), and its result on the real session bus (portal present). |
| Q3 | What does the probe say with **no portal**? | real UI, private bus | `dbus-run-session` (or `dbus-daemon --session` on a private socket) with no portal service, `DBUS_SESSION_BUS_ADDRESS` pointing at it, isolated `XDG_CONFIG_HOME`/`XDG_STATE_HOME`, one instance. Show: the probe reports "missing"; with the unmodified binary Ctrl+O opens nothing and prints nothing (confirm the J5 observation). If a private bus cannot be made to behave, say so and stop at Q2's result. |
| Q4 | "Portal present but **backend** missing": distinguishable? | recorded state | Use `busctl` (or the probe) against the private bus with `org.freedesktop.portal.Desktop` registered by a stub that has no `FileChooser` implementation, **if** you can build such a stub in under 30 lines of shell or python; otherwise report "not reproducible here" (allowed) and state what the probe can and cannot tell. |
| Q5 | Can a dialog that **never answers** be reproduced (flag `chooser_open` stuck)? | real UI, private bus | With the stub from Q4 (a portal that accepts the call and never replies): does `rfd` return, hang, or time out? Report how long you waited (one bounded wait of 20 s, no sampling) and the state of `chooser_open`'s observable effect (Ctrl+O again opens nothing). "Not reproducible" is allowed. |

## 2. Rules for the runs

Real runs follow the GUI rules (unlocked session; isolated XDG dirs and a pinned `settings.json`; one instance; window chosen by pid; focus check before every keystroke; assert after each step; stop at the first anomaly; analyse after the run; repeat limit). **The owner's real `~/.config/logolig/settings.json` modification time before and after** is recorded. Do not touch the owner's real D-Bus session bus beyond reading (`busctl --user list` is allowed); never start or stop a portal service on it.

## 3. Deliverable

`.git-exclude/review-request/rfc-031-dialog-spike/README.md` (report only its path) containing, per question: the answer, the evidence (pasted output), what you could not do and why. Then a **recommendation with reasons** for each of: (a) design A or D for the missing portal; (b) the Save-dialog guard (one dialog of each kind at a time: the same pattern as `chooser_open`; say how many lines and which tests); (c) the never-answers case (recovery needed or document only). Do not implement any of it.

## 4. Acceptance criteria

All five questions answered (a stated "not reproducible" counts); no product file changed (`git status` clean apart from the review-request folder); no new package in `Cargo.lock`; the real configuration untouched; one real-UI set per question that needs it, no repeats without asking.

## 5. Prohibited shortcuts

No timing heuristic (design C is rejected in the RFC); no `ashpd`/`zbus` direct dependency committed; no change to a dialog's behaviour; no starting or stopping system services on the owner's real bus.
