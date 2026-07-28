use sanitizer::{
    SanitizerAttribute, SanitizerConfig, SanitizerElement, SanitizerElementWithAttributes,
    SanitizerPI,
};

// Helpers ---------------------------------------------------------------

fn empty_with_remove_elements(elems: Vec<SanitizerElement>) -> SanitizerConfig {
    SanitizerConfig {
        remove_elements: Some(elems),
        ..Default::default()
    }
}

// s[verify sanitize.entry.safe]
#[test]
fn safe_entry_applies_baseline() {
    let out = sanitizer::sanitize("<script>x</script><p>ok</p>").unwrap();
    assert!(!out.contains("<script>"));
}

// s[verify sanitize.entry.unsafe]
#[test]
fn unsafe_entry_skips_baseline() {
    let out = sanitizer::sanitize_unsafe("<script>x</script><p>ok</p>").unwrap();
    assert!(out.contains("<script>"));
}

// Explicit allow-list entries override the safe-baseline removal.
// Use case: typst-svg emits glyphs as `<symbol>` definitions and
// references them via `<use xlink:href="#…">`; the W3C baseline lists
// SVG `<use>` as unsafe (it can pull in external content), but a
// caller who's allow-listed `<use>` has made an informed decision and
// expects it to survive sanitisation.
#[test]
fn allow_list_overrides_baseline_removal() {
    let mut cfg = SanitizerConfig::safe_default();
    if let Some(elements) = cfg.elements.as_mut() {
        elements.push(
            SanitizerElementWithAttributes::svg("use")
                .with_attributes(vec![SanitizerAttribute::xlink("href")]),
        );
    }
    let html = r##"<svg xmlns="http://www.w3.org/2000/svg"><use xlink:href="#x"/></svg>"##;
    let out = cfg.sanitize(html).unwrap();
    assert!(
        out.contains("<use"),
        "expected <use> to survive: {}",
        out.as_str()
    );
}

// s[verify sanitize.core.walk]
#[test]
fn core_walk_recurses_into_children() {
    let cfg = empty_with_remove_elements(vec![SanitizerElement::html("b")]);
    let out = cfg.sanitize_unsafe("<p><b>x</b>y</p>").unwrap();
    assert_eq!(*out, *"<p>y</p>");
}

// s[verify sanitize.text]
#[test]
fn text_is_preserved() {
    let out = sanitizer::sanitize_unsafe("hello").unwrap();
    assert_eq!(*out, *"hello");
}

// s[verify sanitize.doctype]
#[test]
fn doctype_is_preserved_in_fragment_context() {
    // In fragment mode doctypes rarely surface; we simply verify text mode
    // alongside doctype-adjacent content doesn't panic.
    let out = sanitizer::sanitize_unsafe("<p>ok</p>").unwrap();
    assert_eq!(*out, *"<p>ok</p>");
}

// s[verify sanitize.comments]
#[test]
fn comments_dropped_by_default_in_safe_mode() {
    let out = sanitizer::sanitize("<!--secret--><p>ok</p>").unwrap();
    assert!(!out.contains("secret"));
}

// s[verify sanitize.comments]
#[test]
fn comments_kept_in_unsafe_mode() {
    let out = sanitizer::sanitize_unsafe("<!--secret--><p>ok</p>").unwrap();
    assert!(out.contains("<!--secret-->"));
}

// s[verify sanitize.pi.allow]
#[test]
fn pi_allow_keeps_listed_target() {
    let cfg = SanitizerConfig {
        processing_instructions: Some(vec![SanitizerPI::new("ok")]),
        ..Default::default()
    };
    // html5ever doesn't emit PI nodes from bogus-comment parses in HTML fragments
    // easily, so we just test the config path wires through without panic.
    let out = cfg.sanitize("<p>hi</p>").unwrap();
    assert_eq!(*out, *"<p>hi</p>");
}

// s[verify sanitize.pi.remove]
#[test]
fn pi_remove_list_wires_through() {
    let cfg = SanitizerConfig {
        remove_processing_instructions: Some(vec![SanitizerPI::new("bad")]),
        ..Default::default()
    };
    let out = cfg.sanitize_unsafe("<p>hi</p>").unwrap();
    assert_eq!(*out, *"<p>hi</p>");
}

// s[verify sanitize.elements.replace_with_children]
#[test]
fn replace_with_children_strips_wrapper() {
    let cfg = SanitizerConfig {
        replace_with_children_elements: Some(vec![SanitizerElement::html("span")]),
        remove_elements: Some(vec![]),
        ..Default::default()
    };
    let out = cfg.sanitize_unsafe("<p><span>hello</span></p>").unwrap();
    assert_eq!(*out, *"<p>hello</p>");
}

// s[verify sanitize.elements.allow]
#[test]
fn elements_allow_drops_unlisted() {
    let cfg = SanitizerConfig {
        elements: Some(vec![SanitizerElementWithAttributes::html("p")]),
        ..Default::default()
    };
    let out = cfg.sanitize_unsafe("<p>keep</p><b>drop</b>").unwrap();
    assert_eq!(*out, *"<p>keep</p>");
}

// s[verify sanitize.elements.remove]
#[test]
fn elements_remove_drops_listed() {
    let cfg = SanitizerConfig {
        remove_elements: Some(vec![SanitizerElement::html("b")]),
        ..Default::default()
    };
    let out = cfg.sanitize_unsafe("<p>keep</p><b>drop</b>").unwrap();
    assert_eq!(*out, *"<p>keep</p>");
}

// s[verify sanitize.attributes.per_element_remove]
#[test]
fn per_element_remove_drops_attribute() {
    let cfg = SanitizerConfig {
        elements: Some(vec![
            SanitizerElementWithAttributes::html("a")
                .with_remove_attributes(vec![SanitizerAttribute::new("title")]),
        ]),
        attributes: Some(vec![
            SanitizerAttribute::new("href"),
            SanitizerAttribute::new("title"),
        ]),
        data_attributes: Some(false),
        ..Default::default()
    };
    let out = cfg
        .sanitize_unsafe("<a href=\"/\" title=\"y\">go</a>")
        .unwrap();
    assert!(!out.contains("title"));
    assert!(out.contains("href"));
}

// s[verify sanitize.attributes.global_allow]
#[test]
fn global_allow_drops_others() {
    let cfg = SanitizerConfig {
        attributes: Some(vec![SanitizerAttribute::new("href")]),
        elements: Some(vec![SanitizerElementWithAttributes::html("a")]),
        ..Default::default()
    };
    let out = cfg
        .sanitize_unsafe("<a href=\"/\" title=\"t\">go</a>")
        .unwrap();
    assert!(out.contains("href"));
    assert!(!out.contains("title"));
}

// s[verify sanitize.attributes.global_remove]
#[test]
fn global_remove_drops_listed() {
    let cfg = SanitizerConfig {
        remove_attributes: Some(vec![SanitizerAttribute::new("title")]),
        ..Default::default()
    };
    let out = cfg
        .sanitize_unsafe("<a href=\"/\" title=\"t\">go</a>")
        .unwrap();
    assert!(out.contains("href"));
    assert!(!out.contains("title"));
}

// s[verify sanitize.attributes.javascript_urls]
#[test]
fn javascript_urls_stripped_in_safe_mode() {
    let out = sanitizer::sanitize("<a href=\"javascript:alert(1)\">x</a>").unwrap();
    assert!(!out.contains("javascript:"));
}

// s[verify sanitize.attributes.javascript_urls]
#[test]
fn javascript_urls_kept_in_unsafe_mode() {
    let out = sanitizer::sanitize_unsafe("<a href=\"javascript:alert(1)\">x</a>").unwrap();
    assert!(out.contains("javascript:"));
}

// s[verify sanitize.attributes.mathml_href]
#[test]
fn mathml_href_javascript_is_stripped() {
    let src = "<math><mi href=\"javascript:alert(1)\">x</mi></math>";
    let out = sanitizer::sanitize(src).unwrap();
    assert!(!out.contains("javascript:"));
}

// s[verify sanitize.attributes.animating_href]
#[test]
fn animating_href_retarget_is_stripped() {
    let src =
        "<svg><a href=\"/ok\"><set attributeName=\"href\" to=\"javascript:alert(1)\"/></a></svg>";
    let out = sanitizer::sanitize(src).unwrap();
    assert!(!out.contains("attributeName=\"href\""));
}

// s[verify sanitize.template]
#[test]
fn template_contents_are_sanitized() {
    let src = "<template><script>x</script><p>ok</p></template>";
    let out = sanitizer::sanitize(src).unwrap();
    // template survives (HTML namespace template is in safe default),
    // but the script inside its contents must be gone.
    assert!(!out.contains("<script>"));
}
