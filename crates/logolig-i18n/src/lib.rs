//! Translation dictionaries and locale resolution for logolig (v1.5.0).
//!
//! ## Design
//!
//! - **Keys are `logolig_core::MessageKey` enum** — type-safe, not string keys
//! - **Dictionaries are TOML** bundled at compile time via `include_str!`
//! - **Output is `String`** — the UI layer passes it directly to `text()`
//! - **One-way dependency on logolig-core** — translation keys are part of
//!   core's vocabulary; this is an intentional, healthy coupling
//! - **No file I/O and no localStorage abstraction** — dictionaries are
//!   embedded at build time, no runtime loading needed

mod dictionary;
mod locale;
mod translator;

pub use locale::{Locale, detect_system_locale};
pub use translator::Translator;
