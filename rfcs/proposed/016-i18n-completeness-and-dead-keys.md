# RFC 016 — i18n completeness and dead-key cleanup

**Status.** Proposed
**Priority.** P3
**Tracks.** Hardcoded English in the UI; unused translation keys; no guard against either.
**Touches.** `crates/core/src/message_key.rs`, `crates/i18n` (dictionary, `en.toml`, `ja.toml`), `crates/app` UI and `app.rs`, `task_queue.rs`, a repository check.

## Summary

Project rules require every user-facing string to go through `logolig-i18n`. A scan found literals that bypass it, and 41 of 148 `MessageKey` variants that nothing references.

## Evidence

Verified by source reading and a scripted scan (references to `MessageKey::<Variant>` in `crates/app/src` and `crates/core/src`).

Hardcoded user-visible English:

- `app.rs` `push_error_toast`: title `"Operation failed"` (the code comment admits it).
- `ui/drop_zone.rs` history card: `format!("{} files", n)`.
- `task_queue.rs`: file-dialog title `"Choose a PNG, SVG, WebP, or JPEG to forge favicons from"`, filter names `"Images"`, `"ZIP"`.
- `ui/preview_panel.rs`: `"Browser tab — 16×16 actual size"`, `"Smartphone home — 60pt @2x"`, caption `"Logolig"`. (Mock site text such as `logolig.example.com` is illustrative content and may stay.)
- `ui/accessibility.rs::label` constants are English, unused, and stale ("PNG, SVG, or WebP" omits JPEG).

Unreferenced variants (41), for example: `DropZoneInstruction`, `ChangeFileButton`, `PreviewNoSource`, `AdvancedTitle`, `AdvancedBlurb`, `SectionFiles`, `IncludeIcoLabel`, `SizeAddButton`, `ExportButton`, `ToggleThemeButton`, `LanguageEnglish`, `GroupWhatToExport`, `TooltipClose`, `SettingsTitle`, `SidebarLabelSettings`, `ToastInvalidColorTitle` / `Body`.
Because the enum and `Dictionary` fields are public/exhaustive, the compiler's `dead_code` lint never reports them, contrary to the claim in `message_key.rs` doc comments.

## Design

1. Add keys for each hardcoded string above (en + ja); `ToastErrorTitle` replaces `"Operation failed"`; the history count uses a placeholder key (`{count}`); dialog titles and filter names are keys.
2. **Dead keys.** For each unreferenced variant decide *remove* (default) or *keep for a planned use*. Remove from the enum, `Dictionary`, `en.toml`, `ja.toml` in one change. Keep `ToastInvalidColor*` only if RFC 012 reuses them.
3. Delete or wire `ui::accessibility::label`. Recommendation: delete; re-introduce when a real screen-reader API exists.
4. **Guard.** Add a test (in the app crate's `tests.rs`) that fails when a `MessageKey` variant has no reference in app/core sources, with an explicit allow-list for deliberately reserved keys. Add a CI grep rule (RFC 020) that flags `text("...")` with alphabetic literals outside an allow-list of symbols/mock content.
5. Keep `en` and `ja` parity (`tests/i18n.rs` already enforces it).

## Tests (design-derived)

- Existing i18n parity tests pass with fewer keys.
- New dead-key test fails when a variant is added without use (verified by temporarily adding one).
- Snapshot-style test that `push_error_toast` uses the translated title in `ja`.

## Acceptance criteria

No alphabetic literal reaches the UI outside the allow-list; zero unreferenced keys other than the documented reserve; `en`/`ja` parity holds.

## Risk

Low. Mechanical. Coordinate with RFC 004 (zh-CN), which multiplies the translation burden by the number of keys; fewer keys is cheaper to translate.
