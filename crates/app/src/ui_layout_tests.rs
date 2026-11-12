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
//! | `ui_layout_tests/support.rs` | the probe (layout, widget bounds, hover hit-testing), the states and the matrix |
//! | `ui_layout_tests/overflow.rs` | P1: nothing lies beyond the right edge of the window |
//! | `ui_layout_tests/hit_area.rs` | P3: every interactive area is at least 24 × 24 px |
//! | `ui_layout_tests/reachable.rs` | P2: the main controls are fully visible without scrolling |

mod fonts;
mod hit_area;
mod overflow;
mod reachable;
mod support;
