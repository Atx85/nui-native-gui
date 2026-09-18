// Experimental: requires the official Jai compiler and its Bindings_Generator module.
// Run from the SDK directory. Generate the full API from the authoritative header.
#import "Basic";
#import "Bindings_Generator";
#import "Compiler";

#run,stallable {
    set_build_options_dc(.{do_output=false});
    options: Generate_Bindings_Options;
    array_add(*options.source_files, "native_ui.h");
    array_add(*options.include_paths, ".");
    array_add(*options.library_search_paths, "standalone");
    library: string;
    #if OS == .MACOS { library = "libnative_ui.dylib"; }
    else #if OS == .LINUX { library = "libnative_ui.so"; }
    else { #assert false; }
    array_add(*options.libraries, .{filename=library, identifier="NativeUI"});
    options.generate_library_declarations = true;
    options.generate_compile_time_struct_checks = true;
    if !generate_bindings(options, "examples/controls/raylib/jai/native_ui.jai") {
        compiler_set_workspace_status(.FAILED);
    }
}
