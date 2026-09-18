use native_ui::{Button, Checkbox, SelectItem, Ui};
use native_ui_sdl3::SdlBackend;
use sdl3::{
    pixels::{Color, PixelFormat},
    render::FRect,
    surface::Surface,
};

#[test]
fn controls_catalogue_renders_at_top_and_scrolled_with_native_images_and_canvases() {
    let mut ui = Ui::from_html_css_with_viewport(
        include_str!("../../../examples/showcase/catalog/assets/controls.html"),
        &format!(
            "{}\n{}",
            include_str!("../../../examples/showcase/ui/themes/macos-light.css"),
            include_str!("../../../examples/showcase/catalog/assets/controls.css")
        ),
        1120.0,
        760.0,
    )
    .unwrap();
    ui.get("code-controls")
        .unwrap()
        .add_checkbox(Checkbox::new("codeOption"))
        .unwrap();
    ui.get("code-controls")
        .unwrap()
        .add_button(Button::new("codeToggle").label("Toggle option"))
        .unwrap();
    ui.set_select_options(
        "choice",
        &[
            SelectItem::new("report", "Report — café.txt"),
            SelectItem::new("notes", "Notes.txt"),
        ],
    )
    .unwrap();
    let mut canvas = Surface::new(1120, 760, PixelFormat::RGBA32)
        .unwrap()
        .into_canvas()
        .unwrap();
    let creator = canvas.texture_creator();
    let mut backend = SdlBackend::new(&creator).unwrap();
    let rgba: Vec<u8> = [[82, 202, 246, 255], [143, 202, 246, 255]]
        .repeat(4)
        .into_iter()
        .flatten()
        .collect();
    backend.register_image_rgba("tile", 4, 2, &rgba).unwrap();
    for (name, y) in [
        ("top", 0.0),
        ("middle", 1000.0),
        ("images", 1800.0),
        ("bottom", f32::MAX),
    ] {
        ui.set_scroll_offset("options", 0.0, y).unwrap();
        canvas.set_draw_color(Color::RGB(243, 243, 243));
        canvas.clear();
        let mut calls = 0;
        backend
            .render_cached_with_canvases(&ui, &mut canvas, |region, canvas| {
                calls += 1;
                canvas.set_draw_color(Color::RGB(82, 202, 246));
                canvas.fill_rect(FRect::new(
                    region.content.x + 20.0,
                    region.content.y + 20.0,
                    80.0,
                    80.0,
                ))?;
                Ok(())
            })
            .unwrap();
        assert!(calls >= 1);
        canvas.present();
        if let Ok(dir) = std::env::var("NATIVE_UI_CATALOG_SCREENSHOTS") {
            canvas
                .surface()
                .save_bmp(std::path::Path::new(&dir).join(format!("catalog-{name}.bmp")))
                .unwrap();
        }
    }
    assert_eq!(backend.cache_stats().image_uploads, 1);
    assert_eq!(backend.cache_stats().rebuilds, 4);
}
