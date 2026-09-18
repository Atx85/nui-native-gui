//! Temporary HTML5 tree. No parser nodes or attributes survive UI compilation.
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use html5ever::{Attribute, ExpandedName, QualName, parse_document};
use std::{
    borrow::Cow,
    cell::RefCell,
    rc::{Rc, Weak},
};

pub(crate) type Handle = Rc<Node>;
#[derive(Default)]
pub(crate) struct Node {
    pub name: Option<QualName>,
    pub attrs: RefCell<Vec<Attribute>>,
    pub text: RefCell<String>,
    pub children: RefCell<Vec<Handle>>,
    parent: RefCell<Weak<Node>>,
    template: Option<Handle>,
}
impl Node {
    pub(crate) fn element(
        tag: &str,
        attrs: &[(&str, &str)],
        text: &str,
        children: Vec<Handle>,
    ) -> Self {
        Self {
            name: Some(QualName::new(
                None,
                "http://www.w3.org/1999/xhtml".into(),
                tag.into(),
            )),
            attrs: RefCell::new(
                attrs
                    .iter()
                    .map(|(name, value)| Attribute {
                        name: QualName::new(None, "".into(), (*name).into()),
                        value: (*value).into(),
                    })
                    .collect(),
            ),
            text: RefCell::new(text.into()),
            children: RefCell::new(children),
            ..Self::default()
        }
    }
    pub(crate) fn option(value: &str, class: &str, disabled: bool, selected: bool) -> Self {
        let mut attrs = vec![("value", value), ("class", class)];
        if disabled {
            attrs.push(("disabled", ""));
        }
        if selected {
            attrs.push(("selected", ""));
        }
        Self::element("option", &attrs, "", vec![])
    }
    pub fn attr(&self, name: &str) -> Option<String> {
        self.attrs
            .borrow()
            .iter()
            .find(|a| &*a.name.local == name)
            .map(|a| a.value.to_string())
    }
    pub fn tag(&self) -> &str {
        self.name.as_ref().map_or("", |n| n.local.as_ref())
    }
    pub fn text_content(&self) -> String {
        let mut text = self.text.borrow().clone();
        for child in self.children.borrow().iter() {
            text.push_str(&child.text_content());
        }
        text
    }
}
#[derive(Default)]
struct Sink {
    document: Handle,
}
impl Sink {
    fn insert(&self, parent: &Handle, at: usize, child: NodeOrText<Handle>) {
        let child = match child {
            NodeOrText::AppendNode(node) => {
                self.remove_from_parent(&node);
                node
            }
            NodeOrText::AppendText(text) => {
                if at > 0 {
                    let children = parent.children.borrow();
                    let prev = &children[at - 1];
                    if prev.name.is_none() && !prev.text.borrow().is_empty() {
                        prev.text.borrow_mut().push_str(&text);
                        return;
                    }
                }
                Rc::new(Node {
                    text: RefCell::new(text.to_string()),
                    ..Node::default()
                })
            }
        };
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        let mut children = parent.children.borrow_mut();
        let at = at.min(children.len());
        children.insert(at, child);
    }
}
impl TreeSink for Sink {
    type Handle = Handle;
    type Output = Handle;
    type ElemName<'a> = ExpandedName<'a>;
    fn finish(self) -> Handle {
        self.document
    }
    fn get_document(&self) -> Handle {
        self.document.clone()
    }
    fn elem_name<'a>(&'a self, node: &'a Handle) -> ExpandedName<'a> {
        node.name
            .as_ref()
            .expect("HTML parser requested element name")
            .expanded()
    }
    fn create_element(&self, name: QualName, attrs: Vec<Attribute>, flags: ElementFlags) -> Handle {
        Rc::new(Node {
            name: Some(name),
            attrs: RefCell::new(attrs),
            template: flags.template.then(|| Rc::new(Node::default())),
            ..Node::default()
        })
    }
    fn create_comment(&self, _: StrTendril) -> Handle {
        Rc::new(Node::default())
    }
    fn create_pi(&self, _: StrTendril, _: StrTendril) -> Handle {
        Rc::new(Node::default())
    }
    fn append(&self, parent: &Handle, child: NodeOrText<Handle>) {
        let len = parent.children.borrow().len();
        self.insert(parent, len, child);
    }
    fn append_based_on_parent_node(
        &self,
        node: &Handle,
        previous: &Handle,
        child: NodeOrText<Handle>,
    ) {
        if node.parent.borrow().upgrade().is_some() {
            self.append_before_sibling(node, child);
        } else {
            self.append(previous, child);
        }
    }
    fn append_before_sibling(&self, sibling: &Handle, child: NodeOrText<Handle>) {
        if let NodeOrText::AppendNode(node) = &child {
            self.remove_from_parent(node);
        }
        let parent = sibling
            .parent
            .borrow()
            .upgrade()
            .expect("sibling must have a parent");
        let at = parent
            .children
            .borrow()
            .iter()
            .position(|n| Rc::ptr_eq(n, sibling))
            .unwrap();
        self.insert(&parent, at, child);
    }
    fn remove_from_parent(&self, node: &Handle) {
        if let Some(parent) = std::mem::take(&mut *node.parent.borrow_mut()).upgrade() {
            parent
                .children
                .borrow_mut()
                .retain(|child| !Rc::ptr_eq(child, node));
        }
    }
    fn reparent_children(&self, node: &Handle, parent: &Handle) {
        let children = std::mem::take(&mut *node.children.borrow_mut());
        for child in children {
            *child.parent.borrow_mut() = Rc::downgrade(parent);
            parent.children.borrow_mut().push(child);
        }
    }
    fn add_attrs_if_missing(&self, node: &Handle, attrs: Vec<Attribute>) {
        let mut existing = node.attrs.borrow_mut();
        for attr in attrs {
            if !existing.iter().any(|a| a.name == attr.name) {
                existing.push(attr);
            }
        }
    }
    fn get_template_contents(&self, node: &Handle) -> Handle {
        node.template.as_ref().unwrap().clone()
    }
    fn same_node(&self, a: &Handle, b: &Handle) -> bool {
        Rc::ptr_eq(a, b)
    }
    fn parse_error(&self, _: Cow<'static, str>) {} // Use HTML5's standard error recovery.
    fn set_quirks_mode(&self, _: QuirksMode) {}
    fn append_doctype_to_document(&self, _: StrTendril, _: StrTendril, _: StrTendril) {}
}
pub(crate) fn parse(html: &str) -> Handle {
    parse_document(Sink::default(), Default::default()).one(html)
}
