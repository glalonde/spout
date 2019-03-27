package(default_visibility = ["//visibility:public"])

cc_library(
    name = "logging",
    hdrs = ["logging.h"],
    deps = [
        "@com_google_glog//:glog",
        "@com_github_gflags_gflags//:gflags",
    ],
)

cc_library(
    name = "init",
    hdrs = ["init.h"],
    deps = [":logging"],
)

cc_binary(
    name = "init_example",
    srcs = ["init_example.cc"],
    deps = [":init"],
)

cc_library(
    name = "googletest",
    hdrs = ["googletest.h"],
    deps = [
        ":init",
        ":logging",
        "@com_google_googletest//:gtest_main",
    ],
)

cc_test(
    name = "logging_test",
    srcs = ["logging_test.cc"],
    deps = [
        ":googletest",
    ],
)

cc_test(
    name = "optional_test",
    srcs = ["optional_test.cc"],
    deps = [
        ":googletest",
    ],
)

cc_library(
    name = "format",
    hdrs = ["format.h"],
    deps = [
        "@com_github_c42f_tinyformat//:tinyformat",
    ],
)

cc_test(
    name = "format_test",
    srcs = ["format_test.cc"],
    deps = [
        ":format",
        ":googletest",
    ],
)

cc_library(
    name = "file",
    srcs = ["file.cc"],
    hdrs = ["file.h"],
    deps = [
        ":logging",
    ],
)
