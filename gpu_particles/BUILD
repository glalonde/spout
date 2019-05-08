package(default_visibility = ["//visibility:public"])

filegroup(
    name = "shaders",
    srcs = glob([
        "shaders/*",
    ]),
)


cc_library(
    name = "gl_particle",
    hdrs = ["gl_particle.h"],
    srcs = [],
    deps = [
        "//src:eigen_types",
    ],
)

cc_library(
    name = "gl_emitter",
    hdrs = ["gl_emitter.h"],
    srcs = ["gl_emitter.cc"],
    deps = [
        "//base",
        ":gl_particle",
        "//graphics:check_opengl_errors",
        "//graphics:load_shader",
        "//graphics:opengl",
        "//src:eigen_types",
        "//src:int_grid",
    ],
)

cc_binary(
    name = "game_main",
    srcs = ["game_main.cc"],
    data = [":shaders"],
    deps = [
        ":gl_emitter",
        "//base",
        "//base:init",
        "//base:wall_timer",
        "//graphics:check_opengl_errors",
        "//graphics:load_shader",
        "//graphics:opengl",
        "//src:bresenham",
        "//src:controller_input",
        "//src:demo_utils",
        "//src:eigen_types",
        "//src:int_grid",
        "//src:random",
        "//src:so2",
        "//src/color_maps",
    ],
)
