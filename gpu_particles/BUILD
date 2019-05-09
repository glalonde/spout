package(default_visibility = ["//visibility:public"])

filegroup(
    name = "shaders",
    srcs = glob([
        "shaders/*",
    ]),
)

cc_library(
    name = "game_parameters",
    srcs = [],
    hdrs = ["game_parameters.h"],
    deps = [
    ],
)

cc_library(
    name = "gl_particle",
    srcs = [],
    hdrs = ["gl_particle.h"],
    deps = [
        "//src:eigen_types",
    ],
)

cc_library(
    name = "gl_emitter",
    srcs = ["gl_emitter.cc"],
    hdrs = ["gl_emitter.h"],
    deps = [
        ":game_parameters",
        ":gl_particle",
        "//base",
        "//graphics:check_opengl_errors",
        "//graphics:load_shader",
        "//graphics:opengl",
        "//src:eigen_types",
        "//src:int_grid",
    ],
)

cc_library(
    name = "game_controller",
    srcs = ["game_controller.cc"],
    hdrs = ["game_controller.h"],
    deps = [
        ":game_parameters",
        ":game_window",
        ":gl_emitter",
        "//base",
        "//graphics:check_opengl_errors",
        "//graphics:load_shader",
        "//graphics:opengl",
        "//src:demo_utils",
        "//src:eigen_types",
        "//src:image",
        "//src:int_grid",
        "//src:random",
        "//src/color_maps",
    ],
)

cc_library(
    name = "game_window",
    srcs = ["game_window.cc"],
    hdrs = ["game_window.h"],
    data = [":shaders"],
    deps = [
        "//base",
        "//graphics:load_shader",
        "//graphics:opengl",
        "//src:controller_input",
    ],
)

cc_binary(
    name = "game_main",
    srcs = ["game_main.cc"],
    data = [":shaders"],
    deps = [
        ":game_controller",
        "//base:init",
    ],
)
