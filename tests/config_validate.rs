use sanitizer::{
    ConfigError, SanitizerAttribute, SanitizerConfig, SanitizerElement,
    SanitizerElementWithAttributes, SanitizerPI,
};

fn canonicalized(mut cfg: SanitizerConfig, allow_cpi: bool) -> SanitizerConfig {
    cfg.canonicalize(allow_cpi);
    cfg
}

// s[verify config.validate.no_global_mixing_elements]
#[test]
fn mixing_elements_and_remove_elements_is_rejected() {
    let cfg = SanitizerConfig {
        elements: Some(vec![SanitizerElementWithAttributes::html("a")]),
        remove_elements: Some(vec![SanitizerElement::html("b")]),
        ..Default::default()
    };
    let cfg = canonicalized(cfg, false);
    assert_eq!(cfg.validate(), Err(ConfigError::MixedElementsLists));
}

// s[verify config.validate.no_global_mixing_attributes]
#[test]
fn mixing_attributes_and_remove_attributes_is_rejected() {
    let cfg = SanitizerConfig {
        attributes: Some(vec![SanitizerAttribute::new("id")]),
        remove_attributes: Some(vec![SanitizerAttribute::new("class")]),
        ..Default::default()
    };
    let cfg = canonicalized(cfg, false);
    assert_eq!(cfg.validate(), Err(ConfigError::MixedAttributesLists));
}

// s[verify config.validate.no_global_mixing_pis]
#[test]
fn mixing_pi_lists_is_rejected() {
    let cfg = SanitizerConfig {
        processing_instructions: Some(vec![SanitizerPI::new("x")]),
        remove_processing_instructions: Some(vec![SanitizerPI::new("y")]),
        ..Default::default()
    };
    let cfg = canonicalized(cfg, false);
    assert_eq!(
        cfg.validate(),
        Err(ConfigError::MixedProcessingInstructionsLists)
    );
}

// s[verify config.validate.data_attributes_requires_allow_list]
#[test]
fn data_attributes_requires_attributes_allow_list() {
    let cfg = SanitizerConfig {
        data_attributes: Some(true),
        ..Default::default()
    };
    // Don't canonicalize because canonicalize would never set data_attributes
    // without attributes; we're explicitly testing the rejection path.
    assert_eq!(
        cfg.validate(),
        Err(ConfigError::DataAttributesRequiresAttributesAllowList)
    );
}

// s[verify config.validate.no_duplicates_global]
#[test]
fn duplicate_remove_elements_are_rejected() {
    let cfg = SanitizerConfig {
        remove_elements: Some(vec![
            SanitizerElement::html("script"),
            SanitizerElement::html("script"),
        ]),
        ..Default::default()
    };
    let cfg = canonicalized(cfg, false);
    assert_eq!(cfg.validate(), Err(ConfigError::DuplicateRemoveElements));
}

// s[verify config.validate.no_duplicates_global]
#[test]
fn duplicate_processing_instruction_targets_are_rejected() {
    let cfg = SanitizerConfig {
        processing_instructions: Some(vec![SanitizerPI::new("x"), SanitizerPI::new("x")]),
        ..Default::default()
    };
    let cfg = canonicalized(cfg, false);
    assert_eq!(
        cfg.validate(),
        Err(ConfigError::DuplicateProcessingInstructionTargets)
    );
}

// s[verify config.validate.replaceable_elements]
#[test]
fn replace_with_children_rejects_html_root() {
    let cfg = SanitizerConfig {
        replace_with_children_elements: Some(vec![SanitizerElement::html("html")]),
        ..Default::default()
    };
    let cfg = canonicalized(cfg, false);
    let err = cfg.validate().unwrap_err();
    assert!(matches!(
        err,
        ConfigError::ReplaceWithChildrenContainsNonReplaceable(_)
    ));
}

// s[verify config.validate.replaceable_elements]
#[test]
fn replace_with_children_rejects_svg_root() {
    let cfg = SanitizerConfig {
        replace_with_children_elements: Some(vec![SanitizerElement::svg("svg")]),
        ..Default::default()
    };
    let cfg = canonicalized(cfg, false);
    assert!(matches!(
        cfg.validate(),
        Err(ConfigError::ReplaceWithChildrenContainsNonReplaceable(_))
    ));
}

// s[verify config.validate.replaceable_elements]
#[test]
fn replace_with_children_overlaps_elements_is_rejected() {
    let cfg = SanitizerConfig {
        elements: Some(vec![SanitizerElementWithAttributes::html("span")]),
        replace_with_children_elements: Some(vec![SanitizerElement::html("span")]),
        ..Default::default()
    };
    let cfg = canonicalized(cfg, false);
    assert_eq!(
        cfg.validate(),
        Err(ConfigError::ReplaceWithChildrenOverlapsElements)
    );
}

// s[verify config.validate.per_element_attributes_allow]
#[test]
fn per_element_attributes_overlap_global_allow_is_rejected() {
    let cfg = SanitizerConfig {
        attributes: Some(vec![SanitizerAttribute::new("id")]),
        elements: Some(vec![
            SanitizerElementWithAttributes::html("a")
                .with_attributes(vec![SanitizerAttribute::new("id")]),
        ]),
        ..Default::default()
    };
    let cfg = canonicalized(cfg, false);
    assert!(matches!(
        cfg.validate(),
        Err(ConfigError::PerElementAttributesOverlapGlobalAllow(_))
    ));
}

// s[verify config.validate.per_element_attributes_allow]
#[test]
fn per_element_remove_must_be_subset_of_global_allow() {
    let cfg = SanitizerConfig {
        attributes: Some(vec![SanitizerAttribute::new("id")]),
        elements: Some(vec![
            SanitizerElementWithAttributes::html("a")
                .with_remove_attributes(vec![SanitizerAttribute::new("notfound")]),
        ]),
        ..Default::default()
    };
    let cfg = canonicalized(cfg, false);
    assert!(matches!(
        cfg.validate(),
        Err(ConfigError::PerElementRemoveAttributesNotSubsetOfGlobalAllow(_))
    ));
}

// s[verify config.validate.per_element_attributes_remove]
#[test]
fn per_element_under_global_remove_cannot_have_both_lists() {
    let cfg = SanitizerConfig {
        remove_attributes: Some(vec![SanitizerAttribute::new("href")]),
        elements: Some(vec![
            SanitizerElementWithAttributes::html("a")
                .with_attributes(vec![SanitizerAttribute::new("id")])
                .with_remove_attributes(vec![SanitizerAttribute::new("class")]),
        ]),
        ..Default::default()
    };
    let cfg = canonicalized(cfg, false);
    assert!(matches!(
        cfg.validate(),
        Err(ConfigError::PerElementAttributesBothListsPresent(_))
    ));
}

// s[verify config.validate.per_element_no_duplicates]
#[test]
fn per_element_attributes_duplicate_is_rejected() {
    let cfg = SanitizerConfig {
        attributes: Some(vec![]),
        elements: Some(vec![
            SanitizerElementWithAttributes::html("a").with_attributes(vec![
                SanitizerAttribute::new("href"),
                SanitizerAttribute::new("href"),
            ]),
        ]),
        ..Default::default()
    };
    let cfg = canonicalized(cfg, false);
    assert!(matches!(
        cfg.validate(),
        Err(ConfigError::PerElementAttributesDuplicate(_))
    ));
}
