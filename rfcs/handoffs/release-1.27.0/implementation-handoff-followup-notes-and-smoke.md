# Developer Handoff — Release 1.27.0 candidate follow-up (job J6b): plain release notes and the last smoke run

Same roles, rules and workflow as [`implementation-handoff.md`](./implementation-handoff.md) (directly on `main`, `git pull --ff-only` first, gates chained with `&&`, no red push; no tags, releases, AUR pushes; no edits to `rfcs/` or `ROADMAP.md`). Review of J6: `.git-exclude/reviewed/release-1.27.0-candidate-review-2026-10-28.md`.

## 1. Release notes for users: no RFC numbers

The notes are read by users (they become the GitHub Release text). RFC numbers mean nothing to them ("users must not be confused"); traceability stays in `docs/src/architecture.md` and the review files.

In `docs/release-notes/1.27.0.md` remove every parenthetical RFC reference (all 15, for example `(RFC 010)`, `(RFC 015 phase 2a)`, and the words "RFC 026 is proposed", "RFC 014 is proposed", "RFC 027 is planned", "RFC 031 is proposed", "(RFC 020)", "(RFC 024)"). Where a sentence says a problem "is proposed/planned", say "is planned" or "is a known limitation" in plain words, or just state the limitation. Change nothing else in the notes. Keep the `docs/src/architecture.md` row as it is (it is developer documentation and keeps its RFC references). Check: `rg -n "RFC" docs/release-notes/1.27.0.md` returns nothing.

## 2. The third smoke run (approved, once)

Permission granted by the architect for **one** run, because the two aborted runs failed on the harness's own assertion and not on the application. Use your corrected script (Customize is judged by the rows 900–1000 criterion you already checked offline). Before running, state in the review request the criterion for each assertion and what the three captured states looked like in your earlier captures, so that a threshold is never guessed again.

Required observations (same setup and rules as before: isolated `XDG_CONFIG_HOME` **and** `XDG_STATE_HOME`, never start the binary without them (it has no command-line arguments), `pid` focus check before every key, no resizing, absolute screenshot paths): Ctrl+3 shows Settings with the shortcut list; Ctrl+1 returns to the Result screen; Esc dismisses the newest toast; Esc on the Result screen returns to the start screen. If the run aborts again, **stop and report** (no fourth run); the remaining steps are then recorded as not observed.

## 3. Delivery

One commit for the notes (gates are not needed for a docs-only change, but `git diff --stat` must show only `docs/release-notes/1.27.0.md`); push. Review request at `.git-exclude/review-request/release-1.27.0-smoke/README.md` (short: the commit, the `rg` result, the smoke record). Report only the path. After the architect's review the release recommendation goes to the owner.
