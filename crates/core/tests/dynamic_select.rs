use native_ui::{Color, Key, PaintCommand, Renderer, SelectItem, Ui};
fn ui() -> Ui {
    Ui::from_html_css_with_viewport("<select id=files><option value=a>A</option><option value=b selected>B</option></select><input id=text>","select,input{width:160px;height:30px}option{background:white;color:black}option.hot{color:red}option[value=c]:hover{background:blue}.bad{padding:3px}",300.0,300.0).unwrap()
}
#[test]
fn replace_preserves_value_then_falls_back_or_clears() {
    let mut ui = ui();
    let revision = ui.paint_revision();
    ui.set_select_options(
        "files",
        &[SelectItem::new("b", "Renamed B"), SelectItem::new("c", "C")],
    )
    .unwrap();
    assert_eq!(ui.get_value("files"), Some("b"));
    assert_eq!(ui.selected_index("files"), Some(0));
    assert_eq!(ui.select_option_count("files"), Some(2));
    assert!(!ui.changed("files"));
    assert_ne!(revision, ui.paint_revision());
    ui.set_select_options(
        "files",
        &[
            SelectItem {
                disabled: true,
                ..SelectItem::new("b", "B")
            },
            SelectItem::new("c", "C"),
        ],
    )
    .unwrap();
    assert_eq!(ui.get_value("files"), Some("c"));
    ui.set_select_options("files", &[]).unwrap();
    assert_eq!(ui.get_value("files"), None);
    assert_eq!(ui.selected_index("files"), None);
    assert_eq!(ui.select_option_count("files"), Some(0));
    ui.set_select_options(
        "files",
        &[SelectItem {
            disabled: true,
            ..SelectItem::new("x", "X")
        }],
    )
    .unwrap();
    assert_eq!(ui.get_value("files"), None);
}
#[test]
fn explicit_selection_and_duplicates_match_documented_semantics() {
    let mut ui = ui();
    ui.set_select_options(
        "files",
        &[
            SelectItem {
                selected: true,
                ..SelectItem::new("x", "First")
            },
            SelectItem {
                selected: true,
                disabled: true,
                ..SelectItem::new("x", "Second")
            },
        ],
    )
    .unwrap();
    assert_eq!(ui.selected_index("files"), Some(1));
    ui.set_value("files", "x").unwrap();
    assert_eq!(ui.selected_index("files"), Some(0));
}
#[test]
fn failures_preserve_selection_popup_and_revision() {
    let mut ui = ui();
    ui.pointer_down(10.0, 10.0);
    ui.pointer_up(10.0, 10.0);
    let revision = ui.paint_revision();
    assert!(ui.select_is_open());
    for item in [
        SelectItem::new("c", "bad\0label"),
        SelectItem {
            class: "bad".into(),
            ..SelectItem::new("c", "C")
        },
    ] {
        assert!(
            ui.set_select_options("files", &[SelectItem::new("new", "New"), item])
                .is_err()
        );
        assert_eq!(ui.paint_revision(), revision);
        assert_eq!(ui.get_value("files"), Some("b"));
        assert!(ui.select_is_open());
    }
    assert!(ui.set_select_options("text", &[]).is_err());
    assert!(ui.set_select_options("missing", &[]).is_err());
}
#[test]
fn replacing_open_popup_cancels_stale_press_and_keeps_other_controls() {
    let mut ui = ui();
    ui.set_value("text", "Keep me").unwrap();
    ui.pointer_down(10.0, 10.0);
    ui.pointer_up(10.0, 10.0);
    ui.pointer_down(10.0, 45.0);
    ui.set_select_options("files", &[SelectItem::new("c", "C")])
        .unwrap();
    assert!(!ui.select_is_open());
    ui.pointer_up(10.0, 45.0);
    assert_eq!(ui.get_value("files"), Some("c"));
    assert!(!ui.changed("files"));
    assert_eq!(ui.get_value("text"), Some("Keep me"));
    ui.key_down(Key::Enter, false);
    assert!(ui.select_is_open());
    ui.key_down(Key::Enter, false);
    assert_eq!(ui.get_value("files"), Some("c"));
}
#[derive(Default)]
struct Paint {
    text: Vec<(String, Color)>,
}
impl Renderer for Paint {
    type Error = ();
    fn measure_text(&mut self, t: &str, _: f32) -> Result<(f32, f32), ()> {
        Ok((t.len() as f32 * 8.0, 12.0))
    }
    fn draw(&mut self, p: PaintCommand<'_>) -> Result<(), ()> {
        if let PaintCommand::Text { text, color, .. } = p {
            self.text.push((text.into(), color));
        }
        Ok(())
    }
}
#[test]
fn runtime_items_use_css_classes_and_literal_unicode_labels() {
    let mut ui = ui();
    ui.set_select_options(
        "files",
        &[SelectItem {
            class: "hot".into(),
            ..SelectItem::new("c", "café <file> & Ω")
        }],
    )
    .unwrap();
    ui.pointer_down(10.0, 10.0);
    ui.pointer_up(10.0, 10.0);
    let mut paint = Paint::default();
    ui.render(&mut paint).unwrap();
    assert!(
        paint
            .text
            .iter()
            .any(|(s, c)| s == "café <file> & Ω" && *c == Color(255, 0, 0, 255))
    );
}
