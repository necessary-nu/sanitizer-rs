use sanitizer::{
    SanitizerAttribute, SanitizerConfig, SanitizerElement, SanitizerElementWithAttributes,
    SanitizerPI,
};

// s[verify config.types.element]
#[test]
fn element_constructors_produce_canonical_names() {
    let html = SanitizerElement::html("div");
    assert_eq!(html.name, "div");
    assert_eq!(
        html.namespace.as_deref(),
        Some("http://www.w3.org/1999/xhtml")
    );

    let svg = SanitizerElement::svg("circle");
    assert_eq!(svg.namespace.as_deref(), Some("http://www.w3.org/2000/svg"));

    let mml = SanitizerElement::mathml("mrow");
    assert_eq!(
        mml.namespace.as_deref(),
        Some("http://www.w3.org/1998/Math/MathML")
    );
}

// s[verify config.types.element]
#[test]
fn element_with_attributes_carries_per_element_lists() {
    let el = SanitizerElementWithAttributes::html("a")
        .with_attributes(vec![SanitizerAttribute::new("href")])
        .with_remove_attributes(vec![SanitizerAttribute::new("target")]);
    assert!(el.attributes.is_some());
    assert!(el.remove_attributes.is_some());
}

// s[verify config.types.attribute]
#[test]
fn attribute_defaults_to_null_namespace() {
    let a = SanitizerAttribute::new("href");
    assert_eq!(a.name, "href");
    assert!(a.namespace.is_none());
    let x = SanitizerAttribute::xlink("href");
    assert_eq!(x.namespace.as_deref(), Some("http://www.w3.org/1999/xlink"));
}

// s[verify config.types.pi]
#[test]
fn pi_holds_target() {
    let pi = SanitizerPI::new("xml-stylesheet");
    assert_eq!(pi.target, "xml-stylesheet");
}

// s[verify config.types.sanitizer_config]
#[test]
fn sanitizer_config_default_is_empty() {
    let cfg = SanitizerConfig::default();
    assert!(cfg.elements.is_none());
    assert!(cfg.remove_elements.is_none());
    assert!(cfg.attributes.is_none());
    assert!(cfg.remove_attributes.is_none());
    assert!(cfg.comments.is_none());
    assert!(cfg.data_attributes.is_none());
}
