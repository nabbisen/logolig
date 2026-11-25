# Developer Handoff — RFC 012: web manifest input validation (job K2, release 1.29.0), with a small K3 follow-up first

Role and rules as always (`.git-exclude/roles/*`, `.git-exclude/rules/*`; GUI Evidence and Test Optimization rules; Rust 2024, tests in separate files, English only, `unsafe` forbidden; **relative paths with `cd`, or none**). Governing text: [RFC 012](../../done/012-web-manifest-input-validation.md) with **"Decisions 2026-11-22"** (binding: `#RGB` accepted and canonicalised to `#RRGGBB` upper case, the field shows the canonical form; inline errors use a **lucide icon, not the `⚠` glyph**; invalid text is never saved; the retired toast keys stay). Review of K3: `.git-exclude/reviewed/rfc-013-system-theme-review-2026-11-23.md`. **Workflow:** `main`; `git pull --ff-only`; `git log origin/main..` before every push; one commit per step; gates chained (audit 0 / 12); CI waited for each and **looked at after your push**; no tags, force-push or history rewriting.

## Step 0 — the K3 follow-up (one small commit)

The preview's **Surface picker** shows its System option as "System (not reported; using Light)" (the Settings label). In that narrow picker it is heavy, and in that place "System" is enough: the mock-up beside it shows what System resolves to. **Use the plain `PreviewBackgroundSystem` text ("System") in the picker**, keep the honest long label on the Settings page only, and change the `radio_groups.rs` lookup back to the short text; this also gives `PreviewBackgroundSystem` a real use again (it must not become a dead key). Tests and gates as usual.

## 0. Evidence plan

| Question | Level | Evidence |
|---|---|---|
| The colour rule: `#RRGGBB` and `#RGB` accepted (any letter case), canonicalised to `#RRGGBB` upper case; everything else (no `#`, wrong length, non-hex, empty, whitespace inside) invalid; leading/trailing whitespace trimmed | pure (core, one function) | table test of valid and invalid inputs, including `#FFF`, `#ffF`, `#12345`, `#GGGGGG`, ` #fff `, `#fff;`, `rgb(…)` |
| Names: `name` and `short_name` non-empty after trimming | pure | cases incl. only spaces |
| An invalid value is **never** saved and **never** replaces the previous valid value; half-typed text stays in the edit buffer and is not written to `settings.json` on each keystroke | `update()` tests with the persisted-settings snapshot | before/after leaf comparison per keystroke sequence (`#`, `#F`, `#FF`, `#FFF` …) |
| After a valid commit the field shows the canonical form | update + headless | `#fff` → `#FFFFFF` shown |
| Inline feedback: an icon plus a plain sentence under the field, English and Japanese, both themes, all window sizes, no overflow; the sentence says what is accepted | headless matrix + i18n | extend the matrix with a Customize state that shows an error on each of the four fields; key parity |
| Defence in depth: the exporter returns a typed `AppError::Export` with an i18n key for invalid manifest settings (reachable only by a hand-edited file) | core test | hand-built `WebManifestSettings` |
| Loading: invalid manifest values in `settings.json` are replaced by defaults, with one warning toast, and the application starts | core/app test | hand-written bad files (bad colour, empty name) |
| Keyboard: a field with an error is still a Tab stop; Enter commits; focus is not lost when the error appears | headless (the J14 session driver) | tests |

No real-window run in this job (the logic and headless levels cover it); the release pass includes "type an invalid colour".

## 1. Work (tests first, shown failing before)

1. **Core:** one `normalize_color(&str) -> Option<String>` (or a typed `Result`) used by the app and the exporter; keep `is_valid_color`/`has_required_text` public (nothing public is removed); add what is new as additions. Validation in `exporter::run_in_memory` when `web_manifest` is `Some` (decision: typed `AppError::Export`, new `MessageKey` for the text; the accepted source-compatibility note).
2. **App:** edit buffers for the four manifest fields in `AppState` (like `png_size_input`); `ExportPlan` and `settings.json` receive a value only when it is valid; on commit (Enter, or the field losing focus: check what iced 0.14 `text_input` offers: `on_submit` exists; a blur hook may not: **if there is no blur event, commit on submit and on every keystroke that makes the text valid, and say so**) the field shows the canonical form. `persist_settings` is no longer called with half-typed values (the finding of the RFC).
3. **Inline feedback** per field, shown only once the user has typed or submitted something invalid (not on an untouched default): the lucide warning icon (`snora::lucide::AlertTriangle` exists in the pinned icon font; check the name the `snora::lucide` re-export gives) plus a sentence ("Use a colour like #1A2B3C or #1AB", "Enter a name"); never colour alone; a text-glyph state cue is forbidden (the source test of J10 forbids `⚠`). New keys in English and Japanese; key parity; list the Japanese pairs for `.git-exclude/tmp/ja-review-1.29.0.md` (not blocking).
4. **Loading:** invalid manifest values found in `settings.json` become the defaults with one warning toast (reuse the retired `ToastInvalidColor*` text if it fits, or new keys; say which); the application never fails to start.
5. **Docs:** `docs/src/functional-spec.md` (manifest fields: accepted forms, canonical form, inline feedback, what happens to a bad file value) and the README sentence "If a malformed color is detected at export, a Toast is shown and the export is blocked" corrected to what is true (the UI cannot commit an invalid value; the exporter also refuses one).

## 2. Evidence, acceptance, prohibited

Review request `.git-exclude/review-request/rfc-012-manifest/README.md` (report only its path): the §0 table with failing-before lists; mutation checks for the three protections (a stale value not kept; an invalid value saved; the exporter not validating); SemVer additions (public items added, none removed); where blur is or is not available; test-list diff (named additions; adapted tests listed); gates and CI per commit. Acceptance: invalid text is never saved; `#RGB` accepted and shown canonical; inline icon-and-sentence feedback in both languages at every size; exporter and loader defence in depth; keyboard unchanged; docs true. Prohibited: the `⚠` glyph or any text-glyph cue; removing a public item or the retired toast keys; changing the manifest output format for valid input; a real-window run without my words; absolute `cd`; tags, force-push or history rewriting.
