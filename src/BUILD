package(default_visibility = ["//visibility:public"])

cc_library(
    name = "sdl",
    linkopts = [
        "-lSDL2",
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

cc_test(
    name = "eigen_types_test",
    srcs = ["eigen_types_test.cc"],
    deps = [
        ":eigen_types",
        "//base:googletest",
    ],
)

cc_library(
    name = "so2",
    hdrs = ["so2.h"],
    deps = [":eigen_types"],
)

cc_test(
    name = "so2_test",
    srcs = ["so2_test.cc"],
    deps = [
        ":so2",
        "//base:googletest",
    ],
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
        "//graphics:image_viewer",
        "//src/color_maps",
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

cc_test(
    name = "print_env",
    srcs = ["print_env.cc"],
    deps = [
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
        ":int_grid",
        "//base:logging",
    ],
)

cc_test(
    name = "bresenham_test",
    srcs = ["bresenham_test.cc"],
    tags = ["exclusive"],
    deps = [
        ":bresenham",
        "//base:googletest",
        "//graphics:animated_canvas",
    ],
)

cc_binary(
    name = "bresenham_demo",
    srcs = ["bresenham_demo.cc"],
    deps = [
        ":bresenham",
        ":convert",
        ":demo_utils",
        ":random",
        "//base:format",
        "//base:init",
        "//graphics:animated_canvas",
        "//src/fonts:font_renderer",
    ],
)

cc_binary(
    name = "bresenham_profile",
    srcs = ["bresenham_profile.cc"],
    deps = [
        ":bresenham",
        ":convert",
        ":demo_utils",
        ":random",
        "//base:format",
        "//base:init",
        "//base:scoped_profiler",
        "//base:wall_timer",
        "//src/fonts:font_renderer",
    ],
)

cc_binary(
    name = "exp_demo",
    srcs = ["exp_demo.cc"],
    deps = [
        ":bresenham",
        ":convert",
        ":demo_utils",
        ":int_grid",
        ":random",
        "//base:format",
        "//base:init",
        "//graphics:animated_canvas",
        "//src/fonts:font_renderer",
    ],
)

cc_binary(
    name = "bresenham_benchmark",
    srcs = ["bresenham_benchmark.cc"],
    deps = [
        ":bresenham",
        ":demo_utils",
        ":random",
        ":so2",
        "//base",
        "@com_github_google_benchmark//:benchmark_main",
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
        "//graphics:animated_canvas",
        "//src/color_maps",
    ],
)

cc_library(
    name = "scrolling_manager",
    srcs = ["scrolling_manager.cc"],
    hdrs = ["scrolling_manager.h"],
    deps = [
        ":buffer_stack",
        ":image",
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

cc_library(
    name = "mobile_object",
    hdrs = ["mobile_object.h"],
    deps = [
        ":so2",
        "//base",
    ],
)

cc_library(
    name = "drawing_utils",
    hdrs = ["drawing_utils.h"],
    deps = [
        ":image",
    ],
)

cc_library(
    name = "demo_utils",
    hdrs = ["demo_utils.h"],
    deps = [
        ":buffer_stack",
        ":convert",
        ":image",
        ":random",
        "//src/color_maps",
        "//src/fonts:font_renderer",
    ],
)

cc_binary(
    name = "ship_demo",
    srcs = ["ship_demo.cc"],
    deps = [
        ":bresenham",
        ":convert",
        ":demo_utils",
        ":drawing_utils",
        ":mobile_object",
        ":random",
        "//base:format",
        "//base:init",
        "//graphics:animated_canvas",
    ],
)

cc_library(
    name = "buffer_stack",
    hdrs = ["buffer_stack.h"],
    deps = [
        ":image",
        "//base:logging",
    ],
)

cc_test(
    name = "buffer_stack_test",
    srcs = ["buffer_stack_test.cc"],
    deps = [
        ":buffer_stack",
        "//base:googletest",
    ],
)

cc_library(
    name = "emitter",
    hdrs = ["emitter.h"],
    deps = [
        ":eigen_types",
        ":so2",
        "//base:circular_buffer",
        "//base:logging",
    ],
)

cc_test(
    name = "emitter_test",
    srcs = ["emitter_test.cc"],
    deps = [
        ":emitter",
        "//base:googletest",
    ],
)

cc_binary(
    name = "ship_scrolling_demo",
    srcs = ["ship_scrolling_demo.cc"],
    deps = [
        ":bresenham",
        ":buffer_stack",
        ":convert",
        ":demo_utils",
        ":drawing_utils",
        ":mobile_object",
        ":random",
        ":scrolling_manager",
        "//base:format",
        "//base:init",
        "//graphics:animated_canvas",
    ],
)

cc_binary(
    name = "emitter_demo",
    srcs = ["emitter_demo.cc"],
    deps = [
        ":bresenham",
        ":convert",
        ":demo_utils",
        ":drawing_utils",
        ":emitter",
        ":mobile_object",
        ":random",
        "//base:format",
        "//base:init",
        "//graphics:animated_canvas",
        "//src/fonts:font_renderer",
    ],
)

cc_binary(
    name = "game_demo",
    srcs = ["game_demo.cc"],
    deps = [
        ":bresenham",
        ":buffer_stack",
        ":convert",
        ":demo_utils",
        ":drawing_utils",
        ":emitter",
        ":mobile_object",
        ":random",
        ":scrolling_manager",
        "//base:format",
        "//base:init",
        "//base:scoped_profiler",
        "//graphics:animated_canvas",
        "//src/fonts:font_renderer",
    ],
)

cc_binary(
    name = "get_cell_benchmark",
    srcs = ["get_cell_benchmark.cc"],
    deps = [
        ":random",
        "//base",
        "@com_github_google_benchmark//:benchmark_main",
    ],
)

cc_library(
    name = "int_grid",
    hdrs = ["int_grid.h"],
    deps = [
    ],
)

cc_test(
    name = "int_grid_test",
    srcs = ["int_grid_test.cc"],
    deps = [
        ":int_grid",
        "//base:googletest",
    ],
)
