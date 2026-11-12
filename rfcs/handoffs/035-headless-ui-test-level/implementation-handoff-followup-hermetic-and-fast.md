# Developer Handoff — RFC 035 follow-up: a hermetic font set and a faster matrix (job JH2)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (read the **Test Optimization Rules** again; relative paths with `cd`; English only).

Governing RFC: [RFC 035](../../accepted/035-headless-ui-test-level.md). Review of the previous step: `.git-exclude/reviewed/rfc-035-headless-ui-test-followup-review-2026-11-12.md`. On any conflict, stop and send a clarification request.

**`main` is red on CI** (the `test` job: `no_text_is_squeezed_out`, 48 findings, Japanese only; and +3 minutes). Fixing that is the priority; J12 waits. Owner philosophy: clean, safe and secure, robust design. A test that gives different answers on different machines is not a test.

**Workflow:** `main`; `git log origin/main..` **before every push** (the checkout is shared); `git pull --ff-only` first; one commit per step; gates chained with `&&`; wait for each commit's CI before the next push; no tags, no force-push, no history rewriting. The tree is red until step 1 is green; do not push anything else before it.

## 0. Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Is the matrix independent of the fonts installed on the machine? | headless | the full findings list is **byte-identical** (a) normally and (b) with the system fonts hidden (your `bwrap --tmpfs /usr/share/fonts` method), saved as two files and compared with `cmp`; and green on the CI runner |
| Can a Japanese string silently use a glyph the test font lacks? | logic | the guard test of step 1.4 (shown failing once with a deliberately added missing character, then reverted) |
| Does the faster matrix find exactly what the slow one found? | headless | the findings lists before and after are identical (`cmp`), the coverage test and all self-tests pass |
| How long does the `test` job take now? | recorded state | the CI run of each pushed commit (start and end times of the `test` job) against the baseline of 1 min 53 s (`1030de8`) |

No real-window run is needed in this job; say so.

## 1. Step 1 — Hermetic fonts (the fix; one commit)

1. **The probe uses its own font set, and nothing else.** `Probe::new` makes the font system contain exactly: Fira Sans (the face `iced_test` bundles), the Lucide icon font (as today) and one Japanese test font (below). **System fonts are removed from the probe's font database** (find the right call in the `cosmic_text`/`fontdb` API behind `iced_test::renderer::graphics::text::font_system()`; it is process-global, so do it once, before the first interface is built, and make the generic families (sans-serif, serif, monospace, cursive, fantasy) resolve to Fira Sans so no machine font can answer for them). The app's own runtime behaviour is untouched: only the test probe is affected. Explain in the review request how you made sure no test uses the font database before the cleaning.
2. **The Japanese test font:** a **subset of Noto Sans CJK JP (SIL Open Font License 1.1)**, taken from the system's copy (the JP face of the TTC/OTC) with `pyftsubset` (fonttools) or `hb-subset`; record which. Characters: **every character of every Japanese string of the i18n crate** (read them from the crate, do not retype), plus hiragana, katakana, full-width punctuation and ASCII. Target size ≤ 300 KB; if it is larger, report the size and what drives it before committing. Location: `crates/app/test-assets/fonts/` with `OFL.txt` (the licence and the copyright line of the source font), and a short `README.md` there: source font file name and its **sha256**, the subsetting tool and exact command, the **sha256 of the committed subset**, the reason ("test fixture, not shipped"). Provide `scripts/make-test-font.sh` that regenerates the subset deterministically from the strings and the source font (it is run by hand, never by the tests).
3. **Keep it out of the published crate and the binary:** `exclude = ["test-assets/"]` in `crates/app/Cargo.toml`; the font is read by `include_bytes!` inside `#[cfg(test)]` code only. Show `cargo package --list -p logolig-app --allow-dirty | grep -c test-assets` = 0 (the list only; do not publish) and `cargo tree -e normal -p logolig-app` unchanged. No new package in `Cargo.lock`.
4. **Guard test (the font may never silently lack a glyph):** a test that fails, naming the characters and the command to regenerate the font, when any character of any Japanese string is not covered by the subset. Use the facilities already in the tree (shaping through `cosmic_text` and checking for the `.notdef` glyph, or the font database's own cmap access). If the only way is a direct `ttf-parser` edge: it is already in the lock (but listed as unmaintained by `cargo audit`); report the alternative you tried first, and do not add the edge without asking.
5. **Regenerate the `KNOWN` lists once** (`overflow.rs`, and `hit_area.rs` if its Japanese entries change) on the hermetic font. The diff of the lists must be explained entry by entry in the review request; I expect changes only in Japanese entries.
6. **Invariance evidence** (see §0): save `findings-hermetic-normal.txt` and `findings-hermetic-nofonts.txt` and show `cmp` is silent. Then the gates, then the push, then the CI of that commit, which must be **green**.
7. README: change the sentence about the bundled font (Fira Sans, and a Japanese test subset with its checksum in `crates/app/test-assets/fonts/README.md`); say that the headless layout no longer depends on installed fonts.

## 2. Step 2 — A faster matrix (one commit, after step 1 is green on CI)

Goal: the **`test` job on CI at most 60 seconds longer than the baseline (≤ about 2 min 55 s)**; today it is 4 min 56 s, of which the harness is about 177 s on the runner.

Do these, in order, measuring after each (local, with `taskset -c 0,1` as a proxy for the runner's two cores; the CI run is the proof):

1. **Layout is independent of the theme; prove it, then use it.** The hover scan (P3, and the interactive probes) runs once per (screen, size, language) on **Light**. Add one cheap test that proves the claim over the **whole** matrix: for every (screen, size, language) the walk (`Probe::walk`, text/container/scrollable/text-input bounds, no hover scan) is identical between Light and Dark. The theme dimension of the owner's rule is thereby covered by a proof, not dropped. (Colour contrast belongs to RFC 034's pure-function test.)
2. **Fewer probes per piece:** skip a piece when all its probe points fall inside an area already found (the current test does this per point; order the pieces so large containers come after the controls inside them, or skip containers that contain an already-found area). The coverage test is the safety net.
3. **Only if still above the goal:** a coarser probe grid where the coverage test still passes. Do **not** change `[profile.*]` in `Cargo.toml` (it affects every build); if you believe it would help, measure it and report the numbers, I decide.

Acceptance of the step: the findings list is **identical** to the one before the step (`cmp`); the coverage test and every self-test pass; the harness time on the runner is reported from the CI run; if the goal is not reached, report what you tried and the numbers, and stop.

## 3. Required evidence (review request: update `.git-exclude/review-request/rfc-035-headless-ui-test/README.md` with a new section 10; report only its path)

The §0 table filled in; the size, checksum and licence of the font; the explanation of every `KNOWN` change; `cmp` outputs; the CI runs by id with the `test` job durations; gates chained on every pushed commit (as in earlier jobs, the four existing doc warnings expected; audit 0 / 11); the test-list diff (new tests named, none removed).

## 4. Acceptance criteria

CI green on the `main` head (all five jobs); the matrix gives the same findings with and without system fonts; the guard test exists and has been shown failing once; the font is not in the published package nor in the binary; the `test` job within about 60 s of the baseline or a reasoned report; production code unchanged; no new package in `Cargo.lock`.

## 5. Prohibited shortcuts

No dropping or skipping of Japanese; no `#[ignore]`; no loosened thresholds; no change to `.github/workflows/`; no `[profile]` change without asking; no `KNOWN` entries beyond the regenerated ones; no system-font dependency left in the probe; no tags, force-push or history rewriting.
