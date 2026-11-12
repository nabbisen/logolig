# Handoffs for RFC 035

Governing RFC: [RFC 035](../../accepted/035-headless-ui-test-level.md). The handoffs are execution documents; the RFC remains the authority for design and for lifecycle status. If a handoff conflicts with its RFC, the RFC wins and the handoff is corrected (RFC 000).

- [`implementation-handoff.md`](./implementation-handoff.md) (JH): spike first (does `iced_test` run without a GPU and see the RFC 026 defect?), then the harness and the first properties over the size/language/theme matrix.

- [`implementation-handoff-followup-hermetic-and-fast.md`](./implementation-handoff-followup-hermetic-and-fast.md) (JH2): the Japanese half of the matrix depended on installed fonts (CI red) and the harness added three minutes to the `test` job: a hermetic font set (a subset of an OFL font, checksummed) and a faster scan, with identical findings.

Review requests and evidence produced by the implementer are working material and are not stored here.
