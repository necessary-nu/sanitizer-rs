use sanitizer::SanitizerConfig;

// s[verify config.builtin.default]
#[test]
fn safe_default_parses_and_is_non_empty() {
    let cfg = SanitizerConfig::safe_default();
    let elements = cfg
        .elements
        .as_ref()
        .expect("default has elements allow-list");
    assert!(!elements.is_empty());
    assert!(
        elements.iter().any(
            |e| e.name == "p" && e.namespace.as_deref() == Some("http://www.w3.org/1999/xhtml")
        )
    );
    assert!(
        elements.iter().any(
            |e| e.name == "svg" && e.namespace.as_deref() == Some("http://www.w3.org/2000/svg")
        )
    );
    assert!(elements.iter().any(|e| e.name == "math"
        && e.namespace.as_deref() == Some("http://www.w3.org/1998/Math/MathML")));
}

// s[verify config.builtin.default]
#[test]
fn safe_default_is_validated_after_canonicalization() {
    let mut cfg = SanitizerConfig::safe_default();
    cfg.canonicalize(false);
    cfg.validate().expect("safe default must pass validation");
}

// s[verify config.builtin.baseline]
#[test]
fn baseline_triggers_unsafe_element_removal() {
    // Safe sanitize with a permissive per-user config still applies the
    // baseline. Use an allow-list that includes script — the baseline should
    // strip <script> anyway.
    let mut cfg = SanitizerConfig::default();
    cfg.remove_elements = Some(vec![]); // empty remove list = allow everything except baseline
    let out = cfg.sanitize("<p>ok</p><script>alert(1)</script>").unwrap();
    assert!(!out.contains("<script>"));
    assert!(out.contains("<p>ok</p>"));
}

// s[verify config.builtin.navigating_urls]
#[test]
fn navigating_urls_list_is_exercised_via_safe_sanitize() {
    let out = sanitizer::sanitize("<a href=\"javascript:alert(1)\">x</a>").unwrap();
    assert!(!out.contains("javascript"));
}

// s[verify config.builtin.animating_urls]
#[test]
fn animating_urls_blocks_href_retarget() {
    // SVG animate retargeting href is stripped in safe mode.
    let src = "<svg><a href=\"/ok\"><animate attributeName=\"href\"/></a></svg>";
    let out = sanitizer::sanitize(src).unwrap();
    assert!(!out.contains("attributeName=\"href\""));
}

// s[verify config.builtin.non_replaceable]
#[test]
fn non_replaceable_rejection_enforced() {
    use sanitizer::{ConfigError, SanitizerElement};
    let mut cfg = SanitizerConfig {
        replace_with_children_elements: Some(vec![SanitizerElement::svg("svg")]),
        ..Default::default()
    };
    cfg.canonicalize(false);
    assert!(matches!(
        cfg.validate(),
        Err(ConfigError::ReplaceWithChildrenContainsNonReplaceable(_))
    ));
}
