use native_ui::{SelectItem, Ui};
#[test]
fn shared_catalog_compiles_in_all_themes_and_exposes_all_variants() {
    for theme in [
        include_str!("../../../examples/showcase/ui/themes/macos-light.css"),
        include_str!("../../../examples/showcase/ui/themes/windows-xp-luna.css"),
        include_str!("../../../examples/showcase/ui/themes/windows-11-fluent.css"),
    ] {
        let mut ui = Ui::from_html_css_with_viewport(
            include_str!("../../../examples/showcase/catalog/assets/controls.html"),
            &format!(
                "{theme}\n{}",
                include_str!("../../../examples/showcase/catalog/assets/controls.css")
            ),
            1120.0,
            760.0,
        )
        .unwrap();
        for id in [
            "format-png",
            "format-svg",
            "format-pdf",
            "format-avif",
            "formats-preset",
            "formats-clear",
            "entry",
            "placeholder",
            "readonly",
            "disabled-text",
            "enabled",
            "unchecked",
            "disabled-check",
            "disabled-checked",
            "size",
            "fraction",
            "continuous",
            "default-range",
            "fixed-range",
            "disabled-range",
            "choice",
            "static-select",
            "disabled-select",
            "empty-select",
            "all-disabled",
            "selected-disabled",
            "long-select",
            "apply",
            "reload",
            "clear",
            "disabled-button",
            "rich-label",
            "scene",
            "preview",
            "logo",
            "contain-image",
            "cover-image",
            "fill-image",
            "missing-image",
            "options",
            "visible-panel",
            "hidden-panel",
            "clip-panel",
            "auto-panel",
            "scroll-panel",
            "axis-panel",
        ] {
            assert!(ui.contains_id(id), "{id}");
        }
        assert_eq!(ui.checked_values("formats"), Some(vec!["png", "pdf"]));
        assert_eq!(ui.get_value("fixed-range"), Some("3"));
        assert_eq!(ui.get_value("default-range"), Some("50"));
        assert_eq!(ui.get_value("static-select"), Some("second"));
        assert_eq!(ui.get_value("all-disabled"), None);
        ui.set_select_options("choice", &[SelectItem::new("report", "Report")])
            .unwrap();
        assert_eq!(ui.get_value("choice"), Some("report"));
        assert!(ui.scroll_max("options").unwrap().1 > 500.0);
        ui.set_viewport(900.0, 640.0).unwrap();
        assert!(ui.canvas_region("scene").unwrap().content.w > 0.0);
    }
}
