# Handoffs for RFC 017

Governing RFC: [RFC 017](../../accepted/017-source-structure-conformance.md). The handoffs are execution documents; the RFC remains the authority for design and for lifecycle status. If a handoff conflicts with its RFC, the RFC wins and the handoff is corrected (RFC 000).

RFC 017 is delivered in three jobs, each reviewed before the next is issued:

- [`implementation-handoff-part-1.md`](./implementation-handoff-part-1.md) (J8a): inline tests to their own files, vestigial code, stale test scaffolding, doc comments.
- [`implementation-handoff-part-2.md`](./implementation-handoff-part-2.md) (J8b): retire the four unreachable ICO messages and deprecate `ThemeMode::next` (Step 0), then split `app.rs`.
- J8c: split `advanced_drawer.rs`, `preview_panel.rs`, `core/tests/exporter.rs` and `app/tests.rs` (after J8b; handoff written after J8b is reviewed).

Review requests and evidence produced by the implementer are working material and are not stored here.
