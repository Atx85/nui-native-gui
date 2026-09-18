// Generated from examples/showcase/ui/themes by tools/sync-example-themes.py. Do not edit.
pub const macos =
    \\/* Reusable macOS light skin. Load before application layout/overrides. */
    \\label {
    \\    height: 20px;
    \\}
    \\
    \\body, .panel {
    \\    background: #f3f3f3;
    \\}
    \\
    \\label, input, select, button {
    \\    font-size: 13px;
    \\    color: #262626;
    \\}
    \\
    \\input:not([type="checkbox"]):not([type="range"]) {
    \\    height: 30px;
    \\    padding: 6px 10px;
    \\    background: #ffffff;
    \\    border: 1px solid #c9c9c9;
    \\    border-radius: 5px;
    \\}
    \\
    \\input:not([type="checkbox"]):not([type="range"]):hover {
    \\    border-color: #b5b5b5;
    \\}
    \\
    \\input:not([type="checkbox"]):not([type="range"]):focus {
    \\    border-color: #5399e8;
    \\    outline: 3px solid #a5caf3;
    \\    outline-offset: 0;
    \\}
    \\
    \\input:not([type="checkbox"]):not([type="range"])::selection {
    \\    background: #b3d7ff;
    \\    color: #1d1d1f;
    \\}
    \\
    \\select {
    \\    height: 26px;
    \\    padding: 4px 8px;
    \\    background: #ffffff;
    \\    border: 1px solid #c9c9c9;
    \\    border-radius: 5px;
    \\}
    \\
    \\select:hover {
    \\    border-color: #b5b5b5;
    \\}
    \\
    \\select:focus {
    \\    border-color: #5399e8;
    \\    outline: 3px solid #a5caf3;
    \\    outline-offset: 0;
    \\}
    \\
    \\select::picker-icon {
    \\    width: 18px;
    \\    height: 22px;
    \\    background: #087aff;
    \\    border: none;
    \\    border-radius: 4px;
    \\    color: #ffffff;
    \\    -native-ui-icon: chevron-up-down;
    \\    -native-ui-icon-size: 8px;
    \\    -native-ui-icon-stroke: 1.5px;
    \\}
    \\
    \\select:hover::picker-icon {
    \\    background: #0874ee;
    \\}
    \\
    \\select:active::picker-icon {
    \\    background: #005dce;
    \\}
    \\
    \\select:active {
    \\    background: #e8e8e8;
    \\}
    \\
    \\option {
    \\    background: #ffffff;
    \\    color: #262626;
    \\    border: none;
    \\    font-size: 13px;
    \\}
    \\
    \\option:hover {
    \\    background: #086bdf;
    \\    color: #ffffff;
    \\}
    \\
    \\option:disabled {
    \\    background: #ffffff;
    \\    color: #a0a0a0;
    \\}
    \\
    \\input[type="checkbox"] {
    \\    width: 16px;
    \\    height: 16px;
    \\    padding: 1px;
    \\    background: #ffffff;
    \\    border: 1px solid #bdbdbd;
    \\    border-radius: 3px;
    \\    color: #ffffff;
    \\}
    \\
    \\input[type="checkbox"]:hover {
    \\    border-color: #929292;
    \\}
    \\
    \\input[type="checkbox"]:checked {
    \\    background: #086bdf;
    \\    border-color: #086bdf;
    \\}
    \\
    \\input[type="checkbox"]:checked:hover {
    \\    background: #1378ec;
    \\    border-color: #1378ec;
    \\}
    \\
    \\input[type="checkbox"]:focus {
    \\    outline: 3px solid #a5caf3;
    \\    outline-offset: 1px;
    \\}
    \\
    \\input[type="checkbox"]:active {
    \\    background: #dedede;
    \\}
    \\
    \\input[type="checkbox"]:checked:active {
    \\    background: #0057bc;
    \\    border-color: #0057bc;
    \\}
    \\
    \\button {
    \\    height: 30px;
    \\    padding: 6px 12px;
    \\    border: 1px solid #cacaca;
    \\    border-radius: 5px;
    \\    background: #ffffff;
    \\}
    \\
    \\button.primary {
    \\    background: #086bdf;
    \\    border-color: #086bdf;
    \\    color: #ffffff;
    \\}
    \\
    \\button:hover {
    \\    background: #f9f9f9;
    \\    border-color: #adadad;
    \\}
    \\
    \\button.primary:hover {
    \\    background: #1378ec;
    \\    border-color: #1378ec;
    \\}
    \\
    \\button:focus {
    \\    outline: 3px solid #a5caf3;
    \\    outline-offset: 1px;
    \\}
    \\
    \\button:active {
    \\    background: #dedede;
    \\    border-color: #bdbdbd;
    \\}
    \\
    \\button.primary:active {
    \\    background: #0057bc;
    \\    border-color: #0057bc;
    \\}
    \\
    \\button:disabled, button.primary:disabled, input:not([type="checkbox"]):not([type="range"]):disabled, select:disabled, input[type="checkbox"]:disabled {
    \\    color: #a0a0a0;
    \\    background: #e9e9e9;
    \\    border-color: #d6d6d6;
    \\    outline: none;
    \\}
    \\
    \\input[type="checkbox"]:checked:disabled {
    \\    background: #c4c4c4;
    \\    border-color: #c4c4c4;
    \\    color: #ffffff;
    \\}
    \\
    \\select:disabled::picker-icon {
    \\    background: #dedede;
    \\    color: #999999;
    \\}
    \\
    \\/* Compact round slider handles and light inset scrollbar thumbs. */
    \\input[type="range"] { height: 26px; padding: 2px; border: none; background: transparent; }
    \\input[type="range"]:focus { outline: 2px solid #a5caf3; outline-offset: 1px; border-radius: 5px; }
    \\input::slider-track { height: 4px; background: #c5c5c9; border-radius: 2px; }
    \\input::slider-fill { background: #087aff; border-radius: 2px; }
    \\input::slider-thumb { width: 18px; height: 18px; background: linear-gradient(to bottom,#ffffff,#f7f7f7); border: 1px solid #b9b9bd; border-radius: 9px; }
    \\input:hover::slider-thumb { border-color: #8d8d92; }
    \\input:active::slider-thumb { background: #eaeaea; }
    \\input:disabled::slider-thumb { background: #ededed; border-color: #d0d0d0; }
    \\input:disabled::slider-fill { background: #bbbbbf; }
    \\::scrollbar { width: 12px; height: 12px; background: #fafafa; }
    \\::scrollbar-thumb { width: 26px; height: 26px; background: #b0b0b4; border: 3px solid #fafafa; border-radius: 6px; }
    \\::scrollbar-thumb:hover { background: #8f8f94; }
    \\::scrollbar-thumb:active { background: #727278; }
    \\::scrollbar-corner { background: #fafafa; }
;
pub const windows_11 =
    \\/* Reusable Windows 11 Fluent skin. Load before application layout/overrides. */
    \\label {
    \\    height: 22px;
    \\}
    \\
    \\body, .panel {
    \\    background: #f3f3f3;
    \\}
    \\
    \\label, input, select, button {
    \\    font-size: 14px;
    \\    color: #1a1a1a;
    \\}
    \\
    \\input:not([type="checkbox"]):not([type="range"]) {
    \\    height: 36px;
    \\    padding: 8px 12px;
    \\    background: #fcfcfc;
    \\    border: 1px solid #e0e0e0;
    \\    border-bottom-color: #858585;
    \\    border-radius: 4px;
    \\}
    \\
    \\input:not([type="checkbox"]):not([type="range"]):hover {
    \\    background: #ffffff;
    \\}
    \\
    \\input:not([type="checkbox"]):not([type="range"]):focus {
    \\    background: #ffffff;
    \\    border-bottom-color: #005fb8;
    \\    box-shadow: inset 0 -1px 0 0 #005fb8;
    \\}
    \\
    \\input:not([type="checkbox"]):not([type="range"])::selection {
    \\    background: #005fb8;
    \\    color: #ffffff;
    \\}
    \\
    \\select {
    \\    height: 36px;
    \\    padding: 8px 12px;
    \\    background: #fbfbfb;
    \\    border: 1px solid #e0e0e0;
    \\    border-bottom-color: #c4c4c4;
    \\    border-radius: 4px;
    \\}
    \\
    \\select:hover {
    \\    background: #f7f7f7;
    \\}
    \\
    \\select:active {
    \\    background: #f0f0f0;
    \\    color: #606060;
    \\}
    \\
    \\select:focus {
    \\    outline: 1px solid #1a1a1a;
    \\    outline-offset: -3px;
    \\}
    \\
    \\select::picker-icon {
    \\    -native-ui-icon: chevron-down;
    \\    -native-ui-icon-size: 10px;
    \\    -native-ui-icon-stroke: 1px;
    \\    background: transparent;
    \\    border: none;
    \\    color: #494949;
    \\}
    \\
    \\option {
    \\    background: #fcfcfc;
    \\    color: #1a1a1a;
    \\    border: none;
    \\    font-size: 14px;
    \\}
    \\
    \\option:hover {
    \\    background: #e9e9e9;
    \\}
    \\
    \\option:checked {
    \\    background: #e3edf7;
    \\    color: #003e79;
    \\}
    \\
    \\option:checked:hover {
    \\    background: #d7e7f7;
    \\}
    \\
    \\option:disabled {
    \\    background: #fcfcfc;
    \\    color: #a0a0a0;
    \\}
    \\
    \\input[type="checkbox"] {
    \\    width: 20px;
    \\    height: 20px;
    \\    padding: 2px;
    \\    background: #fcfcfc;
    \\    border: 1px solid #858585;
    \\    border-radius: 3px;
    \\    color: #ffffff;
    \\}
    \\
    \\input[type="checkbox"]:hover {
    \\    background: #e9e9e9;
    \\}
    \\
    \\input[type="checkbox"]:checked {
    \\    background: #005fb8;
    \\    border-color: #005fb8;
    \\}
    \\
    \\input[type="checkbox"]:checked:hover {
    \\    background: #196fc0;
    \\    border-color: #196fc0;
    \\}
    \\
    \\input[type="checkbox"]:active {
    \\    background: #dedede;
    \\}
    \\
    \\input[type="checkbox"]:checked:active {
    \\    background: #367cc2;
    \\    border-color: #367cc2;
    \\}
    \\
    \\input[type="checkbox"]:focus {
    \\    outline: 1px solid #1a1a1a;
    \\    outline-offset: 3px;
    \\}
    \\
    \\button {
    \\    height: 32px;
    \\    padding: 7px 12px;
    \\    background: #fbfbfb;
    \\    border: 1px solid #e0e0e0;
    \\    border-bottom-color: #c4c4c4;
    \\    border-radius: 4px;
    \\}
    \\
    \\button.primary {
    \\    background: #005fb8;
    \\    border-color: #0058aa;
    \\    border-bottom-color: #004f99;
    \\    color: #ffffff;
    \\}
    \\
    \\button:hover {
    \\    background: #f7f7f7;
    \\}
    \\
    \\button.primary:hover {
    \\    background: #196fc0;
    \\}
    \\
    \\button:active {
    \\    background: #f0f0f0;
    \\    color: #606060;
    \\    border-bottom-color: #e0e0e0;
    \\}
    \\
    \\button.primary:active {
    \\    background: #367cc2;
    \\    color: #e4eff9;
    \\    border-bottom-color: #367cc2;
    \\}
    \\
    \\button:focus {
    \\    outline: 1px solid #1a1a1a;
    \\    outline-offset: -3px;
    \\}
    \\
    \\button.primary:focus {
    \\    outline-color: #ffffff;
    \\}
    \\
    \\button:disabled, button.primary:disabled, input:not([type="checkbox"]):not([type="range"]):disabled, select:disabled, input[type="checkbox"]:disabled {
    \\    background: #f0f0f0;
    \\    color: #a0a0a0;
    \\    border-color: #dedede;
    \\    outline: none;
    \\    box-shadow: none;
    \\}
    \\
    \\input[type="checkbox"]:checked:disabled {
    \\    background: #bcbcbc;
    \\    border-color: #bcbcbc;
    \\    color: #eeeeee;
    \\}
    \\
    \\select:disabled::picker-icon {
    \\    color: #bcbcbc;
    \\}
    \\
    \\/* Filled track and white-ring thumb, with narrow rounded scrollbars. */
    \\input[type="range"] { height: 30px; padding: 2px; border: none; background: transparent; }
    \\input[type="range"]:focus { outline: 2px solid #1a1a1a; outline-offset: 1px; border-radius: 4px; }
    \\input::slider-track { height: 4px; background: #8a8a8a; border-radius: 2px; }
    \\input::slider-fill { background: #0067c0; border-radius: 2px; }
    \\input::slider-thumb { width: 22px; height: 22px; background: #0067c0; border: 5px solid #ffffff; border-radius: 11px; outline: 1px solid #d2d2d2; outline-offset: 0; }
    \\input:hover::slider-thumb { background: #1975c5; border-color: #f7f7f7; }
    \\input:active::slider-thumb { background: #00599f; }
    \\input:disabled::slider-thumb { background: #b6b6b6; border-color: #f3f3f3; }
    \\input:disabled::slider-fill { background: #b6b6b6; }
    \\::scrollbar { width: 14px; height: 14px; background: #f9f9f9; }
    \\::scrollbar-thumb { width: 24px; height: 24px; background: #8a8a8a; border: 4px solid #f9f9f9; border-radius: 7px; }
    \\::scrollbar-thumb:hover { background: #626262; }
    \\::scrollbar-thumb:active { background: #444444; }
    \\::scrollbar-corner { background: #f9f9f9; }
;
pub const windows_xp =
    \\/* Reusable Windows XP Luna skin. Load before application layout/overrides. */
    \\label {
    \\    height: 26px;
    \\}
    \\input:not([type="checkbox"]):not([type="range"]), select {
    \\    height: 44px;
    \\    padding: 10px 12px;
    \\}
    \\input[type="checkbox"] {
    \\    width: 26px;
    \\    height: 26px;
    \\    padding: 3px;
    \\}
    \\button {
    \\    height: 56px;
    \\    padding: 12px 20px;
    \\}
    \\
    \\body, .panel {
    \\    background-color: #ece9d8;
    \\}
    \\
    \\button {
    \\    font-size: 13px;
    \\    color: #000000;
    \\    background: linear-gradient(to bottom, #ffffff 0%, #f5f4ea 45%, #e6e3d5 100%);
    \\    border: 1px solid #003c74;
    \\    border-radius: 3px;
    \\    box-shadow: inset 0 0 0 1px #ffffff;
    \\}
    \\
    \\button:hover {
    \\    background: linear-gradient(to bottom, #fffef9, #f5f2df);
    \\    box-shadow: inset 0 0 0 2px #f6c65a;
    \\}
    \\
    \\button:focus {
    \\    box-shadow: inset 0 0 0 2px #a5c5ed;
    \\    outline: 1px dotted #383838;
    \\    outline-offset: -5px;
    \\}
    \\
    \\button:focus:hover {
    \\    box-shadow: inset 0 0 0 2px #f6c65a;
    \\}
    \\
    \\button:active, button:focus:hover:active {
    \\    background: linear-gradient(to bottom, #d7d3c5, #ece9d8);
    \\    box-shadow: inset 0 0 0 1px #b9b5a7;
    \\}
    \\
    \\button:disabled {
    \\    color: #a1a192;
    \\    border-color: #c9c7ba;
    \\    background: #f5f4ea;
    \\    box-shadow: none;
    \\    outline: none;
    \\}
    \\
    \\label, input, select {
    \\    font-size: 13px;
    \\    color: #000000;
    \\}
    \\
    \\input:not([type="checkbox"]):not([type="range"]) {
    \\    background: #ffffff;
    \\    border: 1px solid #7f9db9;
    \\    box-shadow: inset 0 0 0 1px #eef1f5;
    \\}
    \\
    \\input:not([type="checkbox"]):not([type="range"]):hover {
    \\    border-color: #4f83b3;
    \\}
    \\
    \\input:not([type="checkbox"]):not([type="range"]):focus {
    \\    border-color: #316ac5;
    \\}
    \\
    \\input:not([type="checkbox"]):not([type="range"])::selection {
    \\    background: #316ac5;
    \\    color: #ffffff;
    \\}
    \\
    \\select {
    \\    background: #ffffff;
    \\    border: 1px solid #7f9db9;
    \\}
    \\
    \\select:hover, select:focus {
    \\    border-color: #316ac5;
    \\}
    \\
    \\select::picker-icon {
    \\    color: #4a6384;
    \\    background: linear-gradient(to bottom, #e8f1ff, #b8d0f3);
    \\    border: 1px solid #7f9db9;
    \\    border-radius: 2px;
    \\    box-shadow: inset 0 0 0 1px #f5f9ff;
    \\}
    \\
    \\select:hover::picker-icon {
    \\    background: linear-gradient(to bottom, #dceaff, #a4c4ef);
    \\    border-color: #316ac5;
    \\}
    \\
    \\select:active::picker-icon {
    \\    background: linear-gradient(to bottom, #98b7df, #cfdef4);
    \\    box-shadow: inset 0 0 0 1px #8daace;
    \\}
    \\
    \\option {
    \\    background: #ffffff;
    \\    color: #000000;
    \\    border: none;
    \\    font-size: 13px;
    \\}
    \\
    \\option:hover {
    \\    background: #316ac5;
    \\    color: #ffffff;
    \\}
    \\
    \\option:disabled {
    \\    background: #ffffff;
    \\    color: #aca899;
    \\}
    \\
    \\input[type="checkbox"] {
    \\    color: #21a121;
    \\    background: linear-gradient(to bottom, #dcdad0, #ffffff 65%);
    \\    border: 1px solid #1c5180;
    \\    box-shadow: inset 0 0 0 1px #f5f4eb;
    \\}
    \\
    \\input[type="checkbox"]:hover {
    \\    box-shadow: inset 0 0 0 2px #ffcf73;
    \\}
    \\
    \\input[type="checkbox"]:focus {
    \\    outline: 1px dotted #383838;
    \\    outline-offset: 2px;
    \\}
    \\
    \\input[type="checkbox"]:active {
    \\    background: #e0dfd7;
    \\}
    \\
    \\input[type="checkbox"]:disabled {
    \\    border-color: #c9c7ba;
    \\    color: #aca899;
    \\    background: #f5f4ea;
    \\    outline: none;
    \\}
    \\
    \\input:not([type="checkbox"]):not([type="range"]):disabled, select:disabled {
    \\    background: #ece9d8;
    \\    color: #aca899;
    \\    border-color: #c9c7ba;
    \\}
    \\
    \\select:disabled::picker-icon {
    \\    background: #ece9d8;
    \\    color: #aca899;
    \\    border-color: #c9c7ba;
    \\    box-shadow: none;
    \\}
    \\
    \\/* Range and scroll parts: dimensions and decoration belong to the skin. */
    \\input[type="range"] { height: 28px; padding: 2px; border: none; background: transparent; }
    \\input[type="range"]:focus { outline: 1px dotted #383838; outline-offset: 1px; }
    \\input::slider-track { height: 5px; background: #e4e1d5; border: 1px solid #9d9c92; border-radius: 0; }
    \\input::slider-fill { background: #b8cbdc; border-radius: 0; }
    \\input::slider-thumb { width: 12px; height: 22px; background: linear-gradient(to right,#ffffff,#d5e3f4); border: 1px solid #567caa; border-radius: 2px; box-shadow: inset 0 0 0 1px #ffffff; }
    \\input:hover::slider-thumb { background: #f8edc3; border-color: #bc8d32; }
    \\input:active::slider-thumb { background: #d5dfef; }
    \\input:disabled::slider-thumb { background: #e2dfd5; border-color: #aaa89c; }
    \\::scrollbar { width: 17px; height: 17px; background: #edead8; }
    \\::scrollbar-track { background: linear-gradient(to right,#f4f2e8,#e8e5d4); border: 1px solid #dedbc9; }
    \\::scrollbar-thumb { width: 24px; height: 24px; background: linear-gradient(to right,#d6e5f9,#a8c0e2); border: 1px solid #6e91bf; border-radius: 2px; box-shadow: inset 0 0 0 1px #edf4ff; }
    \\::scrollbar-thumb:hover { background: #c4dcfb; }
    \\::scrollbar-thumb:active { background: #9db9df; }
    \\::scrollbar-corner { background: #ece9d8; }
;
