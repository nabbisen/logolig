# Developer Handoff — RFC 009: Result artifact identity and ZIP integrity

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (Rust 2024, 2018+ module style without `mod.rs`, English only, tests separated from implementation, `unsafe` forbidden).

Governing RFC: [RFC 009](../../done/009-result-artifact-identity-and-zip-integrity.md) (Accepted). The RFC defines what and why; this handoff defines how and how to prove it. On any conflict between them, stop and send a clarification request. The RFC's section "Decisions" (Q1, case-insensitive uniqueness, one additive public function) is binding.

Owner philosophy: clean, safe and secure, robust design; users must not be confused or misled.

Dependencies: none. May run in parallel with the RFC 010 handoff. Expect trivial merge conflicts with it in `crates/core/Cargo.toml` (`[[test]]` list) and in the `app.rs` test module; whichever branch is merged second rebases. Do not wait for it.

## 1. Purpose

Fix a data-losing defect: with monochrome output enabled, "Download all (ZIP)" fails and leaves a truncated file, because result items lose their `mono/` prefix and collide by name. Result items get their full relative path as identity, ZIP writing becomes all-or-nothing, and core guarantees that artifact paths are unique.

## 2. Background (verified facts; you prove the defect by execution as your first step)

- `crates/app/src/task_queue.rs::collect_assets` builds `ResultAssetItem::file_name` from `relative_path.file_name()` only, dropping `mono/`.
- With `monochrome = true` and the default plan, the item list contains `favicon-32.png`, `favicon-192.png`, `favicon-512.png` and `favicon.ico` twice.
- `write_zip_blocking` calls `std::fs::File::create(path)` first, then `zip.start_file(item.file_name, ..)` per item. `zip` 2.4.2 (`Cargo.lock`) returns `InvalidArchive("Duplicate filename")` from `ZipWriter::insert_file_data` when a name repeats, so the second colliding `start_file` fails after the file already exists on disk.
- The only ZIP tests (`task_queue.rs`, inline `mod tests`) use unique names.
- Call sites of `ResultAssetItem` fields: `app.rs` (`DownloadOneRequested` uses `item.file_name`; the test helper `dummy_result_assets` builds the struct literally), `ui/result_view.rs:196` (card title), `task_queue.rs` (construction, ZIP, the test helper `asset()`).
- Core: `exporter::run_in_memory` returns `Vec<InMemoryArtifact { relative_path: PathBuf, bytes }>`; paths for mono items are `mono/favicon-<size>.png` and `mono/favicon.ico`.

## 3. Change scope (you may touch)

Core (`crates/core`):
- `src/services/exporter.rs`: add `pub fn ensure_unique_paths(artifacts: &[InMemoryArtifact]) -> Result<(), AppError>` (name and signature fixed; additive public API approved by the architect). It fails with `AppError::export("duplicate artifact path: <path>")` when two artifacts have the same relative path, compared **case-insensitively** (compare lower-cased `to_string_lossy` forms; document why: ZIP extraction onto case-insensitive file systems). `run_in_memory` calls it on its result before returning `Ok`.
- `tests/artifact_paths.rs` (new) and its `[[test]]` entry in `crates/core/Cargo.toml` (**`autotests = false`**: an unregistered file is silently not run; verify with `cargo test -p logolig --test artifact_paths`).

App (`crates/app`):
- `src/result.rs`: `ResultAssetItem` gains `pub relative_path: String` (always `/`-separated, relative, no `..`, no leading `/`). `file_name` stays (used for kind classification and the save-dialog default).
- `src/task_queue.rs`:
  - `collect_assets` fills `relative_path`; it becomes fallible: `Result<ResultAssets, AppError>`; update `run_convert` accordingly. Build the string by joining the path components with `/`; return `AppError::export(..)` if a component is not valid UTF-8 or is not a normal component (`..`, root, prefix). This must work identically on Windows (backslash `PathBuf` input).
  - `write_zip_blocking` is split: a pure `fn build_zip_bytes(items: &[ResultAssetItem]) -> Result<Vec<u8>, AppError>` (builds the archive in memory in a `Cursor<Vec<u8>>`, entry name = `relative_path`, same `Deflated` method and same `last_modified_time(current_zip_modified_time())` as today), and `write_zip_blocking(path, items)` which calls it and then writes with **one** `std::fs::write(path, bytes)`. On any error nothing is created. Keep the function name and signature of `write_zip_blocking`, and keep `write_zip_task` unchanged.
- `src/ui/result_view.rs`: the card title shows `item.relative_path` (for root files this equals the file name, for `mono/` files it is `mono/favicon-32.png`). No other visual change; the "PNG mono" badge stays.
- `src/app.rs`: only the minimum needed to compile (the struct literal in the existing test helper `dummy_result_assets` gets the new field). `DownloadOneRequested` keeps passing `item.file_name` to the save dialog (**Q1 decision: the default save name stays the base name**).
- New test module file `src/task_queue/zip_tests.rs`, declared in `task_queue.rs` with `#[cfg(test)] mod zip_tests;` (a separate file, as the project rules require, so the existing inline tests need not move; moving them belongs to RFC 017). Adapt the existing inline helper `asset()` only as far as the new field requires.

Docs:
- `docs/src/export-spec.md`: state that "Download all (ZIP)" contains the same relative layout as the disk export, including the `mono/` subdirectory, and that artifact paths are unique (case-insensitive).
- `docs/src/functional-spec.md` §7 (result cards): cards for files in a subdirectory show the relative path.

## 4. Non-change scope (you must not touch)

- File names, paths and order of artifacts produced by core (no renaming of mono files; the documented `mono/` layout is a contract).
- `InMemoryArtifact`, `ExportReport`, `exporter::run` (disk export) behaviour, `ExportPlan`, persisted settings.
- ZIP compression method, timestamp logic (`current_zip_modified_time`), the default ZIP file name `favicon-bundle.zip`, the save dialogs.
- `zip` dependency version and features; no new dependency.
- The vestigial or dead code around (RFC 017 owns clean-up), the `app.rs` structure, inline test modules other than what is needed to compile.
- No temp-file-and-rename scheme (the RFC says a single write; do not add more).
- Do not edit RFC files or move them between folders.

## 5. Required behaviour

1. Every `ResultAssetItem` has a unique `relative_path`; for mono items it is `mono/<name>`.
2. "Download all (ZIP)" with monochrome (and with every other optional output enabled) succeeds; the archive contains each artifact exactly once under its relative path.
3. If writing the archive fails for any reason, no file is created or left behind at the target path.
4. `run_in_memory` fails with `AppError::export` if two artifacts share a path (case-insensitive). With all current plan options this never happens.
5. Cards for `mono/` items are distinguishable by their title.
6. Single-file download default name is unchanged (`favicon-32.png` even for the mono item).

## 6. Required tests (write T-Z1 before any production change and show it failing)

Core, `crates/core/tests/artifact_paths.rs`:

| # | Test | Assertion |
|---|---|---|
| C1 | Plan with everything on (`monochrome`, `include_microsoft_app_logos`, `web_manifest = Some`, ICO, apple-touch, svg, html snippet, default PNG sizes plus 16 and 48) from a PNG fixture | all relative paths unique, compared case-insensitively |
| C2 | `ensure_unique_paths` with two identical paths | `Err(AppError::Export { .. })` naming the path |
| C3 | same with `Favicon.ico` vs `favicon.ico` | `Err` |
| C4 | Regression: default-plan artifact path list and order for a PNG source | equals the list recorded **before** your change (capture it first) |
| C5 | Empty slice | `Ok` |

App, `crates/app/src/task_queue/zip_tests.rs`. Build the fixture PNG inside the test with the `image` crate (the app crate depends on it); ingest it with `logolig::services::ingest::ingest_bytes`:

| # | Test | Assertion |
|---|---|---|
| Z1 | Full pipeline: all-options plan → `run_in_memory` → `collect_assets` → `build_zip_bytes` → read back with `zip::ZipArchive` | entry names are exactly the artifact relative paths (`favicon-32.png` **and** `mono/favicon-32.png`, `favicon.ico` and `mono/favicon.ico`, …); entry count equals artifact count; no duplicate names; each entry's bytes equal the artifact bytes. **Must fail on the unmodified code** (record the failure: `Duplicate filename`). |
| Z2 | `collect_assets` classification and paths | mono items: kind `PngMono`/`Ico`, `relative_path` starts with `mono/`; root items: `relative_path == file_name`; separators are `/` |
| Z3 | `collect_assets` path hygiene | a `PathBuf` with a `..` component, an absolute path, and (on Unix) a non-UTF-8 component each return `Err` |
| Z4 | Duplicate `relative_path` handed straight to `build_zip_bytes` | `Err`, no panic |
| Z5 | `write_zip_blocking` into a directory that does not exist | `Err(AppError::Export)`; the target path does not exist afterwards |
| Z6 | `write_zip_blocking` happy path | file exists, readable, entries as Z1 (use a temp file removed on drop, as the existing test does) |
| Z7 | The two existing timestamp tests | unchanged in meaning, still pass |

Manual GUI check (rules: `.git-exclude/rules/project-instructions-gui-common.md`; resize to 1280 × 720; focus the window before typing; screenshots with `niri msg action screenshot-window --id <id> --path <relative path>`, not ImageMagick `import`): enable **Output `mono/` grayscale set** in Customize, drop a PNG, confirm two visually distinguishable cards (`favicon-32.png` and `mono/favicon-32.png`), press "Download all (ZIP)", then `unzip -l` the result and confirm the `mono/` entries; press the download button of a mono card and confirm the dialog proposes `favicon-32.png`. Do the same once with the Microsoft logos and the web manifest also enabled. If any part cannot be run, say which and what risk remains.

## 7. Required evidence

1. Before any change: run the gates (§8) on the base commit; keep the logs.
2. Z1 failing on the unmodified code (log).
3. C4's recorded list (before) and its passing run (after).
4. Gates after the change; the manual GUI evidence; `unzip -l` output.

## 8. Required validation (report exactly what was and was not run)

```
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Baseline at commit `f632996`: fmt clean, clippy clean, 147 tests passed, 0 failed. If you rebase onto another merged branch, record the new baseline first.

## 9. Acceptance criteria

- C1–C5 and Z1–Z7 exist and pass; Z1 shown failing before the fix.
- The manual check shows a working ZIP with `mono/` entries and no partial file after a forced failure (for example save into a directory you then make read-only, or via Z5).
- Docs updated as in §3.
- No change outside §3 (the reviewer will run `git diff --stat` against the base).

## 10. Prohibited shortcuts

- Do not fix the symptom by renaming mono files or skipping duplicates silently.
- Do not weaken or remove the ZIP library's duplicate check.
- Do not change expected values in a test after seeing output; derive them from §5 and §6.
- Do not catch and ignore ZIP or write errors; map them to `AppError::export` with a useful message.
- Do not add a setting, a dependency, or a new persisted field.
- Do not "tidy" surrounding code, split files, or move the existing inline tests (RFC 017).

## 11. Security and compatibility constraints

- Entry names come from core-generated relative paths, but treat them as untrusted anyway: reject `..`, absolute paths and non-UTF-8 (Z3) so a future producer bug can never create a path-traversal ZIP ("zip slip").
- ZIP content for non-mono plans keeps the same entry names as today (documented layout). Persisted settings and the public core API are unchanged except the one additive function.
- Memory: the archive is built in memory; typical bundles are under 1 MB (DEC-003). Do not add buffering beyond that.

## 12. Known risks

- Changing `collect_assets` to return `Result` touches the conversion task path; keep error mapping identical (`ConvertCompleted(Err(..))` already exists).
- The struct literal in `app.rs`'s test helper and the helper in `task_queue.rs` must compile; keep those edits minimal to limit conflicts with the RFC 010 handoff.
- Windows path separators: write a test that exercises the component-joining logic with a `PathBuf` built from components, not only from a literal string.

## 13. Delivery

Branch `rfc-009-zip-integrity`, small commits (tests first, then code, then docs; match the repository's short imperative subject style). Push the **branch**. Do not merge, tag or release; do not change RFC status.

## 14. Required review request

In `.git-exclude/review-request/rfc-009-zip-integrity/` with a `README.md` entry point containing everything required by section 7 of the mid-capability operating instructions, plus: the Z1 "fails before" log; C4's before/after lists; the `unzip -l` output; GUI screenshots; the forced-failure evidence; commands executed with results; skipped checks with reasons; deviations (none expected); and open questions for the architect. Report to the owner only the path of the `README.md`.
