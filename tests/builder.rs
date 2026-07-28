use sanitizer::{
    ConfigError, SanitizerAttribute, SanitizerBuilder, SanitizerElement,
    SanitizerElementWithAttributes,
};

#[test]
fn allow_element_builds_allow_list() {
    let cfg = SanitizerBuilder::new()
        .allow_element(SanitizerElementWithAttributes::html("p"))
        .allow_element(SanitizerElementWithAttributes::html("b"))
        .build()
        .unwrap();
    assert!(cfg.elements.is_some());
    assert!(cfg.remove_elements.is_none());
    let out = cfg
        .sanitize_unsafe("<p>keep <span>drop</span> <b>bold</b></p>")
        .unwrap();
    assert_eq!(*out, *"<p>keep  <b>bold</b></p>");
}

#[test]
fn remove_element_builds_remove_list() {
    let cfg = SanitizerBuilder::new()
        .remove_element(SanitizerElement::html("b"))
        .build()
        .unwrap();
    assert!(cfg.elements.is_none());
    assert!(cfg.remove_elements.is_some());
    let out = cfg.sanitize_unsafe("<p>keep <b>drop</b></p>").unwrap();
    assert_eq!(*out, *"<p>keep </p>");
}

#[test]
fn mixing_allow_and_remove_element_is_rejected() {
    let err = SanitizerBuilder::new()
        .allow_element(SanitizerElementWithAttributes::html("p"))
        .remove_element(SanitizerElement::html("b"))
        .build()
        .unwrap_err();
    assert_eq!(err, ConfigError::MixedElementsLists);
}

#[test]
fn mixing_allow_and_remove_attribute_is_rejected() {
    let err = SanitizerBuilder::new()
        .allow_attribute(SanitizerAttribute::new("href"))
        .remove_attribute(SanitizerAttribute::new("class"))
        .build()
        .unwrap_err();
    assert_eq!(err, ConfigError::MixedAttributesLists);
}

#[test]
fn comments_flag_is_forwarded() {
    let cfg = SanitizerBuilder::new().comments(true).build().unwrap();
    let out = cfg.sanitize_unsafe("<!--x--><p>hi</p>").unwrap();
    assert!(out.contains("<!--x-->"));
}

#[test]
fn build_validates_before_returning() {
    let err = SanitizerBuilder::new()
        .replace_with_children(SanitizerElement::html("html"))
        .build()
        .unwrap_err();
    assert!(matches!(
        err,
        ConfigError::ReplaceWithChildrenContainsNonReplaceable(_)
    ));
}
