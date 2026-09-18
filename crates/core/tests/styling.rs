use native_ui::{Color, Key, PaintCommand, Rect, Renderer, Ui};

const GEOMETRY: &str =
    "button,input,select{position:absolute;left:10px;top:10px;width:120px;height:40px}";
#[derive(Default)]
struct Recorder {
    boxes: Vec<(Rect, Color)>,
    text: Vec<(String, f32, Color, Rect)>,
}
impl Renderer for Recorder {
    type Error = std::convert::Infallible;
    fn measure_text(&mut self, text: &str, size: f32) -> Result<(f32, f32), Self::Error> {
        Ok((text.chars().count() as f32 * size / 2.0, size))
    }
    fn draw(&mut self, command: PaintCommand<'_>) -> Result<(), Self::Error> {
        match command {
            PaintCommand::FillRect { rect, color } => {
                assert!(
                    rect.x.is_finite()
                        && rect.y.is_finite()
                        && rect.w.is_finite()
                        && rect.h.is_finite()
                );
                assert!(rect.w >= 0.0 && rect.h >= 0.0);
                self.boxes.push((rect, color));
            }
            PaintCommand::Text {
                text,
                size,
                color,
                clip,
                ..
            } => self.text.push((text.into(), size, color, clip)),
        }
        Ok(())
    }
}
impl Recorder {
    fn at(&self, x: f32, y: f32) -> Option<Color> {
        self.boxes
            .iter()
            .rev()
            .find(|(rect, c)| c.3 > 0 && rect.contains(x, y))
            .map(|(_, c)| *c)
    }
}
fn draw(ui: &Ui) -> Recorder {
    let mut r = Recorder::default();
    ui.render(&mut r).unwrap();
    r
}
fn button(css: &str) -> Ui {
    Ui::from_html_css("<button id=test>Test</button>", &format!("{GEOMETRY}{css}")).unwrap()
}

#[test]
fn borders_affect_content_box_and_padding_but_visual_states_do_not_change_hit_geometry() {
    let mut ui = button(
        "button{border:3px solid red;padding:2px 4px;font-size:13px;box-sizing:content-box}button:hover{border-color:blue;border-radius:6px;box-shadow:inset 0 0 0 2px white}",
    );
    let bounds = ui.bounds("test").unwrap();
    assert_eq!((bounds.w, bounds.h), (134.0, 50.0));
    let record = draw(&ui);
    assert_eq!(record.text[0].1, 13.0);
    assert_eq!(
        record.text[0].3,
        Rect {
            x: 17.0,
            y: 15.0,
            w: 120.0,
            h: 40.0
        }
    );
    ui.pointer_down(142.0, 55.0);
    ui.pointer_up(142.0, 55.0);
    assert!(ui.clicked("test"));
    assert_eq!(ui.bounds("test"), Some(bounds));
    assert_eq!(draw(&ui).at(50.0, 11.0), Some(Color(0, 0, 255, 255)));
}

#[test]
fn border_shorthand_and_side_colors_follow_normal_cascade() {
    let ui = button(
        "button{border:2px solid black;border-color:red blue green yellow;border-left-color:purple!important;border-color:white}",
    );
    let r = draw(&ui);
    assert_eq!(r.at(10.5, 25.0), Some(Color(128, 0, 128, 255)));
    assert_eq!(r.at(50.0, 10.5), Some(Color(255, 255, 255, 255)));
    assert_eq!(r.at(129.0, 25.0), Some(Color(255, 255, 255, 255)));
    let no_border = button("button{border:none;background:red}");
    assert_eq!(draw(&no_border).boxes.len(), 1);
    assert_eq!(
        draw(&no_border).text[0].3,
        no_border.bounds("test").unwrap()
    );
}

#[test]
fn gradient_direction_stops_and_background_reset_resolve_independently() {
    let mut ui = button(
        "button{border:none;background:linear-gradient(to right,red 0%,blue 50%,lime 100%)}button:hover{background:yellow}",
    );
    let r = draw(&ui);
    assert!(r.at(11.0, 25.0).unwrap().0 > 240);
    assert!(r.at(70.0, 25.0).unwrap().2 > 240);
    assert!(r.at(129.0, 25.0).unwrap().1 > 240);
    ui.pointer_move(20.0, 20.0);
    let r = draw(&ui);
    assert_eq!(
        r.boxes.len(),
        1,
        "solid background shorthand must remove the image"
    );
    assert_eq!(r.at(40.0, 20.0), Some(Color(255, 255, 0, 255)));
    let preserved =
        button("button{background:linear-gradient(0deg,red,blue);background-color:yellow}");
    assert!(draw(&preserved).at(40.0, 12.0).unwrap().2 > 240);
    let reset = button(
        "button{background:linear-gradient(red,blue);background-color:yellow;background-image:none}",
    );
    assert_eq!(draw(&reset).at(40.0, 12.0), Some(Color(255, 255, 0, 255)));
}

#[test]
fn gradient_hard_stops_and_premultiplied_alpha_do_not_create_dark_fringes() {
    let r = draw(&button(
        "button{border:none;background:linear-gradient(red 50%,blue 50%)}",
    ));
    assert_eq!(r.at(40.0, 29.0), Some(Color(255, 0, 0, 255)));
    assert_eq!(r.at(40.0, 30.0), Some(Color(0, 0, 255, 255)));
    let r = draw(&button(
        "button{border:none;background:linear-gradient(transparent,blue)}",
    ));
    let mid = r.at(40.0, 30.0).unwrap();
    assert_eq!((mid.0, mid.1, mid.2), (0, 0, 255));
    assert!((125..=135).contains(&mid.3));
}

#[test]
fn rounded_borders_leave_corners_transparent_and_shadows_stay_inside() {
    let r = draw(&button(
        "button{background:transparent;border:2px solid blue;border-radius:10px;box-shadow:inset 0 0 0 2px red}",
    ));
    assert_eq!(r.at(10.1, 10.1), None);
    assert_eq!(r.at(70.0, 10.5), Some(Color(0, 0, 255, 255)));
    assert_eq!(r.at(70.0, 12.5), Some(Color(255, 0, 0, 255)));
    assert_eq!(r.at(70.0, 30.0), None);
    assert_eq!(r.at(9.0, 30.0), None);
}

#[test]
fn dotted_focus_outline_is_decoration_only_and_clears_on_blur() {
    let mut ui =
        button("button{background:white}button:focus{outline:1px dotted black;outline-offset:2px}");
    let before = draw(&ui).boxes;
    ui.key_down(Key::Tab, false);
    let focused = draw(&ui);
    assert_eq!(focused.at(11.1, 7.2), Some(Color(0, 0, 0, 255)));
    assert_eq!(focused.at(12.1, 7.2), None);
    assert_eq!(ui.bounds("test").unwrap().x, 10.0);
    ui.cancel_input();
    assert_eq!(draw(&ui).boxes, before);
}

#[test]
fn styled_picker_is_separate_from_select_text_and_follows_its_states() {
    let mut ui=Ui::from_html_css("<select id=test><option>Choice</option></select>",&format!("{GEOMETRY}select{{background:white;color:black}}select::picker-icon{{background:blue;color:white;border:1px solid navy}}select:hover::picker-icon{{background:red}}" )).unwrap();
    let r = draw(&ui);
    assert_eq!(r.at(20.0, 20.0), Some(Color(255, 255, 255, 255)));
    assert_eq!(r.at(115.0, 20.0), Some(Color(0, 0, 255, 255)));
    assert_eq!(r.text[0].2, Color(0, 0, 0, 255));
    ui.pointer_move(20.0, 20.0);
    assert_eq!(draw(&ui).at(115.0, 20.0), Some(Color(255, 0, 0, 255)));
}

#[test]
fn option_styles_use_selected_highlighted_and_disabled_states() {
    let mut ui=Ui::from_html_css("<select id=test><option selected>One</option><option>Two</option><option disabled>Three</option></select>",&format!("{GEOMETRY}option{{background:white;color:black;border:none}}option:checked{{color:green}}option:hover{{background:blue;color:white}}option:disabled{{color:gray}}" )).unwrap();
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Enter, false);
    ui.key_down(Key::Down, false);
    let r = draw(&ui);
    assert_eq!(r.text[1].2, Color(0, 128, 0, 255));
    assert_eq!(r.text[2].2, Color(255, 255, 255, 255));
    assert_eq!(r.text[3].2, Color(128, 128, 128, 255));
    assert_eq!(r.at(12.0, 92.0), Some(Color(0, 0, 255, 255)));
    assert_eq!(ui.get_value("test"), Some("One"));
}

#[test]
fn selection_colors_and_font_size_apply_without_changing_text() {
    let mut ui=Ui::from_html_css("<input id=test value=Hello>",&format!("{GEOMETRY}input{{font-size:12px;color:black}}input::selection{{background:blue;color:white}}" )).unwrap();
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::SelectAll, false);
    let r = draw(&ui);
    assert_eq!(r.text.len(), 2);
    assert_eq!(r.text[0].1, 12.0);
    assert_eq!(r.text[1].2, Color(255, 255, 255, 255));
    assert_eq!(r.text[1].3.w, 30.0);
    assert_eq!(ui.get_value("test"), Some("Hello"));
    assert!(!ui.changed("test"));
}

#[test]
fn body_color_is_available_to_the_host_and_does_not_draw_an_extra_control() {
    let ui = button("body{background:#ece9d8}");
    assert_eq!(ui.background_color(), Some(Color(236, 233, 216, 255)));
    assert_eq!(draw(&ui).text.len(), 1);
    assert_eq!(draw(&ui).boxes.len(), 5);
}

#[test]
fn unsupported_decoration_and_state_layout_return_errors() {
    for rule in [
        "button{border:2px dashed red}",
        "button{border-radius:20%}",
        "button{border-width:-1px}",
        "button{border-width:1px 2px}",
        "button{font-size:0}",
        "button{font-size:400px}",
        "button{box-shadow:1px 2px 3px black}",
        "button{box-shadow:inset 1px 0 2px 2px red}",
        "button{background:linear-gradient(45deg,red,blue)}",
        "button{background:linear-gradient(red)}",
        "button{background:linear-gradient(red 10px,blue)}",
        "button{background:linear-gradient(red -5%,blue)}",
        "button{background:linear-gradient(red,red,red,red,red,red,red,red,red)}",
        "button:hover{font-size:12px}",
        "button:hover{border-width:3px}",
        "button:hover{border:1px solid blue}",
        "button{outline:1px dashed black}",
        "button::picker-icon{background:red}",
        "button::selection{background:red}",
        "body{background:linear-gradient(red,blue)}",
    ] {
        assert!(
            Ui::from_html_css("<button>Test</button>", &format!("{GEOMETRY}{rule}")).is_err(),
            "{rule}"
        );
    }
}

#[test]
fn inset_shadow_offsets_and_rounded_focus_rings_do_not_change_bounds() {
    let mut ui = button(
        "button{background:white;border-radius:5px}button:focus{box-shadow:inset 0 -2px 0 0 blue;outline:3px solid red;outline-offset:1px}",
    );
    let bounds = ui.bounds("test");
    ui.key_down(Key::Tab, false);
    let r = draw(&ui);
    assert_eq!(r.at(50.0, 48.0), Some(Color(0, 0, 255, 255)));
    assert_eq!(r.at(50.0, 46.0), Some(Color(255, 255, 255, 255)));
    assert_eq!(
        r.at(6.1, 6.1),
        None,
        "rounded outline corners reveal the host"
    );
    assert_eq!(r.at(50.0, 6.1), Some(Color(255, 0, 0, 255)));
    assert_eq!(ui.bounds("test"), bounds);
    let r = draw(&button(
        "button{background:white;box-shadow:inset 2px 0 0 0 green}",
    ));
    assert_eq!(r.at(11.5, 30.0), Some(Color(0, 128, 0, 255)));
    assert_eq!(r.at(15.0, 30.0), Some(Color(255, 255, 255, 255)));
}

#[test]
fn picker_content_is_css_driven_and_scoped_to_the_existing_indicator() {
    let mut ui=Ui::from_html_css("<select><option>One</option></select>",&format!("{GEOMETRY}select::picker-icon{{content:'˅';font-size:28px;color:black}}select:hover::picker-icon{{content:'˄'}}")).unwrap();
    assert_eq!(draw(&ui).text[1].0, "˅");
    assert_eq!(draw(&ui).text[1].1, 28.0);
    ui.pointer_move(20.0, 20.0);
    assert_eq!(draw(&ui).text[1].0, "˄");
    for css in ["select{content:'x'}", "select::picker-icon{content:'abc'}"] {
        assert!(Ui::from_html_css("<select></select>", &format!("{GEOMETRY}{css}")).is_err());
    }
}

#[test]
fn vector_picker_has_two_separate_chevrons_without_font_glyphs() {
    let css = format!(
        "{GEOMETRY}select{{background:white;color:black}}select::picker-icon{{width:18px;height:22px;background:blue;color:white;border:none;-native-ui-icon:chevron-up-down;-native-ui-icon-size:8px;-native-ui-icon-stroke:1.5px}}select:hover::picker-icon{{background:red}}select:disabled::picker-icon{{background:gray;color:silver}}"
    );
    let mut ui =
        Ui::from_html_css("<select id=test><option>Choice</option></select>", &css).unwrap();
    let bounds = ui.bounds("test");
    let r = draw(&ui);
    assert_eq!(r.text.len(), 1, "only the selected label uses the font");
    assert_eq!(r.text[0].0, "Choice");
    assert_eq!(r.at(110.0, 20.0), Some(Color(0, 0, 255, 255)));
    assert_eq!(
        r.at(110.0, 17.0),
        Some(Color(255, 255, 255, 255)),
        "indicator is inset vertically"
    );
    let strokes: Vec<_> = r
        .boxes
        .iter()
        .filter(|(rect, c)| rect.w <= 1.0 && c.0 == 255 && c.1 == 255 && c.2 == 255)
        .collect();
    assert!(strokes.iter().any(|(rect, _)| rect.y < 29.0));
    assert!(strokes.iter().any(|(rect, _)| rect.y >= 31.0));
    assert!(
        strokes
            .iter()
            .all(|(rect, _)| rect.y < 29.0 || rect.y >= 31.0),
        "chevrons have a clear gap"
    );
    assert!(
        strokes.iter().any(|(_, c)| c.3 > 0 && c.3 < 255),
        "stroke edges have partial coverage"
    );
    assert!(strokes.iter().all(|(rect, _)| rect.x >= 109.0
        && rect.x + rect.w <= 127.0
        && rect.y >= 19.0
        && rect.y + rect.h <= 41.0));
    ui.pointer_move(20.0, 20.0);
    assert_eq!(draw(&ui).at(110.0, 20.0), Some(Color(255, 0, 0, 255)));
    assert_eq!(ui.bounds("test"), bounds);
    let disabled =
        Ui::from_html_css("<select disabled><option>Choice</option></select>", &css).unwrap();
    assert_eq!(
        draw(&disabled).at(110.0, 20.0),
        Some(Color(128, 128, 128, 255))
    );
    let single = Ui::from_html_css(
        "<select><option>Choice</option></select>",
        &css.replace("chevron-up-down", "chevron-down"),
    )
    .unwrap();
    let r = draw(&single);
    assert_eq!(r.text.len(), 1);
    assert!(
        r.boxes
            .iter()
            .filter(|(rect, c)| rect.w <= 1.0 && c.0 == 255)
            .all(|(rect, _)| rect.y >= 27.0 && rect.y < 33.0)
    );
}

#[test]
fn invalid_vector_picker_properties_are_rejected() {
    for rule in [
        "select{-native-ui-icon:chevron-down}",
        "select::picker-icon{-native-ui-icon:macos}",
        "select::picker-icon{-native-ui-icon-size:0px}",
        "select::picker-icon{-native-ui-icon-stroke:65px}",
        "select::picker-icon{content:'v';-native-ui-icon:chevron-down}",
        "select::picker-icon{width:0px}",
        "select:hover::picker-icon{width:18px}",
        "select::picker-icon{left:1px}",
    ] {
        assert!(
            Ui::from_html_css("<select></select>", &format!("{GEOMETRY}{rule}")).is_err(),
            "{rule}"
        );
    }
}

#[test]
fn attribute_selectors_match_html_values_and_use_class_level_specificity() {
    let html = "<input id=MiXeD class=field type=TeXt readonly>";
    let css = format!(
        "{GEOMETRY}input{{background:white}}input[type]{{background:red}}input[type=text]{{background:blue}}input.field{{background:green}}input[type=text s]{{background:yellow}}input[type=TEXT i][readonly]{{color:navy}}input[id=mixed i]:focus{{background:purple}}"
    );
    let mut ui = Ui::from_html_css(html, &css).unwrap();
    assert_eq!(
        draw(&ui).at(20.0, 20.0),
        Some(Color(0, 128, 0, 255)),
        "attribute and class selectors have equal specificity"
    );
    ui.key_down(Key::Tab, false);
    assert_eq!(draw(&ui).at(20.0, 20.0), Some(Color(128, 0, 128, 255)));
    let css = format!(
        "{GEOMETRY}input{{background:white}}input:not([type=checkbox]){{background:red}}input.field{{background:blue}}input:not([type]){{color:green}}input[id=mixed]{{background:yellow}}"
    );
    let ui = Ui::from_html_css("<input id=MiXeD class=field value=Text>", &css).unwrap();
    let r = draw(&ui);
    assert_eq!(
        r.at(20.0, 20.0),
        Some(Color(0, 0, 255, 255)),
        "negation takes its attribute's specificity; id values are case-sensitive"
    );
    assert_eq!(r.text[0].2, Color(0, 128, 0, 255));
    let ui = Ui::from_html_css("<input type=CHECKBOX>", &css).unwrap();
    assert_eq!(
        draw(&ui).at(20.0, 20.0),
        Some(Color(255, 255, 255, 255)),
        "text rules exclude checkboxes regardless of type casing"
    );
    for selector in [
        "input[type^=text]",
        "input[type=text z]",
        "input[type=]",
        "input:not()",
        "input:not(button)",
        "input:not([type], [id])",
    ] {
        assert!(
            Ui::from_html_css("<input>", &format!("{GEOMETRY}{selector}{{color:red}}")).is_err(),
            "unsupported selector {selector} should be diagnosed"
        );
    }
}

#[test]
fn every_theme_styles_arbitrary_and_anonymous_controls_without_demo_layout() {
    let themes = [
        include_str!("../../../examples/showcase/ui/themes/windows-xp-luna.css"),
        include_str!("../../../examples/showcase/ui/themes/macos-light.css"),
        include_str!("../../../examples/showcase/ui/themes/windows-11-fluent.css"),
    ];
    let cases = [
        (
            "<input id=name type=text value=Text>",
            "<input id=nickname type=text value=Text>",
            "<input value=Text>",
        ),
        (
            "<input id=name type=text disabled value=Text>",
            "<input id=address type=text disabled value=Text>",
            "<input disabled value=Text>",
        ),
        (
            "<input id=name type=text readonly value=Text>",
            "<input id=reference type=text readonly value=Text>",
            "<input readonly value=Text>",
        ),
        (
            "<input id=counting type=checkbox>",
            "<input id=newsletter type=CHECKBOX>",
            "<input type=checkbox>",
        ),
        (
            "<input id=counting type=checkbox checked>",
            "<input id=consent type=checkbox checked>",
            "<input type=checkbox checked>",
        ),
        (
            "<input id=counting type=checkbox checked disabled>",
            "<input id=archived type=checkbox checked disabled>",
            "<input type=checkbox checked disabled>",
        ),
        (
            "<select id=step><option>One</option><option>Two</option></select>",
            "<select id=country><option>One</option><option>Two</option></select>",
            "<select><option>One</option><option>Two</option></select>",
        ),
        (
            "<button id=demo class=primary>Save</button>",
            "<button id=submit class=primary>Save</button>",
            "<button class=primary>Save</button>",
        ),
        (
            "<button id=reset>Cancel</button>",
            "<button id=cancel>Cancel</button>",
            "<button>Cancel</button>",
        ),
        (
            "<label id=name-label>Label</label>",
            "<label id=heading>Label</label>",
            "<label>Label</label>",
        ),
    ];
    for (theme_index, theme) in themes.iter().enumerate() {
        for rule in theme.split('}') {
            if let Some((selector, _)) = rule.split_once('{') {
                assert!(
                    !selector.contains('#'),
                    "themes must not target app-specific IDs"
                );
            }
        }
        assert!(!theme.contains("left:") && !theme.contains("top:"));
        // A new application supplies placement and field widths only; all appearance
        // and control heights come from the reusable theme.
        let css = format!(
            "{theme}\ninput,select,button,label{{position:absolute;left:30px;top:20px;width:180px}}"
        );
        for (original, renamed, anonymous) in cases {
            let mut controls: Vec<_> = [original, renamed, anonymous]
                .iter()
                .map(|html| Ui::from_html_css(html, &css).unwrap())
                .collect();
            for state in 0..5 {
                for ui in &mut controls {
                    match state {
                        1 => ui.pointer_move(35.0, 25.0),
                        2 => ui.key_down(Key::Tab, false),
                        3 => ui.pointer_down(35.0, 25.0),
                        4 => ui.pointer_up(35.0, 25.0),
                        _ => {}
                    }
                }
                let expected = draw(&controls[0]);
                for ui in &controls[1..] {
                    let actual = draw(ui);
                    assert_eq!(
                        actual.boxes, expected.boxes,
                        "theme {theme_index}, state {state}, {original}"
                    );
                    assert_eq!(
                        actual.text, expected.text,
                        "theme {theme_index}, state {state}, {original}"
                    );
                }
            }
        }
        let ui = Ui::from_html_css(
            "<input id=nickname>",
            &format!("{css}#nickname{{height:52px;width:280px;background:yellow}} "),
        )
        .unwrap();
        assert_eq!(ui.bounds("nickname").unwrap().h, 52.0);
        assert_eq!(ui.bounds("nickname").unwrap().w, 280.0);
        assert_eq!(
            draw(&ui).at(50.0, 40.0),
            Some(Color(255, 255, 0, 255)),
            "application overrides remain effective"
        );
    }
}
