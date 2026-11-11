# RFC 034 — A readable, comfortable interface for older users and low vision

**Status.** Proposed (question from the owner, 2026-11-12: "I wonder the app UI is optimized for elder people"; the answer is: not yet)
**Priority.** P2 (the owner's philosophy: users must not be confused; the keyboard work of RFC 027 is P1 and comes first)
**Tracks.** Audit of text size, contrast, target size, effects, wording and settings, from the source and the palette constants (no user study).
**Touches.** `crates/app/src/ui/*` (type sizes, paddings), `app.rs` (one `scale_factor` call), Settings page, persisted settings (one new value), i18n texts, `docs/src/ui-a11y.md`. Interacts with RFC 021 (selection cues), RFC 026 (result layout) and RFC 027 (focus): all four touch the same views, so they are ordered, not parallel.

## Summary

The application is simple in structure (three pages; the main task is choose file → look → download) and has several things older users need: no animation, no hover-only features, a visible "Choose file…" button as the alternative to drag and drop, toasts that stay as long as the text takes to read and errors that stay until dismissed. But its **type is small, its target sizes are modest, there is no way to enlarge the interface or raise the contrast, and a part of the Customize page speaks in technical terms**. Nothing here is a defect in the old sense; it is a design that was made for a comfortable desktop reader, not for a reader with weaker eyes or a less steady hand.

## Evidence (source, 2026-11-12)

| Area | Finding |
|---|---|
| Text size | `.size()` values in the UI: 10 px ×1, **11 px ×10**, **12 px ×11**, **13 px ×23**, 14 ×4, 15 ×4, 16 ×1, 17 ×3, 20–28 ×5. The main body text is **13 px**; navigation labels are **11 px** (10 px on the mobile bar); result-card details **11 px**; the app-name footer 11–12 px; descriptions 11–13 px. Common guidance for comfortable reading is 16 px for body text and not below 12–14 px for anything secondary; older readers benefit from more. |
| Resizing | All sizes are fixed logical pixels. The operating system's display scaling (if the platform passes it on) is honoured, but the application offers **no setting of its own** to enlarge the interface. iced 0.14 provides `application(...).scale_factor(|state| f32)`, which scales text, icons, spacing and fixed widths together. |
| Contrast | Body text 21:1 (Light) and 11.1:1 (Dark): very good. **Muted text** (descriptions, labels, card details) is the body colour at 60 % opacity: **5.74:1 (Light)** and **5.02:1 (Dark)**, computed from the iced Light/Dark palette constants. That meets WCAG AA (4.5:1) but not AAA (7:1), and it is applied to the **smallest** text (11–12 px). White on the primary button colour `#5865F2` is **4.61:1**: AA, borderline. Toast colours were fixed to AA in 1.27.0. |
| High contrast | The only themes are System, Light and Dark. No high-contrast theme. snora 0.54 offers AAA presets, but the application builds on iced's own palettes. |
| Target size | Buttons use paddings of 6–12 px around 13–15 px text, so their height is about **30–45 px**; the preset checkboxes are iced's default 16 px box with a 13 px label; the SVG quality slider has a small handle and 11 px end labels; the "Back" button is 13 px text with 8 px padding. WCAG 2.2 asks 24 px at AA and 44 px at AAA; older users and those with tremor benefit from the larger value. |
| Cues | Selection is shown by colour and fill; the ✓ mark has two meanings (RFC 021, accepted). The keyboard cannot operate the interface (RFC 027, accepted). |
| Effects | No animation, no parallax, no auto-advancing content. A drag-hover highlight on the drop zone. Fine. |
| Wording | The main flow is plain ("Choose file…", "Download all (ZIP)", "Back"). Customize shows: "Lanczos3", the tracing preset names ("Sharp", "Default", "PhotoRich"), "favicon", "ICO", "apple-touch-icon", "web manifest", "monochrome", a slider with three unlabeled positions. These are right for a web developer and opaque for a casual user. |
| Layout | Left navigation 90 px wide with icon plus 11 px label; Result cards in three columns of small cards; footer with the app name. Window-size problems at small heights: RFC 026. |

## Requirements (proposed; the owner decides the numbers marked ◆)

1. **A type scale, not scattered numbers.** All sizes come from named steps in one module (`caption`, `body`, `label`, `title`, `heading`), so a later change is one edit and review sees one place. ◆ Proposed: **body 16 px, secondary 14 px, nothing below 13 px, headings 20/24/28 px**. This is a layout-affecting change: card widths, the sidebar width and paddings are adjusted with it and the result is checked at the sizes in RFC 026.
2. **An "Interface size" setting** in Settings (◆ Normal 100 %, Large 125 %, Extra large 150 %, and optionally 200 %), applied with iced's `scale_factor`, persisted, and available in every language. It makes text, icons, spacing and fixed widths grow together and needs no per-widget work. The default stays Normal. Checked at every step against RFC 026's small-window layout (at 150 % a 1280 × 720 window behaves like 853 × 480 logical pixels).
3. **A high-contrast theme** (◆ "High contrast", black/white or the AAA palettes, 7:1 for all text including muted text, no 60 %-opacity text) as a fourth choice next to System/Light/Dark. In every theme, **muted text reaches at least 4.5:1** (it already does) and is not used below 14 px.
4. **Target sizes.** ◆ Primary actions at least 44 × 44 px (Download all, Choose file, Back, navigation items); all other controls at least 32 px high; checkbox rows are clickable on their label with at least 32 px row height; the quality slider gets labeled positions and a larger handle or is replaced by three labelled choices (a radio-like group, consistent with RFC 021).
5. **Plain-language labels in Customize** (en and ja): the algorithm and tracing presets get a one-line plain description each ("Sharp: for logos with clean edges"; "Smooth photos…"), the technical names move to a secondary line or a tooltip; "favicon", "ICO" and "apple-touch-icon" are explained once under the section heading. The Customize page leads with the choices most users need (sizes, transparency) and keeps the rest under the existing "Advanced settings". No change to what the options do.
6. **Documentation and honesty.** `ui-a11y.md` states the sizes, contrasts and settings from the finished work, and what is still not met (screen readers, RFC 027 until done).

## Not in scope

Speech output and screen-reader support (iced 0.14 exposes no accessibility tree); changing the product's flow; new languages; a user study. Reduced-motion needs no work: there is no motion.

## Order and cost

Do after RFC 021 (cues), RFC 026 (result layout) and RFC 027 (focus), because they edit the same views and the type scale and target sizes should be applied once on the final structure. The Interface-size setting (requirement 2) is independent and small; it could ship first (one setting, one `scale_factor` call, one persisted value, tests, Settings texts) as soon as RFC 026 shows the layout survives a 150 % scale. Rough size: requirement 2 S; requirements 1 and 4 L together (every view); requirement 3 M; requirement 5 M (texts in two languages). Proposed for release 1.29.0 with the owner's decision.

## Tests and evidence

- Unit: the type scale has no value below the stated minimum; every `text(...).size(...)` call in `ui/` uses a named step (a source-level test that fails on a numeric literal); the scale-factor setting round-trips through persisted settings; old settings load with Normal.
- Contrast: a test computes the contrast of every text colour pair of every theme against its background and fails below the stated ratio (AA in Light/Dark, AAA in High contrast).
- Target size: measured on screenshots per theme (machine pixel measurement of the control boxes), recorded in the review request.
- GUI: before/after captures at 100 % and 150 % for the Home, Result, Customize and Settings pages at the launch size and at the RFC 026 small-window size; every difference described.

## Questions for the owner

- ◆1 Is **16 px body / 14 px secondary / nothing below 13 px** the right target, or larger?
- ◆2 Which steps for Interface size: 100/125/150 %, with or without 200 %?
- ◆3 A fourth theme "High contrast", or raise contrast in Light and Dark instead (no new choice for the user to understand)?
- ◆4 Release: with 1.28.0 only the Interface-size setting (after RFC 026), the rest in 1.29.0; or everything in 1.29.0?
