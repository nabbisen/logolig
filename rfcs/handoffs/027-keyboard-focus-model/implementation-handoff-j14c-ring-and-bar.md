# Developer Handoff — RFC 027/021 J14c: a focused navigation item must not hide its accent bar; the shared real-run verification (small)

Role and rules as always (`.git-exclude/roles/*`, `.git-exclude/rules/*`; GUI Evidence and Test Optimization rules; relative paths with `cd`). Governing texts: RFC 021 (Q1 and D7: the active page has a filled item, a bold label and a thin accent bar toward the content) and RFC 027 (the ring is 2 px, offset 2 px outside the control). Review: `.git-exclude/reviewed/rfc-027-j14b-pages-review-2026-11-21.md` (addenda). **Workflow:** `main`; `git pull --ff-only`; `git log origin/main..` before every push; one commit per step; gates chained; CI waited for; no tags, force-push or history rewriting. **No real-window run in this job.**

## Why

Your attempt 4 found a real conflict between two cues: the ring of a focused **active** navigation item (2 px, drawn 2 px outside the item) lands on the item's 3 px accent bar (x 104–107 px in the capture: 104 accent, 105–106 black ring, 107 accent), hiding two thirds of the bar. The page is still shown by the fill and the bold label, but the cue the owner chose (the bar) is damaged exactly when a keyboard user looks at it. Both cues must be whole at the same time.

## Step 1 — the fix (tests first)

1. **Test, red first:** in `ui_layout_tests/nav_cue.rs` (or a new sibling file) a pixel test: the **focused active** item (focus the item by Tab through the real key path, as the J14a tests do) on Home, Customize and Settings × desktop/mobile × Light/Dark: the bar is **intact** (3 px × the item's length, all accent colour, none of it covered by ring pixels), and the ring is complete on all four sides. It must fail on today's tree (show it).
2. **Fix:** leave a clear gap between the item and the bar so the ring fits between them: the ring needs the item's edge plus **4 px** (2 px offset + 2 px ring) before the bar starts. Do it in `ui/sidebar.rs` (the sidebar's item column gets the room on the side facing the bar; on the phone the bottom bar's item gets it below the bar), keeping the bar flush with the content edge, 3 px, as now. Do not move the ring (it is snora's) and do not thicken or recolour the bar. Report the item's new width/height and anything that moves by pixels (the sidebar is 90 px wide: say what happens to the item and label widths in Japanese at the narrowest size; the layout properties P1–P3 and the theme proof must stay green).
3. Snapshots of every other page byte-identical is **not** expected (the item narrows): `cmp` before/after, look at the differing pictures only (Home desktop and phone, one theme each, English; plus the Japanese phone case), describe the difference.

## Step 2 — the shared verification pattern (small commit, shell and tests only)

Put into the shared scripts (`scripts/gui-session.sh` or a sibling `scripts/gui-verify.sh`): `capture_scale` (capture width ÷ the window's logical width from `niri msg -j windows` on Wayland, 1 on X11, printed in the log) and the **replay pattern**: verification functions take **file paths** (the run feeds live captures, a test feeds saved ones), written with `set -u` and `local` for every helper variable. Test them on **existing PNGs** from your attempts (the `real-wayland-*` captures: a ring is found where it exists, the Settings bar signal reads as it did, a missing ring is reported). Re-point `real-run-wayland.sh` and `real-run-x11.sh` at the shared functions. `bash -n` and the replay test are the gate. This is the shared fix for J10 run 1, J14b run 1 and attempts 2–4 (four script-side stops).

## Step 3 — docs (one small commit)

`docs/src/ui-a11y.md`, "How it is checked": add what attempt 4 **actually verified** in the real window (native Wayland, `wtype`, niri, English, Light): Tab 1–4 rings on each stop; **Enter on the focused drop-zone card opens exactly one chooser** and Esc closes it; Ctrl+2; Tab ×4 and **Space ticks a checkbox**; Shift+Tab moves the ring back; Ctrl+3. Say plainly that the Settings arrow-key step and the X11 text-field steps were not run in a real window for this release candidate and are covered by the automated tests on the real key path; they are repeated on the release commit (below). Nothing is claimed that was not run.

## What is NOT done in this job and why

The remaining real passes (Wayland: Settings theme group, ↓ ↑ repaint; X11: Tab out of the field keeps `12`, Ctrl+1 in the field goes Home, Esc in the field) **move to the release-commit pass** of the release checklist (once per release, on the release commit), using the fixed, replay-tested scripts of Step 2. The real-run budget of J14b is spent: **a re-sent review pointer is not leave** (the owner re-sends the pointer whenever a request file changes; the architect's mistake was not saying so in the first place). From now on a repeat of a real run needs a reply in words from the architect.

## Evidence, acceptance, prohibited

Review request: `.git-exclude/review-request/rfc-027-j14c/README.md` (report only its path). Acceptance: the new pixel test red then green; every cue whole when focused; all earlier tests green; shared functions replay-tested on saved PNGs; docs say only what was run; gates and CI per commit. Prohibited: changing the ring, the bar's thickness or colour, `shortcuts.rs`, any real-window run, new `#[allow]`, new packages, tags, force-push or history rewriting.
