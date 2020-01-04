pub mod color_maps;
pub mod emitter;
pub mod shader_utils;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        scrub_log::init().unwrap();
        shader_utils::list_shaders();
        let _test_bytes = include_shader!("collatz.comp.spv");
    }
}
