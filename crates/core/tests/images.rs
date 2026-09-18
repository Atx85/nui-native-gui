use native_ui::{ImageDraw, Key, PaintCommand, Rect, Renderer, Ui};

#[derive(Default)]
struct Recorder {
    size: Option<(u32, u32)>,
    images: Vec<(String, Rect, Rect, Rect)>,
    texts: Vec<String>,
    fail: bool,
}
impl Renderer for Recorder {
    type Error = &'static str;
    fn measure_text(&mut self, _: &str, _: f32) -> Result<(f32, f32), Self::Error> {
        Ok((10.0, 10.0))
    }
    fn draw(&mut self, command: PaintCommand<'_>) -> Result<(), Self::Error> {
        if let PaintCommand::Text { text, .. } = command {
            self.texts.push(text.into());
        }
        Ok(())
    }
    fn image_size(&mut self, _: &str) -> Option<(u32, u32)> {
        self.size
    }
    fn draw_image(&mut self, image: ImageDraw<'_>) -> Result<(), Self::Error> {
        self.images.push((
            image.src.into(),
            image.source,
            image.destination,
            image.clip,
        ));
        if self.fail {
            Err("image failed")
        } else {
            Ok(())
        }
    }
}
fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect { x, y, w, h }
}
fn ui(extra: &str) -> Ui {
    Ui::from_html_css("<img id=preview src='asset:landscape' alt='Landscape preview'>", &format!("img{{position:absolute;left:10px;top:20px;width:110px;height:70px;border:2px solid red;padding:3px;{extra}}}")).unwrap()
}
#[test]
fn fitting_preserves_the_content_box_and_crops_centered_in_source_pixels() {
    for (fit, source, destination) in [
        (
            "fill",
            rect(0.0, 0.0, 200.0, 100.0),
            rect(15.0, 25.0, 100.0, 60.0),
        ),
        (
            "contain",
            rect(0.0, 0.0, 200.0, 100.0),
            rect(15.0, 30.0, 100.0, 50.0),
        ),
        (
            "cover",
            rect(50.0 / 3.0, 0.0, 500.0 / 3.0, 100.0),
            rect(15.0, 25.0, 100.0, 60.0),
        ),
    ] {
        let ui = ui(&format!("object-fit:{fit}"));
        let mut renderer = Recorder {
            size: Some((200, 100)),
            ..Default::default()
        };
        ui.render(&mut renderer).unwrap();
        let (src, actual_source, actual_destination, clip) = &renderer.images[0];
        assert_eq!(src, "asset:landscape");
        for (a, b) in [
            (actual_source.x, source.x),
            (actual_source.y, source.y),
            (actual_source.w, source.w),
            (actual_source.h, source.h),
        ] {
            assert!((a - b).abs() < 0.001, "{fit}: {a} vs {b}");
        }
        assert_eq!(*actual_destination, destination);
        assert_eq!(*clip, rect(15.0, 25.0, 100.0, 60.0));
        assert!(renderer.texts.is_empty());
    }
}
#[test]
fn portrait_images_fit_and_crop_in_the_other_axis() {
    let mut renderer = Recorder {
        size: Some((50, 200)),
        ..Default::default()
    };
    ui("object-fit:contain").render(&mut renderer).unwrap();
    assert_eq!(renderer.images[0].2, rect(57.5, 25.0, 15.0, 60.0));
    renderer.images.clear();
    ui("object-fit:cover").render(&mut renderer).unwrap();
    assert_eq!(renderer.images[0].1, rect(0.0, 85.0, 50.0, 30.0));
}
#[test]
fn missing_assets_and_unsupported_backends_paint_alt_text() {
    for size in [None, Some((0, 20)), Some((20, 0))] {
        let mut renderer = Recorder {
            size,
            ..Default::default()
        };
        ui("").render(&mut renderer).unwrap();
        assert_eq!(renderer.texts, ["Landscape preview"]);
        assert!(renderer.images.is_empty());
    }
    let ui = Ui::from_html_css_with_viewport("<img>", "img{height:20px}", 100.0, 50.0).unwrap();
    let mut renderer = Recorder::default();
    ui.render(&mut renderer).unwrap();
    assert!(renderer.texts.is_empty());
    assert_eq!(ui.image_sources().collect::<Vec<_>>(), [""]);
}
#[test]
fn image_sources_are_generic_and_updates_invalidate_only_when_changed() {
    let mut ui = Ui::from_html_css_with_viewport(
        "<img src=a><img id=whatever src=a><button id=b>OK</button>",
        "img,button{height:30px}",
        200.0,
        120.0,
    )
    .unwrap();
    assert_eq!(ui.image_sources().collect::<Vec<_>>(), ["a", "a"]);
    let before = ui.paint_revision();
    ui.set_image_source("whatever", "a").unwrap();
    assert_eq!(before, ui.paint_revision());
    let bounds = ui.bounds("whatever");
    ui.set_image_source("whatever", "b").unwrap();
    assert_ne!(before, ui.paint_revision());
    assert_eq!(ui.image_source("whatever"), Some("b"));
    assert_eq!(ui.bounds("whatever"), bounds);
    assert!(ui.set_image_source("b", "asset").is_err());
    assert!(ui.set_image_source("missing", "asset").is_err());
    assert_eq!(ui.image_source("b"), None);
}
#[test]
fn images_take_part_in_relative_layout_without_asset_dimensions() {
    let mut ui = Ui::from_html_css_with_viewport(
        "<img id=preview src=test><label id=after>After</label>",
        "body{padding:10px}img{width:50%;height:40px;margin:5px 0 7px}label{height:20px}",
        200.0,
        150.0,
    )
    .unwrap();
    assert_eq!(ui.bounds("preview"), Some(rect(10.0, 15.0, 90.0, 40.0)));
    assert_eq!(ui.bounds("after").unwrap().y, 62.0);
    ui.set_viewport(400.0, 150.0).unwrap();
    assert_eq!(ui.bounds("preview").unwrap().w, 190.0);
}
#[test]
fn images_block_click_through_but_do_not_take_focus_or_activate() {
    let mut ui = Ui::from_html_css(
        "<button id=b>OK</button><img id=i src=test>",
        "button,img{position:absolute;left:0;top:0;width:100px;height:50px}",
    )
    .unwrap();
    ui.pointer_down(20.0, 20.0);
    ui.pointer_up(20.0, 20.0);
    assert!(!ui.clicked("b"));
    ui.key_down(Key::Enter, false);
    assert!(!ui.clicked("i") && !ui.clicked("b"));
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Enter, false);
    assert!(ui.clicked("b"));
    ui.end_frame();
    ui.key_down(Key::Tab, false);
    ui.key_down(Key::Enter, false);
    assert!(ui.clicked("b"));
    ui.end_frame();
    assert!(
        Ui::from_html_css_with_viewport(
            "<img id=i><label for=i>X</label>",
            "img,label{height:20px}",
            100.0,
            80.0
        )
        .is_err()
    );
}
#[test]
fn fit_cascades_including_hover_and_backend_errors_propagate() {
    let mut ui = Ui::from_html_css("<img class=preview src=test>","img{position:absolute;left:0;top:0;width:100px;height:100px;object-fit:fill}.preview{object-fit:contain}img:hover{object-fit:cover}").unwrap();
    let mut renderer = Recorder {
        size: Some((200, 100)),
        ..Default::default()
    };
    ui.render(&mut renderer).unwrap();
    assert_eq!(renderer.images[0].2, rect(0.0, 25.0, 100.0, 50.0));
    ui.pointer_move(50.0, 50.0);
    ui.render(&mut renderer).unwrap();
    assert_eq!(renderer.images[1].1, rect(50.0, 0.0, 100.0, 100.0));
    renderer.fail = true;
    assert_eq!(ui.render(&mut renderer), Err("image failed"));
}
#[test]
fn unsupported_image_features_are_rejected_explicitly() {
    for css in [
        "img{object-fit:scale-down}",
        "label{object-fit:cover}",
        "img{object-position:left}",
    ] {
        assert!(
            Ui::from_html_css_with_viewport("<img><label>Test</label>", css, 100.0, 80.0).is_err()
        );
    }
    for html in ["<img srcset='a 1x,b 2x'>", "<button><img src=a></button>"] {
        assert!(Ui::from_html_css_with_viewport(html, "", 100.0, 80.0).is_err());
    }
}
