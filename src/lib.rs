//! Spout game library — GPU-accelerated particle terrain destruction game.

pub mod bloom;
pub mod buffer_util;
pub mod camera;
pub mod color_maps;
pub mod game_params;
#[cfg(test)]
pub(crate) mod gpu_test_utils;
pub mod input;
pub mod level_manager;
pub mod particles;
pub mod render;
pub mod shader_util;
pub mod ship;
pub mod text;
pub mod textured_quad;
