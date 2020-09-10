pub mod color_maps;
pub mod debug_overlay;
pub mod emitter;
pub mod fonts;
pub mod fps_estimator;
pub mod game_params;
pub mod game_viewport;
pub mod glow_pass;
pub mod int_grid;
pub mod level_manager;
// pub mod music_player;
pub mod particle_system;
pub mod shader_utils;
pub mod ship;
// pub mod sound_queue;
pub mod terrain_renderer;
pub mod text_renderer;
pub mod viewport;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        let _test_bytes = shader_utils::Shaders::get("collatz.comp.spv").unwrap();
    }
}
