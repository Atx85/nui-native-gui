/* Tests only: drive the same update function used by the interactive showcase. */
static void act(NuiUi *ui, const char *id) {
    ok(nui_end_frame(ui));
    click(ui, id);
    ok(update(ui));
}
static void check(NuiUi *ui) {
    require(strcmp(value(ui, "name"), "Ada") == 0);
    require(nui_checked(ui, "enabled") == 1);
    require(bounds(ui, "scene").h == 110);
    require(bounds(ui, "picture").w == 80);
    require(bounds(ui, "reset").y + bounds(ui, "reset").h < 640);

    click(ui, "name");
    ok(nui_key(ui, NUI_END, 1, 0, 0));
    ok(nui_text_input(ui, "!"));
    ok(update(ui));
    require(strcmp(value(ui, "echo"), "Ada!") == 0);
    require(nui_changed(ui, "name") == 1);
    require(nui_changed(ui, "name") == 1); // Reading does not consume it.
    ok(nui_end_frame(ui));
    require(nui_changed(ui, "name") == 0);
    require(strcmp(value(ui, "name"), "Ada!") == 0);
    ok(nui_set_value(ui, "name", "Host update"));
    require(nui_changed(ui, "name") == 0);

    act(ui, "enabled");
    require(nui_checked(ui, "enabled") == 0);
    require(strcmp(value(ui, "status"), "Sound off") == 0);
    act(ui, "svg");
    size_t count = 0;
    ok(nui_checked_value_count(ui, "formats", &count));
    require(count == 2 && nui_checkbox_group_changed(ui, "formats") == 1);
    require(strcmp(value(ui, "status"), "2 formats selected") == 0);

    ok(nui_end_frame(ui));
    click(ui, "volume");
    ok(nui_key(ui, NUI_END, 1, 0, 0));
    ok(update(ui));
    require(strcmp(value(ui, "volume"), "100") == 0);
    require(nui_changed(ui, "volume") == 1);
    ok(nui_set_value(ui, "volume", "200"));
    require(strcmp(value(ui, "volume"), "100") == 0); // Clamped by the library.

    act(ui, "options");
    ok(nui_select_option_count(ui, "mode", &count));
    require(count == 3 && nui_changed(ui, "mode") == 0);
    ok(nui_end_frame(ui));
    click(ui, "mode");
    ok(nui_key(ui, NUI_END, 1, 0, 0));
    ok(nui_key(ui, NUI_ENTER, 1, 0, 0));
    ok(update(ui));
    require(strcmp(value(ui, "mode"), "draft") == 0);
    require(nui_changed(ui, "mode") == 1);

    act(ui, "reset");
    require(strcmp(value(ui, "name"), "Ada") == 0);
    require(strcmp(value(ui, "echo"), "Ada") == 0);
    require(strcmp(value(ui, "volume"), "55") == 0);
    require(strcmp(value(ui, "mode"), "normal") == 0);
    require(nui_checked(ui, "enabled") == 1);
    ok(nui_checked_value_count(ui, "formats", &count));
    require(count == 1 && nui_checkbox_group_changed(ui, "formats") == 0);
    require(nui_changed(ui, "name") == 0 && nui_changed(ui, "volume") == 0);
    act(ui, "apply");
    require(strcmp(value(ui, "status"), "Quality: normal / volume: 55") == 0);
    require(nui_clicked(ui, "apply") == 1);
    ok(nui_end_frame(ui));
    require(nui_clicked(ui, "apply") == 0);

    act(ui, "swap");
    act(ui, "swap");
    act(ui, "scene");
    require(strcmp(value(ui, "status"), "Canvas pressed") == 0);
    act(ui, "bottom");
    NuiRect scroll;
    ok(nui_scroll_position(ui, "list", &scroll));
    require(scroll.y > 0 && scroll.y == scroll.h);
    act(ui, "top");
    ok(nui_scroll_position(ui, "list", &scroll));
    require(scroll.y == 0);
    float width = bounds(ui, "preview").w;
    ok(nui_set_viewport(ui, 1140, 640));
    require(bounds(ui, "preview").w > width);
}
