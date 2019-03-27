package(
    default_visibility = ["//visibility:public"],
)

licenses(["permissive"])

cc_library(
    name = "glad",
    srcs = ["src/glad.c"],
    hdrs = ["include/glad/glad.h"],
    includes = ["./include"],
    linkopts = [
        "-lGL",
        "-ldl",
    ],
)

cc_binary(
    name = "glad_example",
    srcs = ["glad_example.cc"],
    linkopts = ["-lglut"],
    deps = [":glad"],
)
