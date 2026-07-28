use url::Url;

// s[impl sanitize.attributes.javascript_urls]
// Returns true if the given attribute value, when parsed as a URL,
// has scheme "javascript".
pub(super) fn contains_javascript_url(value: &str) -> bool {
    // The spec says "basic URL parser"; we use the `url` crate which
    // implements WHATWG URL. Absolute URL parsing is what we need.
    match Url::parse(value.trim()) {
        Ok(u) => u.scheme().eq_ignore_ascii_case("javascript"),
        Err(_) => false,
    }
}
