package(default_visibility = ["//visibility:public"])

cc_library(
    name = "sdl",
    linkopts = [
        "-lSDL2",
        "-lGL",
    ],
)

cc_binary(
    name = "sdl_example",
    srcs = ["sdl_example.cc"],
    deps = [
        ":sdl",
    ],
)

cc_library(
    name = "eigen_types",
    hdrs = ["eigen_types.h"],
    copts = [
        "-DEIGEN_ENABLE_AVX512",
        "-mavx512f",
    ],
    deps = ["@eigen"],
)

cc_library(
    name = "random",
    hdrs = ["random.h"],
    deps = [
        ":eigen_types",
        ":interpolation",
        "//base:logging",
    ],
)

cc_test(
    name = "random_test",
    srcs = ["random_test.cc"],
    deps = [
        ":convert",
        ":image",
        ":random",
        "//base:googletest",
        "//src/color_maps",
        "//src/image_viewer",
    ],
)

filegroup(
    name = "testdata",
    srcs = glob([
        "testdata/*",
    ]),
)

cc_library(
    name = "image",
    hdrs = ["image.h"],
    deps = [":eigen_types"],
)

cc_library(
    name = "image_io",
    srcs = ["image_io.cc"],
    hdrs = ["image_io.h"],
    deps = [
        ":image",
        "//base:logging",
        "@com_github_nothings_stb//:stb_image",
    ],
)

cc_test(
    name = "image_io_test",
    srcs = ["image_io_test.cc"],
    data = [":testdata"],
    deps = [
        ":image_io",
        "//base:googletest",
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

cc_test(
    name = "print_env",
    srcs = ["print_env.cc"],
    deps = [
        "//base:logging",
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
    name = "controller_input",
    hdrs = ["controller_input.h"],
    deps = [":sdl"],
)

cc_library(
    name = "convert",
    hdrs = ["convert.h"],
    deps = [":image"],
)

cc_test(
    name = "convert_test",
    srcs = ["convert_test.cc"],
    deps = [
        ":convert",
        "//base:googletest",
    ],
)

cc_library(
    name = "interpolation",
    hdrs = ["interpolation.h"],
)

cc_library(
    name = "fps_estimator",
    srcs = ["fps_estimator.cc"],
    hdrs = ["fps_estimator.h"],
    deps = [
        "//base:circular_buffer",
        "//base:time",
    ],
)

cc_library(
    name = "bresenham",
    hdrs = ["bresenham.h"],
    deps = [
        ":eigen_types",
        ":image",
        "//base:logging",
    ],
)

cc_test(
    name = "bresenham_test",
    srcs = ["bresenham_test.cc"],
    deps = [
        ":bresenham",
        "//base:googletest",
        "//src/image_viewer:animated_canvas",
    ],
)

cc_binary(
    name = "bresenham_demo",
    srcs = ["bresenham_demo.cc"],
    deps = [
        ":bresenham",
        ":convert",
        "//base:format",
        "//base:init",
        "//src/fonts:font_renderer",
        "//src/image_viewer:animated_canvas",
    ],
)

cc_library(
    name = "level_generators",
    hdrs = ["level_generators.h"],
    deps = [
        ":image",
        ":random",
    ],
)

cc_binary(
    name = "level_generators_demo",
    srcs = ["level_generators_demo.cc"],
    deps = [
        ":convert",
        ":level_generators",
        "//base:init",
        "//base:wall_timer",
        "//src/color_maps",
        "//src/image_viewer:animated_canvas",
    ],
)

cc_library(
    name = "scrolling_manager",
    srcs = ["scrolling_manager.cc"],
    hdrs = ["scrolling_manager.h"],
    deps = [
        "//base",
    ],
)

cc_test(
    name = "scrolling_manager_test",
    srcs = ["scrolling_manager_test.cc"],
    deps = [
        ":scrolling_manager",
        "//base:googletest",
    ],
)

cc_binary(
    name = "scrolling_demo",
    srcs = ["scrolling_demo.cc"],
    deps = [
        ":convert",
        "//base:init",
        "//base:wall_timer",
        "//src/color_maps",
        "//src/image_viewer:animated_canvas",
        ":scrolling_manager",
    ],
)
