//! Headless layout tests of the whole interface (RFC 035).
//!
//! The application's `view` is laid out by `iced` without a window, a GPU or a display, in
//! every combination of window size, screen, language and theme, and properties of the layout
//! are checked on each of them. This is the middle of three levels:
//!
//! 1. logic and state: `update` tests (`app/*_tests.rs`);
//! 2. **the rendered tree, headless: these tests**;
//! 3. the real window, only for what needs eyes or a real portal.
//!
//! | File | Holds |
//! |---|---|
//! | `ui_layout_tests/fonts.rs` | the font set (no machine fonts) and the guard that it covers every Japanese character |
//! | `ui_layout_tests/checkboxes.rs` | the checkboxes: Tab stops, and a click, Enter and Space each publish the one message the iced checkbox published |
//! | `ui_layout_tests/contrast.rs` | contrast measured on the drawn pictures: every text piece of every screen in Light and Dark reaches 7:1, every icon 3:1 (RFC 034, J21) |
//! | `ui_layout_tests/contrast_inventory.rs` | the colours the palette and the helpers give, before and after, against 7:1 (text) and 3:1 (borders and marks), in both themes, and that every colour helper is in the inventory |
//! | `ui_layout_tests/controls.rs` | the helper every migrated control is checked with: Tab stop, and Enter, Space and a click each publish the one message |
//! | `ui_layout_tests/controls_customize.rs` | "+ Add custom size", the "Advanced settings" header and "Reset to defaults" as `snora::control`s |
//! | `ui_layout_tests/controls_home.rs` | the drop-zone card and "View results →" as `snora::control`s |
//! | `ui_layout_tests/controls_result.rs` | the Result page's controls: Back, the Save buttons, the preview toggle, Download all |
//! | `ui_layout_tests/controls_nav.rs` | the navigation items (side bar and bottom bar) as `snora::control`s |
//! | `ui_layout_tests/command_keys.rs` | key events into a focused text field: a command chord is a shortcut, never text |
//! | `ui_layout_tests/nav_cue.rs` | the active page in the navigation (filled, bold, accent bar), and no text glyph as a state cue |
//! | `ui_layout_tests/radio_groups.rs` | the single-choice groups: one Tab stop each, the keys and clicks publish the messages the buttons did |
//! | `ui_layout_tests/scroll_areas.rs` | a page scrolls as one: no scroll area inside another, the Customize page has exactly one |
//! | `ui_layout_tests/result_middle.rs` | the Result screen's scrolling middle is at least 200 px high at every matrix size, in both languages, 7 and 15 cards, preview closed and open (J22) |
//! | `ui_layout_tests/ring.rs` | the focus ring, drawn: present, neutral, outside the control (so never merged with a selection mark), in both themes |
//! | `ui_layout_tests/support.rs` | the probe (layout, widget bounds, hover hit-testing), the states and the matrix |
//! | `ui_layout_tests/overflow.rs` | P1: nothing lies beyond the right edge of the window |
//! | `ui_layout_tests/focus_after.rs` | where the focus is after Back, a conversion, a page change, a toast's close, Esc in a field: never lost, never a trap |
//! | `ui_layout_tests/focus_disclosure.rs` | the "Advanced settings" header and the preview toggle keep the focus when they open or close a section (Space, Enter), and Tab goes on from them |
//! | `ui_layout_tests/focus_in_view.rs` | after every Tab and Shift+Tab the focused control is fully visible, on every page of the matrix |
//! | `ui_layout_tests/focus_keys.rs` | Tab and Shift+Tab through the real key path: routed, in order, out of a text field, no wrap |
//! | `ui_layout_tests/manifest_input.rs` | the web manifest form: an icon and a sentence under an invalid field (both languages), no error on an untouched field, the field is still a Tab stop and keeps its focus when the error appears, Enter submits |
//! | `ui_layout_tests/marks.rs` | the selection marks are the accent in both themes, the same colour as the primary fill, drawn and held at snora's 3:1 against the page and the row tints (J21b) |
//! | `ui_layout_tests/hit_area.rs` | P3: every interactive area is at least 24 × 24 px |
//! | `ui_layout_tests/tab_stops.rs` | the Tab stops: in layout order, each one an interactive area, and every interactive area one of them except a named list (J14b empties it) |
//! | `ui_layout_tests/system_theme.rs` | "System" follows the operating system: a report repaints the page, the Settings text in the three states (both languages), the preview's System surface follows the report |
//! | `ui_layout_tests/type_scale.rs` | the five steps ascend and none is below 13 px; no literal size under `ui/` or in `shell.rs` (a source test that can fail); CAPTION only where nothing depends on the text; no navigation label wraps (J22) |
//! | `ui_layout_tests/theme.rs` | the proof that Light and Dark lay out identically (so the properties run on Light only) |
//! | `ui_layout_tests/reachable.rs` | P2: the main controls are fully visible without scrolling |

mod checkboxes;
mod command_keys;
mod contrast;
mod contrast_inventory;
mod controls;
mod controls_customize;
mod controls_home;
mod controls_nav;
mod controls_result;
mod focus_after;
mod focus_disclosure;
mod focus_in_view;
mod focus_keys;
mod fonts;
mod hit_area;
mod manifest_input;
mod marks;
mod nav_cue;
mod overflow;
mod radio_groups;
mod reachable;
mod result_middle;
mod ring;
mod scroll_areas;
mod support;
mod system_theme;
mod tab_stops;
mod theme;
mod type_scale;
