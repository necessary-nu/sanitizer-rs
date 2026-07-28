use std::cell::RefCell;
use std::default::Default;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

use bumpalo::Bump;
use html5ever::serialize::{SerializeOpts, TraversalScope, serialize};
use html5ever::tendril::TendrilSink;
use html5ever::{ParseOpts, QualName, local_name, ns, parse_document, parse_fragment};
use markup5ever::interface::Attribute;

use crate::config::{
    ANIMATING_URL_ATTRIBUTES, NAVIGATING_URL_ATTRIBUTES, SanitizerAttribute, SanitizerConfig,
    SanitizerElementWithAttributes, UrlAttr,
};
use crate::dom::{ArenaOom, Dom, Handle, NodeData, SerializableHandle};
use crate::error::SanitizeError;
use crate::namespace::{MATHML_NS, XLINK_NS};
use crate::output::SanitizedOutput;

use super::urls::contains_javascript_url;

pub(super) fn run(
    cfg: &SanitizerConfig,
    html: &str,
    handle_js_urls: bool,
) -> Result<SanitizedOutput, SanitizeError> {
    match detect_mode(html) {
        Mode::Document => run_document(cfg, html, handle_js_urls),
        Mode::Fragment => run_fragment(cfg, html, handle_js_urls),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Document,
    Fragment,
}

// s[impl sanitize.doctype]
fn detect_mode(html: &str) -> Mode {
    let trimmed = html.trim_start().as_bytes();
    if has_ascii_prefix_ci(trimmed, b"<!doctype") || has_ascii_prefix_ci(trimmed, b"<html") {
        Mode::Document
    } else {
        Mode::Fragment
    }
}

fn has_ascii_prefix_ci(input: &[u8], needle: &[u8]) -> bool {
    input.len() >= needle.len() && input[..needle.len()].eq_ignore_ascii_case(needle)
}

fn run_fragment(
    cfg: &SanitizerConfig,
    html: &str,
    handle_js_urls: bool,
) -> Result<SanitizedOutput, SanitizeError> {
    let arena = Bump::new();
    // Apply the limit *after* `Dom::new` below (which allocates the document
    // node); see `run_with_arena` for the sequencing.
    run_with_arena(&arena, cfg, || {
        let sink = Dom::new(&arena);
        if let Some(limit) = cfg.max_arena_bytes {
            arena.set_allocation_limit(Some(limit));
        }
        let context = QualName::new(None, ns!(html), local_name!("body"));
        let sink = parse_fragment(sink, ParseOpts::default(), context, vec![], false).one(html);
        let root = fragment_root(&sink);
        sanitize_children(root, cfg, handle_js_urls);
        serialize_to_output(root)
    })
}

fn run_document(
    cfg: &SanitizerConfig,
    html: &str,
    handle_js_urls: bool,
) -> Result<SanitizedOutput, SanitizeError> {
    let arena = Bump::new();
    run_with_arena(&arena, cfg, || {
        let sink = Dom::new(&arena);
        if let Some(limit) = cfg.max_arena_bytes {
            arena.set_allocation_limit(Some(limit));
        }
        let sink = parse_document(sink, ParseOpts::default()).one(html);
        sanitize_children(sink.document, cfg, handle_js_urls);
        serialize_to_output(sink.document)
    })
}

fn run_with_arena<F>(
    _arena: &Bump,
    _cfg: &SanitizerConfig,
    body: F,
) -> Result<SanitizedOutput, SanitizeError>
where
    F: FnOnce() -> Result<SanitizedOutput, SanitizeError>,
{
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(r) => r,
        Err(payload) => {
            if payload.is::<ArenaOom>() {
                Err(SanitizeError::AllocationLimit)
            } else {
                resume_unwind(payload)
            }
        }
    }
}

fn serialize_to_output(root: Handle<'_>) -> Result<SanitizedOutput, SanitizeError> {
    let mut buf = Vec::new();
    serialize(
        &mut buf,
        &SerializableHandle(root),
        SerializeOpts {
            traversal_scope: TraversalScope::ChildrenOnly(None),
            ..Default::default()
        },
    )
    .map_err(SanitizeError::Serialize)?;

    String::from_utf8(buf)
        .map(SanitizedOutput::new)
        .map_err(|e| {
            SanitizeError::Serialize(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
}

fn fragment_root<'a>(dom: &Dom<'a>) -> Handle<'a> {
    let first = dom.document.children.borrow().first().copied();
    first.unwrap_or(dom.document)
}

// --- sanitize walk --------------------------------------------------------
//
// The walk is a table of QualName-level comparisons against the canonicalized
// config. We never construct `SanitizerElement` or `SanitizerAttribute` for
// DOM nodes: their `name`/`namespace` fields are `Atom`-backed and cheap to
// compare against the `&str` inside the config entries.

fn ns_matches(qn_ns: &str, spec_ns: Option<&str>) -> bool {
    match spec_ns {
        None => qn_ns.is_empty(),
        Some(s) => qn_ns == s,
    }
}

fn element_matches_list<T>(
    name: &QualName,
    list: &[T],
    get_name: impl Fn(&T) -> &str,
    get_ns: impl Fn(&T) -> Option<&str>,
) -> bool {
    let local: &str = &name.local;
    let ns: &str = &name.ns;
    list.iter()
        .any(|e| get_name(e) == local && ns_matches(ns, get_ns(e)))
}

fn attr_matches_list(attr: &QualName, list: &[SanitizerAttribute]) -> bool {
    let local: &str = &attr.local;
    let ns: &str = &attr.ns;
    list.iter()
        .any(|a| a.name == local && ns_matches(ns, a.namespace.as_deref()))
}

// s[impl sanitize.core.walk]
fn sanitize_children(root: Handle<'_>, cfg: &SanitizerConfig, handle_js_urls: bool) {
    // Iterative, preorder, explicit stack. Each work item instructs the walk
    // to filter one node's children (for a Document or Element node) and any
    // template-contents nested within. Recursion is avoided so a deeply
    // nested input can't overflow the call stack.
    let mut stack: Vec<Handle<'_>> = Vec::new();
    stack.push(root);

    while let Some(parent) = stack.pop() {
        let mut keep: Vec<Handle<'_>> = Vec::new();
        let current: Vec<Handle<'_>> = parent.children.borrow().clone();

        for child in current {
            // s[impl sanitize.text]
            // s[impl sanitize.doctype]
            match &child.data {
                NodeData::Text { .. } | NodeData::Doctype { .. } | NodeData::Document => {
                    keep.push(child);
                }
                NodeData::Comment { .. } => {
                    // s[impl sanitize.comments]
                    if cfg.comments == Some(true) {
                        keep.push(child);
                    }
                }
                NodeData::ProcessingInstruction { target, .. } => {
                    // s[impl sanitize.pi.allow]
                    // s[impl sanitize.pi.remove]
                    if let Some(allow) = &cfg.processing_instructions {
                        if allow.iter().any(|p| p.target.as_str() == target.as_ref()) {
                            keep.push(child);
                        }
                    } else if let Some(remove) = &cfg.remove_processing_instructions {
                        if !remove.iter().any(|p| p.target.as_str() == target.as_ref()) {
                            keep.push(child);
                        }
                    } else {
                        keep.push(child);
                    }
                }
                NodeData::Element {
                    name,
                    attrs,
                    template_contents,
                    ..
                } => {
                    // s[impl sanitize.elements.replace_with_children]
                    if let Some(list) = &cfg.replace_with_children_elements {
                        let matches = element_matches_list(
                            name,
                            list,
                            |e| e.name.as_str(),
                            |e| e.namespace.as_deref(),
                        );
                        if matches {
                            // Filter the element's subtree in place first,
                            // then splice its (now-sanitized) children into
                            // `keep` so the wrapper itself drops out.
                            stack.push(child);
                            // Drain and push immediately so the subtree has
                            // already been processed before we splice up.
                            // But `stack.pop` order means we'd process the
                            // subtree *after* the current frame pops — wrong
                            // order. We do a bounded inline subtree walk
                            // here using a local sub-stack to keep the spec
                            // semantics while still avoiding recursion.
                            stack.pop();
                            filter_subtree(child, cfg, handle_js_urls);
                            for grand in child.children.borrow().iter() {
                                grand.parent.set(Some(parent));
                                keep.push(*grand);
                            }
                            child.children.borrow_mut().clear();
                            continue;
                        }
                    }

                    // s[impl sanitize.elements.allow]
                    // s[impl sanitize.elements.remove]
                    if let Some(allow) = &cfg.elements {
                        let allowed = element_matches_list(
                            name,
                            allow,
                            |e| e.name.as_str(),
                            |e| e.namespace.as_deref(),
                        );
                        if !allowed {
                            continue;
                        }
                    } else if let Some(remove) = &cfg.remove_elements {
                        let removed = element_matches_list(
                            name,
                            remove,
                            |e| e.name.as_str(),
                            |e| e.namespace.as_deref(),
                        );
                        if removed {
                            continue;
                        }
                    }

                    // s[impl sanitize.template]
                    let is_html_template = name.local.as_ref() == "template"
                        && name.ns.as_ref() == crate::namespace::HTML_NS;
                    if is_html_template {
                        if let Some(tc) = template_contents.get() {
                            stack.push(tc);
                        }
                    }

                    let per_element_entry = cfg.elements.as_ref().and_then(|list| {
                        list.iter().find(|e| {
                            e.name == name.local.as_ref()
                                && ns_matches(&name.ns, e.namespace.as_deref())
                        })
                    });

                    filter_attributes(name, attrs, cfg, per_element_entry, handle_js_urls);

                    // Queue this element so its own children get filtered.
                    stack.push(child);
                    keep.push(child);
                }
            }
        }

        let mut children = parent.children.borrow_mut();
        children.clear();
        children.extend(keep);
    }
}

// Contained subtree walk used only by replace-with-children, so that we can
// sanitize an element's descendants *before* splicing them into the parent's
// child list in the outer walk. Uses the same iterative stack discipline.
fn filter_subtree(root: Handle<'_>, cfg: &SanitizerConfig, handle_js_urls: bool) {
    sanitize_children(root, cfg, handle_js_urls);
}

// s[impl sanitize.attributes.per_element_remove]
// s[impl sanitize.attributes.global_allow]
// s[impl sanitize.attributes.global_remove]
// s[impl sanitize.attributes.javascript_urls]
// s[impl sanitize.attributes.mathml_href]
// s[impl sanitize.attributes.animating_href]
fn filter_attributes(
    element: &QualName,
    attrs: &RefCell<Vec<Attribute>>,
    cfg: &SanitizerConfig,
    per_element: Option<&SanitizerElementWithAttributes>,
    handle_js_urls: bool,
) {
    let mut list = attrs.borrow_mut();
    let mut kept: Vec<Attribute> = Vec::with_capacity(list.len());

    let per_element_allow = per_element
        .and_then(|e| e.attributes.as_deref())
        .unwrap_or(&[]);
    let per_element_remove = per_element
        .and_then(|e| e.remove_attributes.as_deref())
        .unwrap_or(&[]);

    for attr in list.drain(..) {
        if attr_matches_list(&attr.name, per_element_remove) {
            continue;
        }

        if let Some(global_allow) = &cfg.attributes {
            let in_global_allow = attr_matches_list(&attr.name, global_allow);
            let in_local_allow = attr_matches_list(&attr.name, per_element_allow);
            let is_data =
                attr.name.ns.as_ref().is_empty() && attr.name.local.as_ref().starts_with("data-");
            let data_ok = is_data && cfg.data_attributes == Some(true);
            if !(in_global_allow || in_local_allow || data_ok) {
                continue;
            }
        } else {
            if per_element.map_or(false, |e| e.attributes.is_some())
                && !attr_matches_list(&attr.name, per_element_allow)
            {
                continue;
            }
            if let Some(global_remove) = &cfg.remove_attributes {
                if attr_matches_list(&attr.name, global_remove) {
                    continue;
                }
            }
        }

        if handle_js_urls && is_javascript_navigation(element, &attr.name, &attr.value) {
            continue;
        }
        if handle_js_urls && is_javascript_mathml_href(element, &attr.name, &attr.value) {
            continue;
        }
        if handle_js_urls && is_animation_to_href(element, &attr.name, &attr.value) {
            continue;
        }

        kept.push(attr);
    }

    *list = kept;
}

fn url_attr_matches(entry: &UrlAttr, el: &QualName, attr: &QualName) -> bool {
    el.local.as_ref() == entry.element_name
        && el.ns.as_ref() == entry.element_ns
        && attr.local.as_ref() == entry.attr_name
        && ns_matches(&attr.ns, entry.attr_ns)
}

fn is_javascript_navigation(element: &QualName, attr: &QualName, value: &str) -> bool {
    NAVIGATING_URL_ATTRIBUTES
        .iter()
        .any(|e| url_attr_matches(e, element, attr))
        && contains_javascript_url(value)
}

// s[impl sanitize.attributes.mathml_href]
fn is_javascript_mathml_href(element: &QualName, attr: &QualName, value: &str) -> bool {
    let attr_ns: &str = &attr.ns;
    let ns_ok = attr_ns.is_empty() || attr_ns == XLINK_NS;
    element.ns.as_ref() == MATHML_NS
        && attr.local.as_ref() == "href"
        && ns_ok
        && contains_javascript_url(value)
}

// s[impl sanitize.attributes.animating_href]
fn is_animation_to_href(element: &QualName, attr: &QualName, value: &str) -> bool {
    ANIMATING_URL_ATTRIBUTES
        .iter()
        .any(|e| url_attr_matches(e, element, attr))
        && (value == "href" || value == "xlink:href")
}
