use super::*;
#[test]
fn controls_events_and_application_updates() {
    let mut ui =
        Ui::from_html_css_with_viewport(HTML, &format!("{THEME}{CSS}"), 940., 640.).unwrap();
    let mut alternate = false;
    click(&mut ui, "name");
    ui.text_input("!");
    update(&mut ui, &mut alternate).unwrap();
    assert_eq!(ui.get_value("name").unwrap(), ui.get_value("echo").unwrap());
    assert!(ui.changed("name"));
    ui.end_frame();
    assert!(!ui.changed("name"));
    ui.set_value("name", "Host update").unwrap();
    assert!(!ui.changed("name"));
    click(&mut ui, "enabled");
    update(&mut ui, &mut alternate).unwrap();
    assert!(!ui.checked("enabled").unwrap());
    ui.end_frame();
    click(&mut ui, "svg");
    update(&mut ui, &mut alternate).unwrap();
    assert_eq!(ui.checked_values("formats").unwrap().len(), 2);
    ui.end_frame();
    click(&mut ui, "options");
    update(&mut ui, &mut alternate).unwrap();
    ui.set_value("mode", "draft").unwrap();
    assert_eq!(ui.get_value("mode").unwrap(), "draft");
    assert!(!ui.changed("mode"));
    ui.end_frame();
    click(&mut ui, "reset");
    update(&mut ui, &mut alternate).unwrap();
    assert_eq!(ui.get_value("name").unwrap(), "Ada");
    assert_eq!(ui.get_value("volume").unwrap(), "55");
    assert_eq!(ui.get_value("mode").unwrap(), "normal");
    assert!(ui.checked("enabled").unwrap());
    assert_eq!(ui.checked_values("formats").unwrap().len(), 1);
    assert!(!ui.changed("name"));
    ui.end_frame();
    click(&mut ui, "apply");
    update(&mut ui, &mut alternate).unwrap();
    assert_eq!(
        ui.get_value("status").unwrap(),
        "Quality: normal / volume: 55"
    );
    ui.end_frame();
    assert!(!ui.clicked("apply"));
    click(&mut ui, "swap");
    update(&mut ui, &mut alternate).unwrap();
    assert!(alternate);
    ui.end_frame();
    click(&mut ui, "scene");
    update(&mut ui, &mut alternate).unwrap();
    assert_eq!(ui.get_value("status").unwrap(), "Canvas pressed");
    ui.end_frame();
    click(&mut ui, "bottom");
    update(&mut ui, &mut alternate).unwrap();
    assert!(ui.scroll_offset("list").unwrap().1 > 0.);
    ui.end_frame();
    click(&mut ui, "top");
    update(&mut ui, &mut alternate).unwrap();
    assert_eq!(ui.scroll_offset("list").unwrap().1, 0.);
    let width = ui.bounds("preview").unwrap().w;
    ui.set_viewport(1140., 640.).unwrap();
    assert!(ui.bounds("preview").unwrap().w > width);
}

fn click(ui: &mut Ui, id: &str) {
    let r = ui.bounds(id).unwrap();
    ui.pointer_down(r.x + r.w / 2., r.y + r.h / 2.);
    ui.pointer_up(r.x + r.w / 2., r.y + r.h / 2.);
}
