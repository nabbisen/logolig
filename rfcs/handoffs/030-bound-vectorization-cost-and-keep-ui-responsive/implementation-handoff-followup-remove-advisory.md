# Developer Handoff — RFC 030 follow-up: remove the large-SVG advisory (job J4b-fix)

Same roles, rules and workflow as [`implementation-handoff.md`](./implementation-handoff.md) (directly on `main`, `git pull --ff-only` first, gates chained with `&&`, no red push, no tags). Governing text: [RFC 030](../../done/030-bound-vectorization-cost-and-keep-ui-responsive.md) Design 3 (amended 2026-10-25). Review of J4b: `.git-exclude/reviewed/rfc-030-vectorization-cost-review-2026-10-25.md`.

## Why

The cap made the advisory's premise disappear: the traced `favicon.svg` is bounded and the conversion takes well under a second. Your finding 1 showed the 2 MB trigger almost never fires; the architect traced ten real images (1920 to 5120 px): 0.01 to 1.0 MB, none above 2 MB. A warning that cannot fire where intended, or that is made to fire by lowering the threshold, is confusing and adds text and keys to maintain for nothing ("Less is more"). Remove it.

## Scope (remove, nothing else)

1. `crates/app/src/app.rs`: `SVG_ADVISORY_BYTES`, `is_large_traced_svg`, `push_svg_large_advisory`, the call in the `ConvertCompleted(Ok)` arm (restore the arm to its state before `da9a031`), and the 4 advisory tests (including the SVG-source guard test).
2. i18n: `ToastSvgLargeTitle` / `ToastSvgLargeBody` in `message_key.rs`, `dictionary.rs`, `en.toml`, `ja.toml`, `crates/i18n/tests/i18n.rs` (the 2 tests and the entries added to the two lists).
3. Docs: `docs/src/functional-spec.md` remove the "Large traced `favicon.svg`" row; `docs/src/export-spec.md` keep the cap paragraph and the statement that `favicon.svg` of a photograph is an approximation that can reach about 3 MB, but delete the sentence saying the application warns above 2 MB and points to "SVG output".

## Keep unchanged

The cap, `trace_dimensions`, `vectorize_with_cap`, `vectorize` capped with Lanczos3 (decided: stays), `run_blocking`, its 5 tests, the `tokio` dependency edge, the 8 core tests.

## Evidence required

- Gates chained with `&&` (fmt, clippy `-D warnings`, test, `cargo +1.90 check`, audit) on the final tree; test count expected: 259 − 4 − 2 = **253** (report the real number and explain any difference).
- `rg -n "SvgLarge|svg_large|SVG_ADVISORY|large_traced|push_svg_large" crates docs locales` returns nothing.
- No GUI run is required (a removal of a code path nothing in the pictures depends on). Say so in the report.
- CI on the pushed commit: all five jobs, by name.
- A short review request at `.git-exclude/review-request/rfc-030-remove-advisory/README.md`; report only its path.

## Prohibited

No new advisory, no change of the cap, no new setting, no `rfcs/` or `ROADMAP.md` edits.
