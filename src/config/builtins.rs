use std::sync::OnceLock;

use crate::namespace::{HTML_NS, MATHML_NS, SVG_NS, XLINK_NS};

use super::{SanitizerAttribute, SanitizerConfig, SanitizerElement};

const SAFE_DEFAULT_JSON: &str =
    include_str!("../../sanitizer-api/builtins/safe-default-configuration.json");

const SAFE_BASELINE_JSON: &str =
    include_str!("../../sanitizer-api/builtins/safe-baseline-configuration.json");

// s[impl config.builtin.default]
pub(crate) fn safe_default_configuration() -> &'static SanitizerConfig {
    static CELL: OnceLock<SanitizerConfig> = OnceLock::new();
    CELL.get_or_init(|| {
        serde_json::from_str(SAFE_DEFAULT_JSON)
            .expect("embedded safe-default-configuration.json must parse")
    })
}

// s[impl config.builtin.baseline]
pub(crate) fn safe_baseline_configuration() -> &'static SanitizerConfig {
    static CELL: OnceLock<SanitizerConfig> = OnceLock::new();
    CELL.get_or_init(|| {
        serde_json::from_str(SAFE_BASELINE_JSON)
            .expect("embedded safe-baseline-configuration.json must parse")
    })
}

// s[impl config.builtin.non_replaceable]
pub(crate) static NON_REPLACEABLE_ELEMENTS: &[(&str, &str)] =
    &[("html", HTML_NS), ("svg", SVG_NS), ("math", MATHML_NS)];

pub(crate) fn is_non_replaceable(el: &SanitizerElement) -> bool {
    NON_REPLACEABLE_ELEMENTS
        .iter()
        .any(|(name, ns)| el.name == *name && el.namespace.as_deref() == Some(*ns))
}

pub(crate) struct UrlAttr {
    pub element_name: &'static str,
    pub element_ns: &'static str,
    pub attr_name: &'static str,
    pub attr_ns: Option<&'static str>,
}

// s[impl config.builtin.navigating_urls]
pub(crate) static NAVIGATING_URL_ATTRIBUTES: &[UrlAttr] = &[
    UrlAttr {
        element_name: "a",
        element_ns: HTML_NS,
        attr_name: "href",
        attr_ns: None,
    },
    UrlAttr {
        element_name: "area",
        element_ns: HTML_NS,
        attr_name: "href",
        attr_ns: None,
    },
    UrlAttr {
        element_name: "base",
        element_ns: HTML_NS,
        attr_name: "href",
        attr_ns: None,
    },
    UrlAttr {
        element_name: "button",
        element_ns: HTML_NS,
        attr_name: "formaction",
        attr_ns: None,
    },
    UrlAttr {
        element_name: "form",
        element_ns: HTML_NS,
        attr_name: "action",
        attr_ns: None,
    },
    UrlAttr {
        element_name: "input",
        element_ns: HTML_NS,
        attr_name: "formaction",
        attr_ns: None,
    },
    UrlAttr {
        element_name: "a",
        element_ns: SVG_NS,
        attr_name: "href",
        attr_ns: None,
    },
    UrlAttr {
        element_name: "a",
        element_ns: SVG_NS,
        attr_name: "href",
        attr_ns: Some(XLINK_NS),
    },
];

// s[impl config.builtin.animating_urls]
pub(crate) static ANIMATING_URL_ATTRIBUTES: &[UrlAttr] = &[
    UrlAttr {
        element_name: "animate",
        element_ns: SVG_NS,
        attr_name: "attributeName",
        attr_ns: None,
    },
    UrlAttr {
        element_name: "animateTransform",
        element_ns: SVG_NS,
        attr_name: "attributeName",
        attr_ns: None,
    },
    UrlAttr {
        element_name: "set",
        element_ns: SVG_NS,
        attr_name: "attributeName",
        attr_ns: None,
    },
];

// s[impl sanitize.baseline.attributes]
// The list is sourced directly from the vendored W3C/WHATWG spec file
// `event-handler-content-attributes.txt`, which tracks
// https://html.spec.whatwg.org/#ix-event-handlers. Lines beginning with `//`
// are comments; blank lines are ignored.
const EVENT_HANDLERS_TXT: &str =
    include_str!("../../sanitizer-api/builtins/event-handler-content-attributes.txt");

pub(crate) fn event_handler_attributes() -> &'static [SanitizerAttribute] {
    static CELL: OnceLock<Vec<SanitizerAttribute>> = OnceLock::new();
    CELL.get_or_init(|| {
        EVENT_HANDLERS_TXT
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with("//"))
            .map(|name| SanitizerAttribute {
                name: name.to_owned(),
                namespace: None,
            })
            .collect()
    })
    .as_slice()
}
