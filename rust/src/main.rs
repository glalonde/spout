use spout::shader_utils;

fn main() {
    scrub_log::init().unwrap();
    shader_utils::list_shaders();
}
