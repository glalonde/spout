package(default_visibility = ["//visibility:public"])

cc_library(
    name = "color_maps",
    srcs = [
        "color_maps.cc",
        "inferno.cc",
        "linear_color_map.h",
        "magma.cc",
        "parula.cc",
        "plasma.cc",
        "viridis.cc",
    ],
    hdrs = ["color_maps.h"],
    deps = [
        "//base:logging",
        "//src:eigen_types",
        "//src:interpolation",
    ],
)

cc_test(
    name = "color_maps_test",
    srcs = [
        "color_maps_test.cc",
    ],
    deps = [
        ":color_maps",
        "//base:googletest",
    ],
)

cc_binary(
    name = "color_maps_viewer",
    srcs = [
        "color_maps_viewer.cc",
    ],
    deps = [
        ":color_maps",
        "//base:init",
        "//src:convert",
        "//graphics:image_viewer",
    ],
)
