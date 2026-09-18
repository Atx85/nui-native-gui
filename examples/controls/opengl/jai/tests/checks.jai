// Experimental: execute with CHECK_ONLY=true after binding generation.
check :: (ui: *NuiUi) {
    click(ui, "name");
    text_input(ui, "!");
    update(ui);
    assert(changed(ui, "name"));
    assert(nui_end_frame(ui)==1);
    set_value(ui, "name", "Host update");
    assert(!changed(ui, "name"));
    click(ui, "options"); update(ui);
    set_value(ui, "mode", "draft"); expect_value(ui, "mode", "draft");
    assert(nui_end_frame(ui)==1);
    click(ui, "reset"); update(ui);
    expect_value(ui, "name", "Ada"); expect_value(ui, "volume", "55");
    assert(checked(ui, "enabled"));
}
