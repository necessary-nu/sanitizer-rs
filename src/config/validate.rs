use crate::error::ConfigError;

use super::builtins::is_non_replaceable;
use super::{
    SanitizerAttribute, SanitizerConfig, SanitizerElement, SanitizerElementWithAttributes,
};

fn has_duplicates<T: PartialEq>(list: &[T]) -> bool {
    for (i, a) in list.iter().enumerate() {
        for b in &list[i + 1..] {
            if a == b {
                return true;
            }
        }
    }
    false
}

fn has_duplicate_elements_with_attributes(list: &[SanitizerElementWithAttributes]) -> bool {
    for (i, a) in list.iter().enumerate() {
        for b in &list[i + 1..] {
            if a.name == b.name && a.namespace == b.namespace {
                return true;
            }
        }
    }
    false
}

fn element_in_list(el: &SanitizerElement, list: &[SanitizerElement]) -> bool {
    list.iter()
        .any(|e| e.name == el.name && e.namespace == el.namespace)
}

fn element_in_allow_list(el: &SanitizerElement, list: &[SanitizerElementWithAttributes]) -> bool {
    list.iter()
        .any(|e| e.name == el.name && e.namespace == el.namespace)
}

fn attrs_intersect(a: &[SanitizerAttribute], b: &[SanitizerAttribute]) -> bool {
    a.iter().any(|x| b.contains(x))
}

fn attrs_subset(subset: &[SanitizerAttribute], superset: &[SanitizerAttribute]) -> bool {
    subset.iter().all(|x| superset.contains(x))
}

// s[impl config.validate.no_global_mixing_elements]
// s[impl config.validate.no_global_mixing_attributes]
// s[impl config.validate.no_global_mixing_pis]
// s[impl config.validate.data_attributes_requires_allow_list]
// s[impl config.validate.no_duplicates_global]
// s[impl config.validate.replaceable_elements]
// s[impl config.validate.per_element_attributes_allow]
// s[impl config.validate.per_element_attributes_remove]
// s[impl config.validate.per_element_no_duplicates]
pub(super) fn validate(cfg: &SanitizerConfig) -> Result<(), ConfigError> {
    if cfg.elements.is_some() && cfg.remove_elements.is_some() {
        return Err(ConfigError::MixedElementsLists);
    }
    if cfg.processing_instructions.is_some() && cfg.remove_processing_instructions.is_some() {
        return Err(ConfigError::MixedProcessingInstructionsLists);
    }
    if cfg.attributes.is_some() && cfg.remove_attributes.is_some() {
        return Err(ConfigError::MixedAttributesLists);
    }
    if cfg.data_attributes.is_some() && cfg.attributes.is_none() {
        return Err(ConfigError::DataAttributesRequiresAttributesAllowList);
    }

    if let Some(list) = &cfg.elements {
        if has_duplicate_elements_with_attributes(list) {
            return Err(ConfigError::DuplicateElements);
        }
    } else if let Some(list) = &cfg.remove_elements {
        if has_duplicates(list) {
            return Err(ConfigError::DuplicateRemoveElements);
        }
    }

    if let Some(list) = &cfg.replace_with_children_elements {
        if has_duplicates(list) {
            return Err(ConfigError::DuplicateReplaceWithChildren);
        }
        for el in list {
            if is_non_replaceable(el) {
                return Err(ConfigError::ReplaceWithChildrenContainsNonReplaceable(
                    el.display(),
                ));
            }
        }
        if let Some(elements) = &cfg.elements {
            for el in list {
                let simple = el.clone();
                if element_in_allow_list(&simple, elements) {
                    return Err(ConfigError::ReplaceWithChildrenOverlapsElements);
                }
            }
        } else if let Some(remove) = &cfg.remove_elements {
            for el in list {
                if element_in_list(el, remove) {
                    return Err(ConfigError::ReplaceWithChildrenOverlapsRemoveElements);
                }
            }
        }
    }

    if let Some(pis) = &cfg.processing_instructions {
        if has_duplicates(pis) {
            return Err(ConfigError::DuplicateProcessingInstructionTargets);
        }
    } else if let Some(pis) = &cfg.remove_processing_instructions {
        if has_duplicates(pis) {
            return Err(ConfigError::DuplicateRemoveProcessingInstructionTargets);
        }
    }

    if let Some(list) = &cfg.attributes {
        if has_duplicates(list) {
            return Err(ConfigError::DuplicateAttributes);
        }
    } else if let Some(list) = &cfg.remove_attributes {
        if has_duplicates(list) {
            return Err(ConfigError::DuplicateRemoveAttributes);
        }
    }

    let data_attributes_enabled = cfg.data_attributes == Some(true);

    if let Some(global_allow) = &cfg.attributes {
        if data_attributes_enabled && global_allow.iter().any(|a| a.is_data_attribute()) {
            return Err(ConfigError::GlobalAttributesContainsDataAttribute);
        }
        if let Some(elements) = &cfg.elements {
            for el in elements {
                validate_per_element_under_global_allow(el, global_allow, data_attributes_enabled)?;
            }
        }
    } else if let Some(global_remove) = &cfg.remove_attributes {
        if let Some(elements) = &cfg.elements {
            for el in elements {
                validate_per_element_under_global_remove(el, global_remove)?;
            }
        }
    }

    Ok(())
}

fn validate_per_element_under_global_allow(
    el: &SanitizerElementWithAttributes,
    global_allow: &[SanitizerAttribute],
    data_attributes_enabled: bool,
) -> Result<(), ConfigError> {
    if let Some(attrs) = &el.attributes {
        if has_duplicates(attrs) {
            return Err(ConfigError::PerElementAttributesDuplicate(el.display()));
        }
        if attrs_intersect(attrs, global_allow) {
            return Err(ConfigError::PerElementAttributesOverlapGlobalAllow(
                el.display(),
            ));
        }
        if data_attributes_enabled && attrs.iter().any(|a| a.is_data_attribute()) {
            return Err(ConfigError::PerElementAttributesContainsDataAttribute(
                el.display(),
            ));
        }
    }
    if let Some(attrs) = &el.remove_attributes {
        if has_duplicates(attrs) {
            return Err(ConfigError::PerElementRemoveAttributesDuplicate(
                el.display(),
            ));
        }
        if !attrs_subset(attrs, global_allow) {
            return Err(
                ConfigError::PerElementRemoveAttributesNotSubsetOfGlobalAllow(el.display()),
            );
        }
    }
    Ok(())
}

fn validate_per_element_under_global_remove(
    el: &SanitizerElementWithAttributes,
    global_remove: &[SanitizerAttribute],
) -> Result<(), ConfigError> {
    if el.attributes.is_some() && el.remove_attributes.is_some() {
        return Err(ConfigError::PerElementAttributesBothListsPresent(
            el.display(),
        ));
    }
    if let Some(attrs) = &el.attributes {
        if has_duplicates(attrs) {
            return Err(ConfigError::PerElementAttributesDuplicate(el.display()));
        }
        if attrs_intersect(attrs, global_remove) {
            return Err(ConfigError::PerElementAttributesOverlapGlobalRemove(
                el.display(),
            ));
        }
    }
    if let Some(attrs) = &el.remove_attributes {
        if has_duplicates(attrs) {
            return Err(ConfigError::PerElementRemoveAttributesDuplicate(
                el.display(),
            ));
        }
        if attrs_intersect(attrs, global_remove) {
            return Err(ConfigError::PerElementRemoveAttributesOverlapGlobalRemove(
                el.display(),
            ));
        }
    }
    Ok(())
}
