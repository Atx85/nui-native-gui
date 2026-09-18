use native_ui::{CanvasRegion, Color, Key, PaintCommand, Rect, Renderer, Ui};

const CSS: &str = "canvas{position:absolute;left:10px;top:20px;width:120px;height:80px;border:2px solid red;padding:3px 4px 5px 6px}button{position:absolute;left:50px;top:30px;width:30px;height:25px;background:blue}select{position:absolute;left:10px;top:0;width:120px;height:20px}";
#[derive(Debug, PartialEq)]
enum Event {
    Fill(Rect, Color),
    Text(String),
    Canvas(Option<String>, Rect),
}
#[derive(Default)]
struct Recorder {
    events: Vec<Event>,
    fail: bool,
}
impl Renderer for Recorder {
    type Error = &'static str;
    fn measure_text(&mut self, _: &str, _: f32) -> Result<(f32, f32), Self::Error> {
        Ok((10.0, 10.0))
    }
    fn draw(&mut self, command: PaintCommand<'_>) -> Result<(), Self::Error> {
        self.events.push(match command {
            PaintCommand::FillRect { rect, color } => Event::Fill(rect, color),
            PaintCommand::Text { text, .. } => Event::Text(text.into()),
        });
        Ok(())
    }
    fn draw_canvas(&mut self, region: CanvasRegion<'_>) -> Result<(), Self::Error> {
        self.events
            .push(Event::Canvas(region.id.map(str::to_owned), region.content));
        if self.fail {
            Err("native drawing failed")
        } else {
            Ok(())
        }
    }
}
#[test]
fn css_canvas_reports_content_bounds_and_arbitrary_ids() {
    let ui = Ui::from_html_css("<canvas id=editor></canvas><canvas></canvas>", CSS).unwrap();
    let region = ui.canvas_region("editor").unwrap();
    assert_eq!(
        region.bounds,
        Rect {
            x: 10.0,
            y: 20.0,
            w: 120.0,
            h: 80.0
        }
    );
    assert_eq!(
        region.content,
        Rect {
            x: 18.0,
            y: 25.0,
            w: 106.0,
            h: 68.0
        }
    );
    assert_eq!(region.to_local(20.0, 30.0), (2.0, 5.0));
    assert!(ui.canvas_region("missing").is_none());
    let mut r = Recorder::default();
    ui.render(&mut r).unwrap();
    assert_eq!(
        r.events
            .iter()
            .filter(|e| matches!(e, Event::Canvas(..)))
            .count(),
        2
    );
    assert!(r.events.contains(&Event::Canvas(None, region.content)));
}
#[test]
fn unstyled_canvas_is_transparent_and_empty_content_skips_callback() {
    let css = "canvas{position:absolute;left:0;top:0;width:100px;height:80px}";
    let ui = Ui::from_html_css("<canvas></canvas>", css).unwrap();
    let mut r = Recorder::default();
    ui.render(&mut r).unwrap();
    assert_eq!(r.events.len(), 2); // transparent decoration then native drawing
    assert!(matches!(r.events[0], Event::Fill(_, Color(_, _, _, 0))));
    for extra in ["canvas{width:0}", "canvas{padding:80px}"] {
        let ui = Ui::from_html_css("<canvas></canvas>", &format!("{css}{extra}")).unwrap();
        let mut r = Recorder::default();
        ui.render(&mut r).unwrap();
        assert!(!r.events.iter().any(|e| matches!(e, Event::Canvas(..))));
    }
}
#[test]
fn native_drawing_is_ordered_between_decoration_and_overlays() {
    let mut ui = Ui::from_html_css(
        "<canvas id=scene></canvas><button>Overlay</button><select><option>Popup</option></select>",
        CSS,
    )
    .unwrap();
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Enter, false);
    assert!(ui.select_is_open());
    let mut r = Recorder::default();
    ui.render(&mut r).unwrap();
    let native = r
        .events
        .iter()
        .position(|e| matches!(e, Event::Canvas(..)))
        .unwrap();
    assert!(native > 0, "CSS border is drawn first");
    let overlay = r
        .events
        .iter()
        .position(|e| *e == Event::Text("Overlay".into()))
        .unwrap();
    let popup = r
        .events
        .iter()
        .rposition(|e| *e == Event::Text("Popup".into()))
        .unwrap();
    assert!(native < overlay && overlay < popup);
    r.fail = true;
    assert_eq!(ui.render(&mut r), Err("native drawing failed"));
}
#[test]
fn canvas_pointer_capture_uses_content_local_coordinates_and_frame_edges() {
    let mut ui = Ui::from_html_css("<canvas id=scene></canvas>", CSS).unwrap();
    ui.pointer_down(12.0, 22.0);
    assert!(
        !ui.canvas_input("scene").unwrap().captured,
        "padding is not drawing input"
    );
    ui.pointer_down(20.0, 30.0);
    let input = ui.canvas_input("scene").unwrap();
    assert_eq!(input.pressed, Some((2.0, 5.0)));
    assert!(input.captured && input.hovered && input.focused);
    assert_eq!(
        ui.canvas_input("scene"),
        Some(input),
        "polling does not consume edges"
    );
    ui.end_frame();
    assert!(ui.canvas_input("scene").unwrap().pressed.is_none());
    ui.pointer_move(200.0, 150.0);
    let input = ui.canvas_input("scene").unwrap();
    assert!(input.captured && !input.hovered);
    assert_eq!(input.pointer, Some((182.0, 125.0)));
    ui.pointer_up(200.0, 150.0);
    let input = ui.canvas_input("scene").unwrap();
    assert!(!input.captured);
    assert_eq!(input.released, Some((182.0, 125.0)));
    assert!(!ui.clicked("scene") && !ui.changed("scene"));
    ui.end_frame();
    assert!(ui.canvas_input("scene").unwrap().released.is_none());
    ui.pointer_down(20.0, 30.0);
    ui.cancel_input();
    let input = ui.canvas_input("scene").unwrap();
    assert!(input.cancelled && !input.captured && !input.focused);
}
#[test]
fn covering_controls_and_popup_dismissal_do_not_click_through() {
    let mut ui=Ui::from_html_css("<canvas id=scene></canvas><button>Overlay</button><select><option>One</option><option>Two</option></select>",CSS).unwrap();
    assert_eq!(ui.canvas_at(20.0, 30.0).unwrap().id, Some("scene"));
    assert!(ui.canvas_at(55.0, 35.0).is_none());
    ui.pointer_down(55.0, 35.0);
    assert!(ui.canvas_input("scene").unwrap().pressed.is_none());
    ui.cancel_input();
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Enter, false);
    assert!(ui.canvas_at(20.0, 80.0).is_none());
    ui.pointer_down(20.0, 80.0);
    ui.pointer_up(20.0, 80.0);
    assert!(!ui.select_is_open());
    assert!(ui.canvas_input("scene").unwrap().pressed.is_none());
    ui.pointer_down(20.0, 80.0);
    assert!(ui.canvas_input("scene").unwrap().pressed.is_some());
}
#[test]
fn keyboard_focus_is_opt_in_and_does_not_synthesize_canvas_clicks() {
    let mut ui = Ui::from_html_css(
        "<canvas id=scene tabindex=0></canvas><button>Next</button>",
        CSS,
    )
    .unwrap();
    ui.key_down(Key::Tab, false);
    assert!(ui.canvas_input("scene").unwrap().focused);
    ui.key_down(Key::Space, false);
    ui.key_up(Key::Space);
    ui.key_down(Key::Enter, false);
    assert!(!ui.clicked("scene"));
    ui.pointer_down(20.0, 30.0);
    ui.key_down(Key::Tab, false);
    let input = ui.canvas_input("scene").unwrap();
    assert!(!input.focused && !input.captured && input.cancelled);
    ui.key_down(Key::BackTab, false);
    assert!(ui.canvas_input("scene").unwrap().focused);
}
#[test]
fn canvas_markup_rejects_unsupported_browser_semantics() {
    for html in [
        "<canvas tabindex=1></canvas>",
        "<canvas width=300></canvas>",
        "<canvas>fallback</canvas>",
        "<canvas><button>nested</button></canvas>",
        "<button><canvas></canvas></button>",
        "<canvas id=scene></canvas><label for=scene>Wrong target</label>",
    ] {
        assert!(Ui::from_html_css(html, CSS).is_err(), "{html}");
    }
}
