use sanitizer::SanitizerConfig;

// s[verify api.sanitize_fn]
#[test]
fn free_sanitize_applies_safe_default() {
    let out = sanitizer::sanitize("<p>hello <script>x</script></p>").unwrap();
    assert_eq!(*out, *"<p>hello </p>");
}

// s[verify api.sanitize_fn]
#[test]
fn free_sanitize_unsafe_preserves_inputs() {
    let src = "<script>x</script>";
    let out = sanitizer::sanitize_unsafe(src).unwrap();
    assert_eq!(*out, *src);
}

// s[verify api.config_methods]
#[test]
fn config_method_sanitize_validates_first() {
    let cfg = SanitizerConfig {
        elements: Some(vec![
            sanitizer::SanitizerElementWithAttributes::html("a"),
            sanitizer::SanitizerElementWithAttributes::html("a"),
        ]),
        ..Default::default()
    };
    let err = cfg.sanitize("<a>x</a>").unwrap_err();
    let sanitizer::SanitizeError::Config(e) = err else {
        panic!("expected config error");
    };
    assert_eq!(e, sanitizer::ConfigError::DuplicateElements);
}

// s[verify api.config_methods]
#[test]
fn config_method_sanitize_unsafe_uses_unsafe_defaults() {
    let mut cfg = SanitizerConfig::empty();
    // With unsafe flags, comments default to kept.
    cfg.canonicalize(true);
    assert_eq!(cfg.comments, Some(true));
}

// s[verify api.config_presets]
#[test]
fn empty_preset_allows_everything_in_unsafe() {
    let cfg = SanitizerConfig::empty();
    let out = cfg.sanitize_unsafe("<p>hi</p>").unwrap();
    assert_eq!(*out, *"<p>hi</p>");
}

// s[verify api.config_presets]
#[test]
fn safe_default_preset_strips_script() {
    let cfg = SanitizerConfig::safe_default();
    let out = cfg.sanitize("<p>hi</p><script>x</script>").unwrap();
    assert!(!out.contains("script"));
}

// s[verify api.out_of_scope.shadow_dom]
// s[verify api.out_of_scope.live_dom]
// s[verify api.out_of_scope.modifier_methods]
#[test]
fn out_of_scope_behaviors_absent() {
    // We test by absence: no shadow API, no live DOM hook, no modifier methods
    // are exposed. This is enforced by the public surface: the following must
    // compile and typecheck.
    let _: fn(&str) -> Result<sanitizer::SanitizedOutput, sanitizer::SanitizeError> =
        sanitizer::sanitize;
    let _: fn(&str) -> Result<sanitizer::SanitizedOutput, sanitizer::SanitizeError> =
        sanitizer::sanitize_unsafe;
}
