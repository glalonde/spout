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
