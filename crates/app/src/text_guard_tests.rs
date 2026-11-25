//! Guards on the text of the interface (RFC 016, job K4).
//!
//! Two source tests, both run by `cargo test` (so in CI, with no new tooling):
//! - `hardcoded_literals`: no alphabetic string literal in the application's non-test sources
//!   unless it is on a named allow-list that gives a reason for each group. A user-visible
//!   sentence is a `MessageKey`, in English and Japanese.
//! - `unused_keys`: every `MessageKey` is used by the application or the core crate, is
//!   `#[deprecated]`, or is on the named list of keys kept only because removing a public variant
//!   is a SemVer break (until 2.0).
//!
//! They live in the application crate because they read its sources and the core crate's
//! (`CARGO_MANIFEST_DIR/../core`); the i18n crate cannot see either. `lexer` is the small reader
//! of Rust source they share (comments, strings, raw strings and char literals told apart).

mod hardcoded_literals;
mod lexer;
mod unused_keys;
