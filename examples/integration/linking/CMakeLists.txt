cmake_minimum_required(VERSION 3.20)
project(native_ui_linking_example LANGUAGES C)

find_package(native_ui CONFIG REQUIRED COMPONENTS standalone)
add_executable(hello-ui main.c)
target_compile_features(hello-ui PRIVATE c_std_11)
target_link_libraries(hello-ui PRIVATE native_ui::standalone)
if(MSVC)
    target_compile_options(hello-ui PRIVATE /utf-8)
    target_compile_definitions(hello-ui PRIVATE _CRT_SECURE_NO_WARNINGS)
endif()
# Ship the library next to the executable, including in multi-config builds.
add_custom_command(TARGET hello-ui POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "$<TARGET_FILE:native_ui::standalone>" "$<TARGET_FILE_DIR:hello-ui>"
    VERBATIM)
if(APPLE)
    set_target_properties(hello-ui PROPERTIES BUILD_RPATH "@loader_path")
elseif(UNIX)
    set_target_properties(hello-ui PROPERTIES BUILD_RPATH "$ORIGIN")
endif()

include(CTest)
add_test(NAME linked-library COMMAND hello-ui --check)
