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
