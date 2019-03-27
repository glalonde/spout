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
