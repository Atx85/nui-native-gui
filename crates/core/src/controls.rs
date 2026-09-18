use crate::{Error, Key, html::Node};

pub(crate) const POPUP_ROWS: usize = 8;

#[derive(Debug)]
pub(crate) enum ControlKind {
    Container,
    Range(crate::range::RangeInput),
    Image {
        src: String,
        alt: String,
    },
    Canvas(crate::canvas::CanvasState),
    Button {
        text: String,
    },
    Label {
        text: String,
        target: Option<String>,
    },
    TextInput(TextInput),
    Checkbox {
        checked: bool,
        value: String,
    },
    Select {
        options: Vec<SelectOption>,
        selected: Option<usize>,
    },
}

#[derive(Debug)]
pub(crate) struct SelectOption {
    pub style: crate::css::ControlStyle,
    pub states: [crate::decoration::Appearance; crate::css::STATE_COUNT],
    pub label: String,
    pub value: String,
    pub disabled: bool,
}

#[derive(Debug)]
pub(crate) struct SelectPopup {
    pub control: usize,
    pub highlighted: Option<usize>,
    pub first: usize,
    pub pressed: Option<usize>,
}
impl SelectPopup {
    pub fn reveal(&mut self) {
        if let Some(index) = self.highlighted {
            self.first = self
                .first
                .min(index)
                .max(index.saturating_sub(POPUP_ROWS - 1));
        }
    }
}

#[derive(Debug)]
pub(crate) struct TextInput {
    pub value: String,
    pub placeholder: String,
    // UTF-8 byte offsets, always on character boundaries.
    pub cursor: usize,
    pub anchor: usize,
    pub readonly: bool,
    pub max_length: Option<usize>,
    pub hit_positions: std::cell::RefCell<Vec<(usize, f32)>>,
}
impl TextInput {
    pub fn hit_cursor(&self, x: f32) -> usize {
        self.hit_positions
            .borrow()
            .iter()
            .min_by(|a, b| (a.1 - x).abs().total_cmp(&(b.1 - x).abs()))
            .map_or(self.value.len(), |&(index, _)| index)
    }
    pub fn selection(&self) -> std::ops::Range<usize> {
        self.cursor.min(self.anchor)..self.cursor.max(self.anchor)
    }
    pub fn insert(&mut self, text: &str) -> bool {
        if self.readonly {
            return false;
        }
        let text = single_line(text);
        if text.is_empty() {
            return false;
        }
        let selection = self.selection();
        let remaining = self.max_length.map_or(usize::MAX, |max| {
            max.saturating_sub(
                self.value.chars().count() - self.value[selection.clone()].chars().count(),
            )
        });
        let text: String = text.chars().take(remaining).collect();
        if text.is_empty() {
            return false;
        }
        let changed = self.value[selection.clone()] != text;
        self.hit_positions.get_mut().clear();
        self.value.replace_range(selection.clone(), &text);
        self.cursor = selection.start + text.len();
        self.anchor = self.cursor;
        changed
    }
    pub fn key(&mut self, key: Key, shift: bool) -> bool {
        let selection = self.selection();
        let previous = self.value[..self.cursor]
            .char_indices()
            .next_back()
            .map_or(0, |(i, _)| i);
        let next = self.value[self.cursor..]
            .chars()
            .next()
            .map_or(self.cursor, |c| self.cursor + c.len_utf8());
        match key {
            Key::Left | Key::Right | Key::Home | Key::End => {
                self.cursor = match key {
                    Key::Left if !shift && !selection.is_empty() => selection.start,
                    Key::Right if !shift && !selection.is_empty() => selection.end,
                    Key::Left => previous,
                    Key::Right => next,
                    Key::Home => 0,
                    _ => self.value.len(),
                };
                if !shift {
                    self.anchor = self.cursor;
                }
            }
            Key::SelectAll => {
                self.anchor = 0;
                self.cursor = self.value.len();
            }
            Key::Backspace | Key::Delete if !self.readonly => {
                let range = if !selection.is_empty() {
                    selection
                } else if key == Key::Backspace {
                    previous..self.cursor
                } else {
                    self.cursor..next
                };
                if !range.is_empty() {
                    self.hit_positions.get_mut().clear();
                    self.value.replace_range(range.clone(), "");
                    self.cursor = range.start;
                    self.anchor = self.cursor;
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

pub(crate) fn label(node: &Node) -> String {
    node.text_content()
        .split_ascii_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
pub(crate) fn single_line(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '\u{2028}' | '\u{2029}'))
        .collect()
}
pub(crate) fn compile_input(node: &Node) -> Result<ControlKind, Error> {
    if let Some(name) = node.attr("name") {
        if !node
            .attr("type")
            .is_some_and(|t| t.eq_ignore_ascii_case("checkbox"))
        {
            return Err(Error::new("name currently requires type=checkbox"));
        }
        if name.is_empty() {
            return Err(Error::new("checkbox group name must not be empty"));
        }
    }
    match node
        .attr("type")
        .unwrap_or_else(|| "text".into())
        .to_ascii_lowercase()
        .as_str()
    {
        "range" => Ok(ControlKind::Range(crate::range::RangeInput::compile(node)?)),
        "text" => {
            if ["min", "max", "step"]
                .iter()
                .any(|a| node.attr(a).is_some())
            {
                return Err(Error::new("min, max and step require type=range"));
            }
            if node.attr("checked").is_some() {
                return Err(Error::new("checked requires type=checkbox"));
            }
            let max_length = node
                .attr("maxlength")
                .map(|n| {
                    n.parse::<usize>()
                        .map_err(|_| Error::new("maxlength must be a nonnegative integer"))
                })
                .transpose()?;
            let value = single_line(&node.attr("value").unwrap_or_default());
            Ok(ControlKind::TextInput(TextInput {
                cursor: value.len(),
                anchor: value.len(),
                value,
                placeholder: single_line(&node.attr("placeholder").unwrap_or_default()),
                readonly: node.attr("readonly").is_some(),
                max_length,
                hit_positions: Default::default(),
            }))
        }
        "checkbox" => {
            if ["placeholder", "readonly", "maxlength", "min", "max", "step"]
                .iter()
                .any(|attr| node.attr(attr).is_some())
            {
                return Err(Error::new(
                    "checkbox supports checked and disabled, not text input attributes",
                ));
            }
            Ok(ControlKind::Checkbox {
                checked: node.attr("checked").is_some(),
                value: node.attr("value").unwrap_or_else(|| "on".into()),
            })
        }
        _ => Err(Error::new(
            "supported input types are text, checkbox and range",
        )),
    }
}
