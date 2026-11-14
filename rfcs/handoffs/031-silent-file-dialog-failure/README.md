# Handoffs for RFC 031

Governing RFC: [RFC 031](../../accepted/031-silent-file-dialog-failure.md). The handoffs are execution documents; the RFC remains the authority for design and for lifecycle status. If a handoff conflicts with its RFC, the RFC wins and the handoff is corrected (RFC 000).

- [`implementation-handoff-spike.md`](./implementation-handoff-spike.md) (J12, phase 0): evidence only. Answers whether the Save dialogs double-open, whether a missing portal can be detected without a new package, and whether a never-answering dialog can be reproduced. An implementation handoff follows the architect's decision.

- [`implementation-handoff.md`](./implementation-handoff.md) (J12b): after the spike. One dialog at a time (D1), a conservative notice when no dialog can appear (D2/D3), documentation and packaging text (D4/D5).

- [`implementation-handoff-followup-no-stacked-notice.md`](./implementation-handoff-followup-no-stacked-notice.md) (J12c): the notice does not stack on repeated requests.

Review requests and evidence produced by the implementer are working material and are not stored here.
