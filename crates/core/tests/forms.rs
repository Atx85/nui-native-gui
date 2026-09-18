use native_ui::{Color, Key, PaintCommand, Rect, Renderer, Ui};

const CSS: &str = "input,select,button {position:absolute;left:10px;top:10px;width:160px;height:32px;padding:4px;background:white;color:black} #check{top:55px;width:28px;height:28px} #choice{top:95px} #button{top:230px}";
fn form() -> Ui {
    Ui::from_html_css("<input id=text placeholder='Your name'><input id=check type=checkbox><select id=choice><option value=a>Alpha</option><option value=b disabled>Beta</option><option value=c>Gamma</option></select><button id=button>Save</button>", CSS).unwrap()
}
fn click(ui: &mut Ui, x: f32, y: f32) {
    ui.pointer_down(x, y);
    ui.pointer_up(x, y);
}

#[derive(Default)]
struct Recorder {
    text: Vec<String>,
    rects: Vec<(Rect, Color)>,
}
impl Renderer for Recorder {
    type Error = std::convert::Infallible;
    fn measure_text(&mut self, text: &str, _: f32) -> Result<(f32, f32), Self::Error> {
        Ok((text.chars().count() as f32 * 10.0, 16.0))
    }
    fn draw(&mut self, command: PaintCommand<'_>) -> Result<(), Self::Error> {
        match command {
            PaintCommand::Text { text, .. } => self.text.push(text.into()),
            PaintCommand::FillRect { rect, color } => self.rects.push((rect, color)),
        }
        Ok(())
    }
}
fn paint(ui: &Ui) -> Recorder {
    let mut r = Recorder::default();
    ui.render(&mut r).unwrap();
    r
}

#[test]
fn initial_values_entities_and_typed_queries() {
    let ui = Ui::from_html_css("<input id=text value='café &amp; tea'><input id=check type=CHECKBOX checked><select id=choice><option> First  choice </option><option value=x selected>Second</option><option value=y selected>Third</option></select>", CSS).unwrap();
    assert_eq!(ui.get_value("text"), Some("café & tea"));
    assert_eq!(ui.checked("check"), Some(true));
    assert_eq!(ui.get_value("choice"), Some("y"));
    assert_eq!(ui.selected_index("choice"), Some(2));
    assert_eq!(ui.checked("text"), None);
    assert_eq!(ui.get_value("check"), Some("on")); // Default checkbox submission value.
    assert_eq!(ui.get_value("missing"), None);
    assert!(!ui.changed("text"));
    assert!(paint(&ui).text.contains(&"Third".into()));
}

#[test]
fn text_uses_platform_commits_and_unicode_boundaries() {
    let mut ui = form();
    ui.text_input("ignored");
    assert_eq!(ui.get_value("text"), Some(""));
    ui.key_down(Key::Tab, false);
    assert!(ui.wants_text_input());
    ui.text_input("aé🙂z");
    ui.key_down(Key::Left, false);
    ui.key_down(Key::Backspace, true);
    assert_eq!(ui.get_value("text"), Some("aéz"));
    ui.key_down(Key::Left, false);
    ui.key_down(Key::Delete, true);
    assert_eq!(ui.get_value("text"), Some("az"));
    ui.text_input("界");
    assert_eq!(ui.get_value("text"), Some("a界z"));
    assert!(ui.changed("text"));
    assert!(ui.changed("text"));
    assert!(!ui.clicked("text"));
    ui.end_frame();
    assert!(!ui.changed("text"));
    ui.cancel_input();
    assert!(!ui.wants_text_input());
    ui.text_input("ignored");
    assert_eq!(ui.get_value("text"), Some("a界z"));
}

#[test]
fn text_selection_replacement_shift_navigation_and_noop_edits() {
    let mut ui = form();
    ui.set_value("text", "aé🙂z").unwrap();
    ui.key_down(Key::Tab, false);
    ui.key_down_with_shift(Key::Left, false, true);
    ui.key_down_with_shift(Key::Left, true, true);
    ui.text_input("!");
    assert_eq!(ui.get_value("text"), Some("aé!"));
    ui.key_down(Key::SelectAll, false);
    ui.text_input("aé!");
    ui.end_frame();
    ui.key_down(Key::End, false);
    ui.key_down(Key::Delete, false);
    ui.text_input("\r\n\t");
    assert!(!ui.changed("text"));
    ui.key_down(Key::SelectAll, false);
    ui.key_down(Key::Backspace, false);
    assert_eq!(ui.get_value("text"), Some(""));
    assert!(ui.changed("text"));
}

#[test]
fn space_is_inserted_once_by_text_event_and_enter_does_not_click() {
    let mut ui = form();
    click(&mut ui, 20.0, 20.0);
    ui.key_down(Key::Space, false);
    ui.text_input(" ");
    ui.key_up(Key::Space);
    ui.key_down(Key::Enter, false);
    assert_eq!(ui.get_value("text"), Some(" "));
    assert!(!ui.clicked("text"));
}

#[test]
fn maxlength_counts_characters_and_selection_frees_capacity() {
    let mut ui = Ui::from_html_css("<input id=text maxlength=3>", CSS).unwrap();
    click(&mut ui, 20.0, 20.0);
    ui.text_input("é🙂界four");
    assert_eq!(ui.get_value("text"), Some("é🙂界"));
    ui.end_frame();
    ui.text_input("x");
    assert!(!ui.changed("text"));
    ui.key_down_with_shift(Key::Left, false, true);
    ui.text_input("ab");
    assert_eq!(ui.get_value("text"), Some("é🙂a"));
    assert!(ui.set_value("text", "1234").is_err());
    assert_eq!(ui.get_value("text"), Some("é🙂a"));
}

#[test]
fn readonly_text_can_focus_but_cannot_edit_and_disabled_controls_are_skipped() {
    let mut ui = Ui::from_html_css("<input id=text readonly value=locked><input id=check type=checkbox disabled><select id=choice disabled><option>A</option></select><button id=button>Go</button>", CSS).unwrap();
    ui.key_down(Key::Tab, false);
    assert!(!ui.wants_text_input());
    ui.key_down(Key::SelectAll, false);
    ui.text_input("new");
    ui.key_down(Key::Backspace, false);
    assert_eq!(ui.get_value("text"), Some("locked"));
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Enter, false);
    assert!(ui.clicked("button"));
    click(&mut ui, 20.0, 65.0);
    assert_eq!(ui.checked("check"), Some(false));
    click(&mut ui, 20.0, 100.0);
    assert!(!ui.select_is_open());
}

#[test]
fn checkbox_mouse_space_capture_and_persistent_state() {
    let mut ui = form();
    ui.pointer_down(20.0, 65.0);
    ui.pointer_up(200.0, 65.0);
    assert_eq!(ui.checked("check"), Some(false));
    click(&mut ui, 20.0, 65.0);
    assert_eq!(ui.checked("check"), Some(true));
    assert!(ui.changed("check"));
    assert!(!ui.clicked("check"));
    ui.end_frame();
    assert_eq!(ui.checked("check"), Some(true));
    ui.key_down(Key::Space, false);
    ui.key_down(Key::Space, true);
    assert!(!ui.changed("check"));
    ui.key_up(Key::Space);
    assert_eq!(ui.checked("check"), Some(false));
    ui.key_down(Key::Space, false);
    ui.cancel_input();
    ui.key_up(Key::Space);
    assert_eq!(ui.checked("check"), Some(false));
}

#[test]
fn select_defaults_empty_and_disabled_options() {
    for (html, expected) in [
        ("<select id=choice></select>", None),
        (
            "<select id=choice><option disabled>A</option></select>",
            None,
        ),
        (
            "<select id=choice><option disabled>A</option><option> B &amp; C </option></select>",
            Some("B & C"),
        ),
        (
            "<select id=choice><option disabled selected value=a>A</option><option>B</option></select>",
            Some("a"),
        ),
    ] {
        let ui = Ui::from_html_css(html, CSS).unwrap();
        assert_eq!(ui.get_value("choice"), expected);
    }
}

#[test]
fn select_mouse_dropdown_overlays_other_controls_and_skips_disabled_options() {
    let mut ui = form();
    click(&mut ui, 20.0, 100.0);
    assert!(ui.select_is_open());
    ui.pointer_up(20.0, 200.0); // No option press: no commit.
    assert_eq!(ui.get_value("choice"), Some("a"));
    click(&mut ui, 20.0, 170.0); // Disabled Beta.
    assert!(ui.select_is_open());
    assert_eq!(ui.get_value("choice"), Some("a"));
    click(&mut ui, 20.0, 200.0);
    assert_eq!(ui.get_value("choice"), Some("c"));
    assert!(ui.changed("choice"));
    assert!(!ui.select_is_open());
    assert!(!ui.clicked("button"));
}

#[test]
fn select_dismissal_consumes_outside_click_and_focus_loss_cancels_pending_choice() {
    let mut ui = form();
    click(&mut ui, 20.0, 100.0);
    click(&mut ui, 20.0, 240.0);
    assert!(!ui.select_is_open());
    assert!(!ui.clicked("button"));
    click(&mut ui, 20.0, 100.0);
    ui.pointer_down(20.0, 200.0);
    ui.cancel_input();
    ui.pointer_up(20.0, 200.0);
    assert_eq!(ui.get_value("choice"), Some("a"));
    assert!(!ui.changed("choice"));
}

#[test]
fn select_keyboard_preview_escape_commit_and_closed_navigation() {
    let mut ui = form();
    for _ in 0..3 {
        ui.key_down(Key::Tab, false);
    }
    ui.key_down(Key::Enter, false);
    ui.key_down(Key::Down, false);
    assert_eq!(ui.get_value("choice"), Some("a"));
    ui.key_down(Key::Escape, false);
    assert!(!ui.select_is_open());
    assert!(!ui.changed("choice"));
    ui.key_down(Key::Space, false);
    ui.key_up(Key::Space);
    ui.key_down(Key::End, false);
    ui.key_down(Key::Enter, false);
    assert_eq!(ui.get_value("choice"), Some("c"));
    assert!(ui.changed("choice"));
    ui.end_frame();
    ui.key_down(Key::Up, true);
    assert_eq!(ui.get_value("choice"), Some("a"));
    ui.key_down(Key::Enter, false);
    ui.key_down(Key::Tab, false);
    assert!(!ui.select_is_open());
    ui.key_down(Key::Enter, false);
    assert!(ui.clicked("button"));
}

#[test]
fn long_selects_scroll_and_keyboard_reveals_last_option() {
    let options = (0..20)
        .map(|i| format!("<option value={i}>Option {i}</option>"))
        .collect::<String>();
    let mut ui = Ui::from_html_css(&format!("<select id=choice>{options}</select>"), CSS).unwrap();
    click(&mut ui, 20.0, 100.0);
    assert_eq!(paint(&ui).text.len(), 9); // Selected label plus eight rows.
    ui.scroll(100);
    assert!(paint(&ui).text.contains(&"Option 19".into()));
    click(&mut ui, 20.0, 140.0);
    assert_eq!(ui.get_value("choice"), Some("12"));
    click(&mut ui, 20.0, 100.0);
    ui.key_down(Key::End, false);
    assert!(paint(&ui).text.contains(&"Option 19".into()));
    ui.key_down(Key::Enter, false);
    assert_eq!(ui.get_value("choice"), Some("19"));
}

#[test]
fn host_setters_validate_types_and_do_not_emit_user_changes() {
    let mut ui = form();
    ui.set_value("text", "Hello\nworld").unwrap();
    ui.set_value("choice", "c").unwrap();
    ui.set_checked("check", true).unwrap();
    assert_eq!(ui.get_value("text"), Some("Helloworld"));
    assert_eq!(ui.get_value("choice"), Some("c"));
    assert_eq!(ui.checked("check"), Some(true));
    assert!(!ui.changed("text") && !ui.changed("choice") && !ui.changed("check"));
    assert!(ui.set_value("choice", "missing").is_err());
    assert_eq!(ui.get_value("choice"), Some("c"));
    assert!(ui.set_value("check", "x").is_err());
    assert!(ui.set_checked("text", false).is_err());
    assert!(ui.set_value("missing", "x").is_err());
}

#[test]
fn checked_css_and_form_focus_colors_reach_renderer() {
    let mut ui = Ui::from_html_css(
        "<input id=check type=checkbox>",
        &format!("{CSS}input:checked{{background:red}}input:checked:focus{{color:blue}}"),
    )
    .unwrap();
    assert_eq!(paint(&ui).rects[4].1, Color(255, 255, 255, 255));
    click(&mut ui, 20.0, 65.0);
    let painted = paint(&ui);
    assert_eq!(painted.rects[4].1, Color(255, 0, 0, 255));
    // The tick now starts with a partially covered edge pixel, not an opaque block.
    let tick = &painted.rects[5..];
    assert!(
        tick.iter()
            .all(|(_, c)| c.0 == 0 && c.1 == 0 && c.2 == 255 && c.3 > 0)
    );
    assert!(tick.iter().any(|(_, c)| c.3 == 255));
    assert!(tick.iter().any(|(_, c)| c.3 < 255));
}

#[test]
fn placeholder_caret_and_long_text_are_clipped_to_visible_content() {
    let mut ui = form();
    assert!(paint(&ui).text.contains(&"Your name".into()));
    ui.set_value("text", &"abcé🙂".repeat(2000)).unwrap();
    click(&mut ui, 20.0, 20.0);
    let r = paint(&ui);
    assert!(r.text[0].chars().count() <= 16);
    ui.key_down(Key::Home, false);
    assert!(paint(&ui).text[0].starts_with("abcé🙂"));
    ui.key_down(Key::SelectAll, false);
    ui.render(&mut Recorder::default()).unwrap();
}

#[test]
fn unsupported_form_features_and_duplicate_option_ids_fail() {
    for html in [
        "<input type=password>",
        "<input type=radio>",
        "<input maxlength=no>",
        "<input checked>",
        "<input type=checkbox readonly>",
        "<select multiple></select>",
        "<select size=3></select>",
        "<option>Orphan</option>",
        "<select><option id=x>A</option><option id=x>B</option></select>",
        "<button><span><input></span></button>",
        "<span>Standalone</span>",
    ] {
        assert!(Ui::from_html_css(html, CSS).is_err(), "{html}");
    }
    assert!(Ui::from_html_css("<input>", "").is_err());
    assert!(Ui::from_html_css("<select></select>", "").is_err());
}

#[test]
fn labels_toggle_associated_checkboxes_and_are_not_tab_stops() {
    let css = format!(
        "{CSS}label{{position:absolute;left:50px;top:55px;width:120px;height:28px}}label:hover{{color:red}}"
    );
    let mut ui = Ui::from_html_css("<input id=check type=checkbox><label for=check>Enable <span>counting</span></label><button id=button>Save</button>", &css).unwrap();
    assert!(paint(&ui).text.contains(&"Enable counting".into()));
    click(&mut ui, 80.0, 60.0);
    assert_eq!(ui.checked("check"), Some(true));
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Enter, false);
    assert!(ui.clicked("button"));
    let mut disabled = Ui::from_html_css(
        "<input id=check type=checkbox disabled><label for=check>Disabled</label>",
        &css,
    )
    .unwrap();
    click(&mut disabled, 80.0, 60.0);
    assert_eq!(disabled.checked("check"), Some(false));
    assert!(Ui::from_html_css("<label for=missing>Missing</label>", &css).is_err());
    assert!(Ui::from_html_css("<label><input></label>", &css).is_err());
}

#[test]
fn rendered_text_supports_caret_placement_and_drag_selection() {
    let mut ui = form();
    ui.set_value("text", "aé🙂z").unwrap();
    paint(&ui); // Text starts at x=15, with ten-pixel character advances.
    click(&mut ui, 35.0, 20.0);
    ui.text_input("!");
    assert_eq!(ui.get_value("text"), Some("aé!🙂z"));
    paint(&ui);
    ui.pointer_down(25.0, 20.0);
    ui.pointer_move(55.0, 20.0);
    ui.pointer_up(55.0, 20.0);
    assert_eq!(ui.selected_text(), Some("é!🙂"));
    ui.text_input("x");
    assert_eq!(ui.get_value("text"), Some("axz"));
    assert_eq!(ui.selected_text(), None);
}

#[test]
fn popup_occludes_later_painted_controls_and_same_selection_is_not_a_change() {
    let mut ui = Ui::from_html_css("<select id=choice><option value=a>A</option><option value=b>B</option></select><button id=button>Behind popup</button>", &format!("{CSS}#button{{top:127px}}" )).unwrap();
    click(&mut ui, 20.0, 100.0);
    let r = paint(&ui);
    assert_eq!(r.text.last().map(String::as_str), Some("B"));
    click(&mut ui, 20.0, 140.0);
    assert!(!ui.clicked("button"));
    assert_eq!(ui.get_value("choice"), Some("a"));
    assert!(!ui.changed("choice"));
}
