use super::BackendError;
use native_ui::{Key, Ui};
use raylib::prelude::*;

/// Feed the current raylib input snapshot once per frame, before rendering.
/// Leaves raylib's key and character queues available to the host.
/// The host sets the UI viewport, handles closing and calls `Ui::end_frame` after consuming UI events.
/// Disable raylib's default Escape exit key to let Escape dismiss select popups.
pub fn handle_input_snapshot(ui: &mut Ui, rl: &mut RaylibHandle) -> Result<(), BackendError> {
    if !rl.is_window_focused() {
        ui.cancel_input();
        return Ok(());
    }
    let point = rl.get_mouse_position();
    // Keep captured slider/canvas drags moving even beyond the window edge.
    if rl.is_cursor_on_screen() || rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
        ui.pointer_move(point.x, point.y);
    }
    if !rl.is_cursor_on_screen() {
        ui.pointer_leave();
    }
    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        ui.pointer_down(point.x, point.y);
    }
    if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
        ui.pointer_up(point.x, point.y);
    }
    let wheel = rl.get_mouse_wheel_move_v();
    if wheel.x != 0.0 || wheel.y != 0.0 {
        ui.scroll_wheel(-wheel.x * 40.0, -wheel.y * 40.0);
    }
    use KeyboardKey::*;
    let shift = rl.is_key_down(KEY_LEFT_SHIFT) || rl.is_key_down(KEY_RIGHT_SHIFT);
    let shortcut = [
        KEY_LEFT_CONTROL,
        KEY_RIGHT_CONTROL,
        KEY_LEFT_SUPER,
        KEY_RIGHT_SUPER,
    ]
    .into_iter()
    .any(|key| rl.is_key_down(key));
    if shortcut {
        if (rl.is_key_pressed(KEY_C) || rl.is_key_pressed(KEY_X))
            && let Some(text) = ui.selected_text()
        {
            rl.set_clipboard_text(text)?;
            if rl.is_key_pressed(KEY_X) {
                ui.key_down(Key::Delete, false);
            }
        }
        if rl.is_key_pressed(KEY_V) && ui.wants_text_input() {
            ui.text_input(&rl.get_clipboard_text()?);
        }
        if rl.is_key_pressed(KEY_A) {
            ui.key_down(Key::SelectAll, false);
        }
    }
    for (native, key) in [
        (KEY_TAB, if shift { Key::BackTab } else { Key::Tab }),
        (KEY_ENTER, Key::Enter),
        (KEY_KP_ENTER, Key::Enter),
        (KEY_SPACE, Key::Space),
        (KEY_LEFT, Key::Left),
        (KEY_RIGHT, Key::Right),
        (KEY_UP, Key::Up),
        (KEY_DOWN, Key::Down),
        (KEY_HOME, Key::Home),
        (KEY_END, Key::End),
        (KEY_BACKSPACE, Key::Backspace),
        (KEY_DELETE, Key::Delete),
        (KEY_ESCAPE, Key::Escape),
        (KEY_PAGE_UP, Key::PageUp),
        (KEY_PAGE_DOWN, Key::PageDown),
    ] {
        if rl.is_key_pressed(native) {
            ui.key_down_with_shift(key, false, shift);
        } else if rl.is_key_pressed_repeat(native) {
            ui.key_down_with_shift(key, true, shift);
        }
        if rl.is_key_released(native) {
            ui.key_up(key);
        }
    }
    Ok(())
}

/// Convenience forwarding including the host character queue. For shared input,
/// call `handle_input_snapshot` and route `get_char_pressed()` in the host loop.
pub fn handle_input(ui: &mut Ui, rl: &mut RaylibHandle) -> Result<(), BackendError> {
    let (w, h) = (rl.get_screen_width(), rl.get_screen_height());
    if w > 0 && h > 0 {
        ui.set_viewport(w as f32, h as f32)?;
    }
    handle_input_snapshot(ui, rl)?;
    let shortcut = [
        KeyboardKey::KEY_LEFT_CONTROL,
        KeyboardKey::KEY_RIGHT_CONTROL,
        KeyboardKey::KEY_LEFT_SUPER,
        KeyboardKey::KEY_RIGHT_SUPER,
    ]
    .into_iter()
    .any(|key| rl.is_key_down(key));
    while let Some(ch) = rl.get_char_pressed() {
        if rl.is_window_focused() && !shortcut && !ch.is_control() {
            ui.text_input(ch.encode_utf8(&mut [0; 4]));
        }
    }
    Ok(())
}
