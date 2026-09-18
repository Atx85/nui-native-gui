use native_ui::{Key, Ui};
fn ui() -> Ui {
    Ui::from_html_css_with_viewport(
        "<input id=a type=checkbox name=formats value=png checked><label for=a>PNG</label><input id=b type=checkbox name=formats value=svg><input id=c type=checkbox name=formats value=locked disabled checked><input id=other type=checkbox name=other checked><input id=solo type=checkbox>",
        "input{width:20px;height:20px}label{height:20px;width:80px}",300.0,200.0,
    ).unwrap()
}
#[test]
fn groups_read_set_clear_preserve_order_and_distinguish_missing_groups() {
    let mut ui = ui();
    assert_eq!(ui.checked_values("formats"), Some(vec!["png", "locked"]));
    assert_eq!(ui.checked_values("other"), Some(vec!["on"]));
    assert_eq!(ui.checked_values("missing"), None);
    assert_eq!(ui.get_value("b"), Some("svg")); // value is independent of checked state.
    assert_eq!(ui.get_value("solo"), Some("on"));
    ui.set_checked_values("formats", &["svg", "png"]).unwrap();
    assert_eq!(ui.checked_values("formats"), Some(vec!["png", "svg"]));
    assert_eq!(ui.checked("c"), Some(false)); // Host can set disabled members.
    assert_eq!(ui.checked_values("other"), Some(vec!["on"]));
    assert_eq!(ui.checkbox_group_changed("formats"), Some(false));
    let revision = ui.paint_revision();
    ui.set_checked_values("formats", &["png", "svg", "svg"])
        .unwrap();
    assert_eq!(ui.paint_revision(), revision);
    assert!(
        ui.set_checked_values("formats", &["png", "unknown"])
            .is_err()
    );
    assert_eq!(ui.paint_revision(), revision);
    assert_eq!(ui.checked_values("formats"), Some(vec!["png", "svg"]));
    assert!(ui.set_checked_values("missing", &[]).is_err());
    ui.set_checked_values("formats", &[]).unwrap();
    assert_eq!(ui.checked_values("formats"), Some(vec![]));
    ui.set_checked("b", true).unwrap();
    assert_eq!(ui.checked_values("formats"), Some(vec!["svg"]));
    ui.set_viewport(400.0, 300.0).unwrap();
    assert_eq!(ui.checked_values("formats"), Some(vec!["svg"]));
}
#[test]
fn labels_keyboard_and_disabled_controls_keep_group_events_consistent() {
    let mut ui = ui();
    ui.pointer_down(10.0, 30.0);
    ui.pointer_up(10.0, 30.0); // associated label
    assert_eq!(ui.checked("a"), Some(false));
    assert_eq!(ui.checkbox_group_changed("formats"), Some(true));
    assert_eq!(ui.checkbox_group_changed("formats"), Some(true)); // non-consuming
    assert!(ui.changed("a"));
    ui.end_frame();
    assert_eq!(ui.checkbox_group_changed("formats"), Some(false));
    ui.key_down(Key::Space, false);
    ui.key_up(Key::Space);
    assert_eq!(ui.checked("a"), Some(true));
    assert_eq!(ui.checkbox_group_changed("formats"), Some(true));
    ui.end_frame();
    ui.pointer_down(10.0, 70.0);
    ui.pointer_up(10.0, 70.0); // disabled checkbox
    assert_eq!(ui.checked("c"), Some(true));
    assert_eq!(ui.checkbox_group_changed("formats"), Some(false));
    assert_eq!(ui.checkbox_group_changed("missing"), None);
}
#[test]
fn duplicate_empty_unicode_values_and_members_without_ids_are_supported() {
    let mut ui=Ui::from_html_css_with_viewport("<input type=checkbox name=g value=''><input type=checkbox name=g value=café><input type=checkbox name=g value=café>","input{width:20px;height:20px}",200.0,200.0).unwrap();
    ui.set_checked_values("g", &["café", ""]).unwrap();
    assert_eq!(ui.checked_values("g"), Some(vec!["", "café", "café"]));
    for html in [
        "<input name=g>",
        "<input type=range name=g>",
        "<input type=checkbox name=''>",
    ] {
        assert!(Ui::from_html_css_with_viewport(html, "", 200.0, 200.0).is_err());
    }
}
