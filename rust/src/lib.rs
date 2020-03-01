pub mod color_maps;
pub mod compositor;
pub mod debug_overlay;
pub mod emitter;
pub mod fps_estimator;
pub mod game_params;
pub mod glow_pass;
pub mod int_grid;
pub mod level_buffer;
pub mod music_player;
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
