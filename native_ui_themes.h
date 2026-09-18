/* Generated from examples/showcase/ui/themes by tools/sync-example-themes.py. Do not edit. */
#ifndef NATIVE_UI_THEMES_H
#define NATIVE_UI_THEMES_H
/* Pass NUI_THEME_MACOS before layout CSS. Use NUI_THEME_WINDOWS_11 or
   NUI_THEME_WINDOWS_XP to switch skins; no renderer changes are needed. */
#define NUI_THEME_MACOS \
    "/* Reusable macOS light skin. Load before application layout/overrides. */\n" \
    "label {\n" \
    "    height: 20px;\n" \
    "}\n" \
    "\n" \
    "body, .panel {\n" \
    "    background: #f3f3f3;\n" \
    "}\n" \
    "\n" \
    "label, input, select, button {\n" \
    "    font-size: 13px;\n" \
    "    color: #262626;\n" \
    "}\n" \
    "\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"]) {\n" \
    "    height: 30px;\n" \
    "    padding: 6px 10px;\n" \
    "    background: #ffffff;\n" \
    "    border: 1px solid #c9c9c9;\n" \
    "    border-radius: 5px;\n" \
    "}\n" \
    "\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"]):hover {\n" \
    "    border-color: #b5b5b5;\n" \
    "}\n" \
    "\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"]):focus {\n" \
    "    border-color: #5399e8;\n" \
    "    outline: 3px solid #a5caf3;\n" \
    "    outline-offset: 0;\n" \
    "}\n" \
    "\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"])::selection {\n" \
    "    background: #b3d7ff;\n" \
    "    color: #1d1d1f;\n" \
    "}\n" \
    "\n" \
    "select {\n" \
    "    height: 26px;\n" \
    "    padding: 4px 8px;\n" \
    "    background: #ffffff;\n" \
    "    border: 1px solid #c9c9c9;\n" \
    "    border-radius: 5px;\n" \
    "}\n" \
    "\n" \
    "select:hover {\n" \
    "    border-color: #b5b5b5;\n" \
    "}\n" \
    "\n" \
    "select:focus {\n" \
    "    border-color: #5399e8;\n" \
    "    outline: 3px solid #a5caf3;\n" \
    "    outline-offset: 0;\n" \
    "}\n" \
    "\n" \
    "select::picker-icon {\n" \
    "    width: 18px;\n" \
    "    height: 22px;\n" \
    "    background: #087aff;\n" \
    "    border: none;\n" \
    "    border-radius: 4px;\n" \
    "    color: #ffffff;\n" \
    "    -native-ui-icon: chevron-up-down;\n" \
    "    -native-ui-icon-size: 8px;\n" \
    "    -native-ui-icon-stroke: 1.5px;\n" \
    "}\n" \
    "\n" \
    "select:hover::picker-icon {\n" \
    "    background: #0874ee;\n" \
    "}\n" \
    "\n" \
    "select:active::picker-icon {\n" \
    "    background: #005dce;\n" \
    "}\n" \
    "\n" \
    "select:active {\n" \
    "    background: #e8e8e8;\n" \
    "}\n" \
    "\n" \
    "option {\n" \
    "    background: #ffffff;\n" \
    "    color: #262626;\n" \
    "    border: none;\n" \
    "    font-size: 13px;\n" \
    "}\n" \
    "\n" \
    "option:hover {\n" \
    "    background: #086bdf;\n" \
    "    color: #ffffff;\n" \
    "}\n" \
    "\n" \
    "option:disabled {\n" \
    "    background: #ffffff;\n" \
    "    color: #a0a0a0;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"] {\n" \
    "    width: 16px;\n" \
    "    height: 16px;\n" \
    "    padding: 1px;\n" \
    "    background: #ffffff;\n" \
    "    border: 1px solid #bdbdbd;\n" \
    "    border-radius: 3px;\n" \
    "    color: #ffffff;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:hover {\n" \
    "    border-color: #929292;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:checked {\n" \
    "    background: #086bdf;\n" \
    "    border-color: #086bdf;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:checked:hover {\n" \
    "    background: #1378ec;\n" \
    "    border-color: #1378ec;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:focus {\n" \
    "    outline: 3px solid #a5caf3;\n" \
    "    outline-offset: 1px;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:active {\n" \
    "    background: #dedede;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:checked:active {\n" \
    "    background: #0057bc;\n" \
    "    border-color: #0057bc;\n" \
    "}\n" \
    "\n" \
    "button {\n" \
    "    height: 30px;\n" \
    "    padding: 6px 12px;\n" \
    "    border: 1px solid #cacaca;\n" \
    "    border-radius: 5px;\n" \
    "    background: #ffffff;\n" \
    "}\n" \
    "\n" \
    "button.primary {\n" \
    "    background: #086bdf;\n" \
    "    border-color: #086bdf;\n" \
    "    color: #ffffff;\n" \
    "}\n" \
    "\n" \
    "button:hover {\n" \
    "    background: #f9f9f9;\n" \
    "    border-color: #adadad;\n" \
    "}\n" \
    "\n" \
    "button.primary:hover {\n" \
    "    background: #1378ec;\n" \
    "    border-color: #1378ec;\n" \
    "}\n" \
    "\n" \
    "button:focus {\n" \
    "    outline: 3px solid #a5caf3;\n" \
    "    outline-offset: 1px;\n" \
    "}\n" \
    "\n" \
    "button:active {\n" \
    "    background: #dedede;\n" \
    "    border-color: #bdbdbd;\n" \
    "}\n" \
    "\n" \
    "button.primary:active {\n" \
    "    background: #0057bc;\n" \
    "    border-color: #0057bc;\n" \
    "}\n" \
    "\n" \
    "button:disabled, button.primary:disabled, input:not([type=\"checkbox\"]):not([type=\"range\"]):disabled, select:disabled, input[type=\"checkbox\"]:disabled {\n" \
    "    color: #a0a0a0;\n" \
    "    background: #e9e9e9;\n" \
    "    border-color: #d6d6d6;\n" \
    "    outline: none;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:checked:disabled {\n" \
    "    background: #c4c4c4;\n" \
    "    border-color: #c4c4c4;\n" \
    "    color: #ffffff;\n" \
    "}\n" \
    "\n" \
    "select:disabled::picker-icon {\n" \
    "    background: #dedede;\n" \
    "    color: #999999;\n" \
    "}\n" \
    "\n" \
    "/* Compact round slider handles and light inset scrollbar thumbs. */\n" \
    "input[type=\"range\"] { height: 26px; padding: 2px; border: none; background: transparent; }\n" \
    "input[type=\"range\"]:focus { outline: 2px solid #a5caf3; outline-offset: 1px; border-radius: 5px; }\n" \
    "input::slider-track { height: 4px; background: #c5c5c9; border-radius: 2px; }\n" \
    "input::slider-fill { background: #087aff; border-radius: 2px; }\n" \
    "input::slider-thumb { width: 18px; height: 18px; background: linear-gradient(to bottom,#ffffff,#f7f7f7); border: 1px solid #b9b9bd; border-radius: 9px; }\n" \
    "input:hover::slider-thumb { border-color: #8d8d92; }\n" \
    "input:active::slider-thumb { background: #eaeaea; }\n" \
    "input:disabled::slider-thumb { background: #ededed; border-color: #d0d0d0; }\n" \
    "input:disabled::slider-fill { background: #bbbbbf; }\n" \
    "::scrollbar { width: 12px; height: 12px; background: #fafafa; }\n" \
    "::scrollbar-thumb { width: 26px; height: 26px; background: #b0b0b4; border: 3px solid #fafafa; border-radius: 6px; }\n" \
    "::scrollbar-thumb:hover { background: #8f8f94; }\n" \
    "::scrollbar-thumb:active { background: #727278; }\n" \
    "::scrollbar-corner { background: #fafafa; }\n"

#define NUI_THEME_WINDOWS_11 \
    "/* Reusable Windows 11 Fluent skin. Load before application layout/overrides. */\n" \
    "label {\n" \
    "    height: 22px;\n" \
    "}\n" \
    "\n" \
    "body, .panel {\n" \
    "    background: #f3f3f3;\n" \
    "}\n" \
    "\n" \
    "label, input, select, button {\n" \
    "    font-size: 14px;\n" \
    "    color: #1a1a1a;\n" \
    "}\n" \
    "\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"]) {\n" \
    "    height: 36px;\n" \
    "    padding: 8px 12px;\n" \
    "    background: #fcfcfc;\n" \
    "    border: 1px solid #e0e0e0;\n" \
    "    border-bottom-color: #858585;\n" \
    "    border-radius: 4px;\n" \
    "}\n" \
    "\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"]):hover {\n" \
    "    background: #ffffff;\n" \
    "}\n" \
    "\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"]):focus {\n" \
    "    background: #ffffff;\n" \
    "    border-bottom-color: #005fb8;\n" \
    "    box-shadow: inset 0 -1px 0 0 #005fb8;\n" \
    "}\n" \
    "\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"])::selection {\n" \
    "    background: #005fb8;\n" \
    "    color: #ffffff;\n" \
    "}\n" \
    "\n" \
    "select {\n" \
    "    height: 36px;\n" \
    "    padding: 8px 12px;\n" \
    "    background: #fbfbfb;\n" \
    "    border: 1px solid #e0e0e0;\n" \
    "    border-bottom-color: #c4c4c4;\n" \
    "    border-radius: 4px;\n" \
    "}\n" \
    "\n" \
    "select:hover {\n" \
    "    background: #f7f7f7;\n" \
    "}\n" \
    "\n" \
    "select:active {\n" \
    "    background: #f0f0f0;\n" \
    "    color: #606060;\n" \
    "}\n" \
    "\n" \
    "select:focus {\n" \
    "    outline: 1px solid #1a1a1a;\n" \
    "    outline-offset: -3px;\n" \
    "}\n" \
    "\n" \
    "select::picker-icon {\n" \
    "    -native-ui-icon: chevron-down;\n" \
    "    -native-ui-icon-size: 10px;\n" \
    "    -native-ui-icon-stroke: 1px;\n" \
    "    background: transparent;\n" \
    "    border: none;\n" \
    "    color: #494949;\n" \
    "}\n" \
    "\n" \
    "option {\n" \
    "    background: #fcfcfc;\n" \
    "    color: #1a1a1a;\n" \
    "    border: none;\n" \
    "    font-size: 14px;\n" \
    "}\n" \
    "\n" \
    "option:hover {\n" \
    "    background: #e9e9e9;\n" \
    "}\n" \
    "\n" \
    "option:checked {\n" \
    "    background: #e3edf7;\n" \
    "    color: #003e79;\n" \
    "}\n" \
    "\n" \
    "option:checked:hover {\n" \
    "    background: #d7e7f7;\n" \
    "}\n" \
    "\n" \
    "option:disabled {\n" \
    "    background: #fcfcfc;\n" \
    "    color: #a0a0a0;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"] {\n" \
    "    width: 20px;\n" \
    "    height: 20px;\n" \
    "    padding: 2px;\n" \
    "    background: #fcfcfc;\n" \
    "    border: 1px solid #858585;\n" \
    "    border-radius: 3px;\n" \
    "    color: #ffffff;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:hover {\n" \
    "    background: #e9e9e9;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:checked {\n" \
    "    background: #005fb8;\n" \
    "    border-color: #005fb8;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:checked:hover {\n" \
    "    background: #196fc0;\n" \
    "    border-color: #196fc0;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:active {\n" \
    "    background: #dedede;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:checked:active {\n" \
    "    background: #367cc2;\n" \
    "    border-color: #367cc2;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:focus {\n" \
    "    outline: 1px solid #1a1a1a;\n" \
    "    outline-offset: 3px;\n" \
    "}\n" \
    "\n" \
    "button {\n" \
    "    height: 32px;\n" \
    "    padding: 7px 12px;\n" \
    "    background: #fbfbfb;\n" \
    "    border: 1px solid #e0e0e0;\n" \
    "    border-bottom-color: #c4c4c4;\n" \
    "    border-radius: 4px;\n" \
    "}\n" \
    "\n" \
    "button.primary {\n" \
    "    background: #005fb8;\n" \
    "    border-color: #0058aa;\n" \
    "    border-bottom-color: #004f99;\n" \
    "    color: #ffffff;\n" \
    "}\n" \
    "\n" \
    "button:hover {\n" \
    "    background: #f7f7f7;\n" \
    "}\n" \
    "\n" \
    "button.primary:hover {\n" \
    "    background: #196fc0;\n" \
    "}\n" \
    "\n" \
    "button:active {\n" \
    "    background: #f0f0f0;\n" \
    "    color: #606060;\n" \
    "    border-bottom-color: #e0e0e0;\n" \
    "}\n" \
    "\n" \
    "button.primary:active {\n" \
    "    background: #367cc2;\n" \
    "    color: #e4eff9;\n" \
    "    border-bottom-color: #367cc2;\n" \
    "}\n" \
    "\n" \
    "button:focus {\n" \
    "    outline: 1px solid #1a1a1a;\n" \
    "    outline-offset: -3px;\n" \
    "}\n" \
    "\n" \
    "button.primary:focus {\n" \
    "    outline-color: #ffffff;\n" \
    "}\n" \
    "\n" \
    "button:disabled, button.primary:disabled, input:not([type=\"checkbox\"]):not([type=\"range\"]):disabled, select:disabled, input[type=\"checkbox\"]:disabled {\n" \
    "    background: #f0f0f0;\n" \
    "    color: #a0a0a0;\n" \
    "    border-color: #dedede;\n" \
    "    outline: none;\n" \
    "    box-shadow: none;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:checked:disabled {\n" \
    "    background: #bcbcbc;\n" \
    "    border-color: #bcbcbc;\n" \
    "    color: #eeeeee;\n" \
    "}\n" \
    "\n" \
    "select:disabled::picker-icon {\n" \
    "    color: #bcbcbc;\n" \
    "}\n" \
    "\n" \
    "/* Filled track and white-ring thumb, with narrow rounded scrollbars. */\n" \
    "input[type=\"range\"] { height: 30px; padding: 2px; border: none; background: transparent; }\n" \
    "input[type=\"range\"]:focus { outline: 2px solid #1a1a1a; outline-offset: 1px; border-radius: 4px; }\n" \
    "input::slider-track { height: 4px; background: #8a8a8a; border-radius: 2px; }\n" \
    "input::slider-fill { background: #0067c0; border-radius: 2px; }\n" \
    "input::slider-thumb { width: 22px; height: 22px; background: #0067c0; border: 5px solid #ffffff; border-radius: 11px; outline: 1px solid #d2d2d2; outline-offset: 0; }\n" \
    "input:hover::slider-thumb { background: #1975c5; border-color: #f7f7f7; }\n" \
    "input:active::slider-thumb { background: #00599f; }\n" \
    "input:disabled::slider-thumb { background: #b6b6b6; border-color: #f3f3f3; }\n" \
    "input:disabled::slider-fill { background: #b6b6b6; }\n" \
    "::scrollbar { width: 14px; height: 14px; background: #f9f9f9; }\n" \
    "::scrollbar-thumb { width: 24px; height: 24px; background: #8a8a8a; border: 4px solid #f9f9f9; border-radius: 7px; }\n" \
    "::scrollbar-thumb:hover { background: #626262; }\n" \
    "::scrollbar-thumb:active { background: #444444; }\n" \
    "::scrollbar-corner { background: #f9f9f9; }\n"

#define NUI_THEME_WINDOWS_XP \
    "/* Reusable Windows XP Luna skin. Load before application layout/overrides. */\n" \
    "label {\n" \
    "    height: 26px;\n" \
    "}\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"]), select {\n" \
    "    height: 44px;\n" \
    "    padding: 10px 12px;\n" \
    "}\n" \
    "input[type=\"checkbox\"] {\n" \
    "    width: 26px;\n" \
    "    height: 26px;\n" \
    "    padding: 3px;\n" \
    "}\n" \
    "button {\n" \
    "    height: 56px;\n" \
    "    padding: 12px 20px;\n" \
    "}\n" \
    "\n" \
    "body, .panel {\n" \
    "    background-color: #ece9d8;\n" \
    "}\n" \
    "\n" \
    "button {\n" \
    "    font-size: 13px;\n" \
    "    color: #000000;\n" \
    "    background: linear-gradient(to bottom, #ffffff 0%, #f5f4ea 45%, #e6e3d5 100%);\n" \
    "    border: 1px solid #003c74;\n" \
    "    border-radius: 3px;\n" \
    "    box-shadow: inset 0 0 0 1px #ffffff;\n" \
    "}\n" \
    "\n" \
    "button:hover {\n" \
    "    background: linear-gradient(to bottom, #fffef9, #f5f2df);\n" \
    "    box-shadow: inset 0 0 0 2px #f6c65a;\n" \
    "}\n" \
    "\n" \
    "button:focus {\n" \
    "    box-shadow: inset 0 0 0 2px #a5c5ed;\n" \
    "    outline: 1px dotted #383838;\n" \
    "    outline-offset: -5px;\n" \
    "}\n" \
    "\n" \
    "button:focus:hover {\n" \
    "    box-shadow: inset 0 0 0 2px #f6c65a;\n" \
    "}\n" \
    "\n" \
    "button:active, button:focus:hover:active {\n" \
    "    background: linear-gradient(to bottom, #d7d3c5, #ece9d8);\n" \
    "    box-shadow: inset 0 0 0 1px #b9b5a7;\n" \
    "}\n" \
    "\n" \
    "button:disabled {\n" \
    "    color: #a1a192;\n" \
    "    border-color: #c9c7ba;\n" \
    "    background: #f5f4ea;\n" \
    "    box-shadow: none;\n" \
    "    outline: none;\n" \
    "}\n" \
    "\n" \
    "label, input, select {\n" \
    "    font-size: 13px;\n" \
    "    color: #000000;\n" \
    "}\n" \
    "\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"]) {\n" \
    "    background: #ffffff;\n" \
    "    border: 1px solid #7f9db9;\n" \
    "    box-shadow: inset 0 0 0 1px #eef1f5;\n" \
    "}\n" \
    "\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"]):hover {\n" \
    "    border-color: #4f83b3;\n" \
    "}\n" \
    "\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"]):focus {\n" \
    "    border-color: #316ac5;\n" \
    "}\n" \
    "\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"])::selection {\n" \
    "    background: #316ac5;\n" \
    "    color: #ffffff;\n" \
    "}\n" \
    "\n" \
    "select {\n" \
    "    background: #ffffff;\n" \
    "    border: 1px solid #7f9db9;\n" \
    "}\n" \
    "\n" \
    "select:hover, select:focus {\n" \
    "    border-color: #316ac5;\n" \
    "}\n" \
    "\n" \
    "select::picker-icon {\n" \
    "    color: #4a6384;\n" \
    "    background: linear-gradient(to bottom, #e8f1ff, #b8d0f3);\n" \
    "    border: 1px solid #7f9db9;\n" \
    "    border-radius: 2px;\n" \
    "    box-shadow: inset 0 0 0 1px #f5f9ff;\n" \
    "}\n" \
    "\n" \
    "select:hover::picker-icon {\n" \
    "    background: linear-gradient(to bottom, #dceaff, #a4c4ef);\n" \
    "    border-color: #316ac5;\n" \
    "}\n" \
    "\n" \
    "select:active::picker-icon {\n" \
    "    background: linear-gradient(to bottom, #98b7df, #cfdef4);\n" \
    "    box-shadow: inset 0 0 0 1px #8daace;\n" \
    "}\n" \
    "\n" \
    "option {\n" \
    "    background: #ffffff;\n" \
    "    color: #000000;\n" \
    "    border: none;\n" \
    "    font-size: 13px;\n" \
    "}\n" \
    "\n" \
    "option:hover {\n" \
    "    background: #316ac5;\n" \
    "    color: #ffffff;\n" \
    "}\n" \
    "\n" \
    "option:disabled {\n" \
    "    background: #ffffff;\n" \
    "    color: #aca899;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"] {\n" \
    "    color: #21a121;\n" \
    "    background: linear-gradient(to bottom, #dcdad0, #ffffff 65%);\n" \
    "    border: 1px solid #1c5180;\n" \
    "    box-shadow: inset 0 0 0 1px #f5f4eb;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:hover {\n" \
    "    box-shadow: inset 0 0 0 2px #ffcf73;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:focus {\n" \
    "    outline: 1px dotted #383838;\n" \
    "    outline-offset: 2px;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:active {\n" \
    "    background: #e0dfd7;\n" \
    "}\n" \
    "\n" \
    "input[type=\"checkbox\"]:disabled {\n" \
    "    border-color: #c9c7ba;\n" \
    "    color: #aca899;\n" \
    "    background: #f5f4ea;\n" \
    "    outline: none;\n" \
    "}\n" \
    "\n" \
    "input:not([type=\"checkbox\"]):not([type=\"range\"]):disabled, select:disabled {\n" \
    "    background: #ece9d8;\n" \
    "    color: #aca899;\n" \
    "    border-color: #c9c7ba;\n" \
    "}\n" \
    "\n" \
    "select:disabled::picker-icon {\n" \
    "    background: #ece9d8;\n" \
    "    color: #aca899;\n" \
    "    border-color: #c9c7ba;\n" \
    "    box-shadow: none;\n" \
    "}\n" \
    "\n" \
    "/* Range and scroll parts: dimensions and decoration belong to the skin. */\n" \
    "input[type=\"range\"] { height: 28px; padding: 2px; border: none; background: transparent; }\n" \
    "input[type=\"range\"]:focus { outline: 1px dotted #383838; outline-offset: 1px; }\n" \
    "input::slider-track { height: 5px; background: #e4e1d5; border: 1px solid #9d9c92; border-radius: 0; }\n" \
    "input::slider-fill { background: #b8cbdc; border-radius: 0; }\n" \
    "input::slider-thumb { width: 12px; height: 22px; background: linear-gradient(to right,#ffffff,#d5e3f4); border: 1px solid #567caa; border-radius: 2px; box-shadow: inset 0 0 0 1px #ffffff; }\n" \
    "input:hover::slider-thumb { background: #f8edc3; border-color: #bc8d32; }\n" \
    "input:active::slider-thumb { background: #d5dfef; }\n" \
    "input:disabled::slider-thumb { background: #e2dfd5; border-color: #aaa89c; }\n" \
    "::scrollbar { width: 17px; height: 17px; background: #edead8; }\n" \
    "::scrollbar-track { background: linear-gradient(to right,#f4f2e8,#e8e5d4); border: 1px solid #dedbc9; }\n" \
    "::scrollbar-thumb { width: 24px; height: 24px; background: linear-gradient(to right,#d6e5f9,#a8c0e2); border: 1px solid #6e91bf; border-radius: 2px; box-shadow: inset 0 0 0 1px #edf4ff; }\n" \
    "::scrollbar-thumb:hover { background: #c4dcfb; }\n" \
    "::scrollbar-thumb:active { background: #9db9df; }\n" \
    "::scrollbar-corner { background: #ece9d8; }\n"

#endif
