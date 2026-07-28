use sanitizer::{
    SanitizerAttribute, SanitizerConfig, SanitizerElement, SanitizerElementWithAttributes,
    SanitizerPI,
};

fn cfg_with_elements(els: Vec<SanitizerElementWithAttributes>) -> SanitizerConfig {
    SanitizerConfig {
        elements: Some(els),
        ..Default::default()
    }
}

// s[verify config.canonicalize.defaults]
#[test]
fn canonicalize_fills_missing_remove_elements() {
    let mut cfg = SanitizerConfig::default();
    cfg.canonicalize(false);
    assert_eq!(cfg.remove_elements.as_deref().unwrap().len(), 0);
    assert_eq!(cfg.remove_attributes.as_deref().unwrap().len(), 0);
}

// s[verify config.canonicalize.defaults]
#[test]
fn canonicalize_pi_defaults_depend_on_flag() {
    let mut cfg_safe = SanitizerConfig::default();
    cfg_safe.canonicalize(false);
    assert!(cfg_safe.processing_instructions.is_some());
    assert!(cfg_safe.remove_processing_instructions.is_none());

    let mut cfg_unsafe = SanitizerConfig::default();
    cfg_unsafe.canonicalize(true);
    assert!(cfg_unsafe.processing_instructions.is_none());
    assert!(cfg_unsafe.remove_processing_instructions.is_some());
}

// s[verify config.canonicalize.element]
#[test]
fn canonicalize_element_fills_html_namespace() {
    let mut cfg = SanitizerConfig {
        remove_elements: Some(vec![SanitizerElement {
            name: "script".into(),
            namespace: None,
        }]),
        ..Default::default()
    };
    cfg.canonicalize(false);
    let elt = &cfg.remove_elements.as_ref().unwrap()[0];
    assert_eq!(
        elt.namespace.as_deref(),
        Some("http://www.w3.org/1999/xhtml")
    );
}

// s[verify config.canonicalize.element]
#[test]
fn canonicalize_normalizes_empty_namespace_to_none() {
    let mut cfg = SanitizerConfig {
        remove_elements: Some(vec![SanitizerElement {
            name: "weird".into(),
            namespace: Some(String::new()),
        }]),
        ..Default::default()
    };
    cfg.canonicalize(false);
    assert!(cfg.remove_elements.as_ref().unwrap()[0].namespace.is_none());
}

// s[verify config.canonicalize.element_with_attributes]
#[test]
fn canonicalize_element_with_attributes_fills_empty_remove() {
    let mut cfg = cfg_with_elements(vec![SanitizerElementWithAttributes {
        name: "a".into(),
        namespace: None,
        attributes: None,
        remove_attributes: None,
    }]);
    cfg.canonicalize(false);
    let el = &cfg.elements.as_ref().unwrap()[0];
    assert_eq!(
        el.namespace.as_deref(),
        Some("http://www.w3.org/1999/xhtml")
    );
    assert!(el.remove_attributes.is_some());
}

// s[verify config.canonicalize.attribute]
#[test]
fn canonicalize_attribute_normalizes_empty_namespace() {
    let mut cfg = SanitizerConfig {
        attributes: Some(vec![SanitizerAttribute {
            name: "x".into(),
            namespace: Some(String::new()),
        }]),
        ..Default::default()
    };
    cfg.canonicalize(false);
    assert!(cfg.attributes.as_ref().unwrap()[0].namespace.is_none());
}

// s[verify config.canonicalize.pi]
#[test]
fn canonicalize_pi_is_identity() {
    let mut cfg = SanitizerConfig {
        processing_instructions: Some(vec![SanitizerPI::new("xml-stylesheet")]),
        ..Default::default()
    };
    cfg.canonicalize(false);
    assert_eq!(
        cfg.processing_instructions.as_ref().unwrap()[0].target,
        "xml-stylesheet"
    );
}

// s[verify config.canonicalize.booleans]
#[test]
fn canonicalize_booleans_defaults() {
    let mut cfg_safe = SanitizerConfig::default();
    cfg_safe.canonicalize(false);
    assert_eq!(cfg_safe.comments, Some(false));

    let mut cfg_unsafe = SanitizerConfig::default();
    cfg_unsafe.canonicalize(true);
    assert_eq!(cfg_unsafe.comments, Some(true));

    let mut cfg_with_allow = SanitizerConfig {
        attributes: Some(vec![SanitizerAttribute::new("id")]),
        ..Default::default()
    };
    cfg_with_allow.canonicalize(true);
    assert_eq!(cfg_with_allow.data_attributes, Some(true));
}
