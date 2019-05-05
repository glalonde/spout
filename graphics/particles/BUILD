package(default_visibility = ["//visibility:public"])

filegroup(
    name = "shaders",
    srcs = [
        "bresenham.cs",
        "emitter.cs",
        "color_map_texture.frag",
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

cc_binary(
    name = "particle_shader_test",
    srcs = ["particle_shader_test.cc"],
    data = [":shaders"],
    deps = [
        "//base",
        "//base:init",
        "//base:wall_timer",
        "//graphics:check_opengl_errors",
        "//graphics:load_shader",
        "//graphics:opengl",
        "//src:bresenham",
        "//src:controller_input",
        "//src:demo_utils",
        "//src:eigen_types",
        "//src:int_grid",
        "//src:random",
        "//src:so2",
        "//src/color_maps",
    ],
)

cc_binary(
    name = "cpu_particle_test",
    srcs = ["cpu_particle_test.cc"],
    deps = [
        "//base:format",
        "//base:init",
        "//graphics:animated_canvas",
        "//src:bresenham",
        "//src:convert",
        "//src:demo_utils",
        "//src:random",
        "//src:so2",
        "//src/fonts:font_renderer",
    ],
)
