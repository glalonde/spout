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
        "@com_github_gtruc_glm//:glm",
    ],
)
