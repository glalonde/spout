package(default_visibility = ["//visibility:public"])

filegroup(
    name = "shaders",
    srcs = glob([
        "shaders/*",
    ]),
)

cc_library(
    name = "image_viewer",
    srcs = ["image_viewer.cc"],
    hdrs = ["image_viewer.h"],
    data = [
        "//vulkan/shaders:frag_shader",
        "//vulkan/shaders:vert_shader",
    ],
    deps = [
        "//base",
        "//src:eigen_glm",
        "//src:fps_estimator",
        "//src:image_io",
        "//vulkan:glfw",
        "//vulkan:vma_wrapper",
        "//vulkan:vulkan_utils",
    ],
)

cc_binary(
    name = "image_viewer_main",
    srcs = ["image_viewer_main.cc"],
    deps = [
        ":image_viewer",
        "//base:init",
    ],
)
