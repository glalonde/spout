package(default_visibility = ["//visibility:public"])

filegroup(
    name = "testdata",
    srcs = glob([
        "testdata/*",
    ]),
)

filegroup(
    name = "shaders",
    srcs = [
        "draw_particles.cs",
        "shader.frag",
        "shader.vert",
    ],
)

cc_binary(
    name = "compute_shader_test",
    srcs = ["compute_shader_test.cc"],
    data = [":shaders"],
    deps = [
        "//base:init",
        "//base:wall_timer",
        "//graphics:check_opengl_errors",
        "//graphics:load_shader",
        "//graphics:opengl",
        "//src:controller_input",
        "//src:eigen_types",
        "//src:random",
        "//src/color_maps",
    ],
)
