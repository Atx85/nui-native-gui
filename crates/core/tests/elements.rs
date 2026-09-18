use native_ui::{Button, Checkbox, Key, PaintCommand, Renderer, Ui};
#[derive(Default)]
struct Paint(Vec<String>);
impl Renderer for Paint {
    type Error = ();
    fn measure_text(&mut self, s: &str, _: f32) -> Result<(f32, f32), ()> {
        Ok((s.len() as f32 * 7.0, 14.0))
    }
    fn draw(&mut self, cmd: PaintCommand<'_>) -> Result<(), ()> {
        if let PaintCommand::Text { text, .. } = cmd {
            self.0.push(text.into());
        }
        Ok(())
    }
}
fn ui() -> Ui {
    Ui::from_html_css_with_viewport("<aside id=options><input id=edit value=Keep></aside><button id=after>After</button><select id=pick><option id=reserved>First</option></select>",
        "aside{width:220px;height:100px;overflow:auto}input,button,select,label{height:24px}input[type=checkbox]{width:20px;height:20px}button.primary{background:blue;color:white}label{font-size:13px}",400.0,300.0).unwrap()
}
#[test]
fn get_returns_elements_and_typed_additions_keep_values_and_groups() {
    let mut ui = ui();
    let mut parent = ui.get("options").unwrap();
    parent
        .add_checkbox(
            Checkbox::new("showFPS")
                .checked(true)
                .group("view")
                .value("fps"),
        )
        .unwrap();
    parent
        .add_button(Button::new("resetView").class("primary"))
        .unwrap();
    assert_eq!(ui.get("edit").unwrap().get_value(), Some("Keep"));
    ui.get("edit").unwrap().set_value("Still here").unwrap();
    assert_eq!(ui.get_value("edit"), Some("Still here"));
    assert_eq!(ui.checked_values("view"), Some(vec!["fps"]));
    assert!(ui.get("showFPS").unwrap().checked().unwrap());
    let mut paint = Paint::default();
    ui.render(&mut paint).unwrap();
    assert!(paint.0.contains(&"Show FPS".into()));
    assert!(paint.0.contains(&"Reset View".into()));
    assert!(
        paint.0.iter().position(|s| s == "Reset View").unwrap()
            < paint.0.iter().position(|s| s == "After").unwrap()
    );
    let before = ui.paint_revision();
    assert!(ui.get("missing").is_err());
    assert!(
        ui.get("showFPS")
            .unwrap()
            .add_button(Button::new("badParent"))
            .is_err()
    );
    assert_eq!(ui.paint_revision(), before);
    ui.get("showFPS").unwrap().set_checked(false).unwrap();
    assert!(!ui.get("showFPS").unwrap().changed());
    assert_eq!(ui.checked_values("view"), Some(vec![]));
}
#[test]
fn labels_are_literal_can_be_overridden_or_hidden_and_clicks_follow_tree_order() {
    let mut ui = ui();
    ui.get("options")
        .unwrap()
        .add_checkbox(Checkbox::new("showGrid").label("<Grid> & Ω"))
        .unwrap();
    ui.get("options")
        .unwrap()
        .add_button(Button::new("child").label("Child"))
        .unwrap();
    ui.add_checkbox(Checkbox::new("rootBox").label("")).unwrap();
    let mut p = Paint::default();
    ui.render(&mut p).unwrap();
    assert!(p.0.contains(&"<Grid> & Ω".into()));
    assert!(!p.0.contains(&"Root Box".into()));
    let r = ui.bounds("showGrid").unwrap();
    ui.pointer_down(r.x + r.w + 12.0, r.y + 5.0);
    ui.pointer_up(r.x + r.w + 12.0, r.y + 5.0);
    assert_eq!(ui.checked("showGrid"), Some(true));
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Enter, false);
    ui.key_up(Key::Enter);
    assert!(ui.get("child").unwrap().clicked());
    assert!(!ui.clicked("after"));
}
#[test]
fn failed_additions_roll_back_ids_groups_geometry_and_cache_revision() {
    let mut ui = ui();
    let before = ui.paint_revision();
    let rect = ui.bounds("edit");
    for value in [
        Checkbox::new("edit"),
        Checkbox::new("reserved"),
        Checkbox::new(""),
        Checkbox::new("bad id"),
        Checkbox::new("new").group(""),
        Checkbox::new("new").label("bad\0label"),
    ] {
        assert!(ui.get("options").unwrap().add_checkbox(value).is_err());
        assert_eq!(ui.paint_revision(), before);
        assert_eq!(ui.bounds("edit"), rect);
        assert!(!ui.contains_id("new"));
    }
    ui.get("options")
        .unwrap()
        .add_checkbox(Checkbox::new("new"))
        .unwrap();
    assert!(ui.contains_id("new"));
    // A runtime-only class can reveal a layout error; no partial addition survives.
    let mut ui = Ui::from_html_css_with_viewport(
        "<div id=parent></div>",
        ".broken{height:3e38px;margin-top:3e38px}",
        200.0,
        200.0,
    )
    .unwrap();
    let revision = ui.paint_revision();
    assert!(
        ui.get("parent")
            .unwrap()
            .add_button(Button::new("huge").class("broken"))
            .is_err()
    );
    assert!(!ui.contains_id("huge"));
    assert_eq!(ui.paint_revision(), revision);
}
#[test]
fn inserting_into_earlier_parent_keeps_later_overlay_on_top_and_existing_focus() {
    let mut ui=Ui::from_html_css_with_viewport("<div id=parent><input id=edit value=Hi></div><button id=cover>Cover</button>",
        "#parent{position:absolute;left:0;top:0;width:200px;height:100px}#cover{position:absolute;left:0;top:30px;width:100px;height:30px}input{height:30px}button{height:30px}",300.0,200.0).unwrap();
    ui.pointer_down(10.0, 10.0);
    ui.pointer_up(10.0, 10.0);
    ui.get("parent")
        .unwrap()
        .add_button(Button::new("behind"))
        .unwrap();
    ui.text_input("!");
    assert!(ui.get_value("edit").unwrap().contains('!'));
    ui.pointer_down(10.0, 40.0);
    ui.pointer_up(10.0, 40.0);
    assert!(ui.clicked("cover"));
    assert!(!ui.clicked("behind"));
}
