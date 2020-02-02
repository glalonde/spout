pub mod color_maps;
pub mod emitter;
pub mod fps_estimator;
pub mod int_grid;
pub mod particle_system;
pub mod shader_utils;
pub mod ship;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        shader_utils::list_shaders();
        let _test_bytes = include_shader!("collatz.comp.spv");
    }
}
