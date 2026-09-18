use crate::{ControlKind, Error, Rect, Ui, controls, css, html};

/// Application-owned data for one select option. Labels are plain text, not HTML.
/// Values need not be unique; `set_value` uses the first match, preservation the first enabled match.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectItem {
    pub value: String,
    pub label: String,
    pub disabled: bool,
    pub selected: bool,
    /// Optional CSS class names. `option`, class, value and state selectors apply.
    pub class: String,
}
impl SelectItem {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            ..Self::default()
        }
    }
}
impl Ui {
    pub fn select_option_count(&self, id: &str) -> Option<usize> {
        match &self.controls.get(*self.ids.get(id)?)?.kind {
            ControlKind::Select { options, .. } => Some(options.len()),
            _ => None,
        }
    }
    /// Atomically replace options. Copies all data; no caller strings are retained.
    /// Last explicit `selected` wins (even if disabled), otherwise preserve the old
    /// value if enabled in the new list, otherwise choose the first enabled item.
    /// Empty/all-disabled lists without an explicit selection have no value.
    /// Host updates do not emit `changed`. Closes this select's popup/capture.
    pub fn set_select_options(&mut self, id: &str, items: &[SelectItem]) -> Result<(), Error> {
        let i = *self
            .ids
            .get(id)
            .ok_or_else(|| Error::new("unknown select ID"))?;
        let ControlKind::Select {
            options: old,
            selected,
        } = &self.controls[i].kind
        else {
            return Err(Error::new("set_select_options requires a select"));
        };
        let old_value = selected.map(|n| old[n].value.as_str());
        let mut options = Vec::with_capacity(items.len());
        for item in items {
            if [&item.value, &item.label, &item.class]
                .iter()
                .any(|s| s.contains('\0'))
            {
                return Err(Error::new("select item strings cannot contain NUL"));
            }
            let node = html::Node::option(&item.value, &item.class, item.disabled, item.selected);
            options.push(controls::SelectOption {
                style: css::compute(&node, &self.option_rules, self.layout.is_some())?,
                states: css::state_styles(
                    &node,
                    &self.option_rules,
                    css::Part::Control,
                    self.layout.is_some(),
                )?,
                label: controls::single_line(&item.label),
                value: item.value.clone(),
                disabled: item.disabled,
            });
        }
        let selected = items
            .iter()
            .rposition(|o| o.selected)
            .or_else(|| {
                old_value.and_then(|v| items.iter().position(|o| o.value == v && !o.disabled))
            })
            .or_else(|| items.iter().position(|o| !o.disabled));
        let rect = self.controls[i].rect;
        Rect {
            y: rect.y + rect.h,
            h: rect.h * options.len().min(controls::POPUP_ROWS) as f32,
            ..rect
        }
        .validate()?;
        self.controls[i].kind = ControlKind::Select { options, selected };
        if self.popup.as_ref().is_some_and(|p| p.control == i) {
            self.popup = None;
        }
        if self.pointer_capture == Some(i) {
            self.pointer_capture = None;
        }
        if self.keyboard_capture == Some(i) {
            self.keyboard_capture = None;
        }
        self.invalidate_paint();
        Ok(())
    }
}
