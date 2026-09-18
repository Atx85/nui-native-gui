use crate::{ControlKind, Key, Rect, SelectPopup, Ui, controls::POPUP_ROWS};

impl Ui {
    /// Topmost canvas content at a UI-space point. Open popups and covering controls
    /// block hit testing. For primary button events, poll `canvas_input` so a popup's
    /// dismissal press cannot be mistaken for a canvas press.
    pub fn canvas_at(&self, x: f32, y: f32) -> Option<crate::CanvasRegion<'_>> {
        if self.popup.is_some() {
            return None;
        }
        let region = self.controls[self.hit(x, y)?].canvas_region()?;
        (region.content.contains(x, y) && region.clip.contains(x, y)).then_some(region)
    }
    fn cancel_pointer_capture(&mut self) {
        self.scroll_capture = None;
        if let Some(index) = self.pointer_capture.take()
            && let ControlKind::Canvas(state) = &mut self.controls[index].kind
        {
            state.cancelled = true;
        }
    }
    pub(crate) fn hit(&self, x: f32, y: f32) -> Option<usize> {
        self.paint_order().rev().find_map(|(i, bar)| {
            let c = &self.controls[i];
            if !c.clip.contains(x, y) {
                return None;
            }
            if bar {
                self.bars(i)
                    .iter()
                    .any(|b| b.track.contains(x, y))
                    .then_some(i)
            } else {
                c.rect.contains(x, y).then_some(i)
            }
        })
    }
    pub(crate) fn cancel_layout_captures(&mut self) {
        self.cancel_pointer_capture();
        self.keyboard_capture = None;
        self.popup = None;
        for control in &mut self.controls {
            if let ControlKind::Canvas(state) = &mut control.kind {
                state.pressed = None;
                state.released = None;
            }
        }
    }
    fn activation_target(&self) -> Option<usize> {
        let i = self.pointer_target()?;
        match &self.controls[i].kind {
            ControlKind::Container => self.controls[i].tab_stop.then_some(i),
            ControlKind::Image { .. } => None,
            ControlKind::Label { target, .. } => {
                target.as_ref().and_then(|id| self.ids.get(id).copied())
            }
            _ => Some(i),
        }
    }
    pub(crate) fn pointer_target(&self) -> Option<usize> {
        self.pointer.and_then(|(x, y)| {
            if let Some(popup) = &self.popup {
                return (self.controls[popup.control].rect.contains(x, y)
                    || self.popup_hit(x, y).is_some())
                .then_some(popup.control);
            }
            self.hit(x, y)
        })
    }
    pub(crate) fn option_rect(&self, control: usize, row: usize) -> Rect {
        let rect = self.controls[control].rect;
        Rect {
            y: rect.y + rect.h * (row + 1) as f32,
            ..rect
        }
    }
    fn popup_hit(&self, x: f32, y: f32) -> Option<usize> {
        let popup = self.popup.as_ref()?;
        let ControlKind::Select { options, .. } = &self.controls[popup.control].kind else {
            return None;
        };
        (popup.first..options.len().min(popup.first + POPUP_ROWS)).find(|&i| {
            self.option_rect(popup.control, i - popup.first)
                .contains(x, y)
        })
    }
    pub fn pointer_move(&mut self, x: f32, y: f32) {
        let before = self.pointer_appearance();
        self.pointer = Some((x, y));
        if self.scrollbar_move(x, y) {
            return;
        }
        if let Some(i) = self.pointer_capture {
            self.range_pointer(i, x, false);
        }
        if let Some(i) = self.pointer_capture
            && let ControlKind::TextInput(input) = &mut self.controls[i].kind
        {
            input.cursor = input.hit_cursor(x);
        }
        if let Some(index) = self.popup_hit(x, y) {
            let popup = self.popup.as_mut().unwrap();
            if let ControlKind::Select { options, .. } = &self.controls[popup.control].kind
                && !options[index].disabled
            {
                popup.highlighted = Some(index);
            }
        }
        if before != self.pointer_appearance() {
            self.invalidate_paint();
        }
    }
    pub fn pointer_leave(&mut self) {
        let hovered = self.pointer_target();
        self.pointer = None;
        if hovered.is_some() {
            self.invalidate_paint();
        }
    }
    pub fn pointer_down(&mut self, x: f32, y: f32) {
        self.invalidate_paint();
        self.pointer_move(x, y);
        self.keyboard_capture = None;
        self.cancel_pointer_capture();
        if self.popup.is_some() {
            if let Some(index) = self.popup_hit(x, y) {
                self.popup.as_mut().unwrap().pressed = Some(index);
            } else {
                // Dismissal consumes the press; an underlying control must not activate.
                self.popup = None;
            }
            return;
        }
        if self.scrollbar_down(x, y) {
            return;
        }
        let target = self.activation_target();
        if let Some(i) = target
            && matches!(self.controls[i].kind, ControlKind::Range(_))
            && self.hit(x, y) != Some(i)
        {
            if !self.controls[i].disabled {
                self.focused = Some(i);
                self.reveal_focus();
            }
            return;
        }
        self.pointer_capture = target.filter(|&i| {
            !self.controls[i].disabled
                && self.controls[i]
                    .canvas_region()
                    .is_none_or(|region| region.content.contains(x, y))
        });
        self.focused = self.pointer_capture;
        if let Some(i) = self.pointer_capture {
            self.range_pointer(i, x, true);
        }
        if let Some(index) = self.pointer_capture {
            let content = self.controls[index].content_rect();
            if let ControlKind::Canvas(state) = &mut self.controls[index].kind {
                state.pressed = Some((x - content.x, y - content.y));
            }
        }
        if let Some(i) = self.focused
            && let ControlKind::TextInput(input) = &mut self.controls[i].kind
        {
            input.cursor = input.hit_cursor(x);
            input.anchor = input.cursor;
        }
    }
    pub fn pointer_up(&mut self, x: f32, y: f32) {
        self.invalidate_paint();
        self.pointer_move(x, y);
        if self.scroll_capture.take().is_some() {
            return;
        }
        if self.popup.is_some() {
            let hit = self.popup_hit(x, y);
            let popup = self.popup.as_mut().unwrap();
            if let Some(index) = popup.pressed.take()
                && hit == Some(index)
            {
                self.commit_option(index);
            }
            return;
        }
        if let Some(index) = self.pointer_capture.take() {
            let content = self.controls[index].content_rect();
            if let ControlKind::Canvas(state) = &mut self.controls[index].kind {
                state.released = Some((x - content.x, y - content.y));
            } else if self.activation_target() == Some(index) {
                self.activate(index);
            }
        }
    }
    fn activate(&mut self, index: usize) {
        self.cancel_pointer_capture();
        self.keyboard_capture = None;
        let control = &mut self.controls[index];
        match &mut control.kind {
            ControlKind::Button { .. } => control.clicked = true,
            ControlKind::Checkbox { checked, .. } => {
                *checked = !*checked;
                control.changed = true;
            }
            ControlKind::Select { options, selected } if !options.is_empty() => {
                let highlighted = selected
                    .filter(|&i| !options[i].disabled)
                    .or_else(|| options.iter().position(|option| !option.disabled));
                let mut popup = SelectPopup {
                    control: index,
                    highlighted,
                    first: 0,
                    pressed: None,
                };
                popup.reveal();
                self.popup = Some(popup);
            }
            _ => {}
        }
    }
    fn commit_option(&mut self, index: usize) {
        let Some(popup) = &self.popup else {
            return;
        };
        let control = &mut self.controls[popup.control];
        if let ControlKind::Select { options, selected } = &mut control.kind
            && !options[index].disabled
        {
            control.changed |= *selected != Some(index);
            *selected = Some(index);
            self.popup = None;
        }
    }
    /// Scroll select rows, or scroll the container under the pointer (40px per row).
    pub fn scroll(&mut self, rows: i32) {
        if self.popup.is_none() {
            self.scroll_wheel(0.0, rows as f32 * 40.0);
            return;
        }
        if rows != 0 && self.popup.is_some() {
            self.invalidate_paint();
        }
        if let Some(popup) = &mut self.popup
            && let ControlKind::Select { options, .. } = &self.controls[popup.control].kind
        {
            popup.first = popup
                .first
                .saturating_add_signed(rows as isize)
                .min(options.len().saturating_sub(POPUP_ROWS));
            popup.pressed = None;
        }
    }
    /// Feed committed platform text, not characters guessed from physical key codes.
    pub fn text_input(&mut self, text: &str) {
        if let Some(i) = self.focused
            && let ControlKind::TextInput(input) = &mut self.controls[i].kind
        {
            self.controls[i].changed |= input.insert(text);
            self.invalidate_paint();
        }
    }
    /// Cancel captures and dismiss the select on window focus loss.
    pub fn cancel_input(&mut self) {
        if self.pointer_capture.is_none()
            && self.keyboard_capture.is_none()
            && self.scroll_capture.is_none()
            && self.pointer.is_none()
            && self.focused.is_none()
            && self.popup.is_none()
        {
            return;
        }
        self.invalidate_paint();
        self.cancel_pointer_capture();
        self.keyboard_capture = None;
        self.pointer = None;
        self.focused = None;
        self.popup = None;
    }
    pub fn key_down(&mut self, key: Key, repeat: bool) {
        self.key_down_with_shift(key, repeat, false);
    }
    /// Shift extends a text selection; BackTab handles reverse focus traversal.
    pub fn key_down_with_shift(&mut self, key: Key, repeat: bool, shift: bool) {
        if matches!(key, Key::Tab | Key::BackTab)
            || self.popup.is_some()
            || self
                .focused
                .is_some_and(|i| !matches!(self.controls[i].kind, ControlKind::Canvas(_)))
        {
            self.invalidate_paint();
        }
        if repeat
            && matches!(
                key,
                Key::Tab | Key::BackTab | Key::Enter | Key::Space | Key::Escape
            )
        {
            return;
        }
        if matches!(key, Key::Tab | Key::BackTab) {
            self.popup = None;
            self.cancel_pointer_capture();
            self.keyboard_capture = None;
            let eligible: Vec<_> = self
                .paint_order()
                .filter(|(_, bar)| !bar)
                .map(|(i, _)| (i, &self.controls[i]))
                .filter(|(_, c)| {
                    !c.disabled
                        && !matches!(c.kind, ControlKind::Label { .. } | ControlKind::Image { .. })
                        && (!matches!(c.kind,ControlKind::Container)||c.tab_stop)
                        && !matches!(&c.kind, ControlKind::Canvas(state) if !state.tab_stop || c.content_rect().w <= 0.0 || c.content_rect().h <= 0.0)
                        && c.rect.w > 0.0
                        && c.rect.h > 0.0
                })
                .map(|(i, _)| i)
                .collect();
            if eligible.is_empty() {
                self.focused = None;
                return;
            }
            let current = eligible.iter().position(|&i| Some(i) == self.focused);
            let next = match (current, key == Key::BackTab) {
                (Some(i), false) => (i + 1) % eligible.len(),
                (Some(i), true) => (i + eligible.len() - 1) % eligible.len(),
                (None, false) => 0,
                (None, true) => eligible.len() - 1,
            };
            self.focused = Some(eligible[next]);
            self.reveal_focus();
            return;
        }
        if self.popup.is_some() {
            match key {
                Key::Escape => self.popup = None,
                Key::Enter | Key::Space => {
                    if let Some(index) = self.popup.as_ref().and_then(|p| p.highlighted) {
                        self.commit_option(index);
                    } else {
                        self.popup = None;
                    }
                }
                Key::Up | Key::Down | Key::Home | Key::End => self.navigate_select(key, true),
                _ => {}
            }
            return;
        }
        let Some(i) = self.focused else {
            return;
        };
        if self.scroll_key(i, key) {
            return;
        }
        if let ControlKind::Range(r) = &mut self.controls[i].kind {
            self.controls[i].changed |= r.key(key);
            return;
        }
        if matches!(self.controls[i].kind, ControlKind::Canvas(_)) {
            // The host handles native canvas keyboard input; no synthetic button activation.
            return;
        }
        if let ControlKind::TextInput(input) = &mut self.controls[i].kind {
            self.controls[i].changed |= input.key(key, shift);
            return;
        }
        match key {
            Key::Enter if !matches!(self.controls[i].kind, ControlKind::Checkbox { .. }) => {
                self.activate(i)
            }
            Key::Space => self.keyboard_capture = Some(i),
            Key::Up | Key::Down | Key::Home | Key::End => self.navigate_select(key, false),
            _ => {}
        }
    }
    fn navigate_select(&mut self, key: Key, open: bool) {
        let Some(i) = self.focused else {
            return;
        };
        let ControlKind::Select { options, selected } = &mut self.controls[i].kind else {
            return;
        };
        let current = if open {
            self.popup.as_ref().and_then(|p| p.highlighted)
        } else {
            *selected
        };
        let next = match key {
            Key::Home => options.iter().position(|option| !option.disabled),
            Key::End => options.iter().rposition(|option| !option.disabled),
            Key::Up => (0..current.unwrap_or(options.len()))
                .rev()
                .find(|&i| !options[i].disabled),
            _ => (current.map_or(0, |i| i + 1)..options.len()).find(|&i| !options[i].disabled),
        };
        if let Some(next) = next {
            if open {
                let popup = self.popup.as_mut().unwrap();
                popup.highlighted = Some(next);
                popup.pressed = None;
                popup.reveal();
            } else {
                self.controls[i].changed |= *selected != Some(next);
                if let ControlKind::Select { selected, .. } = &mut self.controls[i].kind {
                    *selected = Some(next);
                }
            }
        }
    }
    pub fn key_up(&mut self, key: Key) {
        if key == Key::Space && self.keyboard_capture.is_some() {
            self.invalidate_paint();
        }
        if key == Key::Space
            && let Some(i) = self.keyboard_capture.take()
            && Some(i) == self.focused
        {
            self.activate(i);
        }
    }
}
