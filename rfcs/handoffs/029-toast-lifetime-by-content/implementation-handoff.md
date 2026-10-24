# Developer Handoff — RFC 029: Toast lifetime proportional to its content (job J3b)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (Rust 2024, 2018+ module style without `mod.rs`, English only, tests separated from implementation, `unsafe` forbidden; relative paths with `cd`; GUI evidence rules in `.git-exclude/rules/project-instructions-gui-common.md`, "GUI Evidence and Automation Rules").

Governing RFC: [RFC 029](../../accepted/029-toast-lifetime-by-content.md) (Accepted; owner authorized it into release 1.27.0 on 2026-10-24). The RFC (with its **corrected formula**) is authoritative. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused or misled.

**Workflow (owner decision): no topic branches, no concurrent jobs.** Work directly on `main`: `git pull --ff-only` first (the checkout is shared with the architect; do not leave uncommitted work lying around between sessions), small commits, **chain every gate with `&&` and push only when all pass**, never force-push, no tags. Job J3b in `ROADMAP.md`; prerequisite: J3 reviewed (it is).

## 1. Purpose

Every warning toast lives 4 seconds (snora's default), but the advisory texts need 9 to 17 seconds to read. Give each transient toast a lifetime proportional to what it says, in one tested place.

## 2. Background (verified facts)

- `crates/app/src/app.rs`: `push_warning_toast` (line about 1254) and `push_success_toast` create `Toast::new(..)` with no lifetime; `push_long_success_toast` sets `ToastLifetime::seconds(7)`; `push_error_toast` is `.persistent()`.
- snora API (0.53, `snora-core/src/toast.rs`): `ToastLifetime::seconds(u64)` takes **whole seconds**; `Toast::with_lifetime(ToastLifetime)`; `ToastLifetime::DEFAULT` = 4 s. The formula below therefore yields a whole number of seconds (round **up**).
- Callers of `push_warning_toast`: invalid size / duplicate size / out of range (PNG, ICO), transparency advisory, JPEG advisory, non-square advisory, SVG parts advisory, settings load/save failures. All go through the one function.
- The body of the SVG advisory contains `\n` between sentences.

## 3. The rule (RFC 029, binding)

For every **transient** toast: `seconds = clamp(ceil(2.0 + 0.3 × units), 4, 20)` where `units` is the reading size of **title + body** as displayed:

- split the non-CJK text on whitespace; each token counts `max(1, ceil(chars / 6))` (a long path counts as several units, not one);
- each CJK character (Han, Hiragana, Katakana, Hangul; use `char::is_alphabetic` plus Unicode block ranges you list in code comments, no new dependency) counts `0.4`;
- punctuation and whitespace count 0.

`push_long_success_toast` keeps 7 s as a **minimum**: `max(7, formula)`. Errors stay persistent; do not touch them. Reference values (computed by the architect on the real texts with a reference implementation of this rule; your tests must reproduce them): a 6-unit toast → 4 s; 31 units → 12 s; 48 units → 17 s; 57 units → 20 s. Long words count as several units (`ceil(chars / 6)`), so unit counts are higher than plain word counts.

## 4. Change scope

- New module `crates/app/src/toast_time.rs` (declare `mod toast_time;` in `main.rs`) with `pub(crate) fn lifetime_for(title: &str, body: &str) -> snora::ToastLifetime` (and a smaller `pub(crate) fn seconds_for(title, body) -> u64` that the tests call), tests in `crates/app/src/toast_time/tests.rs` declared with `#[cfg(test)] mod tests;` (separate file per the project rules).
- `crates/app/src/app.rs`: `push_warning_toast` and `push_success_toast` call `.with_lifetime(lifetime_for(title, body))`; `push_long_success_toast` uses `max(7, ...)`. Nothing else in `app.rs`.
- `docs/src/functional-spec.md` §10: one sentence: toast time scales with the length of the message (minimum 4 s, maximum 20 s; errors stay until dismissed).

## 5. Non-change scope

Toast texts, colours, positions, `×`, persistence of errors, snora version/features, any dependency, `Cargo.toml`/`Cargo.lock`, other files. No hover/focus extension logic (snora has no such hook). No new setting.

## 6. Required tests (design-derived; write them first)

In `toast_time/tests.rs`:

| # | Test | Assertion |
|---|---|---|
| 1 | empty title and body; one-word toast | 4 |
| 2 | the three real advisories (use the English texts from `crates/i18n/locales/en.toml`: `toast_non_square_input_*`, `toast_jpeg_input_*`, and the SVG title plus both bodies joined by `\n`) | non-square **12**, JPEG **17**, SVG combined **20** (cap); SVG images-only **13**, transparency (fully opaque) advisory **12** |
| 3 | the same three in Japanese (`ja.toml`) | architect's reference values: non-square **11**, JPEG **15**, SVG combined **20**, SVG images-only **12**, fully-opaque **11**; assert them exactly (title and body joined by a space; the SVG bodies joined by `\n` as the app does). If your tokenisation of punctuation inside a Japanese token differs by one second from these, report it instead of editing the expectation |
| 4 | a 200-character single token (long path) | counts as `ceil(200/6)` = 34 units → `ceil(2 + 0.3 × 34)` = **13**, not 4 |
| 5 | a text far beyond the cap | exactly 20 |
| 6 | monotonic: adding words never lowers the result | property over a few lengths |
| 7 | CJK weights: 25 CJK characters = 10 units | `ceil(2 + 0.3 × 10)` = 5 |
| 8 | whitespace/punctuation only | 4 |

In the existing `app.rs` test module: for each of the pushed advisories (non-square, SVG parts, JPEG via its helper path, an invalid-size warning) the toast's lifetime is `Transient(d)` with `d` equal to `seconds_for(its own title, its own body)`; the long success toast is at least 7 s; an error toast is persistent. (Read the lifetime through the public accessors of `snora::Toast`; if none exists in 0.53, say so and compare through `Debug` output instead, naming the limit in the review request.)

## 7. Required evidence

1. Gates, chained: `cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit` (expected: 0 vulnerabilities, 12 warnings unchanged).
2. **One** GUI check (rules file): isolated config, binary named `logolig`, one instance; drop a PNG that is not square so the non-square advisory appears, then assert by machine that the toast is **still visible** at about 8 s and **gone** by about 14 s (two captures plus the first one to show it appeared; no frame loop). State the backend. One picture is enough (the one at 8 s).
3. CI: the run of the final commit and each of the five job conclusions (if a job shows `in_progress` while the run is completed, read the raw job log).

## 8. Acceptance criteria

The helper exists once and is tested against the RFC's reference values; the app uses it for warning and success toasts; errors untouched; docs updated; gates and CI green; the GUI check shows the toast living past 8 s and gone by 14 s.

## 9. Prohibited shortcuts

No constants copied into callers, no per-toast hand-tuned seconds, no changes to toast texts, no new dependency, no test weakening, no red pushes, no tags or history rewriting.

## 10. Known risks

A longer toast covers content longer (RFC 026 addresses placement); the `×` stays the mitigation. If your implementation differs from a reference value in section 3 or in the tests by even one second, stop and report instead of adjusting the tests: either the tokenisation rule or the reference is wrong, and the architect decides.

## 11. Required review request

`.git-exclude/review-request/rfc-029-toast-lifetime/README.md` with everything required by section 7 of the mid-capability operating instructions, plus: the helper code, the table of computed lifetimes for every existing toast text in English and Japanese, gates and CI verdicts, the GUI evidence, skipped checks, deviations, open questions. Report to the owner only the path of the `README.md`.
