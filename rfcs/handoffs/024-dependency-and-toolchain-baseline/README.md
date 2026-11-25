# Handoffs for RFC 024

Governing RFC: [RFC 024](../../done/024-dependency-and-toolchain-baseline.md). The handoffs are execution documents; the RFC remains the authority for design and for lifecycle status. If a handoff conflicts with its RFC, the RFC wins and the handoff is corrected (RFC 000).

Implementation handoff (J1, done) and follow-ups (J1b, done; J2a).

- [`implementation-handoff.md`](./implementation-handoff.md)
- [`implementation-handoff-followup-lock-updates.md`](./implementation-handoff-followup-lock-updates.md)
- [`implementation-handoff-followup-tokio.md`](./implementation-handoff-followup-tokio.md)

Review requests and evidence produced by the implementer are working material and are not stored here.

- [`implementation-handoff-h1-spike.md`](./implementation-handoff-h1-spike.md) (H1, 1.29.0): evidence only. Replacement options for `chrono` (unmaintained, one use) and for the four warnings that come through `vtracer`'s old dependency chain; a recommendation and an outline for the implementation job.
- [`implementation-handoff-h3-replace-chrono.md`](./implementation-handoff-h3-replace-chrono.md) (H3): replace `chrono` with std-only code (no new package, no `unsafe`) and measure whether the ZIP can keep right times through the extended-timestamp field (owner, 2026-11-26: "Security is really important").
