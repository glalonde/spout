package(default_visibility = ["//visibility:public"])

filegroup(
    name = "shaders",
    srcs = glob([
        "shaders/*",
    ]),
)

cc_library(
    name = "vulkan_memory_allocator",
    srcs = ["vulkan_memory_allocator.cc"],
    hdrs = ["vulkan_memory_allocator.h"],
    deps = [
        "@com_github_vulkan_memory_allocator//:vulkan_memory_allocator",
    ],
)

cc_library(
    name = "vulkan_utils",
    srcs = ["vulkan_utils.cc"],
    hdrs = ["vulkan_utils.h"],
    linkopts = [
        "-lvulkan",
    ],
    deps = [
        "//base:error_xor",
        "//base:file",
        "//base:logging",
        "@com_google_absl//absl/algorithm:container",
    ],
)

cc_library(
    name = "vma_wrapper",
    srcs = ["vma_wrapper.cc"],
    hdrs = ["vma_wrapper.h"],
    linkopts = [
        "-lvulkan",
    ],
    deps = [
        ":vulkan_memory_allocator",
        "//base:logging",
    ],
)

cc_library(
    name = "hello_quad",
    srcs = ["hello_quad.cc"],
    hdrs = ["hello_quad.h"],
    data = [
        "//vulkan/shaders:frag_shader",
        "//vulkan/shaders:vert_shader",
    ],
    linkopts = [
        "-lglfw",
        "-lvulkan",
    ],
    deps = [
        ":vma_wrapper",
        ":vulkan_utils",
        "//base",
        "//src:eigen_glm",
        "//src:fps_estimator",
    ],
)

cc_binary(
    name = "hello_quad_main",
    srcs = ["hello_quad_main.cc"],
    deps = [
        ":hello_quad",
        "//base:init",
    ],
)

cc_library(
    name = "hello_compute",
    srcs = ["hello_compute.cc"],
    hdrs = ["hello_compute.h"],
    data = [
        "//vulkan/shaders:mandelbrot",
    ],
    linkopts = [
        "-lvulkan",
    ],
    deps = [
        ":vma_wrapper",
        ":vulkan_utils",
        "//base:logging",
        "//src:convert",
        "//src:image",
        "//src:image_io",
    ],
)

cc_binary(
    name = "hello_compute_main",
    srcs = ["hello_compute_main.cc"],
    deps = [
        ":hello_compute",
        "//base:init",
    ],
)
