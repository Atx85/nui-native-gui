use native_ui::{CanvasRegion, Color, Key, PaintCommand, Rect, Renderer, Ui};
fn range(attrs: &str) -> Ui {
    Ui::from_html_css(&format!("<input type=range id=volume {attrs}>"),"input{position:absolute;left:10px;top:20px;width:116px;height:30px;border:none}input::slider-track{height:4px;background:red}input::slider-thumb{width:16px;height:16px;background:blue}").unwrap()
}
#[test]
fn range_defaults_constraints_and_host_updates() {
    assert_eq!(range("").get_value("volume"), Some("50"));
    let mut ui = range("min=-2 max=4 step=0.5 value=0");
    ui.set_value("volume", "1.3").unwrap();
    assert_eq!(ui.get_value("volume"), Some("1.5"));
    assert!(!ui.changed("volume"));
    ui.set_value("volume", "100").unwrap();
    assert_eq!(ui.get_value("volume"), Some("4"));
    let revision = ui.paint_revision();
    ui.set_value("volume", "4").unwrap();
    assert_eq!(ui.paint_revision(), revision);
    assert!(ui.set_value("volume", "NaN").is_err());
    assert_eq!(ui.get_value("volume"), Some("4"));
    let mut equal = range("min=3 max=3");
    equal.key_down(Key::Tab, false);
    equal.key_down(Key::Right, false);
    assert_eq!(equal.get_value("volume"), Some("3"));
    let mut stepped = range("min=0 max=10 step=3");
    stepped.set_value("volume", "10").unwrap();
    assert_eq!(stepped.get_value("volume"), Some("9"));
}
#[test]
fn range_pointer_capture_keyboard_disabled_and_change_edges() {
    let mut ui = range("value=0");
    ui.pointer_down(68.0, 35.0);
    assert_eq!(ui.get_value("volume"), Some("50"));
    assert!(ui.changed("volume"));
    ui.end_frame();
    ui.pointer_move(1000.0, 35.0);
    assert_eq!(ui.get_value("volume"), Some("100"));
    assert!(ui.changed("volume"));
    ui.pointer_up(1000.0, 35.0);
    ui.end_frame();
    ui.pointer_move(10.0, 35.0);
    assert_eq!(ui.get_value("volume"), Some("100"));
    ui.key_down(Key::Home, false);
    assert_eq!(ui.get_value("volume"), Some("0"));
    ui.key_down(Key::PageUp, false);
    assert_eq!(ui.get_value("volume"), Some("10"));
    ui.key_down(Key::Right, true);
    assert_eq!(ui.get_value("volume"), Some("11"));
    assert!(!ui.clicked("volume"));
    let mut disabled = range("disabled value=20");
    disabled.pointer_down(118.0, 35.0);
    disabled.pointer_up(118.0, 35.0);
    disabled.key_down(Key::Tab, false);
    disabled.key_down(Key::End, false);
    assert_eq!(disabled.get_value("volume"), Some("20"));
}
#[test]
fn thumb_grab_offset_and_step_any_are_preserved() {
    let mut ui = range("step=any value=50");
    ui.pointer_down(73.0, 35.0);
    assert_eq!(ui.get_value("volume"), Some("50"));
    ui.pointer_move(83.0, 35.0);
    let value = ui.get_value("volume").unwrap().parse::<f64>().unwrap();
    assert!((value - 60.0).abs() < 0.001);
    ui.cancel_input();
    ui.pointer_move(118.0, 35.0);
    assert_eq!(
        ui.get_value("volume").unwrap().parse::<f64>().unwrap(),
        value
    );
}
#[test]
fn invalid_range_attributes_are_rejected() {
    for a in [
        "min=8 max=2",
        "step=0",
        "step=-1",
        "min=NaN",
        "value=inf",
        "readonly",
        "maxlength=4",
    ] {
        assert!(
            Ui::from_html_css_with_viewport(&format!("<input type=range {a}>"), "", 100.0, 100.0)
                .is_err(),
            "{a}"
        );
    }
    assert!(Ui::from_html_css_with_viewport("<input min=1>", "", 100.0, 100.0).is_err());
}
#[derive(Default)]
struct Recorder {
    fills: Vec<(Rect, Color)>,
    texts: Vec<(String, Rect)>,
    canvases: Vec<(Rect, Rect)>,
}
impl Renderer for Recorder {
    type Error = ();
    fn measure_text(&mut self, t: &str, _: f32) -> Result<(f32, f32), ()> {
        Ok((t.len() as f32 * 8.0, 12.0))
    }
    fn draw(&mut self, p: PaintCommand<'_>) -> Result<(), ()> {
        match p {
            PaintCommand::FillRect { rect, color } => self.fills.push((rect, color)),
            PaintCommand::Text { text, clip, .. } => self.texts.push((text.into(), clip)),
        }
        Ok(())
    }
    fn draw_canvas(&mut self, c: CanvasRegion<'_>) -> Result<(), ()> {
        self.canvases.push((c.content, c.clip));
        Ok(())
    }
}
fn panel(overflow: &str) -> Ui {
    Ui::from_html_css_with_viewport("<aside id=panel><button id=a>A</button><button id=b>B</button><button id=c>C</button></aside>",&format!("aside{{width:120px;height:100px;overflow:{overflow}}}button{{width:100px;height:60px;border:none;background:red}}"),300.0,200.0).unwrap()
}
#[test]
fn overflow_modes_distinguish_clipping_scrolling_and_gutters() {
    for mode in ["visible", "hidden", "clip", "auto", "scroll"] {
        let mut ui = panel(mode);
        let mut rec = Recorder::default();
        ui.render(&mut rec).unwrap();
        let max = ui.scroll_max("panel").unwrap();
        if mode == "visible" {
            assert!(
                rec.fills
                    .iter()
                    .any(|(r, c)| *c == Color(255, 0, 0, 255) && r.y + r.h > 100.0)
            );
        } else {
            assert!(
                rec.fills
                    .iter()
                    .filter(|(_, c)| *c == Color(255, 0, 0, 255))
                    .all(|(r, _)| r.y + r.h <= 100.0)
            );
        }
        if matches!(mode, "visible" | "clip") {
            assert_eq!(max, (0.0, 0.0));
        } else {
            assert!(max.1 >= 80.0);
        }
        ui.set_scroll_offset("panel", 0.0, 50.0).unwrap();
        assert_eq!(
            ui.scroll_offset("panel").unwrap().1,
            if matches!(mode, "visible" | "clip") {
                0.0
            } else {
                50.0
            }
        );
    }
    let ui = Ui::from_html_css_with_viewport(
        "<aside id=p><button>B</button></aside>",
        "aside{width:100px;height:100px;overflow:scroll}button{height:20px}",
        200.0,
        200.0,
    )
    .unwrap();
    assert_eq!(ui.scroll_max("p"), Some((0.0, 0.0)));
    let mut rec = Recorder::default();
    ui.render(&mut rec).unwrap();
    assert!(rec.fills.iter().any(|(r, _)| r.x == 86.0));
}
#[test]
fn scrolling_updates_hit_testing_and_clamps_on_resize() {
    let mut ui = panel("auto");
    ui.set_scroll_offset("panel", 0.0, 80.0).unwrap();
    assert_eq!(ui.bounds("c").unwrap().y, 40.0);
    ui.pointer_down(20.0, 50.0);
    ui.pointer_up(20.0, 50.0);
    assert!(ui.clicked("c"));
    assert!(!ui.clicked("a"));
    ui.end_frame();
    ui.pointer_down(20.0, 130.0);
    ui.pointer_up(20.0, 130.0);
    assert!(!ui.clicked("c"));
    let old = ui.paint_revision();
    ui.set_scroll_offset("panel", 0.0, 80.0).unwrap();
    assert_eq!(old, ui.paint_revision());
    assert!(ui.set_scroll_offset("panel", f32::NAN, 0.0).is_err());
    assert_eq!(ui.scroll_offset("panel"), Some((0.0, 80.0)));
    let mut responsive = Ui::from_html_css_with_viewport(
        "<aside id=p><label>A</label></aside>",
        "aside{height:100%;overflow:auto}label{height:200px}",
        100.0,
        100.0,
    )
    .unwrap();
    responsive.set_scroll_offset("p", 0.0, 100.0).unwrap();
    responsive.set_viewport(100.0, 300.0).unwrap();
    assert_eq!(responsive.scroll_offset("p"), Some((0.0, 0.0)));
}
#[test]
fn nested_wheel_scrolls_inner_then_chains_to_outer_and_hidden_ignores_wheel() {
    let mut ui=Ui::from_html_css_with_viewport("<main id=outer><aside id=inner><label>tall</label></aside><label>below</label></main>","main{width:200px;height:100px;overflow:auto}aside{height:80px;overflow:auto}label{height:200px}",300.0,300.0).unwrap();
    ui.pointer_move(10.0, 10.0);
    ui.scroll_wheel(0.0, 40.0);
    assert_eq!(ui.scroll_offset("inner").unwrap().1, 40.0);
    assert_eq!(ui.scroll_offset("outer").unwrap().1, 0.0);
    ui.scroll_wheel(0.0, 200.0);
    assert_eq!(ui.scroll_offset("inner").unwrap().1, 120.0);
    assert_eq!(ui.scroll_offset("outer").unwrap().1, 120.0);
    let mut hidden = panel("hidden");
    hidden.pointer_move(20.0, 20.0);
    hidden.scroll_wheel(0.0, 50.0);
    assert_eq!(hidden.scroll_offset("panel"), Some((0.0, 0.0)));
}
#[test]
fn scrollbar_drag_track_paging_horizontal_axis_and_focus_reveal() {
    let mut ui = panel("auto");
    ui.pointer_down(113.0, 10.0);
    ui.pointer_move(113.0, 95.0);
    ui.pointer_up(113.0, 95.0);
    assert_eq!(ui.scroll_offset("panel").unwrap().1, 80.0);
    assert!(!ui.clicked("b"));
    ui.key_down(Key::Home, false);
    assert_eq!(ui.scroll_offset("panel").unwrap().1, 0.0);
    ui.pointer_down(113.0, 95.0);
    ui.pointer_up(113.0, 95.0);
    assert_eq!(ui.scroll_offset("panel").unwrap().1, 80.0);
    ui.set_scroll_offset("panel", 0.0, 0.0).unwrap();
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Tab, false);
    assert_eq!(ui.scroll_offset("panel").unwrap().1, 80.0);
    let mut x=Ui::from_html_css_with_viewport("<aside id=p><label id=text>wide</label></aside>","aside{width:100px;height:60px;overflow-x:auto;overflow-y:hidden}label{width:250px;height:20px}",200.0,100.0).unwrap();
    x.pointer_move(20.0, 20.0);
    x.scroll_wheel(70.0, 0.0);
    assert_eq!(x.scroll_offset("p"), Some((70.0, 0.0)));
    assert_eq!(x.bounds("text").unwrap().x, -70.0);
}
#[test]
fn native_canvas_keeps_full_geometry_and_receives_separate_clip() {
    let mut ui=Ui::from_html_css_with_viewport("<aside id=p><canvas id=scene></canvas></aside>","aside{width:100px;height:80px;overflow:hidden}canvas{width:200px;height:160px;padding:4px}",300.0,200.0).unwrap();
    ui.set_scroll_offset("p", 30.0, 40.0).unwrap();
    let region = ui.canvas_region("scene").unwrap();
    assert_eq!(
        region.content,
        Rect {
            x: -26.0,
            y: -36.0,
            w: 192.0,
            h: 152.0
        }
    );
    assert_eq!(
        region.clip,
        Rect {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 80.0
        }
    );
    assert!(ui.canvas_at(110.0, 20.0).is_none());
    assert!(ui.canvas_at(20.0, 20.0).is_some());
    let mut rec = Recorder::default();
    ui.render(&mut rec).unwrap();
    assert_eq!(rec.canvases, [(region.content, region.clip)]);
    ui.pointer_down(10.0, 10.0);
    ui.set_scroll_offset("p", 0.0, 0.0).unwrap();
    assert!(ui.canvas_input("scene").unwrap().cancelled);
}
#[test]
fn styled_parts_are_reusable_and_gutters_reflow_percentage_children() {
    let mut ui=Ui::from_html_css_with_viewport("<aside id=p><input type=range id=r value=0><label>long</label></aside>","aside{width:120px;height:90px;overflow:auto}input{height:30px}label{height:150px}::-webkit-scrollbar{width:10px;height:8px}::-webkit-scrollbar-thumb{background:#ff00ff;border-radius:3px}input::slider-thumb{width:24px;height:24px;background:#00ff00}input::slider-track{height:6px;background:#ff0000}",300.0,200.0).unwrap();
    assert_eq!(ui.bounds("r").unwrap().w, 110.0);
    let mut rec = Recorder::default();
    ui.render(&mut rec).unwrap();
    assert!(
        rec.fills
            .iter()
            .any(|(r, c)| *c == Color(255, 0, 255, 255) && r.x >= 110.0)
    );
    assert!(rec.fills.iter().any(|(_, c)| *c == Color(0, 255, 0, 255)));
    ui.pointer_down(55.0, 15.0);
    ui.pointer_up(55.0, 15.0);
    assert_eq!(ui.get_value("r"), Some("50"));
}

#[test]
fn axis_normalization_bar_interdependence_and_clipped_descendants() {
    let mut ui = Ui::from_html_css_with_viewport(
        "<aside id=p><label>wide</label></aside>",
        "aside{width:100px;height:100px;overflow-y:auto}label{width:95px;height:120px}",
        200.0,
        200.0,
    )
    .unwrap();
    // The vertical gutter induces horizontal overflow; visible computes to auto.
    assert_eq!(ui.scroll_max("p"), Some((9.0, 34.0)));
    ui.set_scroll_offset("p", 100.0, 100.0).unwrap();
    assert_eq!(ui.scroll_offset("p"), Some((9.0, 34.0)));
    let clipped=Ui::from_html_css_with_viewport("<main id=p><aside><label>long</label></aside></main>","main{width:200px;height:100px;overflow:auto}aside{width:100px;height:50px;overflow:clip}label{height:300px}",300.0,300.0).unwrap();
    assert_eq!(clipped.scroll_max("p"), Some((0.0, 0.0)));
    let mut hidden = Ui::from_html_css_with_viewport(
        "<aside id=p tabindex=0><label>long</label></aside>",
        "aside{height:50px;overflow:hidden}label{height:200px}",
        100.0,
        100.0,
    )
    .unwrap();
    hidden.key_down(Key::Tab, false);
    hidden.key_down(Key::End, false);
    assert_eq!(hidden.scroll_offset("p"), Some((0.0, 0.0)));
}
#[test]
fn keyboard_accessible_container_and_range_label_focus() {
    let mut ui=Ui::from_html_css_with_viewport("<aside id=p tabindex=0><label for=r>Volume</label><input id=r type=range value=20><label>long</label></aside>","aside{width:150px;height:100px;overflow:auto}label{height:40px}input{height:30px}",200.0,200.0).unwrap();
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::End, false);
    assert_eq!(ui.scroll_offset("p").unwrap().1, 10.0);
    ui.set_scroll_offset("p", 0.0, 0.0).unwrap();
    ui.pointer_down(90.0, 10.0);
    ui.pointer_up(90.0, 10.0);
    assert_eq!(ui.get_value("r"), Some("20"));
    ui.key_down(Key::Right, false);
    assert_eq!(ui.get_value("r"), Some("21"));
}

#[test]
fn unsupported_part_geometry_has_diagnostics_instead_of_silent_fallbacks() {
    for css in [
        "input::slider-thumb{width:50%}",
        "input::slider-track{height:auto}",
        "input::slider-fill{height:8px}",
        "aside::scrollbar-track{width:9px}",
        "input::slider-thumb.extra{background:red}",
    ] {
        assert!(
            Ui::from_html_css_with_viewport("<aside><input type=range></aside>", css, 200.0, 200.0)
                .is_err(),
            "{css}"
        );
    }
}
