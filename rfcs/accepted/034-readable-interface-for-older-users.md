# RFC 034 — A readable, comfortable interface for older users and low vision

**Status.** Accepted (question from the owner, 2026-11-12: "I wonder the app UI is optimized for elder people"; the answer was: not yet. The owner delegated the four questions to the architect on 2026-11-12 under the philosophy "clean, safe and secure, robust and sophisticated; users must not be confused"; the decisions are below. Release 1.29.0.)
**Priority.** P2 (the owner's philosophy: users must not be confused; the keyboard work of RFC 027 is P1 and comes first)
**Tracks.** Audit of text size, contrast, target size, effects, wording and settings, from the source and the palette constants (no user study).
**Touches.** `crates/app/src/ui/*` (type sizes, paddings), `app.rs` (one `scale_factor` call), Settings page, persisted settings (one new value), i18n texts, `docs/src/ui-a11y.md`. Depends on the headless test level of RFC 035. Interacts with RFC 021 (selection cues), RFC 026 (result layout) and RFC 027 (focus): all four touch the same views, so they are ordered, not parallel.

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

## Requirements (decisions D1–D5 below fix the numbers)

1. **A type scale, not scattered numbers.** All sizes come from named steps in one module (`caption`, `body`, `label`, `title`, `heading`), so a later change is one edit and review sees one place. Decided (D1): **body 16 px, secondary 14 px, caption 13 px (the floor), headings 20 and 28 px** (fewer steps: if the existing views use only two heading sizes, the scale has two). This is a layout-affecting change: card widths, the sidebar width and paddings are adjusted with it and the result is checked at the sizes in RFC 026.
2. **An "Interface size" setting** in Settings (decided, D2: **Normal 100 %, Large 125 %, Extra large 150 %**, shown with words, not percentages, each name rendered at its own size), applied with iced's `scale_factor`, persisted, and available in every language. It makes text, icons, spacing and fixed widths grow together and needs no per-widget work. The default stays Normal. Checked at every step against RFC 026's small-window layout (at 150 % a 1280 × 720 window behaves like 853 × 480 logical pixels).
3. **Higher contrast in the existing themes, no fourth theme** (decided, D3). In Light and Dark **every text colour reaches at least 7:1 (WCAG AAA)**, including secondary text, which becomes a solid colour instead of the body colour at 60 % opacity (`muted_text` in `ui/colors.rs`); the primary button colour is darkened until white text on it reaches 7:1; borders and the focus/selection cues of controls reach 3:1 against their surroundings (WCAG 1.4.11). Caption text (13 px) is never used for information the user needs in order to act.
4. **Target sizes.** Primary actions at least 44 × 44 px (Download all, Choose file, Back, navigation items); all other controls at least 32 px high; checkbox rows are clickable on their label with at least 32 px row height; the SVG quality slider (three positions) is **replaced by three labelled choices** (a radio-like group, consistent with RFC 021), same stored values, so old settings files keep working (decided, D5).
5. **Plain-language labels in Customize** (en and ja): the algorithm and tracing presets get a one-line plain description each ("Sharp: for logos with clean edges"; "Smooth photos…"), the technical names move to a secondary line or a tooltip; "favicon", "ICO" and "apple-touch-icon" are explained once under the section heading. The Customize page leads with the choices most users need (sizes, transparency) and keeps the rest under the existing "Advanced settings". No change to what the options do.
6. **Documentation and honesty.** `ui-a11y.md` states the sizes, contrasts and settings from the finished work, and what is still not met (screen readers, RFC 027 until done).

## Not in scope

Speech output and screen-reader support (iced 0.14 exposes no accessibility tree); changing the product's flow; new languages; a user study. Reduced-motion needs no work: there is no motion.

## Decisions (architect, under the owner's delegation, 2026-11-12)

- **D1 Sizes.** Body 16 px, secondary 14 px, caption 13 px as the floor, used only for text the user does not need in order to act (footer, version). Reason: 16 px is the common comfortable floor for body text; a floor of 13 px keeps the layout workable; larger text is the job of the Interface-size setting, so no second, competing mechanism.
- **D2 Interface size.** Three steps (Normal, Large, Extra large = 100/125/150 %), named in words. No 200 %: at the default 1280 × 720 window it would behave like a 640 × 360 window, below what the layout is designed and tested for; users who need more already have the operating system's display scaling, which the app honours (documented, see below). Fewer choices, each one guaranteed to work. **Robustness rules:** the control that changes the size stays reachable at every step (a headless test at every step; a user who enlarges the interface must always be able to take it back); an unknown or invalid stored value loads as Normal; the mobile breakpoint is evaluated in scaled logical pixels, so at 150 % a window narrower than 1152 physical pixels uses the mobile layout (intended, tested).
- **D3 Contrast.** Raise it in Light and Dark; no fourth theme. Reason: one fewer thing to understand and choose; the benefit reaches everybody; a user with a real need for black-and-white contrast is served by the operating system. Revisit (a separate RFC) only if a user asks for it.
- **D4 Release and order.** Everything in **1.29.0**, in the order: contrast (pure functions, small), then type scale with target sizes and the quality choices, then plain-language labels, then Interface size last, so the steps are defined once on the final scale. Reason: the Interface-size steps and the type scale change what "Normal" means; shipping them in different releases would make a user's chosen step jump in meaning. The views are also still being changed by RFC 021, 026 and 027 in 1.28.0. **Interim, in 1.28.0:** the user documentation says how to enlarge the application with the operating system's display scaling (one paragraph, job J15).
- **D5 Quality control.** The slider becomes three labelled choices (bigger targets, no unlabeled positions, one control style). Stored values unchanged.

## Order and cost

After RFC 021 (cues), 026 (result layout), 027 (focus) and the headless test level (RFC 035). Jobs (ROADMAP J21–J24, in this order): contrast (M), type scale + target sizes + quality choices (L), plain-language Customize labels (M), Interface size (S). Each is one reviewed step; none is released unless the previous ones are complete.

## Tests and evidence

Levels follow the owner's test rules (2026-11-12) and the headless level of RFC 035:

- **Logic and state:** the type scale has no value below the stated minimum; every `text(...).size(...)` call in `ui/` uses a named step (a source-level test that fails on a numeric literal); the scale-factor setting round-trips through persisted settings; old settings load with Normal.
- **Pure functions:** contrast of every text colour pair of every theme against its background, failing below the stated ratio (AAA, 7:1, in both Light and Dark; 3:1 for control borders).
- **Headless UI (every CI run):** target sizes (≥ 44 / ≥ 32 px, the property P3 of RFC 035 with its thresholds raised and its exception list emptied), no overflow and primary controls reachable at every window size, **at every Interface-size step (100/125/150 %)**, in every language, Light and Dark.
- **Real UI (human eyes only):** the worst case (largest Interface size, longest-labelled language, narrowest window) plus one baseline, in Light and Dark, for "does it read well"; every difference described. Nothing else is photographed.
