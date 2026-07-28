use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

pub mod attribute;
pub mod element;
pub mod pi;

mod builder;
mod builtins;
mod canonicalize;
mod validate;

pub use attribute::SanitizerAttribute;
pub use builder::SanitizerBuilder;
pub use element::{SanitizerElement, SanitizerElementWithAttributes};
pub use pi::SanitizerPI;

pub(crate) use builtins::{
    ANIMATING_URL_ATTRIBUTES, NAVIGATING_URL_ATTRIBUTES, UrlAttr, event_handler_attributes,
    safe_baseline_configuration, safe_default_configuration,
};

// s[impl config.types.sanitizer_config]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SanitizerConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elements: Option<Vec<SanitizerElementWithAttributes>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remove_elements: Option<Vec<SanitizerElement>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replace_with_children_elements: Option<Vec<SanitizerElement>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processing_instructions: Option<Vec<SanitizerPI>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remove_processing_instructions: Option<Vec<SanitizerPI>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<SanitizerAttribute>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remove_attributes: Option<Vec<SanitizerAttribute>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comments: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_attributes: Option<bool>,

    /// Optional upper bound on bytes that the bump arena may allocate per
    /// sanitize call. When set, inputs that would cause the arena to grow
    /// past this limit fail fast with [`SanitizeError::AllocationLimit`]
    /// instead of letting the process balloon. Intended for server contexts
    /// where user-supplied HTML is sanitized on a shared host.
    #[serde(skip)]
    pub max_arena_bytes: Option<usize>,
}

impl SanitizerConfig {
    // s[impl api.config_presets]
    pub fn empty() -> Self {
        Self::default()
    }

    // s[impl config.builtin.default]
    // s[impl api.config_presets]
    pub fn safe_default() -> Self {
        safe_default_configuration().clone()
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        validate::validate(self)
    }

    // s[impl config.canonicalize.defaults]
    pub fn canonicalize(&mut self, allow_comments_pis_and_data_attributes: bool) {
        canonicalize::canonicalize(self, allow_comments_pis_and_data_attributes);
    }

    // s[impl api.config_methods]
    // s[impl sanitize.entry.safe]
    pub fn sanitize(
        &self,
        html: &str,
    ) -> Result<crate::output::SanitizedOutput, crate::error::SanitizeError> {
        let mut cfg = self.clone();
        cfg.canonicalize(false);
        cfg.validate()?;
        crate::sanitize::sanitize_with(&cfg, html, true)
    }

    // s[impl api.config_methods]
    // s[impl sanitize.entry.unsafe]
    pub fn sanitize_unsafe(
        &self,
        html: &str,
    ) -> Result<crate::output::SanitizedOutput, crate::error::SanitizeError> {
        let mut cfg = self.clone();
        cfg.canonicalize(true);
        cfg.validate()?;
        crate::sanitize::sanitize_with(&cfg, html, false)
    }
}
