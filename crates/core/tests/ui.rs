use native_ui::{Color, Key, PaintCommand, Rect, Renderer, Ui};
const GEOMETRY: &str = "button {position:absolute; left:10px; top:20px; width:100px; height:40px}";
fn button() -> Ui {
    Ui::from_html_css("<button id=save>Save</button>", GEOMETRY).unwrap()
}
#[derive(Default, Debug, PartialEq)]
struct Recorder {
    rectangles: Vec<(Rect, Color)>,
    text: Vec<(String, f32, f32, Color, Rect)>,
}
impl Renderer for Recorder {
    type Error = std::convert::Infallible;
    fn measure_text(&mut self, _: &str, _: f32) -> Result<(f32, f32), Self::Error> {
        Ok((30.0, 12.0))
    }
    fn draw(&mut self, command: PaintCommand<'_>) -> Result<(), Self::Error> {
        match command {
            PaintCommand::FillRect { rect, color } => self.rectangles.push((rect, color)),
            PaintCommand::Text {
                text,
                x,
                y,
                color,
                clip,
                ..
            } => self.text.push((text.into(), x, y, color, clip)),
        };
        Ok(())
    }
}
fn draw(ui: &Ui) -> Recorder {
    let mut r = Recorder::default();
    ui.render(&mut r).unwrap();
    r
}
#[test]
fn html_is_compiled_and_entities_and_nested_labels_are_decoded() {
    let ui=Ui::from_html_css("<!doctype html><html><head><title>Demo</title></head><body><div><button id=save> Save &amp; <span>café</span> </button></div></body></html>", GEOMETRY).unwrap();
    assert_eq!(
        ui.bounds("save"),
        Some(Rect {
            x: 10.0,
            y: 20.0,
            w: 100.0,
            h: 40.0
        })
    );
    assert_eq!(draw(&ui).text[0].0, "Save & café");
}
#[test]
fn unique_ids_include_structural_elements() {
    let result = Ui::from_html_css("<div id=save><button id=save>Save</button></div>", GEOMETRY);
    assert!(result.unwrap_err().to_string().contains("duplicate"));
}
#[test]
fn anonymous_buttons_render_without_becoming_native_ids() {
    let ui = Ui::from_html_css("<button>Decorative identity</button>", GEOMETRY).unwrap();
    assert_eq!(draw(&ui).text.len(), 1);
    assert!(!ui.contains_id(""));
}
#[test]
fn click_lives_until_end_of_frame_and_reads_do_not_consume() {
    let mut ui = button();
    ui.pointer_down(20.0, 30.0);
    ui.pointer_up(20.0, 30.0);
    ui.pointer_move(400.0, 400.0);
    assert!(ui.clicked("save"));
    assert!(ui.clicked("save"));
    ui.end_frame();
    assert!(!ui.clicked("save"));
    assert!(!ui.clicked("missing"));
}
#[test]
fn only_topmost_button_gets_capture() {
    let mut ui = Ui::from_html_css(
        "<button id=back>Back</button><button id=front>Front</button>",
        GEOMETRY,
    )
    .unwrap();
    ui.pointer_down(20.0, 30.0);
    ui.pointer_up(20.0, 30.0);
    assert!(ui.clicked("front"));
    assert!(!ui.clicked("back"));
}
#[test]
fn release_without_press_and_drag_in_do_not_click() {
    let mut ui = button();
    ui.pointer_up(20.0, 30.0);
    assert!(!ui.clicked("save"));
    ui.pointer_down(0.0, 0.0);
    ui.pointer_up(20.0, 30.0);
    assert!(!ui.clicked("save"));
}
#[test]
fn drag_out_cancels_release() {
    let mut ui = button();
    ui.pointer_down(20.0, 30.0);
    ui.pointer_move(300.0, 300.0);
    ui.pointer_up(300.0, 300.0);
    assert!(!ui.clicked("save"));
}
#[test]
fn drag_out_and_back_keeps_capture() {
    let mut ui = button();
    ui.pointer_down(20.0, 30.0);
    ui.pointer_leave();
    ui.pointer_move(20.0, 30.0);
    ui.pointer_up(20.0, 30.0);
    assert!(ui.clicked("save"));
}
#[test]
fn focus_loss_cancels_mouse_and_space_without_fake_clicks() {
    let mut ui = button();
    ui.pointer_down(20.0, 30.0);
    ui.cancel_input();
    ui.pointer_up(20.0, 30.0);
    assert!(!ui.clicked("save"));
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Space, false);
    ui.cancel_input();
    ui.key_up(Key::Space);
    assert!(!ui.clicked("save"));
}
#[test]
fn disabled_topmost_button_blocks_underlying_button_and_is_skipped_by_tab() {
    let mut ui = Ui::from_html_css(
        "<button id=back>Back</button><button id=front disabled>Front</button>",
        GEOMETRY,
    )
    .unwrap();
    ui.pointer_down(20.0, 30.0);
    ui.pointer_up(20.0, 30.0);
    assert!(!ui.clicked("back"));
    assert!(!ui.clicked("front"));
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Enter, false);
    assert!(ui.clicked("back"));
    assert!(!ui.clicked("front"));
}
#[test]
fn keyboard_activation_and_repeat_suppression() {
    let mut ui = button();
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Space, false);
    assert!(!ui.clicked("save"));
    ui.key_up(Key::Space);
    assert!(ui.clicked("save"));
    ui.end_frame();
    ui.key_down(Key::Enter, true);
    assert!(!ui.clicked("save"));
    ui.key_down(Key::Enter, false);
    assert!(ui.clicked("save"));
}
#[test]
fn tab_shift_tab_wrap_in_document_order() {
    let mut ui =
        Ui::from_html_css("<button id=a>A</button><button id=b>B</button>", GEOMETRY).unwrap();
    ui.key_down(Key::BackTab, false);
    ui.key_down(Key::Enter, false);
    assert!(ui.clicked("b"));
    ui.end_frame();
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Enter, false);
    assert!(ui.clicked("a"));
}
#[test]
fn shared_edges_and_zero_size_are_not_double_hits() {
    let mut ui = Ui::new();
    ui.add_button_at(
        "a",
        "A",
        Rect {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
        },
    )
    .unwrap();
    ui.add_button_at(
        "b",
        "B",
        Rect {
            x: 10.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
        },
    )
    .unwrap();
    ui.add_button_at(
        "zero",
        "Z",
        Rect {
            x: 10.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        },
    )
    .unwrap();
    ui.pointer_down(10.0, 0.0);
    ui.pointer_up(10.0, 0.0);
    assert!(!ui.clicked("a"));
    assert!(ui.clicked("b"));
    assert!(!ui.clicked("zero"));
    assert_eq!(draw(&ui).text.len(), 2);
}
#[test]
fn invalid_rectangles_and_duplicate_programmatic_ids_are_errors() {
    let mut ui = button();
    assert!(ui.add_button_at("save", "dup", Rect::default()).is_err());
    assert!(
        ui.add_button_at(
            "nan",
            "NaN",
            Rect {
                x: f32::NAN,
                ..Rect::default()
            }
        )
        .is_err()
    );
    assert!(
        ui.add_button_at(
            "neg",
            "Negative",
            Rect {
                w: -1.0,
                ..Rect::default()
            }
        )
        .is_err()
    );
}
#[test]
fn css_cascade_specificity_source_order_inline_and_important() {
    let ui=Ui::from_html_css("<button id=save class=action style='left:90px;top:80px!important'>Save</button>",
        &format!("{GEOMETRY} #save {{left:30px!important;top:50px!important}} .action {{left:40px}} button.action {{width:110px}} .action {{height:45px}} .action {{height:50px}}" )).unwrap();
    assert_eq!(
        ui.bounds("save"),
        Some(Rect {
            x: 30.0,
            y: 80.0,
            w: 110.0,
            h: 50.0
        })
    );
}
#[test]
fn css_comments_case_escapes_and_selector_lists() {
    let ui=Ui::from_html_css("<button id=save>Save</button>",
        "/* hi */ #sa\\76 e, .other { POSITION: absolute; left:0; top:-2.5PX; width:1e2px; height:40px; }").unwrap();
    assert_eq!(
        ui.bounds("save"),
        Some(Rect {
            x: 0.0,
            y: -2.5,
            w: 100.0,
            h: 40.0
        })
    );
}
#[test]
fn embedded_css_and_html_error_recovery_work() {
    let ui = Ui::from_html_css(
        &format!("<style>{GEOMETRY}</style><button id=save>Save"),
        "",
    )
    .unwrap();
    assert!(ui.contains_id("save"));
}
#[test]
fn unsupported_markup_and_css_return_diagnostics() {
    for html in [
        "<input id=save>",
        "<script>alert(1)</script>",
        "<canvas></canvas>",
        "<button hidden>Save</button>",
        "<p>Hello</p>",
    ] {
        assert!(Ui::from_html_css(html, GEOMETRY).is_err(), "{html}");
    }
    for extra in [
        "button{width:20%}",
        "button{width:-1px}",
        "button{height:2}",
        "button{display:flex}",
        "button:hover{left:1px}",
        "div button{left:1px}",
        "@media screen{button{left:1px}}",
    ] {
        assert!(
            Ui::from_html_css("<button>Save</button>", &format!("{GEOMETRY}{extra}")).is_err(),
            "{extra}"
        );
    }
    assert!(Ui::from_html_css("<button>Save</button>", "").is_err());
}
#[test]
fn padding_shorthand_individual_sides_and_important_resolve_per_side() {
    let ui=Ui::from_html_css("<button id=save>Save</button>",
        &format!("{GEOMETRY} button{{padding:2px 4px 6px 8px; padding-left:10px!important; padding:3px 5px}}" )).unwrap();
    let record = draw(&ui);
    let (_, x, y, _, clip) = record.text[0];
    assert_eq!(
        clip,
        Rect {
            x: 21.0,
            y: 24.0,
            w: 83.0,
            h: 32.0
        }
    );
    assert_eq!(x, 47.5);
    assert_eq!(y, 34.0);
}
#[test]
fn each_padding_shorthand_form_matches_css_order() {
    for (css, expected) in [
        ("2px", [2.0, 2.0, 2.0, 2.0]),
        ("2px 4px", [2.0, 4.0, 2.0, 4.0]),
        ("2px 4px 6px", [2.0, 4.0, 6.0, 4.0]),
        ("2px 4px 6px 8px", [2.0, 4.0, 6.0, 8.0]),
    ] {
        let ui = Ui::from_html_css(
            "<button>Save</button>",
            &format!("{GEOMETRY}button{{padding:{css}}}"),
        )
        .unwrap();
        let clip = draw(&ui).text[0].4;
        assert_eq!(clip.x, 11.0 + expected[3]);
        assert_eq!(clip.y, 21.0 + expected[0]);
        assert_eq!(clip.w, 98.0 - expected[1] - expected[3]);
        assert_eq!(clip.h, 38.0 - expected[0] - expected[2]);
    }
}
#[test]
fn box_sizing_and_padding_define_geometry_and_hit_area() {
    let css = format!("{GEOMETRY}button{{padding:4px 8px;box-sizing:content-box}}");
    let mut ui = Ui::from_html_css("<button id=save>Save</button>", &css).unwrap();
    assert_eq!(ui.bounds("save").unwrap().w, 118.0);
    assert_eq!(ui.bounds("save").unwrap().h, 50.0);
    ui.pointer_down(125.0, 65.0);
    ui.pointer_up(125.0, 65.0);
    assert!(ui.clicked("save"));
}
#[test]
fn padding_overflow_clips_text_and_negative_padding_is_rejected() {
    let ui = Ui::from_html_css(
        "<button>Save</button>",
        &format!("{GEOMETRY}button{{padding:100px}}"),
    )
    .unwrap();
    assert!(draw(&ui).text.is_empty());
    assert_eq!(draw(&ui).rectangles[0].0.w, 202.0);
    for css in [
        "padding:-1px",
        "padding:1px 2px 3px 4px 5px",
        "padding-left:2%",
    ] {
        assert!(
            Ui::from_html_css(
                "<button>Save</button>",
                &format!("{GEOMETRY}button{{{css}}}")
            )
            .is_err()
        );
    }
}
#[test]
fn colors_reach_paint_commands_with_alpha_and_background_alias() {
    let ui = Ui::from_html_css(
        "<button>Save</button>",
        &format!("{GEOMETRY}button{{background:#1234;color:rgba(100, 50, 0, 0.5)}}"),
    )
    .unwrap();
    let record = draw(&ui);
    assert_eq!(record.rectangles[4].1, Color(17, 34, 51, 68));
    assert_eq!(record.text[0].3, Color(100, 50, 0, 128));
    let ui = Ui::from_html_css(
        "<button>Save</button>",
        &format!("{GEOMETRY}button{{background-color:rgb(100% 0% 0% / 25%);color:#abcdef}}"),
    )
    .unwrap();
    assert_eq!(draw(&ui).rectangles[4].1, Color(255, 0, 0, 64));
    assert_eq!(draw(&ui).text[0].3, Color(171, 205, 239, 255));
}
#[test]
fn invalid_colors_fail_without_panics() {
    for color in [
        "#xxff00",
        "#12",
        "#ééé",
        "linear-gradient(red,blue)",
        "rgb(1 2)",
        "rgb(1,2,3) junk",
    ] {
        assert!(
            Ui::from_html_css(
                "<button>Save</button>",
                &format!("{GEOMETRY}button{{color:{color}}}")
            )
            .is_err(),
            "{color}"
        );
    }
}
#[test]
fn renderer_errors_propagate_to_the_host() {
    struct Fails;
    impl Renderer for Fails {
        type Error = &'static str;
        fn measure_text(&mut self, _: &str, _: f32) -> Result<(f32, f32), Self::Error> {
            Err("font error")
        }
        fn draw(&mut self, _: PaintCommand<'_>) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    assert_eq!(button().render(&mut Fails), Err("font error"));
}

#[test]
fn pointer_and_keyboard_states_do_not_invent_visual_styles() {
    for colors in ["", "button {background-color:#112233;color:#eeddcc}"] {
        let mut ui = Ui::from_html_css(
            "<button id=save>Save</button>",
            &format!("{GEOMETRY}{colors}"),
        )
        .unwrap();
        let normal = draw(&ui);
        ui.pointer_move(20.0, 30.0);
        assert_eq!(draw(&ui), normal, "hover must not change paint");
        ui.pointer_down(20.0, 30.0);
        assert_eq!(
            draw(&ui),
            normal,
            "press and pointer focus must not change paint"
        );
        ui.pointer_up(20.0, 30.0);
        assert_eq!(draw(&ui), normal);
        ui.cancel_input();
        ui.key_down(Key::Tab, false);
        assert_eq!(draw(&ui), normal, "keyboard focus must not change paint");
        ui.key_down(Key::Space, false);
        assert_eq!(draw(&ui), normal, "keyboard press must not change paint");
    }
}
#[test]
fn disabled_attribute_affects_interaction_without_overriding_css() {
    let css = format!("{GEOMETRY} button {{background:#112233;color:#eeddcc}}");
    let enabled = Ui::from_html_css("<button id=save>Save</button>", &css).unwrap();
    let mut disabled = Ui::from_html_css("<button id=save disabled>Save</button>", &css).unwrap();
    assert_eq!(draw(&disabled), draw(&enabled));
    disabled.pointer_down(20.0, 30.0);
    disabled.pointer_up(20.0, 30.0);
    assert!(!disabled.clicked("save"));
    assert_eq!(draw(&disabled), draw(&enabled));
}

fn state_ui(rules: &str, attrs: &str) -> Ui {
    Ui::from_html_css(
        &format!("<button id=save class=action {attrs}>Save</button>"),
        &format!("{GEOMETRY}button{{background:#112233;color:white}}{rules}"),
    )
    .unwrap()
}
fn background(ui: &Ui) -> Color {
    draw(ui).rectangles[4].1
}
#[test]
fn css_hover_and_active_track_input_and_restore_the_base_colors() {
    let mut ui = state_ui(
        "button:hover{background:red}button:active{background:blue}",
        "",
    );
    assert_eq!(background(&ui), Color(17, 34, 51, 255));
    ui.pointer_move(20.0, 30.0);
    assert_eq!(background(&ui), Color(255, 0, 0, 255));
    ui.pointer_down(20.0, 30.0);
    assert_eq!(background(&ui), Color(0, 0, 255, 255));
    ui.pointer_leave();
    assert_eq!(background(&ui), Color(0, 0, 255, 255));
    ui.pointer_up(400.0, 400.0);
    assert_eq!(background(&ui), Color(17, 34, 51, 255));
    assert!(!ui.clicked("save"));
    ui.pointer_move(20.0, 30.0);
    assert_eq!(background(&ui), Color(255, 0, 0, 255));
    ui.pointer_leave();
    assert_eq!(background(&ui), Color(17, 34, 51, 255));
}
#[test]
fn css_focus_and_keyboard_active_clear_on_release_and_focus_loss() {
    let mut ui = state_ui(
        "button:focus{color:yellow}button:active{background:blue}",
        "",
    );
    ui.key_down(Key::Tab, false);
    assert_eq!(draw(&ui).text[0].3, Color(255, 255, 0, 255));
    assert_eq!(background(&ui), Color(17, 34, 51, 255));
    ui.key_down(Key::Space, false);
    assert_eq!(background(&ui), Color(0, 0, 255, 255));
    ui.key_up(Key::Space);
    assert_eq!(background(&ui), Color(17, 34, 51, 255));
    assert!(ui.clicked("save"));
    ui.key_down(Key::Space, false);
    ui.cancel_input();
    assert_eq!(background(&ui), Color(17, 34, 51, 255));
    assert_eq!(draw(&ui).text[0].3, Color(255, 255, 255, 255));
}
#[test]
fn disabled_css_matches_the_attribute_and_combines_with_hover_without_activation() {
    let rules = "button:hover{background:red}button:disabled{background:gray;color:silver}button:disabled:hover{color:yellow}button:active{background:blue}";
    let mut ui = state_ui(rules, "disabled");
    assert_eq!(background(&ui), Color(128, 128, 128, 255));
    assert_eq!(draw(&ui).text[0].3, Color(192, 192, 192, 255));
    ui.pointer_down(20.0, 30.0);
    assert_eq!(background(&ui), Color(128, 128, 128, 255));
    assert_eq!(draw(&ui).text[0].3, Color(255, 255, 0, 255));
    ui.pointer_up(20.0, 30.0);
    assert!(!ui.clicked("save"));
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Space, false);
    assert_eq!(background(&ui), Color(128, 128, 128, 255));
    assert_eq!(background(&state_ui(rules, "")), Color(17, 34, 51, 255));
}
#[test]
fn compound_pseudo_classes_and_pseudo_class_specificity_are_respected() {
    let mut ui = state_ui(
        "button.action:hover:active{background:green}button.action:active{background:blue}button:hover{color:yellow}.action{color:red}",
        "",
    );
    ui.pointer_move(20.0, 30.0);
    assert_eq!(draw(&ui).text[0].3, Color(255, 255, 0, 255));
    ui.pointer_down(20.0, 30.0);
    assert_eq!(background(&ui), Color(0, 128, 0, 255));
}
#[test]
fn pseudo_classes_obey_inline_and_important_cascade() {
    let mut ui = state_ui(
        "#save:hover{background:red!important;color:yellow!important}",
        "style='background:blue;color:lime!important'",
    );
    ui.pointer_move(20.0, 30.0);
    assert_eq!(background(&ui), Color(255, 0, 0, 255));
    assert_eq!(draw(&ui).text[0].3, Color(0, 255, 0, 255));
    let mut ui = state_ui("#save:hover{background:red}", "style='background:blue'");
    ui.pointer_move(20.0, 30.0);
    assert_eq!(background(&ui), Color(0, 0, 255, 255));
}
#[test]
fn only_matching_selectors_in_a_list_contribute_specificity() {
    let mut ui = state_ui(
        ".action, #save:hover{background:red}button.action{background:blue}",
        "",
    );
    assert_eq!(background(&ui), Color(0, 0, 255, 255));
    ui.pointer_move(20.0, 30.0);
    assert_eq!(background(&ui), Color(255, 0, 0, 255));
}
#[test]
fn pseudo_class_case_repetition_and_source_order_are_supported() {
    let mut ui = state_ui(
        "button:HoVeR:hover{background:red}button:hover{background:blue}button:focus{color:yellow}button:hover{color:red}",
        "",
    );
    ui.pointer_down(20.0, 30.0);
    assert_eq!(background(&ui), Color(255, 0, 0, 255));
    assert_eq!(draw(&ui).text[0].3, Color(255, 0, 0, 255));
}
#[test]
fn state_selectors_target_only_the_hit_button_and_do_not_change_geometry() {
    let mut ui = Ui::from_html_css(
        "<button id=a>A</button><button id=b>B</button>",
        &format!("{GEOMETRY}button{{background:blue}}button:hover{{background:red}}"),
    )
    .unwrap();
    let bounds = ui.bounds("b");
    ui.pointer_move(20.0, 30.0);
    let painted = draw(&ui);
    assert_eq!(painted.rectangles[4].1, Color(0, 0, 255, 255));
    assert_eq!(painted.rectangles[9].1, Color(255, 0, 0, 255));
    assert_eq!(ui.bounds("b"), bounds);
    for rule in [
        "button:hover{padding:2px}",
        "button:active{left:30px}",
        "button:visited{color:red}",
        "button::before{color:red}",
    ] {
        assert!(
            Ui::from_html_css("<button>Save</button>", &format!("{GEOMETRY}{rule}")).is_err(),
            "{rule}"
        );
    }
}
