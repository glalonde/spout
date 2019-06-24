package(default_visibility = ["//visibility:public"])

cc_library(
    name = "logging",
    hdrs = ["logging.h"],
    copts = ["-DHAVE_UNWIND_H"],
    deps = [
        "@com_google_absl//absl/flags:flag",
        "@com_google_absl//absl/flags:parse",
        "@com_google_glog//:glog",
    ],
)

cc_library(
    name = "construction_macros",
    hdrs = ["construction_macros.h"],
    deps = [
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

cc_library(
    name = "time",
    srcs = ["time.cc"],
    hdrs = ["time.h"],
    deps = [":logging"],
)

cc_library(
    name = "wall_timer",
    hdrs = ["wall_timer.h"],
    deps = [
        ":logging",
        ":time",
    ],
)

cc_library(
    name = "circular_buffer",
    hdrs = ["circular_buffer.h"],
    deps = [
    ],
)

cc_test(
    name = "circular_buffer_test",
    srcs = ["circular_buffer_test.cc"],
    deps = [
        ":circular_buffer",
        ":googletest",
    ],
)

cc_library(
    name = "scoped_profiler",
    hdrs = ["scoped_profiler.h"],
    linkopts = ["-lprofiler"],
    deps = [
        ":construction_macros",
    ],
)

cc_test(
    name = "scoped_profiler_test",
    srcs = ["scoped_profiler_test.cc"],
    deps = [
        ":file",
        ":googletest",
        ":scoped_profiler",
        ":time",
    ],
)

cc_library(
    name = "base",
    deps = [
        ":error_xor",
        ":file",
        ":format",
        ":init",
        ":logging",
        ":time",
        ":wall_timer",
        "@com_google_absl//absl/container:fixed_array",
        "@com_google_absl//absl/container:flat_hash_map",
        "@com_google_absl//absl/container:flat_hash_set",
        "@com_google_absl//absl/container:inlined_vector",
    ],
)

cc_binary(
    name = "benchmark_example",
    srcs = ["benchmark_example.cc"],
    deps = [
        ":logging",
        "@com_github_google_benchmark//:benchmark_main",
    ],
)

cc_test(
    name = "time_test",
    srcs = ["time_test.cc"],
    deps = [
        ":googletest",
        ":time",
        ":wall_timer",
    ],
)

cc_library(
    name = "error_xor",
    hdrs = ["error_xor.h"],
    deps = [
        ":logging",
    ],
)

cc_test(
    name = "error_xor_test",
    srcs = ["error_xor_test.cc"],
    deps = [
        ":error_xor",
        ":googletest",
    ],
)
