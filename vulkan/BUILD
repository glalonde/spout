package(default_visibility = ["//visibility:public"])

filegroup(
    name = "shaders",
    srcs = glob([
        "shaders/*",
    ]),
)

cc_binary(
    name = "glfw_main",
    srcs = ["glfw_main.cc"],
    data = [
        "//vulkan/shaders:frag_shader",
        "//vulkan/shaders:vert_shader",
    ],
    linkopts = [
        "-lglfw",
        "-lvulkan",
    ],
    deps = [
        "//base",
        "//src:eigen_glm",
        "//src:fps_estimator",
        "@com_github_gtruc_glm//:glm",
    ],
)

cc_library(
    name = "vulkan_memory_allocator",
    srcs = ["vulkan_memory_allocator.cc"],
    hdrs = ["vulkan_memory_allocator.h"],
    deps = [
        "@com_github_vulkan_memory_allocator//:vulkan_memory_allocator",
    ],
)
