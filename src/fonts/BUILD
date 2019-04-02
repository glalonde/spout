package(default_visibility = ["//visibility:public"])

cc_library(
    name = "font8x8",
    hdrs = [
        "font8x8.h",
        "font8x8_basic.h",
        "font8x8_block.h",
        "font8x8_box.h",
        "font8x8_control.h",
        "font8x8_ext_latin.h",
        "font8x8_greek.h",
        "font8x8_hiragana.h",
        "font8x8_latin.h",
        "font8x8_misc.h",
        "font8x8_sga.h",
    ],
)

cc_binary(
    name = "font_example",
    srcs = [
        "font_example.cc",
    ],
    deps = [
        ":font8x8",
        "//base:init",
        "//base:logging",
    ],
)

cc_library(
    name = "font_renderer",
    srcs = [
        "font_renderer.cc",
    ],
    hdrs = [
        "font_renderer.h",
    ],
    deps = [
        ":font8x8",
        "//src:eigen_types",
    ],
)
