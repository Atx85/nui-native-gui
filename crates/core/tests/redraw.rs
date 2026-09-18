use native_ui::{Key, Ui};

fn ui() -> Ui {
    Ui::from_html_css(
        "<canvas id=game tabindex=0></canvas><input id=text value=hello><input id=check type=checkbox><select id=pick><option value=a>A</option><option value=b>B</option></select>",
        "canvas,input,select{position:absolute;left:0;top:0;width:100px;height:30px}#text{top:40px}#check{top:80px}#pick{top:120px}",
    ).unwrap()
}

#[test]
fn idle_frames_and_game_motion_leave_ui_cache_valid() {
    let mut ui = ui();
    ui.pointer_down(10.0, 10.0);
    let revision = ui.paint_revision();
    for x in 11..90 {
        ui.pointer_move(x as f32, 15.0);
        ui.key_down(Key::Left, false);
        ui.key_up(Key::Left);
        ui.end_frame();
        assert_eq!(ui.paint_revision(), revision);
        assert_eq!(
            ui.canvas_input("game").unwrap().pointer,
            Some((x as f32, 15.0))
        );
    }
    ui.pointer_move(10.0, 45.0);
    assert_ne!(
        ui.paint_revision(),
        revision,
        "crossing hover targets invalidates paint"
    );
}

#[test]
fn repeated_focus_loss_does_not_rebuild_an_already_idle_ui() {
    let mut ui = ui();
    ui.pointer_down(10.0, 10.0);
    let active = ui.paint_revision();
    ui.cancel_input();
    assert_ne!(ui.paint_revision(), active);
    assert!(ui.canvas_input("game").unwrap().cancelled);
    let idle = ui.paint_revision();
    ui.end_frame();
    for _ in 0..10 {
        ui.cancel_input();
        assert_eq!(ui.paint_revision(), idle);
        assert!(!ui.canvas_input("game").unwrap().cancelled);
    }
}

#[test]
fn values_selection_focus_and_explicit_invalidation_refresh_paint() {
    let mut ui = ui();
    let revision = ui.paint_revision();
    ui.set_value("text", "hello").unwrap();
    ui.set_value("pick", "a").unwrap();
    ui.set_checked("check", false).unwrap();
    ui.end_frame();
    assert_eq!(
        ui.paint_revision(),
        revision,
        "unchanged host values stay cached"
    );
    ui.set_checked("check", true).unwrap();
    assert_ne!(ui.paint_revision(), revision);
    let revision = ui.paint_revision();
    ui.set_value("text", "world").unwrap();
    assert_ne!(ui.paint_revision(), revision);
    ui.pointer_down(10.0, 45.0);
    ui.pointer_up(10.0, 45.0);
    let revision = ui.paint_revision();
    ui.key_down_with_shift(Key::Left, false, true);
    assert_ne!(
        ui.paint_revision(),
        revision,
        "selection is a paint change without an edit"
    );
    assert!(!ui.changed("text"));
    let revision = ui.paint_revision();
    ui.text_input("!");
    assert_ne!(ui.paint_revision(), revision);
    let revision = ui.paint_revision();
    ui.cancel_input();
    assert_ne!(ui.paint_revision(), revision);
    let revision = ui.paint_revision();
    ui.invalidate_paint();
    assert_ne!(ui.paint_revision(), revision);
}

#[test]
fn popup_highlighting_and_dismissal_invalidate_without_form_changes() {
    let mut ui = ui();
    ui.pointer_down(10.0, 125.0);
    ui.pointer_up(10.0, 125.0);
    let revision = ui.paint_revision();
    ui.key_down(Key::Down, false);
    assert_ne!(ui.paint_revision(), revision);
    assert!(!ui.changed("pick"));
    let revision = ui.paint_revision();
    ui.key_down(Key::Escape, false);
    assert_ne!(ui.paint_revision(), revision);
}

#[test]
fn reloads_and_layout_changes_have_distinct_tokens() {
    assert_ne!(ui().paint_revision(), ui().paint_revision());
    assert_ne!(
        Ui::default().paint_revision(),
        Ui::default().paint_revision()
    );
    let mut ui = Ui::from_html_css_with_viewport(
        "<button id=b>OK</button>",
        "button{width:100%;height:30px}",
        200.0,
        100.0,
    )
    .unwrap();
    let revision = ui.paint_revision();
    ui.set_viewport(200.0, 100.0).unwrap();
    assert_eq!(ui.paint_revision(), revision);
    ui.set_viewport(300.0, 100.0).unwrap();
    assert_ne!(ui.paint_revision(), revision);
    assert_eq!(ui.bounds("b").unwrap().w, 300.0);
}
