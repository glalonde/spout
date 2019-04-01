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

cc_binary(
    name = "eigen_test",
    srcs = ["eigen_test.cc"],
    deps = [
        ":eigen_types",
        "//base:googletest",
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
    hdrs = ["fps_estimator.h"],
    srcs = ["fps_estimator.cc"],
    deps = [
        "//base:circular_buffer",
        "//base:time",
    ],
)
