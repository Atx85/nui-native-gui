use native_ui::{Key, Rect, Ui};
fn rect(ui: &Ui, id: &str) -> Rect {
    ui.bounds(id).unwrap()
}
#[test]
fn block_flow_uses_parent_padding_additive_margins_and_auto_height() {
    let ui=Ui::from_html_css_with_viewport("<main id=panel><label id=a>A</label><input id=b></main>","body{padding:10px}main{padding:8px;border:2px solid black}label{height:20px;margin:3px 4px 5px 6px}input{height:30px;margin-top:7px}",300.0,200.0).unwrap();
    assert_eq!(
        rect(&ui, "panel"),
        Rect {
            x: 10.0,
            y: 10.0,
            w: 280.0,
            h: 85.0
        }
    );
    assert_eq!(
        rect(&ui, "a"),
        Rect {
            x: 26.0,
            y: 23.0,
            w: 250.0,
            h: 20.0
        }
    );
    assert_eq!(
        rect(&ui, "b"),
        Rect {
            x: 20.0,
            y: 55.0,
            w: 260.0,
            h: 30.0
        }
    );
}
#[test]
fn flex_canvas_and_sidebar_resize_without_reparsing_or_losing_values() {
    let html = "<main id=workspace><canvas id=scene></canvas><aside id=options><input id=rate value=2><select id=mode><option>One</option><option>Two</option></select></aside></main>";
    let css = "body{padding:20px}main{display:flex;gap:16px;height:100%}canvas{flex-grow:1;min-width:100px;padding:10px}aside{width:240px;padding:12px}input,select{height:30px;margin-bottom:8px}";
    let mut ui = Ui::from_html_css_with_viewport(html, css, 1000.0, 600.0).unwrap();
    assert_eq!(
        rect(&ui, "scene"),
        Rect {
            x: 20.0,
            y: 20.0,
            w: 704.0,
            h: 560.0
        }
    );
    assert_eq!(
        rect(&ui, "options"),
        Rect {
            x: 740.0,
            y: 20.0,
            w: 240.0,
            h: 560.0
        }
    );
    assert_eq!(
        rect(&ui, "rate"),
        Rect {
            x: 752.0,
            y: 32.0,
            w: 216.0,
            h: 30.0
        }
    );
    ui.set_value("rate", "3.5").unwrap();
    ui.set_value("mode", "Two").unwrap();
    ui.pointer_down(40.0, 40.0);
    assert!(ui.canvas_input("scene").unwrap().captured);
    ui.set_viewport(1000.0, 600.0).unwrap();
    assert!(
        ui.canvas_input("scene").unwrap().captured,
        "same-size calls do no layout work or input cancellation"
    );
    ui.set_viewport(800.0, 500.0).unwrap();
    assert_eq!(
        rect(&ui, "scene"),
        Rect {
            x: 20.0,
            y: 20.0,
            w: 504.0,
            h: 460.0
        }
    );
    assert_eq!(rect(&ui, "rate").x, 552.0);
    assert_eq!(ui.get_value("rate"), Some("3.5"));
    assert_eq!(ui.get_value("mode"), Some("Two"));
    let input = ui.canvas_input("scene").unwrap();
    assert!(input.cancelled && !input.captured);
    assert!(ui.canvas_at(560.0, 40.0).is_none());
    assert_eq!(ui.canvas_at(40.0, 40.0).unwrap().id, Some("scene"));
    ui.key_down(Key::Tab, false);
    ui.text_input("!");
    assert_eq!(
        ui.get_value("rate"),
        Some("3.5!"),
        "containers are not tab stops"
    );
}
#[test]
fn relative_offsets_do_not_change_sibling_flow_and_absolute_children_are_out_of_flow() {
    let ui=Ui::from_html_css_with_viewport("<main><label id=a>A</label><label id=overlay>Overlay</label><label id=b>B</label></main>","body{padding:10px}main{padding:5px;gap:4px}label{height:20px}#a{position:relative;left:7px;top:3px}#overlay{position:absolute;left:30px;top:50px;width:70px}",200.0,200.0).unwrap();
    assert_eq!(
        rect(&ui, "a"),
        Rect {
            x: 22.0,
            y: 18.0,
            w: 170.0,
            h: 20.0
        }
    );
    assert_eq!(rect(&ui, "b").y, 39.0);
    assert_eq!(
        rect(&ui, "overlay"),
        Rect {
            x: 45.0,
            y: 65.0,
            w: 70.0,
            h: 20.0
        }
    );
}
#[test]
fn percentages_box_sizing_grow_limits_and_alignment_are_resolved() {
    let ui=Ui::from_html_css_with_viewport("<main><canvas id=a></canvas><canvas id=b></canvas></main>","main{display:flex;height:100px;align-items:center;gap:10px}canvas{height:50%;flex-grow:1;margin:5px}#a{max-width:100px}#b{box-sizing:content-box;padding:4px}",400.0,200.0).unwrap();
    assert_eq!(
        rect(&ui, "a"),
        Rect {
            x: 5.0,
            y: 25.0,
            w: 100.0,
            h: 50.0
        }
    );
    assert_eq!(
        rect(&ui, "b"),
        Rect {
            x: 125.0,
            y: 21.0,
            w: 270.0,
            h: 58.0
        }
    );
    let ui=Ui::from_html_css_with_viewport("<main><canvas id=a></canvas><canvas id=b></canvas></main>","main{display:flex;flex-direction:column;height:100%;gap:10px;padding:10px}canvas{flex-grow:1}#a{height:40px;max-height:60px}#b{width:50%;margin-left:5px}",300.0,200.0).unwrap();
    assert_eq!(
        rect(&ui, "a"),
        Rect {
            x: 10.0,
            y: 10.0,
            w: 280.0,
            h: 60.0
        }
    );
    assert_eq!(
        rect(&ui, "b"),
        Rect {
            x: 15.0,
            y: 80.0,
            w: 140.0,
            h: 110.0
        }
    );
}
#[test]
fn invalid_resize_is_transactional_and_layout_states_are_rejected() {
    let mut ui = Ui::from_html_css_with_viewport(
        "<canvas id=scene></canvas>",
        "canvas{height:100%}",
        100.0,
        100.0,
    )
    .unwrap();
    let before = rect(&ui, "scene");
    for size in [(f32::NAN, 100.0), (-1.0, 100.0), (100.0, f32::INFINITY)] {
        assert!(ui.set_viewport(size.0, size.1).is_err());
        assert_eq!(rect(&ui, "scene"), before);
    }
    for css in [
        "canvas{display:grid}",
        "canvas{margin:auto}",
        "canvas{flex-grow:-1}",
        "canvas{width:-10%}",
        "canvas{min-width:40px;max-width:20px}",
        "canvas:hover{margin:10px}",
        "canvas{flex-wrap:wrap}",
    ] {
        assert!(
            Ui::from_html_css_with_viewport("<canvas></canvas>", css, 100.0, 100.0).is_err(),
            "{css}"
        );
    }
}
#[test]
fn min_sizes_overflow_predictably_and_zero_viewports_stay_finite() {
    let mut ui=Ui::from_html_css_with_viewport("<main><canvas id=scene></canvas><aside id=aside></aside></main>","main{display:flex;gap:12px;height:100%}canvas{flex-grow:1;min-width:80px}aside{width:200px}",100.0,100.0).unwrap();
    assert_eq!(rect(&ui, "scene").w, 80.0);
    assert_eq!(rect(&ui, "aside").x, 92.0);
    ui.set_viewport(0.0, 0.0).unwrap();
    assert_eq!(rect(&ui, "scene").h, 0.0);
    assert!(rect(&ui, "aside").x.is_finite());
}
