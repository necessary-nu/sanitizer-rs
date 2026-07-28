# sanitizer-rs

A Rust implementation of the [W3C Sanitizer API](https://wicg.github.io/sanitizer-api/).
It accepts an HTML string and returns a sanitized HTML string, filtering elements, attributes,
and processing instructions against a configuration. In *safe* mode it additionally removes a
baseline of XSS vectors (script hosts, `javascript:` URLs, event-handler attributes).

The rules below are numbered requirements for the library. The vendored W3C
specification lives at `sanitizer-api/index.bs` and is the canonical source for
language; these rules restate it in the form Tracey tracks.

## Configuration model

> s[config.types.element]
> The library models `SanitizerElement` as either a bare local name (short form)
> or a `{name, namespace}` pair (long form). `SanitizerElementWithAttributes`
> additionally carries optional per-element `attributes` and `removeAttributes`
> allow/remove lists. See `sanitizer-api/index.bs` §5 Configuration Dictionary.

> s[config.types.attribute]
> `SanitizerAttribute` is either a local name (short form) or a `{name, namespace}`
> pair (long form). The default namespace for attribute canonicalization is `null`.

> s[config.types.pi]
> `SanitizerPI` (processing instruction specifier) is either a target string
> (short form) or a `{target}` dictionary.

> s[config.types.sanitizer_config]
> `SanitizerConfig` carries optional `elements`/`removeElements`/
> `replaceWithChildrenElements`, optional `processingInstructions`/
> `removeProcessingInstructions`, optional `attributes`/`removeAttributes`,
> plus booleans `comments` and `dataAttributes`.

## Built-in configurations

> s[config.builtin.default]
> The library exposes a built-in safe default configuration equivalent to
> `sanitizer-api/builtins/safe-default-configuration.json`. It is the default
> for the safe sanitize entry point.

> s[config.builtin.baseline]
> The library exposes a built-in safe baseline configuration equivalent to
> `sanitizer-api/builtins/safe-baseline-configuration.json`, used by the
> `remove unsafe` pass in safe mode.

> s[config.builtin.navigating_urls]
> The library carries the built-in navigating URL attributes list: (a,href) and
> (area,href) and (base,href) and (button,formaction) and (form,action) and
> (input,formaction) in HTML, and (a,href) in SVG (both `null` and XLink
> namespaces for the attribute).

> s[config.builtin.animating_urls]
> The library carries the built-in animating URL attributes list:
> (animate,attributeName), (animateTransform,attributeName), (set,attributeName),
> all in the SVG namespace with `null` attribute namespace.

> s[config.builtin.non_replaceable]
> The library carries the built-in non-replaceable elements list —
> (html, HTML), (svg, SVG), (math, MathML) — which may not appear in
> `replaceWithChildrenElements`.

## Canonicalization

> s[config.canonicalize.defaults]
> Canonicalize the configuration fills in missing allow/remove lists: if neither
> `elements` nor `removeElements` is set, set `removeElements` to `[]`; same
> logic for attributes; for processing instructions, the empty list is placed in
> `removeProcessingInstructions` when `allowCommentsPIsAndDataAttributes` is
> true, otherwise in `processingInstructions`.

> s[config.canonicalize.element]
> Canonicalizing a `SanitizerElement` promotes a bare name to
> `{name, namespace: HTML_NAMESPACE}`. An explicit empty-string namespace is
> normalized to `null`. (Spec: canonicalize a sanitizer name.)

> s[config.canonicalize.attribute]
> Canonicalizing a `SanitizerAttribute` promotes a bare name to
> `{name, namespace: null}`. An explicit empty-string namespace is normalized to
> `null`.

> s[config.canonicalize.element_with_attributes]
> Canonicalizing a `SanitizerElementWithAttributes` canonicalizes its nested
> `attributes` / `removeAttributes` lists as attributes. If the element has
> neither list after canonicalization, `removeAttributes` is set to `[]`.

> s[config.canonicalize.pi]
> Canonicalizing a `SanitizerPI` promotes a bare target to `{target}`.

> s[config.canonicalize.booleans]
> After canonicalization, `comments` defaults to the
> `allowCommentsPIsAndDataAttributes` flag; when `attributes` is present and
> `dataAttributes` is absent, `dataAttributes` defaults to that flag.

## Configuration validation

> s[config.validate.no_global_mixing_elements]
> `elements` and `removeElements` must not both be present.

> s[config.validate.no_global_mixing_attributes]
> `attributes` and `removeAttributes` must not both be present.

> s[config.validate.no_global_mixing_pis]
> `processingInstructions` and `removeProcessingInstructions` must not both be
> present.

> s[config.validate.data_attributes_requires_allow_list]
> `dataAttributes` may only be present when a global `attributes` allow-list is
> present.

> s[config.validate.no_duplicates_global]
> No global list (`elements`, `removeElements`, `replaceWithChildrenElements`,
> `attributes`, `removeAttributes`) has duplicate entries; PI lists have no
> duplicate targets.

> s[config.validate.replaceable_elements]
> `replaceWithChildrenElements` must not contain any element from the non-replaceable
> list (html/svg/math), and must not intersect with `elements` or `removeElements`.

> s[config.validate.per_element_attributes_allow]
> When a global `attributes` allow-list is present: a per-element `attributes`
> list must not overlap with the global list; a per-element `removeAttributes`
> list must be a subset of the global list; when `dataAttributes` is true, no
> per-element or global allow-list may contain a `data-*` attribute.

> s[config.validate.per_element_attributes_remove]
> When a global `removeAttributes` remove-list is present: a per-element element
> entry may carry at most one of `attributes` / `removeAttributes`, but not
> both; neither may duplicate entries in the global `removeAttributes` list.

> s[config.validate.per_element_no_duplicates]
> Per-element `attributes` and `removeAttributes` must each be internally
> duplicate-free and must not overlap each other.

## Sanitize algorithm

> s[sanitize.entry.safe]
> The safe entry point parses the input as an HTML fragment, applies the
> `remove unsafe` pass to its configuration, then runs `sanitize core` on the
> parsed tree with `handleJavascriptNavigationUrls = true`, and serializes back
> to an HTML string.

> s[sanitize.entry.unsafe]
> The unsafe entry point runs `sanitize core` on the parsed tree with
> `handleJavascriptNavigationUrls = false` and does *not* apply the `remove
> unsafe` pass.

> s[sanitize.core.walk]
> `sanitize core` iterates the children of a node; for each child it dispatches
> on the node kind (text, comment, PI, element, doctype) and recurses into the
> children of kept elements after per-element filtering has been applied.

> s[sanitize.text]
> Text nodes are always kept.

> s[sanitize.doctype]
> Doctype nodes are always kept (they only appear in full-document parse).

> s[sanitize.comments]
> A comment node is removed unless `configuration.comments` is true.

> s[sanitize.pi.allow]
> When `processingInstructions` exists, a processing instruction is kept iff its
> target appears in that list; otherwise it is removed.

> s[sanitize.pi.remove]
> When `removeProcessingInstructions` exists, a processing instruction is
> removed iff its target appears in that list; otherwise it is kept.

> s[sanitize.elements.replace_with_children]
> If `replaceWithChildrenElements` contains the element, sanitize core first
> recurses into the element (so its children get filtered), then replaces the
> element with its children and continues — the element itself is not kept.

> s[sanitize.elements.allow]
> When `elements` exists, an element is removed if its `{name, namespace}` is
> not in the list.

> s[sanitize.elements.remove]
> When `removeElements` exists, an element is removed if its `{name, namespace}`
> is in the list.

> s[sanitize.attributes.per_element_remove]
> For a kept element whose per-element entry in `elements` carries a
> `removeAttributes` list, attributes in that list are removed first.

> s[sanitize.attributes.global_allow]
> When `attributes` exists, an attribute is removed unless: it appears in the
> global `attributes` list; or it appears in the per-element `attributes` list;
> or it is a `data-*` attribute with `null` namespace and `dataAttributes` is
> true.

> s[sanitize.attributes.global_remove]
> When `removeAttributes` exists (the else branch), an attribute is removed if
> either: the per-element `attributes` allow-list exists and does not contain
> it; or the global `removeAttributes` list contains it.

> s[sanitize.attributes.javascript_urls]
> When `handleJavascriptNavigationUrls` is true and an `{element, attribute}`
> pair matches the built-in navigating URL attributes list, the attribute is
> removed if its value parses as a URL with scheme `javascript`.

> s[sanitize.attributes.mathml_href]
> When `handleJavascriptNavigationUrls` is true, any element in the MathML
> namespace whose attribute is local-name `href` in either the `null` or XLink
> namespace is removed if its value is a `javascript:` URL.

> s[sanitize.attributes.animating_href]
> When `handleJavascriptNavigationUrls` is true and an `{element, attribute}`
> pair matches the built-in animating URL attributes list, the attribute is
> removed if its value equals `href` or `xlink:href` (animation target blocks
> declarative rewriting of navigating URLs).

> s[sanitize.template]
> When a kept element is an HTML `<template>`, `sanitize core` is also called
> on its template contents fragment with the same configuration.

## Remove unsafe pass

> s[sanitize.baseline.elements]
> The `remove unsafe` pass, applied only in safe mode, calls `remove an element`
> for each entry of the built-in safe baseline configuration's `removeElements`
> list against the configuration.

> s[sanitize.baseline.attributes]
> The `remove unsafe` pass calls `remove an attribute` for each entry of the
> built-in safe baseline configuration's `removeAttributes` list and for every
> HTML event-handler content attribute (e.g. `onclick`, `onerror`, …).

## Library entry points

> s[api.sanitize_fn]
> The library exposes free functions `sanitize(html: &str) -> String` (safe
> mode, default config) and `sanitize_unsafe(html: &str) -> String` (no filter,
> empty config). Both accept HTML fragments.

> s[api.config_methods]
> `SanitizerConfig` exposes `sanitize(html)` and `sanitize_unsafe(html)`
> methods. Both canonicalize and validate before running.

> s[api.config_presets]
> `SanitizerConfig::safe_default()` returns the built-in safe default
> configuration. `SanitizerConfig::empty()` returns a configuration equivalent
> to `{}` (allow everything when used with `sanitize_unsafe`).

## Out of scope

> s[api.out_of_scope.shadow_dom]
> Shadow DOM sanitization is not implemented — shadow trees do not exist
> outside a browser runtime.

> s[api.out_of_scope.live_dom]
> The library does not mutate a live DOM or implement WebIDL surfaces
> (`Element.setHTML`, `Document.parseHTML`); only string input / string output
> is provided.

> s[api.out_of_scope.modifier_methods]
> The configuration-modifier methods from §6.3 of the W3C spec (`allowElement`,
> `removeElement`, `allowAttribute`, `removeAttribute`, …) are not exposed as
> a public API; the internal `remove an element` / `remove an attribute` steps
> are used only by the `remove unsafe` pass.
