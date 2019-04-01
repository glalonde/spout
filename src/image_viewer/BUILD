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
        "shader.vert",
        "shader.frag",
        ]
)



cc_library(
    name = "image_viewer",
    hdrs = ["image_viewer.h"],
    srcs = ["image_viewer.cc"],
    data = [":shaders"],
    deps = [
        "//src:controller_input",
        "//src:eigen_types",
        "//src:opengl",
        "//src:image",
        "//src:check_opengl_errors",
        "//src:load_shader",
        "//base:format",
        "//base:logging",
    ],
)

cc_binary(
    name = "image_viewer_main",
    srcs = ["image_viewer_main.cc"],
    data = [":testdata"],
    deps = [
        ":image_viewer",
        "//src:image_io",
        "//base:init",
        "//base:file",
    ],
)

cc_binary(
    name = "color_map_vis",
    srcs = ["color_map_vis.cc"],
    deps = [
        ":image_viewer",
        "//base:init",
        "//src/color_maps",
        "//src:convert",
        "//src:fps_estimator",
        "//base:time",
        "//base:wall_timer",
    ],
)
