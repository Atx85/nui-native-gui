//! Compile an HTML/CSS control UI once, then poll it from your own event loop.
mod canvas;
pub use canvas::{CanvasInput, CanvasRegion};
mod checkbox_group;
mod elements;
pub use elements::{Button, Checkbox, Element};
mod controls;
mod css;
mod decoration;
mod image;
mod input;
use controls::{ControlKind, SelectOption, SelectPopup};
pub use image::ImageDraw;
mod html;
mod layout;
mod paint;
mod parts;
mod range;
mod raster;
mod redraw;
mod scrolling;
mod select;
pub use paint::{Color, PaintCommand, Renderer, Theme};
pub use select::SelectItem;
use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::Path,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(String);
impl Error {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl Rect {
    /// Half-open bounds: a shared edge belongs to only one adjacent control.
    pub fn contains(self, x: f32, y: f32) -> bool {
        self.w > 0.0
            && self.h > 0.0
            && x >= self.x
            && x < self.x + self.w
            && y >= self.y
            && y < self.y + self.h
    }
    pub(crate) fn validate(self) -> Result<(), Error> {
        if [
            self.x,
            self.y,
            self.w,
            self.h,
            self.x + self.w,
            self.y + self.h,
        ]
        .iter()
        .any(|n| !n.is_finite())
            || self.w < 0.0
            || self.h < 0.0
        {
            return Err(Error::new(
                "rectangle must be finite, with nonnegative width and height",
            ));
        }
        Ok(())
    }
}
#[derive(Debug)]
struct Control {
    kind: ControlKind,
    parts: parts::Parts,
    layout_rect: Rect,
    clip: scrolling::Clip,
    scroll: scrolling::ScrollState,
    rect: Rect,
    style: css::ControlStyle,
    state_styles: [decoration::Appearance; css::STATE_COUNT],
    picker: Option<Box<css::CompiledPart>>,
    selection: Option<Box<css::CompiledPart>>,
    tab_stop: bool,
    disabled: bool,
    clicked: bool,
    changed: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Tab,
    BackTab,
    Enter,
    Space,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Backspace,
    Delete,
    SelectAll,
    Escape,
    PageUp,
    PageDown,
}

#[derive(Debug, Default)]
pub struct Ui {
    paint_revision: redraw::Revision,
    option_rules: Vec<css::Rule>,
    rules: Vec<css::Rule>,
    used_ids: HashSet<String>,
    root: Option<usize>,
    checkbox_groups: HashMap<String, Vec<usize>>,
    controls: Vec<Control>,
    layout: Option<layout::Tree>,
    background_color: Option<Color>,
    popup: Option<SelectPopup>,
    scroll_capture: Option<scrolling::Drag>,
    ids: HashMap<String, usize>,
    pointer: Option<(f32, f32)>,
    pointer_capture: Option<usize>,
    keyboard_capture: Option<usize>,
    focused: Option<usize>,
}
impl Ui {
    pub fn new() -> Self {
        Self::default()
    }
    /// Parse once; retain compiled controls and parsed CSS for explicit additions.
    /// All controls require position:absolute and explicit left/top/width/height.
    pub fn from_html_css(html: &str, css: &str) -> Result<Self, Error> {
        Self::compile_html_css(html, css, None)
    }
    /// Compile a retained relative layout at a logical viewport size. Reuse
    /// `set_viewport` on resize; HTML and CSS are not parsed again.
    pub fn from_html_css_with_viewport(
        html: &str,
        css: &str,
        width: f32,
        height: f32,
    ) -> Result<Self, Error> {
        Rect {
            x: 0.0,
            y: 0.0,
            w: width,
            h: height,
        }
        .validate()?;
        Self::compile_html_css(html, css, Some((width, height)))
    }
    fn compile_html_css(
        html: &str,
        css: &str,
        viewport: Option<(f32, f32)>,
    ) -> Result<Self, Error> {
        let document = html::parse(html);
        let mut styles = css.to_owned();
        fn embedded(node: &html::Handle, out: &mut String) {
            if node.tag() == "style" {
                out.push('\n');
                out.push_str(&node.text_content());
            }
            for child in node.children.borrow().iter() {
                embedded(child, out);
            }
        }
        embedded(&document, &mut styles);
        let rules = css::parse(&styles)?;
        let mut ui = Self::new();
        ui.option_rules = css::option_rules(&rules);
        ui.layout = viewport.map(|_| layout::Tree::default());
        let mut ids = HashSet::new();
        compile_node(
            &document,
            &rules,
            &mut ui,
            &mut ids,
            ("", false, false, None),
        )?;
        for control in &ui.controls {
            if let ControlKind::Label {
                target: Some(target),
                ..
            } = &control.kind
                && !ui.ids.get(target).is_some_and(|&i| {
                    !matches!(
                        ui.controls[i].kind,
                        ControlKind::Label { .. }
                            | ControlKind::Canvas(_)
                            | ControlKind::Container
                            | ControlKind::Image { .. }
                    )
                })
            {
                return Err(Error::new(format!(
                    "label target is not a control: {target}"
                )));
            }
        }
        if let Some((width, height)) = viewport {
            ui.set_viewport(width, height)?;
        }
        ui.used_ids = ids;
        ui.rules = rules;
        Ok(ui)
    }
    /// Load two explicitly supplied local files. Relative paths are relative to the host process.
    pub fn load(html: impl AsRef<Path>, css: impl AsRef<Path>) -> Result<Self, Error> {
        let read = |path: &Path| {
            std::fs::read_to_string(path)
                .map_err(|e| Error::new(format!("{}: {e}", path.display())))
        };
        Self::from_html_css(&read(html.as_ref())?, &read(css.as_ref())?)
    }
    /// Optional procedural construction for tools. Declarative loading is the default path.
    pub fn add_button_at(&mut self, id: &str, text: &str, rect: Rect) -> Result<(), Error> {
        rect.validate()?;
        if id.is_empty()
            || id.chars().any(|c| c.is_ascii_whitespace())
            || self.used_ids.contains(id)
            || self.ids.contains_key(id)
        {
            return Err(Error::new(format!("invalid or duplicate UI id: {id}")));
        }
        self.invalidate_paint();
        self.used_ids.insert(id.into());
        self.ids.insert(id.into(), self.controls.len());
        if let Some(tree) = &mut self.layout {
            tree.push(None);
            tree.rebuild_order();
            tree.viewport = None;
        }
        self.controls.push(Control {
            parts: Default::default(),
            layout_rect: rect,
            clip: Default::default(),
            scroll: Default::default(),
            kind: ControlKind::Button { text: text.into() },
            rect,
            style: css::ControlStyle {
                rect,
                layout: layout::Style {
                    position: layout::Position::Absolute,
                    width: layout::Dimension::Px(rect.w),
                    height: layout::Dimension::Px(rect.h),
                    ..Default::default()
                },
                ..Default::default()
            },
            state_styles: [decoration::Appearance::default(); css::STATE_COUNT],
            picker: None,
            selection: None,
            tab_stop: false,
            disabled: false,
            clicked: false,
            changed: false,
        });
        Ok(())
    }
    /// True if this button clicked at least once this frame. Repeated reads do not consume it.
    /// Unknown IDs return false; use `contains_id` to validate application bindings at startup.
    pub fn clicked(&self, id: &str) -> bool {
        self.ids.get(id).is_some_and(|&i| self.controls[i].clicked)
    }
    pub fn contains_id(&self, id: &str) -> bool {
        self.ids.contains_key(id)
    }
    /// Solid body background requested by CSS. The host controls clearing its canvas.
    pub fn background_color(&self) -> Option<Color> {
        self.background_color
    }
    pub fn bounds(&self, id: &str) -> Option<Rect> {
        self.ids.get(id).map(|&i| self.controls[i].rect)
    }
    /// Current text, checkbox submission value or selected option value. Wrong control kinds and missing selections return None.
    pub fn get_value(&self, id: &str) -> Option<&str> {
        match &self.controls.get(*self.ids.get(id)?)?.kind {
            ControlKind::TextInput(input) => Some(&input.value),
            ControlKind::Checkbox { value, .. } => Some(value),
            ControlKind::Range(r) => Some(&r.text),
            ControlKind::Select { options, selected } => {
                selected.map(|i| options[i].value.as_str())
            }
            _ => None,
        }
    }
    /// Persistent checkbox state; None means an unknown ID or a different control kind.
    pub fn checked(&self, id: &str) -> Option<bool> {
        match self.controls.get(*self.ids.get(id)?)?.kind {
            ControlKind::Checkbox { checked, .. } => Some(checked),
            _ => None,
        }
    }
    pub fn selected_index(&self, id: &str) -> Option<usize> {
        match self.controls.get(*self.ids.get(id)?)?.kind {
            ControlKind::Select { selected, .. } => selected,
            _ => None,
        }
    }
    /// True after a user edit this frame. Reading does not consume the flag.
    pub fn changed(&self, id: &str) -> bool {
        self.ids.get(id).is_some_and(|&i| self.controls[i].changed)
    }
    /// Update text or select a matching option value. Host updates do not emit changed events.
    pub fn set_value(&mut self, id: &str, value: &str) -> Result<(), Error> {
        let i = *self
            .ids
            .get(id)
            .ok_or_else(|| Error::new(format!("unknown UI id: {id}")))?;
        match &mut self.controls[i].kind {
            ControlKind::Range(r) => {
                if !r.set_text(value)? {
                    return Ok(());
                }
            }
            ControlKind::TextInput(input) => {
                let value = controls::single_line(value);
                if input
                    .max_length
                    .is_some_and(|max| value.chars().count() > max)
                {
                    return Err(Error::new("text exceeds maxlength"));
                }
                if input.value == value
                    && input.cursor == value.len()
                    && input.anchor == value.len()
                {
                    return Ok(());
                }
                input.hit_positions.get_mut().clear();
                input.value = value;
                input.cursor = input.value.len();
                input.anchor = input.cursor;
            }
            ControlKind::Select { options, selected } => {
                let next = options
                    .iter()
                    .position(|option| option.value == value)
                    .ok_or_else(|| Error::new(format!("unknown option value: {value}")))?;
                if *selected == Some(next) && !self.popup.as_ref().is_some_and(|p| p.control == i) {
                    return Ok(());
                }
                *selected = Some(next);
                if self.popup.as_ref().is_some_and(|popup| popup.control == i) {
                    self.popup = None;
                }
            }
            _ => {
                return Err(Error::new(
                    "set_value requires a text input, range or select",
                ));
            }
        }
        self.invalidate_paint();
        Ok(())
    }
    /// Set a checkbox from the host without emitting a changed event.
    pub fn set_checked(&mut self, id: &str, value: bool) -> Result<(), Error> {
        let i = *self
            .ids
            .get(id)
            .ok_or_else(|| Error::new(format!("unknown UI id: {id}")))?;
        if let ControlKind::Checkbox { checked, .. } = &mut self.controls[i].kind {
            if *checked != value {
                *checked = value;
                self.invalidate_paint();
            }
            Ok(())
        } else {
            Err(Error::new("set_checked requires a checkbox"))
        }
    }
    /// Whether the host should enable platform text input for the focused control.
    pub fn wants_text_input(&self) -> bool {
        self.focused.is_some_and(
            |i| matches!(&self.controls[i].kind, ControlKind::TextInput(input) if !input.readonly),
        )
    }
    /// Selected text for host clipboard integration. Readonly fields can still be copied.
    pub fn selected_text(&self) -> Option<&str> {
        if let ControlKind::TextInput(input) = &self.controls.get(self.focused?)?.kind {
            let range = input.selection();
            if !range.is_empty() {
                return Some(&input.value[range]);
            }
        }
        None
    }
    pub fn select_is_open(&self) -> bool {
        self.popup.is_some()
    }
    /// Call once, after application polling and rendering for the frame.
    pub fn end_frame(&mut self) {
        for control in &mut self.controls {
            control.clicked = false;
            control.changed = false;
            if let ControlKind::Canvas(state) = &mut control.kind {
                state.pressed = None;
                state.released = None;
                state.cancelled = false;
            }
        }
    }
}

fn compile_node(
    node: &html::Handle,
    rules: &[css::Rule],
    ui: &mut Ui,
    ids: &mut HashSet<String>,
    context: (&str, bool, bool, Option<usize>),
) -> Result<(), Error> {
    let (parent, in_label, metadata, layout_parent) = context;
    let mut child_layout_parent = layout_parent;
    let flow = ui.layout.is_some();
    if let Some(id) = node.attr("id") {
        if id.is_empty() || id.chars().any(|c| c.is_ascii_whitespace()) {
            return Err(Error::new(
                "IDs must be nonempty and contain no ASCII whitespace",
            ));
        }
        if !ids.insert(id.clone()) {
            return Err(Error::new(format!("duplicate UI id: {id}")));
        }
    }
    let tag = node.tag();
    if node
        .name
        .as_ref()
        .is_some_and(|name| &*name.ns != "http://www.w3.org/1999/xhtml")
    {
        return Err(Error::new(
            "SVG and other non-HTML elements are not supported",
        ));
    }
    let metadata = metadata || matches!(tag, "head" | "title" | "style" | "meta" | "link");
    match tag {
        "" | "html" | "head" | "body" | "title" | "meta" | "link" | "style" | "div" | "span"
        | "button" | "input" | "select" | "option" | "label" | "canvas" | "main" | "aside"
        | "section" | "header" | "footer" | "img" => {}
        _ => return Err(Error::new(format!("unsupported HTML element <{tag}>"))),
    }
    if !metadata {
        for attr in node.attrs.borrow().iter() {
            let name: &str = &attr.name.local;
            if !matches!(name, "id" | "class" | "style" | "lang")
                && !(tag == "button" && matches!(name, "disabled" | "type"))
                && !(tag == "input"
                    && matches!(
                        name,
                        "disabled"
                            | "name"
                            | "type"
                            | "value"
                            | "placeholder"
                            | "checked"
                            | "readonly"
                            | "maxlength"
                            | "min"
                            | "max"
                            | "step"
                    ))
                && !(tag == "select" && name == "disabled")
                && !(matches!(
                    tag,
                    "canvas" | "body" | "div" | "main" | "aside" | "section" | "header" | "footer"
                ) && name == "tabindex")
                && !(tag == "img" && matches!(name, "src" | "alt"))
                && !(tag == "label" && name == "for")
                && !(tag == "option" && matches!(name, "value" | "selected" | "disabled"))
            {
                return Err(Error::new(format!(
                    "unsupported attribute '{name}' on <{tag}>"
                )));
            }
        }
        let style = css::compute(node, rules, flow)?;
        let rect = style.rect;
        if tag == "body" {
            ui.background_color = style.appearance.background;
        }
        if tag == "option" && parent != "select" {
            return Err(Error::new("<option> must be a direct child of <select>"));
        }
        if parent == "select"
            && !(tag == "option" || (tag.is_empty() && node.text.borrow().trim().is_empty()))
        {
            return Err(Error::new("<select> supports only <option> children"));
        }
        if in_label && matches!(tag, "button" | "input" | "select" | "canvas" | "img") {
            return Err(Error::new(
                "nested controls and images inside text labels are not supported",
            ));
        }
        if parent == "canvas" && !(tag.is_empty() && node.text.borrow().trim().is_empty()) {
            return Err(Error::new(
                "native canvas does not support fallback content or nested controls",
            ));
        }
        let kind = match tag {
            "body" | "div" | "main" | "aside" | "section" | "header" | "footer"
                if flow && !in_label =>
            {
                Some(ControlKind::Container)
            }
            "img" => Some(ControlKind::Image {
                src: node.attr("src").unwrap_or_default(),
                alt: node.attr("alt").unwrap_or_default(),
            }),
            "canvas" => {
                let tab_stop = match node.attr("tabindex").as_deref() {
                    None | Some("-1") => false,
                    Some("0") => true,
                    _ => return Err(Error::new("canvas tabindex supports 0 or -1 only")),
                };
                Some(ControlKind::Canvas(canvas::CanvasState {
                    id: node.attr("id"),
                    tab_stop,
                    ..Default::default()
                }))
            }
            "button" => {
                if node
                    .attr("type")
                    .is_some_and(|kind| !kind.eq_ignore_ascii_case("button"))
                {
                    return Err(Error::new(
                        "only type=button is supported (no form submission)",
                    ));
                }
                Some(ControlKind::Button {
                    text: controls::label(node),
                })
            }
            "input" => Some(controls::compile_input(node)?),
            "label" => Some(ControlKind::Label {
                text: controls::label(node),
                target: node.attr("for"),
            }),
            "select" => {
                let options: Vec<_> = node
                    .children
                    .borrow()
                    .iter()
                    .filter(|child| child.tag() == "option")
                    .map(|child| {
                        Ok(SelectOption {
                            style: css::compute(child, rules, flow)?,
                            states: css::state_styles(child, rules, css::Part::Control, flow)?,
                            label: controls::label(child),
                            value: child
                                .attr("value")
                                .unwrap_or_else(|| controls::label(child)),
                            disabled: child.attr("disabled").is_some(),
                        })
                    })
                    .collect::<Result<_, Error>>()?;
                // Single selects use the last explicit selection, then the first enabled option.
                let selected = node
                    .children
                    .borrow()
                    .iter()
                    .filter(|child| child.tag() == "option")
                    .enumerate()
                    .filter(|(_, child)| child.attr("selected").is_some())
                    .map(|(i, _)| i)
                    .last()
                    .or_else(|| options.iter().position(|option| !option.disabled));
                Rect {
                    y: rect.y + rect.h,
                    h: rect.h * options.len().min(controls::POPUP_ROWS) as f32,
                    ..rect
                }
                .validate()?;
                Some(ControlKind::Select { options, selected })
            }
            _ => None,
        };
        if let Some(kind) = kind {
            let index = ui.controls.len();
            if tag == "body" && flow {
                ui.root = Some(index);
            }
            if matches!(kind, ControlKind::Checkbox { .. })
                && let Some(name) = node.attr("name")
            {
                ui.checkbox_groups.entry(name).or_default().push(index);
            }
            if let Some(tree) = &mut ui.layout {
                tree.push(layout_parent);
                if matches!(kind, ControlKind::Container) {
                    child_layout_parent = Some(index);
                }
            }
            if let Some(id) = node.attr("id") {
                ui.ids.insert(id, index);
            }
            ui.controls.push(Control {
                kind,
                parts: parts::Parts::compile(node, rules, flow)?,
                layout_rect: rect,
                clip: Default::default(),
                scroll: Default::default(),
                rect,
                style,
                state_styles: css::state_styles(node, rules, css::Part::Control, flow)?,
                picker: css::compile_part(node, rules, css::Part::PickerIcon, flow)?,
                selection: css::compile_part(node, rules, css::Part::Selection, flow)?,
                tab_stop: match node.attr("tabindex").as_deref() {
                    None | Some("-1") => false,
                    Some("0") => true,
                    _ => return Err(Error::new("tabindex supports 0 or -1 only")),
                },
                disabled: node.attr("disabled").is_some(),
                clicked: false,
                changed: false,
            });
        } else if !in_label && !node.text.borrow().trim().is_empty() {
            return Err(Error::new(
                "standalone text is not supported yet; use button or option labels",
            ));
        }
    }
    for child in node.children.borrow().iter() {
        compile_node(
            child,
            rules,
            ui,
            ids,
            (
                if tag.is_empty() { parent } else { tag },
                in_label || matches!(tag, "button" | "option" | "label"),
                metadata,
                child_layout_parent,
            ),
        )?;
    }
    Ok(())
}
