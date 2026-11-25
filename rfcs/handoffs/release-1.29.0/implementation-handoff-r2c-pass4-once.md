# Developer Handoff — Release 1.29.0, job R2c: pass 4 once, on the fixed binary

Role and rules as always. **These are my words for exactly one run of pass 4 (the invalid manifest colour in a real window), once, on a release binary built from the head of `main` after `53aaf73`** (build from a `git archive` into a fresh directory; report the sha256). Review of R2b: `.git-exclude/reviewed/release-1.29.0-r2b-review-2026-11-26.md`.

## Work
1. The corrected `evidence/real/manifest.sh` of R2b (the "box empty before typing" assertion removed). **Every number comes from the headless layout of this tree** (`headless-expected-geometry.txt`: after typing, ≥ 500 dark px in the box x 312..540, y 998..1044); if you add any assertion, derive its number from the headless render **first**, and state it in the report before the run. **Do not add a state-before-typing assertion**: the field at the bottom edge before typing is the defect's own picture, not a failure.
2. Replay test first (ALL REPLAY CASES BEHAVE); the usual discipline (unlocked session, isolated XDG, one instance by pid, never start the binary except through the isolated session script, fixed size only at launch, focus before every key, second signal after every step, owner's `settings.json` mtime before and after, first anomaly stops). The owner is asked not to start the other project's prototype during the run.
3. The run: Customize; 14 Tabs; Space opens Advanced; 8 Tabs to Theme color; type `not-a-colour`: **the icon and the sentence are in the capture, inside the visible area** (also one **crop of the sentence at full size** for my eyes); the file never contains the text; Tab out: nothing saved; Shift+Tab back: the line is in view again; select all, type `#abc`: the file reads `#AABBCC` and the line goes. One capture per step.
4. **If it stops again, stop and report; do not repeat.** A stop on your own expectation after this handoff is a third occurrence: say so plainly.

## Prohibited
Any code change (if the run shows the line **not** in view, report with the capture: that is a defect for me, not for you to fix); tags, releases, crates.io, submissions; edits to `rfcs/`, `ROADMAP.md`, notes; a second run; `#[allow]`; absolute `cd`; resizing a running window; scratch left in `/tmp`.

## Delivery
`.git-exclude/review-request/release-1.29.0-r2c/README.md` (report only its path): the binary's sha256, replay output, the numbers expected (from headless) and measured, the captures, the settings-file reads, anything not verified.
