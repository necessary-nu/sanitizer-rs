use crate::namespace::HTML_NS;

use super::{
    SanitizerAttribute, SanitizerConfig, SanitizerElement, SanitizerElementWithAttributes,
    SanitizerPI,
};

// s[impl config.canonicalize.defaults]
// s[impl config.canonicalize.element]
// s[impl config.canonicalize.element_with_attributes]
// s[impl config.canonicalize.attribute]
// s[impl config.canonicalize.pi]
// s[impl config.canonicalize.booleans]
pub(super) fn canonicalize(
    cfg: &mut SanitizerConfig,
    allow_comments_pis_and_data_attributes: bool,
) {
    if cfg.elements.is_none() && cfg.remove_elements.is_none() {
        cfg.remove_elements = Some(Vec::new());
    }

    if cfg.processing_instructions.is_none() && cfg.remove_processing_instructions.is_none() {
        if allow_comments_pis_and_data_attributes {
            cfg.remove_processing_instructions = Some(Vec::new());
        } else {
            cfg.processing_instructions = Some(Vec::new());
        }
    }

    if cfg.attributes.is_none() && cfg.remove_attributes.is_none() {
        cfg.remove_attributes = Some(Vec::new());
    }

    if let Some(list) = cfg.elements.as_mut() {
        for e in list {
            canonicalize_element_with_attributes(e);
        }
    }
    if let Some(list) = cfg.remove_elements.as_mut() {
        for e in list {
            canonicalize_element(e);
        }
    }
    if let Some(list) = cfg.replace_with_children_elements.as_mut() {
        for e in list {
            canonicalize_element(e);
        }
    }

    if let Some(list) = cfg.processing_instructions.as_mut() {
        for p in list {
            canonicalize_pi(p);
        }
    }
    if let Some(list) = cfg.remove_processing_instructions.as_mut() {
        for p in list {
            canonicalize_pi(p);
        }
    }

    if let Some(list) = cfg.attributes.as_mut() {
        for a in list {
            canonicalize_attribute(a);
        }
    }
    if let Some(list) = cfg.remove_attributes.as_mut() {
        for a in list {
            canonicalize_attribute(a);
        }
    }

    if cfg.comments.is_none() {
        cfg.comments = Some(allow_comments_pis_and_data_attributes);
    }

    if cfg.attributes.is_some() && cfg.data_attributes.is_none() {
        cfg.data_attributes = Some(allow_comments_pis_and_data_attributes);
    }
}

pub(super) fn canonicalize_element(el: &mut SanitizerElement) {
    if el.namespace.is_none() {
        el.namespace = Some(HTML_NS.into());
    } else {
        el.normalize();
    }
}

pub(super) fn canonicalize_element_with_attributes(el: &mut SanitizerElementWithAttributes) {
    if el.namespace.is_none() {
        el.namespace = Some(HTML_NS.into());
    } else {
        el.normalize();
    }
    if let Some(attrs) = el.attributes.as_mut() {
        for a in attrs {
            canonicalize_attribute(a);
        }
    }
    if let Some(attrs) = el.remove_attributes.as_mut() {
        for a in attrs {
            canonicalize_attribute(a);
        }
    }
    if el.attributes.is_none() && el.remove_attributes.is_none() {
        el.remove_attributes = Some(Vec::new());
    }
}

pub(super) fn canonicalize_attribute(attr: &mut SanitizerAttribute) {
    // Default namespace for attributes is null.
    attr.normalize();
}

pub(super) fn canonicalize_pi(pi: &mut SanitizerPI) {
    // Nothing to canonicalize; the struct already matches canonical form.
    let _ = pi;
}
