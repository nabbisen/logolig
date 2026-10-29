# RFC 009 — Result artifact identity and ZIP integrity

**Status.** Implemented (1.27.0)
**Priority.** P1 (functional defect; a shipped option makes "Download all (ZIP)" fail)
**Handoffs.** [rfcs/handoffs/009-result-artifact-identity-and-zip-integrity/](../handoffs/009-result-artifact-identity-and-zip-integrity/implementation-handoff.md)
**Tracks.** Defect: ZIP download fails and leaves a partial file when monochrome output is enabled.
**Touches.** `crates/app/src/result.rs`, `crates/app/src/task_queue.rs` (`collect_assets`, `write_zip_blocking`), `crates/app/src/ui/result_view.rs`, tests.

## Summary

Result items are identified by file name only. The `mono/` prefix is discarded, so colour and monochrome outputs collide. The ZIP writer then fails on the duplicate name and leaves a partially written archive. This RFC gives every result item its full relative path as identity and makes ZIP writing all-or-nothing.

## Evidence

Verified by source reading; **not executed**.

- `collect_assets` builds `file_name` from `relative_path.file_name()` only, so `mono/favicon-32.png` becomes `favicon-32.png`.
- With `monochrome = true` and the default plan, the item list contains `favicon-32.png`, `favicon-192.png`, `favicon-512.png` and `favicon.ico` twice.
- `zip` 2.4.2 (`Cargo.lock`) `ZipWriter::insert_file_data` returns `InvalidArchive("Duplicate filename")` when a name repeats, so the second `start_file` fails.
- `write_zip_blocking` calls `File::create(path)` before writing, so a failure leaves a truncated file at the user's chosen path.
- The existing ZIP test uses two unique names, so it cannot see this.
- `docs/src/export-spec.md` and the README show the intended on-disk layout with a `mono/` subdirectory.

## Design

1. `ResultAssetItem` gains `relative_path: String` (always `/`-separated, never absolute, no `..`). `file_name` stays for display and for the save-dialog default.
2. `collect_assets` fills `relative_path` from `InMemoryArtifact::relative_path`.
3. `write_zip_blocking` uses `relative_path` as the entry name. The ZIP layout therefore equals the layout of the legacy disk export.
4. **All-or-nothing write.** Build the archive in memory (`Cursor<Vec<u8>>`; typical bundle is under 1 MB per DEC-003) and write it with a single `std::fs::write`. On any error nothing is created.
5. **Uniqueness invariant in core.** `exporter::run_in_memory` verifies that all `relative_path` values are unique and returns `AppError::Export` otherwise. This turns a future collision (for example a new optional output) into a test failure instead of a user-facing ZIP error.
6. Result cards for files under `mono/` show the path (`mono/favicon-32.png`) as their title so two cards are never visually identical. The existing "PNG mono" badge stays; it is not the only distinguishing cue.
7. Individual download: default file name stays the base name (`favicon-32.png`). Open question Q1 below.

## Alternatives considered

- Rename mono files (`favicon-32-mono.png`): changes the documented, user-visible layout and the README guidance on `mask-image`. Rejected.
- Skip duplicates silently: hides data loss. Rejected.

## Binding details fixed while preparing the handoff

- `ResultAssetItem` gains `relative_path: String` (always `/`-separated, relative). `collect_assets` becomes fallible and rejects `..`, absolute and non-UTF-8 components, so a producer bug can never create a path-traversal ("zip slip") archive.
- The archive is built by a pure `build_zip_bytes` and written with a single `std::fs::write`; compression method and timestamp logic are unchanged.
- Card titles show the relative path; single-file save keeps the base name (see Decisions).

## Decisions

- **Q1 — default name in the single-file save dialog: decided (architect, under the owner's delegation, 2026-10-21): keep the base name** (`favicon-32.png`), also for items under `mono/`. This reverses the draft's recommendation. Reason: the README and the generated guidance tell users to reference `/mono/favicon-32.png`, so a different default (`favicon-mono-32.png`) would force them to rename the file, and ZIP, disk layout and single-file names would then follow three schemes. The confusion the draft worried about (overwriting a colour file saved earlier into the same folder) is handled by the operating system's save dialog, which asks before overwriting, and by the card title, which now shows `mono/favicon-32.png`. Revisit only if user feedback shows overwrites in practice.
- **Case-insensitive uniqueness.** The core uniqueness check (Design 5) compares paths case-insensitively, because ZIP extraction onto case-insensitive file systems (Windows, macOS) would otherwise silently overwrite entries.
- **Public API.** `exporter::ensure_unique_paths` is added as an additive public function so the invariant is testable without crafting an impossible plan.

## Tests (design-derived)

- Core: `run_in_memory` with `monochrome + include_microsoft_app_logos + web_manifest + all PNG/ICO` yields unique relative paths.
- Core: injected duplicate is rejected with `AppError::Export`.
- App: building items from a mono-enabled artifact list then writing a ZIP produces entries `favicon-32.png` **and** `mono/favicon-32.png`, readable back with correct bytes.
- App: a failing write (unwritable target directory) leaves no file behind.
- Tests live in `tests.rs` files per the Rust project rules.

## Acceptance criteria

- With monochrome enabled, "Download all (ZIP)" succeeds and the archive contains the `mono/` entries.
- No partial file remains after a failed ZIP write.
- Existing 147 tests still pass.

## Compatibility and release

No persisted-data change. ZIP content for non-mono plans is byte-identical in entry names to today. Suitable for a patch release; release timing is the owner's decision.

## Risk

Low. The change is local to result assembly and ZIP writing.
