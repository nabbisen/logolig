# Handoffs for RFC 027

Governing RFC: [RFC 027](../../accepted/027-keyboard-focus-model.md). The handoffs are execution documents; the RFC remains the authority for design and for lifecycle status. If a handoff conflicts with its RFC, the RFC wins and the handoff is corrected (RFC 000).

- [`implementation-handoff-phase-0-spike.md`](./implementation-handoff-phase-0-spike.md) (J13, with the shared session script of the former job JS): evidence only. Option A (app-owned model) against Option D (snora's engine) and the hybrid; the four step handoffs follow the architect's review.

- [`implementation-handoff-ctrl-digit-in-field.md`](./implementation-handoff-ctrl-digit-in-field.md) (J13b): a command key must never type into a text field (reproduced defect).

- [`implementation-handoff-j14a-engine.md`](./implementation-handoff-j14a-engine.md) (J14a): Tab is routed to snora's `traverse`, the focused control stays in view, Customize is one scroll area; headless properties. The page steps (J14b) follow.

- [`implementation-handoff-j14b-pages.md`](./implementation-handoff-j14b-pages.md) (J14b): every remaining control becomes a `snora::control`, in five reviewed steps (navigation, Home, Result, Customize, Settings and docs), with two real key runs at the end.

Review requests and evidence produced by the implementer are working material and are not stored here.
