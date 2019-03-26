package(default_visibility = ["//visibility:public"])

cc_library(
    name = "sdl",
    linkopts = [
        "-lSDL2",
        "-lGL",
    ],
)

cc_test(
    name = "sdl_example",
    srcs = ["sdl_example.cc"],
    deps = [
        "//base:googletest",
        ":sdl",
    ],
)
