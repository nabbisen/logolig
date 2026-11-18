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
//! | `ui_layout_tests/command_keys.rs` | key events into a focused text field: a command chord is a shortcut, never text |
//! | `ui_layout_tests/radio_groups.rs` | the single-choice groups: one Tab stop each, the keys and clicks publish the messages the buttons did |
//! | `ui_layout_tests/ring.rs` | the focus ring, drawn: present, neutral, outside the control (so never merged with a selection mark), in both themes |
//! | `ui_layout_tests/support.rs` | the probe (layout, widget bounds, hover hit-testing), the states and the matrix |
//! | `ui_layout_tests/overflow.rs` | P1: nothing lies beyond the right edge of the window |
//! | `ui_layout_tests/hit_area.rs` | P3: every interactive area is at least 24 × 24 px |
//! | `ui_layout_tests/theme.rs` | the proof that Light and Dark lay out identically (so the properties run on Light only) |
//! | `ui_layout_tests/reachable.rs` | P2: the main controls are fully visible without scrolling |

mod checkboxes;
mod command_keys;
mod fonts;
mod hit_area;
mod overflow;
mod radio_groups;
mod reachable;
mod ring;
mod support;
mod theme;
