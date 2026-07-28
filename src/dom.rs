//! Arena-backed DOM used as the `TreeSink` for html5ever.
//!
//! A sanitize call parses once, filters the tree in place, serializes, and
//! drops the whole tree. That is the exact shape for a bump allocator, so
//! nodes live in a [`bumpalo::Bump`] and handles are plain `&'arena Node`
//! references: zero per-node refcount overhead, zero per-node `malloc`,
//! bump-pointer allocation, stable addresses across the arena's lifetime, and
//! one bulk free at the end of the sanitize call.

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{HashSet, VecDeque};
use std::io;
use std::mem;

use bumpalo::Bump;
use html5ever::serialize::{Serialize, Serializer, TraversalScope};
use html5ever::tendril::StrTendril;
use markup5ever::interface::tree_builder;
use markup5ever::interface::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use markup5ever::{Attribute, ExpandedName, QualName};

#[derive(Debug)]
pub(crate) enum NodeData<'a> {
    Document,
    Doctype {
        name: StrTendril,
        #[allow(dead_code)]
        public_id: StrTendril,
        #[allow(dead_code)]
        system_id: StrTendril,
    },
    Text {
        contents: RefCell<StrTendril>,
    },
    Comment {
        contents: StrTendril,
    },
    Element {
        name: QualName,
        attrs: RefCell<Vec<Attribute>>,
        template_contents: Cell<Option<Handle<'a>>>,
        mathml_annotation_xml_integration_point: bool,
    },
    ProcessingInstruction {
        target: StrTendril,
        contents: StrTendril,
    },
}

pub(crate) struct Node<'a> {
    pub parent: Cell<Option<Handle<'a>>>,
    pub children: RefCell<Vec<Handle<'a>>>,
    pub data: NodeData<'a>,
}

impl<'a> std::fmt::Debug for Node<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("data", &self.data)
            .field("children", &self.children)
            .finish()
    }
}

/// A handle is just a shared reference into the arena — `Copy`, a single
/// machine word, no refcount bumps.
pub(crate) type Handle<'a> = &'a Node<'a>;

/// Private panic payload used to signal arena exhaustion from inside a
/// `TreeSink` method (whose signature is infallible). The top-level sanitize
/// call catches this and maps it to [`crate::error::SanitizeError::AllocationLimit`];
/// any other panic is re-raised.
pub(crate) struct ArenaOom;

pub(crate) struct Dom<'a> {
    arena: &'a Bump,
    pub document: Handle<'a>,
    #[allow(dead_code)]
    pub errors: RefCell<Vec<Cow<'static, str>>>,
    #[allow(dead_code)]
    pub quirks_mode: Cell<QuirksMode>,
}

impl<'a> Dom<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        let document = try_alloc_node(arena, NodeData::Document)
            .expect("initial document allocation must succeed before limit is set");
        Self {
            arena,
            document,
            errors: RefCell::new(Vec::new()),
            quirks_mode: Cell::new(tree_builder::NoQuirks),
        }
    }

    fn new_node(&self, data: NodeData<'a>) -> Handle<'a> {
        match try_alloc_node(self.arena, data) {
            Some(n) => n,
            None => std::panic::panic_any(ArenaOom),
        }
    }
}

fn try_alloc_node<'a>(arena: &'a Bump, data: NodeData<'a>) -> Option<Handle<'a>> {
    arena
        .try_alloc(Node {
            parent: Cell::new(None),
            children: RefCell::new(Vec::new()),
            data,
        })
        .ok()
        .map(|n| &*n)
}

// --- tree manipulation helpers --------------------------------------------

fn append_child<'a>(parent: Handle<'a>, child: Handle<'a>) {
    let previous = child.parent.replace(Some(parent));
    debug_assert!(previous.is_none(), "node already attached to a parent");
    parent.children.borrow_mut().push(child);
}

fn parent_and_index<'a>(node: Handle<'a>) -> Option<(Handle<'a>, usize)> {
    let parent = node.parent.get()?;
    let index = parent
        .children
        .borrow()
        .iter()
        .position(|child| std::ptr::eq(*child, node))
        .expect("node not in parent's child list");
    Some((parent, index))
}

fn detach<'a>(node: Handle<'a>) {
    if let Some((parent, i)) = parent_and_index(node) {
        parent.children.borrow_mut().remove(i);
        node.parent.set(None);
    }
}

fn append_to_text(tail: Handle<'_>, text: &str) -> bool {
    if let NodeData::Text { ref contents } = tail.data {
        contents.borrow_mut().push_slice(text);
        true
    } else {
        false
    }
}

// --- TreeSink impl --------------------------------------------------------

impl<'a> TreeSink for Dom<'a> {
    type Output = Self;
    type Handle = Handle<'a>;
    type ElemName<'b>
        = ExpandedName<'b>
    where
        Self: 'b;

    fn finish(self) -> Self {
        self
    }

    fn parse_error(&self, msg: Cow<'static, str>) {
        self.errors.borrow_mut().push(msg);
    }

    fn get_document(&self) -> Self::Handle {
        self.document
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        if let NodeData::Element {
            ref template_contents,
            ..
        } = target.data
        {
            template_contents
                .get()
                .expect("template element missing template_contents")
        } else {
            panic!("get_template_contents called on non-element");
        }
    }

    fn set_quirks_mode(&self, mode: QuirksMode) {
        self.quirks_mode.set(mode);
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        std::ptr::eq(*x, *y)
    }

    fn elem_name<'b>(&'b self, target: &'b Self::Handle) -> ExpandedName<'b> {
        if let NodeData::Element { ref name, .. } = target.data {
            name.expanded()
        } else {
            panic!("elem_name called on non-element");
        }
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags,
    ) -> Self::Handle {
        let template_contents = Cell::new(if flags.template {
            Some(self.new_node(NodeData::Document))
        } else {
            None
        });
        self.new_node(NodeData::Element {
            name,
            attrs: RefCell::new(attrs),
            template_contents,
            mathml_annotation_xml_integration_point: flags.mathml_annotation_xml_integration_point,
        })
    }

    fn create_comment(&self, text: StrTendril) -> Self::Handle {
        self.new_node(NodeData::Comment { contents: text })
    }

    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Self::Handle {
        self.new_node(NodeData::ProcessingInstruction {
            target,
            contents: data,
        })
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        if let NodeOrText::AppendText(ref text) = child {
            if let Some(tail) = parent.children.borrow().last().copied() {
                if append_to_text(tail, text) {
                    return;
                }
            }
        }
        let node = match child {
            NodeOrText::AppendText(text) => self.new_node(NodeData::Text {
                contents: RefCell::new(text),
            }),
            NodeOrText::AppendNode(n) => n,
        };
        append_child(*parent, node);
    }

    fn append_before_sibling(&self, sibling: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let (parent, i) =
            parent_and_index(*sibling).expect("append_before_sibling called on detached node");

        let node = match (child, i) {
            (NodeOrText::AppendText(text), 0) => self.new_node(NodeData::Text {
                contents: RefCell::new(text),
            }),
            (NodeOrText::AppendText(text), idx) => {
                let children = parent.children.borrow();
                let prev = children[idx - 1];
                if append_to_text(prev, &text) {
                    return;
                }
                drop(children);
                self.new_node(NodeData::Text {
                    contents: RefCell::new(text),
                })
            }
            (NodeOrText::AppendNode(n), _) => n,
        };

        detach(node);
        node.parent.set(Some(parent));
        parent.children.borrow_mut().insert(i, node);
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        if element.parent.get().is_some() {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn append_doctype_to_document(
        &self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        append_child(
            self.document,
            self.new_node(NodeData::Doctype {
                name,
                public_id,
                system_id,
            }),
        );
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<Attribute>) {
        let mut existing = match target.data {
            NodeData::Element { ref attrs, .. } => attrs.borrow_mut(),
            _ => panic!("add_attrs_if_missing called on non-element"),
        };
        let have: HashSet<_> = existing.iter().map(|a| a.name.clone()).collect();
        existing.extend(attrs.into_iter().filter(|a| !have.contains(&a.name)));
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        detach(*target);
    }

    fn reparent_children(&self, from: &Self::Handle, to: &Self::Handle) {
        let mut src = from.children.borrow_mut();
        let mut dst = to.children.borrow_mut();
        for child in src.iter() {
            let previous = child.parent.replace(Some(*to));
            debug_assert!(previous.map(|p| std::ptr::eq(p, *from)).unwrap_or(true));
        }
        dst.extend(mem::take(&mut *src));
    }

    fn is_mathml_annotation_xml_integration_point(&self, target: &Self::Handle) -> bool {
        match target.data {
            NodeData::Element {
                mathml_annotation_xml_integration_point,
                ..
            } => mathml_annotation_xml_integration_point,
            _ => panic!("is_mathml_annotation_xml_integration_point on non-element"),
        }
    }
}

// --- Serialization --------------------------------------------------------

pub(crate) struct SerializableHandle<'a>(pub Handle<'a>);

enum SerializeOp<'a> {
    Open(Handle<'a>),
    Close(QualName),
}

impl<'a> Serialize for SerializableHandle<'a> {
    fn serialize<S>(&self, out: &mut S, scope: TraversalScope) -> io::Result<()>
    where
        S: Serializer,
    {
        let mut ops: VecDeque<SerializeOp<'a>> = VecDeque::new();
        match scope {
            TraversalScope::IncludeNode => ops.push_back(SerializeOp::Open(self.0)),
            TraversalScope::ChildrenOnly(_) => {
                for child in self.0.children.borrow().iter() {
                    ops.push_back(SerializeOp::Open(*child));
                }
            }
        }

        while let Some(op) = ops.pop_front() {
            match op {
                SerializeOp::Open(handle) => match handle.data {
                    NodeData::Element {
                        ref name,
                        ref attrs,
                        ..
                    } => {
                        out.start_elem(
                            name.clone(),
                            attrs.borrow().iter().map(|a| (&a.name, &a.value[..])),
                        )?;
                        let children = handle.children.borrow();
                        ops.reserve(1 + children.len());
                        ops.push_front(SerializeOp::Close(name.clone()));
                        for child in children.iter().rev() {
                            ops.push_front(SerializeOp::Open(*child));
                        }
                    }
                    NodeData::Doctype { ref name, .. } => out.write_doctype(name)?,
                    NodeData::Text { ref contents } => out.write_text(&contents.borrow())?,
                    NodeData::Comment { ref contents } => out.write_comment(contents)?,
                    NodeData::ProcessingInstruction {
                        ref target,
                        ref contents,
                    } => out.write_processing_instruction(target, contents)?,
                    NodeData::Document => {
                        for child in handle.children.borrow().iter().rev() {
                            ops.push_front(SerializeOp::Open(*child));
                        }
                    }
                },
                SerializeOp::Close(name) => out.end_elem(name)?,
            }
        }
        Ok(())
    }
}
