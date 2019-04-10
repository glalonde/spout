package(default_visibility = ["//visibility:public"])

filegroup(
    name = "testdata",
    srcs = glob([
        "testdata/*",
    ]),
)

filegroup(
    name = "shaders",
    srcs = [
        "shader.frag",
        "shader.vert",
    ],
)

cc_library(
    name = "opengl",
    hdrs = ["opengl.h"],
    linkopts = [
        "-lSDL2",
        "-lGL",
        "-lGLU",
    ],
    deps = [
        "//third_party/glad",
    ],
)

cc_library(
    name = "load_shader",
    srcs = ["load_shader.cc"],
    hdrs = ["load_shader.h"],
    deps = [
        ":opengl",
        "//base:file",
    ],
)

cc_test(
    name = "load_shader_test",
    srcs = ["load_shader_test.cc"],
    data = [":testdata"],
    local = 1,
    deps = [
        ":load_shader",
        "//base:googletest",
    ],
)

cc_library(
    name = "check_opengl_errors",
    hdrs = ["check_opengl_errors.h"],
    deps = [
        ":opengl",
        "//base:logging",
    ],
)

cc_library(
    name = "image_viewer",
    srcs = ["image_viewer.cc"],
    hdrs = ["image_viewer.h"],
    data = [":shaders"],
    deps = [
        "//base:format",
        "//base:logging",
        ":check_opengl_errors",
        "//src:controller_input",
        "//src:eigen_types",
        "//src:image",
        ":load_shader",
        ":opengl",
    ],
)

cc_binary(
    name = "image_viewer_main",
    srcs = ["image_viewer_main.cc"],
    data = [":testdata"],
    deps = [
        ":image_viewer",
        "//base:file",
        "//base:init",
        "//src:image_io",
    ],
)

cc_library(
    name = "animated_canvas",
    srcs = ["animated_canvas.cc"],
    hdrs = ["animated_canvas.h"],
    deps = [
        ":image_viewer",
        "//base:time",
        "//src:fps_estimator",
        "//src:image",
    ],
)

cc_binary(
    name = "color_map_vis",
    srcs = ["color_map_vis.cc"],
    deps = [
        ":animated_canvas",
        "//base:init",
        "//base:scoped_profiler",
        "//base:wall_timer",
        "//src:convert",
        "//src/color_maps",
        "//src/fonts:font_renderer",
    ],
)
