use crate::error::ConfigError;

use super::{
    SanitizerAttribute, SanitizerConfig, SanitizerElement, SanitizerElementWithAttributes,
    SanitizerPI,
};

// Which side of the allow-xor-remove split a category has committed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Unset,
    Allow,
    Remove,
}

/// Fluent builder for [`SanitizerConfig`].
///
/// The builder enforces the W3C spec's "allow-list xor remove-list" invariant
/// at build time: the first call to `allow_*` locks the category into
/// allow-list mode, and a later `remove_*` call on the same category returns
/// an error from [`SanitizerBuilder::build`]. This mirrors §6.3 of the spec's
/// modifier methods without exposing live-mutation semantics.
#[derive(Debug, Clone, Default)]
pub struct SanitizerBuilder {
    elements_mode: Mode,
    attributes_mode: Mode,
    pis_mode: Mode,

    allow_elements: Vec<SanitizerElementWithAttributes>,
    remove_elements: Vec<SanitizerElement>,
    replace_with_children: Vec<SanitizerElement>,

    allow_attributes: Vec<SanitizerAttribute>,
    remove_attributes: Vec<SanitizerAttribute>,

    allow_pis: Vec<SanitizerPI>,
    remove_pis: Vec<SanitizerPI>,

    comments: Option<bool>,
    data_attributes: Option<bool>,
    max_arena_bytes: Option<usize>,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Unset
    }
}

impl SanitizerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_element(mut self, el: SanitizerElementWithAttributes) -> Self {
        self.elements_mode = match self.elements_mode {
            Mode::Remove => Mode::Remove,
            _ => Mode::Allow,
        };
        self.allow_elements.push(el);
        self
    }

    pub fn remove_element(mut self, el: SanitizerElement) -> Self {
        self.elements_mode = match self.elements_mode {
            Mode::Allow => Mode::Allow,
            _ => Mode::Remove,
        };
        self.remove_elements.push(el);
        self
    }

    pub fn replace_with_children(mut self, el: SanitizerElement) -> Self {
        self.replace_with_children.push(el);
        self
    }

    pub fn allow_attribute(mut self, a: SanitizerAttribute) -> Self {
        self.attributes_mode = match self.attributes_mode {
            Mode::Remove => Mode::Remove,
            _ => Mode::Allow,
        };
        self.allow_attributes.push(a);
        self
    }

    pub fn remove_attribute(mut self, a: SanitizerAttribute) -> Self {
        self.attributes_mode = match self.attributes_mode {
            Mode::Allow => Mode::Allow,
            _ => Mode::Remove,
        };
        self.remove_attributes.push(a);
        self
    }

    pub fn allow_processing_instruction(mut self, pi: SanitizerPI) -> Self {
        self.pis_mode = match self.pis_mode {
            Mode::Remove => Mode::Remove,
            _ => Mode::Allow,
        };
        self.allow_pis.push(pi);
        self
    }

    pub fn remove_processing_instruction(mut self, pi: SanitizerPI) -> Self {
        self.pis_mode = match self.pis_mode {
            Mode::Allow => Mode::Allow,
            _ => Mode::Remove,
        };
        self.remove_pis.push(pi);
        self
    }

    pub fn comments(mut self, on: bool) -> Self {
        self.comments = Some(on);
        self
    }

    pub fn data_attributes(mut self, on: bool) -> Self {
        self.data_attributes = Some(on);
        self
    }

    /// Cap the bump arena that backs each sanitize call at `n` bytes.
    /// Oversized inputs return [`SanitizeError::AllocationLimit`] instead of
    /// being allowed to grow without bound.
    pub fn max_arena_bytes(mut self, n: usize) -> Self {
        self.max_arena_bytes = Some(n);
        self
    }

    /// Finalizes the builder. Returns `Err` if the builder mixed allow and
    /// remove in any category, or if the resulting config fails validation
    /// after canonicalization.
    pub fn build(self) -> Result<SanitizerConfig, ConfigError> {
        if !self.allow_elements.is_empty() && !self.remove_elements.is_empty() {
            return Err(ConfigError::MixedElementsLists);
        }
        if !self.allow_attributes.is_empty() && !self.remove_attributes.is_empty() {
            return Err(ConfigError::MixedAttributesLists);
        }
        if !self.allow_pis.is_empty() && !self.remove_pis.is_empty() {
            return Err(ConfigError::MixedProcessingInstructionsLists);
        }

        let mut cfg = SanitizerConfig::default();
        match self.elements_mode {
            Mode::Allow => cfg.elements = Some(self.allow_elements),
            Mode::Remove => cfg.remove_elements = Some(self.remove_elements),
            Mode::Unset => {}
        }
        if !self.replace_with_children.is_empty() {
            cfg.replace_with_children_elements = Some(self.replace_with_children);
        }
        match self.attributes_mode {
            Mode::Allow => cfg.attributes = Some(self.allow_attributes),
            Mode::Remove => cfg.remove_attributes = Some(self.remove_attributes),
            Mode::Unset => {}
        }
        match self.pis_mode {
            Mode::Allow => cfg.processing_instructions = Some(self.allow_pis),
            Mode::Remove => cfg.remove_processing_instructions = Some(self.remove_pis),
            Mode::Unset => {}
        }
        cfg.comments = self.comments;
        cfg.data_attributes = self.data_attributes;
        cfg.max_arena_bytes = self.max_arena_bytes;

        // We do not canonicalize here — callers consume the builder's output
        // via SanitizerConfig::sanitize / sanitize_unsafe, which canonicalize
        // with the right flag. But we validate the raw shape to catch
        // developer mistakes as early as possible.
        let mut probe = cfg.clone();
        probe.canonicalize(false);
        probe.validate()?;
        Ok(cfg)
    }
}
