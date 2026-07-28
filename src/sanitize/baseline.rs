use crate::config::{
    SanitizerAttribute, SanitizerConfig, SanitizerElement, SanitizerElementWithAttributes,
    event_handler_attributes, safe_baseline_configuration,
};

// s[impl sanitize.baseline.elements]
// s[impl sanitize.baseline.attributes]
pub(super) fn remove_unsafe(cfg: &mut SanitizerConfig) {
    let baseline = safe_baseline_configuration();

    if let Some(elements_to_remove) = baseline.remove_elements.as_ref() {
        for el in elements_to_remove {
            remove_an_element(cfg, el);
        }
    }

    if let Some(attrs_to_remove) = baseline.remove_attributes.as_ref() {
        for attr in attrs_to_remove {
            remove_an_attribute(cfg, attr);
        }
    }

    for attr in event_handler_attributes() {
        remove_an_attribute(cfg, attr);
    }
}

fn contains_element(list: &[SanitizerElement], el: &SanitizerElement) -> bool {
    list.iter()
        .any(|e| e.name == el.name && e.namespace == el.namespace)
}

fn remove_from_allow_list(
    list: &mut Vec<SanitizerElementWithAttributes>,
    el: &SanitizerElement,
) -> bool {
    let before = list.len();
    list.retain(|e| !(e.name == el.name && e.namespace == el.namespace));
    list.len() != before
}

fn remove_from_element_list(list: &mut Vec<SanitizerElement>, el: &SanitizerElement) -> bool {
    let before = list.len();
    list.retain(|e| !(e.name == el.name && e.namespace == el.namespace));
    list.len() != before
}

fn remove_from_attr_list(list: &mut Vec<SanitizerAttribute>, attr: &SanitizerAttribute) -> bool {
    let before = list.len();
    list.retain(|a| !(a.name == attr.name && a.namespace == attr.namespace));
    list.len() != before
}

// Spec: "remove an element" from a config (§6.3.1 in index.bs).
//
// Deviation from §6.3.1: an explicit entry in the user's allow-list
// (`cfg.elements`) overrides the baseline removal. The spec strips
// the entry; we keep it. Callers who allow-list `<use>` or `<symbol>`
// (e.g. for SVG icon-set output from typst-svg) have made an
// informed decision; the baseline shouldn't second-guess them. The
// attribute-level baseline still applies — `on*` event handlers,
// `javascript:`-scheme URLs, etc. are still scrubbed even on
// explicitly-allowed elements.
fn remove_an_element(cfg: &mut SanitizerConfig, el: &SanitizerElement) {
    if let Some(list) = cfg.elements.as_ref() {
        if list
            .iter()
            .any(|e| e.name == el.name && e.namespace == el.namespace)
        {
            return;
        }
    }
    if let Some(list) = cfg.replace_with_children_elements.as_mut() {
        remove_from_element_list(list, el);
    }
    if let Some(list) = cfg.elements.as_mut() {
        remove_from_allow_list(list, el);
        return;
    }
    if let Some(list) = cfg.remove_elements.as_mut() {
        if !contains_element(list, el) {
            list.push(el.clone());
        }
    }
}

// Spec: "remove an attribute" from a config.
fn remove_an_attribute(cfg: &mut SanitizerConfig, attr: &SanitizerAttribute) {
    if let Some(list) = cfg.attributes.as_mut() {
        let removed_global = remove_from_attr_list(list, attr);
        if let Some(elements) = cfg.elements.as_mut() {
            for el in elements {
                if let Some(attrs) = el.attributes.as_mut() {
                    remove_from_attr_list(attrs, attr);
                }
                if let Some(attrs) = el.remove_attributes.as_mut() {
                    if removed_global {
                        remove_from_attr_list(attrs, attr);
                    }
                }
            }
        }
        return;
    }
    if let Some(list) = cfg.remove_attributes.as_mut() {
        if !list
            .iter()
            .any(|a| a.name == attr.name && a.namespace == attr.namespace)
        {
            if let Some(elements) = cfg.elements.as_mut() {
                for el in elements {
                    if let Some(attrs) = el.attributes.as_mut() {
                        remove_from_attr_list(attrs, attr);
                    }
                    if let Some(attrs) = el.remove_attributes.as_mut() {
                        remove_from_attr_list(attrs, attr);
                    }
                }
            }
            list.push(attr.clone());
        }
    }
}
