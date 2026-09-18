//! Typed additions compile transient nodes using the same path as loaded HTML.
use crate::{ControlKind, Error, Rect, Ui, html};
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
pub struct Checkbox {
    pub id: String,
    /// None generates a readable label from the ID; Some("") hides the label.
    pub label: Option<String>,
    pub class: String,
    pub checked: bool,
    pub disabled: bool,
    pub name: Option<String>,
    pub value: Option<String>,
}
#[derive(Clone, Debug, Default)]
pub struct Button {
    pub id: String,
    pub label: Option<String>,
    pub class: String,
    pub disabled: bool,
}
macro_rules! common {
    ($kind:ty) => {
        impl $kind {
            pub fn new(id: impl Into<String>) -> Self {
                Self {
                    id: id.into(),
                    ..Default::default()
                }
            }
            pub fn label(mut self, value: impl Into<String>) -> Self {
                self.label = Some(value.into());
                self
            }
            pub fn class(mut self, value: impl Into<String>) -> Self {
                self.class = value.into();
                self
            }
            pub fn disabled(mut self, value: bool) -> Self {
                self.disabled = value;
                self
            }
        }
    };
}
common!(Checkbox);
common!(Button);
impl Checkbox {
    pub fn checked(mut self, value: bool) -> Self {
        self.checked = value;
        self
    }
    pub fn group(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
}
fn readable_label(id: &str) -> String {
    let chars: Vec<_> = id.chars().collect();
    let mut result = String::new();
    for (i, &ch) in chars.iter().enumerate() {
        if ch == '_' || ch == '-' || ch.is_whitespace() {
            if !result.ends_with(' ') && !result.is_empty() {
                result.push(' ');
            }
            continue;
        }
        if i > 0
            && ch.is_uppercase()
            && !result.ends_with(' ')
            && (chars[i - 1].is_lowercase()
                || chars[i - 1].is_numeric()
                || (chars[i - 1].is_uppercase()
                    && chars.get(i + 1).is_some_and(|c| c.is_lowercase())))
        {
            result.push(' ');
        }
        if result.is_empty() || result.ends_with(' ') {
            result.extend(ch.to_uppercase());
        } else {
            result.push(ch);
        }
    }
    result.trim().to_owned()
}

/// A borrowed element, not a DOM node. It cannot outlive the Ui or be retained
/// across another mutable borrow. Reacquire it with get(id) when needed.
pub struct Element<'a> {
    ui: &'a mut Ui,
    id: String,
}
impl Element<'_> {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn clicked(&self) -> bool {
        self.ui.clicked(&self.id)
    }
    pub fn changed(&self) -> bool {
        self.ui.changed(&self.id)
    }
    pub fn get_value(&self) -> Option<&str> {
        self.ui.get_value(&self.id)
    }
    pub fn set_value(&mut self, value: &str) -> Result<(), Error> {
        self.ui.set_value(&self.id, value)
    }
    pub fn checked(&self) -> Option<bool> {
        self.ui.checked(&self.id)
    }
    pub fn set_checked(&mut self, checked: bool) -> Result<(), Error> {
        self.ui.set_checked(&self.id, checked)
    }
    pub fn bounds(&self) -> Rect {
        self.ui.bounds(&self.id).unwrap()
    }
    pub fn add_checkbox(&mut self, value: Checkbox) -> Result<Element<'_>, Error> {
        let parent = self.ui.ids[&self.id];
        self.ui.insert_checkbox(Some(parent), value)
    }
    pub fn add_button(&mut self, value: Button) -> Result<Element<'_>, Error> {
        let parent = self.ui.ids[&self.id];
        self.ui.insert_button(Some(parent), value)
    }
}
impl Ui {
    pub fn get(&mut self, id: &str) -> Result<Element<'_>, Error> {
        if !self.contains_id(id) {
            return Err(Error::new(format!("unknown element ID: {id}")));
        }
        Ok(Element {
            ui: self,
            id: id.into(),
        })
    }
    pub fn add_checkbox(&mut self, value: Checkbox) -> Result<Element<'_>, Error> {
        self.insert_checkbox(self.root, value)
    }
    pub fn add_button(&mut self, value: Button) -> Result<Element<'_>, Error> {
        self.insert_button(self.root, value)
    }
    fn insert_checkbox(
        &mut self,
        parent: Option<usize>,
        value: Checkbox,
    ) -> Result<Element<'_>, Error> {
        let label = value.label.unwrap_or_else(|| readable_label(&value.id));
        let mut attrs = vec![
            ("id", value.id.as_str()),
            ("type", "checkbox"),
            ("class", value.class.as_str()),
        ];
        if value.checked {
            attrs.push(("checked", ""));
        }
        if value.disabled {
            attrs.push(("disabled", ""));
        }
        if let Some(name) = &value.name {
            attrs.push(("name", name));
        }
        if let Some(v) = &value.value {
            attrs.push(("value", v));
        }
        let input = Rc::new(html::Node::element("input", &attrs, "", vec![]));
        let node = if label.is_empty() {
            input
        } else {
            let label = Rc::new(html::Node::element(
                "label",
                &[("for", &value.id), ("style", "flex-grow:1")],
                &label,
                vec![],
            ));
            Rc::new(html::Node::element(
                "div",
                &[
                    ("class", "native-checkbox-row"),
                    ("style", "display:flex;align-items:center;gap:8px"),
                ],
                "",
                vec![input, label],
            ))
        };
        self.insert_node(parent, node)?;
        self.get(&value.id)
    }
    fn insert_button(
        &mut self,
        parent: Option<usize>,
        value: Button,
    ) -> Result<Element<'_>, Error> {
        let label = value.label.unwrap_or_else(|| readable_label(&value.id));
        let mut attrs = vec![("id", value.id.as_str()), ("class", value.class.as_str())];
        if value.disabled {
            attrs.push(("disabled", ""));
        }
        self.insert_node(
            parent,
            Rc::new(html::Node::element("button", &attrs, &label, vec![])),
        )?;
        self.get(&value.id)
    }
    fn insert_node(&mut self, parent: Option<usize>, node: html::Handle) -> Result<(), Error> {
        let viewport = self
            .layout
            .as_ref()
            .and_then(|t| t.viewport)
            .ok_or_else(|| Error::new("typed additions require a UI with viewport layout"))?;
        if let Some(p) = parent
            && !matches!(self.controls[p].kind, ControlKind::Container)
        {
            return Err(Error::new("only containers accept children"));
        }
        fn validate(node: &html::Node) -> Result<(), Error> {
            if node.text_content().contains('\0')
                || node.attrs.borrow().iter().any(|a| a.value.contains('\0'))
            {
                return Err(Error::new("element strings cannot contain NUL"));
            }
            for c in node.children.borrow().iter() {
                validate(c)?;
            }
            Ok(())
        }
        validate(&node)?;
        let count = self.controls.len();
        let mut ids = self.used_ids.clone();
        // Temporarily move rules out to avoid a conflicting mutable Ui borrow.
        let rules = std::mem::take(&mut self.rules);
        let result =
            crate::compile_node(&node, &rules, self, &mut ids, ("div", false, false, parent));
        self.rules = rules;
        let result = result.and_then(|()| {
            self.layout.as_mut().unwrap().viewport = None;
            self.set_viewport(viewport.0, viewport.1)
        });
        if let Err(error) = result {
            self.controls.truncate(count);
            self.ids.retain(|_, i| *i < count);
            self.checkbox_groups.retain(|_, members| {
                members.retain(|i| *i < count);
                !members.is_empty()
            });
            let tree = self.layout.as_mut().unwrap();
            tree.nodes.truncate(count);
            for n in &mut tree.nodes {
                n.children.retain(|i| *i < count);
            }
            tree.viewport = Some(viewport);
            return Err(error);
        }
        self.used_ids = ids;
        Ok(())
    }
}
