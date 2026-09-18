// Experimental: compile and verify with your Jai toolchain before use.
#import "Basic";
#load "native_ui.jai";
// Set CHECK_ONLY for input/state checks, or SMOKE_TEST for eight hidden frames.
CHECK_ONLY :: false;
SMOKE_TEST :: false;
// Set to NUI_BACKEND_OPENGL to use the same examples with OpenGL.
BACKEND :: NUI_BACKEND_SDL;
set_value :: (ui:*NuiUi,id:string,value:string) {
    name:=to_c_string(id);defer free(name);text:=to_c_string(value);defer free(text);
    assert(nui_set_value(ui,name,text)==1);
}
value :: (ui:*NuiUi,id:string)->string {
    name:=to_c_string(id);defer free(name);count:=nui_get_value(ui,name,null,0);assert(count>0);
    data:=cast(*u8) alloc(count);assert(nui_get_value(ui,name,data,count)==count);
    return .{data=data,count=cast(s64)(count-1)}; // Caller frees the returned data.
}
clicked :: (ui:*NuiUi,id:string)->bool {name:=to_c_string(id);defer free(name);return nui_clicked(ui,name)==1;}
changed :: (ui:*NuiUi,id:string)->bool {name:=to_c_string(id);defer free(name);return nui_changed(ui,name)==1;}
checked :: (ui:*NuiUi,id:string)->bool {name:=to_c_string(id);defer free(name);return nui_checked(ui,name)==1;}
bounds :: (ui:*NuiUi,id:string)->NuiRect {name:=to_c_string(id);defer free(name);r:NuiRect;assert(nui_bounds(ui,name,*r)==1);return r;}
click :: (ui:*NuiUi,id:string) {r:=bounds(ui,id);assert(nui_pointer(ui,NUI_DOWN,r.x+r.w/2,r.y+r.h/2)==1);assert(nui_pointer(ui,NUI_UP,r.x+r.w/2,r.y+r.h/2)==1);}
text_input :: (ui:*NuiUi,text:string) {s:=to_c_string(text);defer free(s);assert(nui_text_input(ui,s)==1);}
expect_value :: (ui:*NuiUi,id:string,expected:string) {s:=value(ui,id);defer free(s.data);assert(s==expected);}
scroll_to :: (ui:*NuiUi,id:string,y:float32) {name:=to_c_string(id);defer free(name);assert(nui_scroll_to(ui,name,0,y)==1);}
// Use THEME_WINDOWS_11 or THEME_WINDOWS_XP to select another shared skin.
#load "themes.jai";
THEME :: THEME_MACOS;
BASE_CSS :: "";
