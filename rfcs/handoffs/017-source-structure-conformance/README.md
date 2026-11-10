# Handoffs for RFC 017

Governing RFC: [RFC 017](../../accepted/017-source-structure-conformance.md). The handoffs are execution documents; the RFC remains the authority for design and for lifecycle status. If a handoff conflicts with its RFC, the RFC wins and the handoff is corrected (RFC 000).

RFC 017 is delivered in three jobs, each reviewed before the next is issued:

- [`implementation-handoff-part-1.md`](./implementation-handoff-part-1.md) (J8a): inline tests to their own files, vestigial code, stale test scaffolding, doc comments.
- J8b: split `app.rs` (handoff written after J8a is reviewed, from the real file layout).
- J8c: split `advanced_drawer.rs`, `preview_panel.rs`, `core/tests/exporter.rs` (after J8b).

Review requests and evidence produced by the implementer are working material and are not stored here.
