//! Rust implementation of the W3C Sanitizer API.
//!
//! The canonical specification is <https://github.com/WICG/sanitizer-api>
//! and the requirements tracked by this crate are in `docs/spec/spec.md`.
//!
//! # Scope
//!
//! This crate is a string-in / string-out HTML sanitizer and does not model
//! browser-specific objects:
//!
//! - s[impl api.out_of_scope.shadow_dom]: shadow roots are not parsed or
//!   walked; the W3C spec step that recurses into a shadow host's shadow root
//!   has no counterpart here.
//! - s[impl api.out_of_scope.live_dom]: there is no `Element.setHTML` or
//!   `Document.parseHTML` surface; inputs and outputs are always `&str` /
//!   `String`.
//! - s[impl api.out_of_scope.modifier_methods]: `allowElement`,
//!   `removeElement`, `allowAttribute`, `removeAttribute`, … from §6.3 of the
//!   W3C spec are not exposed as a public API. The internal remove-element /
//!   remove-attribute steps are used only by the `remove unsafe` baseline pass
//!   (see `sanitize::baseline`).

pub mod config;
mod dom;
pub mod error;
pub mod namespace;
pub mod output;
pub mod sanitize;

pub use config::{
    SanitizerAttribute, SanitizerBuilder, SanitizerConfig, SanitizerElement,
    SanitizerElementWithAttributes, SanitizerPI,
};
pub use error::{ConfigError, SanitizeError};
pub use output::SanitizedOutput;

// s[impl api.sanitize_fn]
// s[impl sanitize.entry.safe]
pub fn sanitize(html: &str) -> Result<SanitizedOutput, SanitizeError> {
    SanitizerConfig::safe_default().sanitize(html)
}

// s[impl api.sanitize_fn]
// s[impl sanitize.entry.unsafe]
pub fn sanitize_unsafe(html: &str) -> Result<SanitizedOutput, SanitizeError> {
    SanitizerConfig::empty().sanitize_unsafe(html)
}
