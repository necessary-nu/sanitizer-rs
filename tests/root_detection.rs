#[test]
fn doctype_in_input_round_trips_as_document() {
    let src = "<!DOCTYPE html><html><body><p>hi</p><script>x</script></body></html>";
    let out = sanitizer::sanitize(src).unwrap();
    assert!(out.to_ascii_lowercase().contains("<!doctype html>"));
    assert!(!out.contains("<script>"));
    assert!(out.contains("<p>hi</p>"));
}

#[test]
fn html_root_in_input_round_trips_as_document() {
    let src = "<html><body><p>hi</p></body></html>";
    let out = sanitizer::sanitize(src).unwrap();
    assert!(out.contains("<html>") || out.contains("<html "));
    assert!(out.contains("<p>hi</p>"));
}

#[test]
fn plain_fragment_stays_fragment() {
    let src = "<p>hi</p>";
    let out = sanitizer::sanitize_unsafe(src).unwrap();
    assert_eq!(*out, *"<p>hi</p>");
    assert!(!out.contains("<html>"));
}

#[test]
fn leading_whitespace_before_doctype_is_tolerated() {
    let src = "  \n<!DOCTYPE html><p>x</p>";
    let out = sanitizer::sanitize(src).unwrap();
    assert!(out.to_ascii_lowercase().contains("<!doctype html>"));
}
