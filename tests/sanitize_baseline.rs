use sanitizer::{SanitizerConfig, SanitizerElement};

// s[verify sanitize.baseline.elements]
#[test]
fn baseline_strips_script_iframe_object_etc() {
    for src in [
        "<script>alert(1)</script>",
        "<iframe></iframe>",
        "<object></object>",
        "<embed>",
        "<frame>",
    ] {
        // Use a permissive config (remove nothing beyond baseline) so
        // the baseline is the only mechanism removing them.
        let cfg = SanitizerConfig {
            remove_elements: Some(vec![]),
            ..Default::default()
        };
        let out = cfg.sanitize(src).unwrap();
        assert!(!out.contains("<script"), "script not stripped: {out}");
        assert!(!out.contains("<iframe"), "iframe not stripped: {out}");
        assert!(!out.contains("<object"), "object not stripped: {out}");
        assert!(!out.contains("<embed"), "embed not stripped: {out}");
        assert!(!out.contains("<frame"), "frame not stripped: {out}");
    }
}

// s[verify sanitize.baseline.elements]
#[test]
fn baseline_strips_svg_use_and_svg_script() {
    let src = "<svg><use href=\"#x\"/></svg><svg><script>x</script></svg>";
    let cfg = SanitizerConfig {
        remove_elements: Some(vec![]),
        ..Default::default()
    };
    let out = cfg.sanitize(src).unwrap();
    assert!(!out.contains("<use"));
    assert!(!out.contains("<script"));
}

// s[verify sanitize.baseline.attributes]
#[test]
fn baseline_strips_event_handler_attributes() {
    let cfg = SanitizerConfig {
        remove_elements: Some(vec![]),
        ..Default::default()
    };
    let out = cfg
        .sanitize("<p onclick=\"x\" onmouseover=\"y\">hi</p>")
        .unwrap();
    assert!(!out.contains("onclick"));
    assert!(!out.contains("onmouseover"));
}

// s[verify sanitize.baseline.elements]
#[test]
fn unsafe_mode_does_not_apply_baseline() {
    let cfg = SanitizerConfig {
        remove_elements: Some(vec![SanitizerElement::html("p")]),
        ..Default::default()
    };
    let out = cfg.sanitize_unsafe("<script>x</script><p>y</p>").unwrap();
    assert!(out.contains("<script>"));
    assert!(!out.contains("<p>y</p>"));
}
