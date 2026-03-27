//! Game configuration structs deserialized from `game_config.toml`.

use serde::{Deserialize, Serialize};

// Parameters that define the game. These don't change at runtime.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct GameParams {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub level_width: u32,
    pub level_height: u32,

    pub fps: f64,
    pub music_starts_on: bool,
    pub render_ship: bool,

    #[serde(default)]
    pub particle_system_params: ParticleSystemParams,

    #[serde(default)]
    pub ship_params: ShipParams,

    #[serde(default)]
    pub level_params: LevelParams,

    #[serde(default)]
    pub visual_params: VisualParams,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct VisualParams {
    /// Index into the particle color map palette (see color_maps.rs).
    pub color_map: i32,

    /// Brightness level above which pixels contribute to bloom (0.0–1.0).
    /// Lower = more of the scene glows; higher = only the brightest hotspots.
    /// Set to 1.1 to effectively disable bloom.
    pub bloom_threshold: f32,

    /// Multiplier on the bloom contribution in the final composite.
    /// 0.0 = no bloom, 1.0 = full additive bloom, 2.0+ = oversaturated.
    pub bloom_strength: f32,

    /// Number of separable H+V blur iterations applied to the thresholded image.
    /// Each additional pass widens the halo by roughly √2 (Gaussian convolution).
    /// 1 = tight (~4 px radius), 2 = moderate (~6 px), 4 = wide (~8 px).
    pub bloom_passes: u32,

    /// CRT post-process intensity. 0.0 = bypass, 1.0 = full effect.
    /// Applies barrel distortion, chromatic aberration, phosphor mask,
    /// scanlines, and vignette.
    #[serde(default)]
    pub crt_strength: f32,
}

impl Default for VisualParams {
    fn default() -> Self {
        VisualParams {
            color_map: 0,
            bloom_threshold: 0.6,
            bloom_strength: 1.0,
            bloom_passes: 2,
            crt_strength: 0.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ParticleSystemParams {
    pub emission_rate: f32,
    pub emission_speed: f32,
    pub max_particle_life: f32,
    pub damage_rate: f32,
    pub gravity: f32,
    pub elasticity: f32,
}

impl Default for ParticleSystemParams {
    fn default() -> Self {
        ParticleSystemParams {
            emission_rate: 100000.0,
            emission_speed: 500.0,
            max_particle_life: 2.0,
            damage_rate: 0.00001,
            gravity: -5.0,
            elasticity: 0.5,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ShipParams {
    pub acceleration: f32,
    pub rotation_rate: f32,
    pub max_speed: f32,
}

impl Default for ShipParams {
    fn default() -> Self {
        ShipParams {
            acceleration: 50.0,
            rotation_rate: 15.0,
            max_speed: 100.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct LevelParams {
    pub starting_terrain_health: i32,
}

impl Default for LevelParams {
    fn default() -> Self {
        LevelParams {
            starting_terrain_health: 100000,
        }
    }
}

impl std::str::FromStr for GameParams {
    type Err = toml::de::Error;
    fn from_str(serialized: &str) -> Result<Self, Self::Err> {
        let params = toml::from_str(serialized)?;
        Ok(params)
    }
}

impl Default for GameParams {
    fn default() -> Self {
        GameParams {
            viewport_width: 640,
            viewport_height: 320,
            level_width: 640,
            level_height: 960,
            fps: 60.0,
            music_starts_on: false,
            render_ship: false,
            particle_system_params: ParticleSystemParams::default(),
            ship_params: ShipParams::default(),
            level_params: LevelParams::default(),
            visual_params: VisualParams::default(),
        }
    }
}

/// Embedded default config, baked in at compile time.
const EMBEDDED_CONFIG: &str = include_str!("../game_config.toml");

/// Load game config. Tries `game_config.toml` in the current directory first
/// (for development / user overrides), then falls back to the embedded default.
/// This way packaged .app bundles work without an external config file.
pub fn get_game_config_from_default_file() -> GameParams {
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(disk_config) = std::fs::read_to_string("game_config.toml") {
        match disk_config.parse() {
            Ok(params) => {
                log::info!("Loaded config from disk: game_config.toml");
                return params;
            }
            Err(e) => {
                log::warn!(
                    "Failed to parse game_config.toml from disk: {e}; using embedded default"
                );
            }
        }
    }

    match EMBEDDED_CONFIG.parse() {
        Ok(params) => params,
        Err(e) => {
            log::error!("Failed to parse embedded config: {:?}", e);
            GameParams::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let params = GameParams {
            viewport_width: 640,
            viewport_height: 320,
            level_width: 640,
            level_height: 320 * 3,
            fps: 60.0,
            music_starts_on: false,
            render_ship: true,
            particle_system_params: ParticleSystemParams::default(),
            ship_params: ShipParams::default(),
            level_params: LevelParams::default(),
            visual_params: VisualParams::default(),
        };
        let serialized = toml::to_string(&params).unwrap();
        let deserialized: GameParams = toml::from_str(&serialized).unwrap();
        assert_eq!(params.viewport_width, deserialized.viewport_width);
        assert_eq!(params.viewport_height, deserialized.viewport_height);
        assert_eq!(params.level_width, deserialized.level_width);
        assert_eq!(params.level_height, deserialized.level_height);
    }

    #[test]
    fn embedded_config_parses() {
        let params = get_game_config_from_default_file();
        assert!(params.viewport_width > 0);
        assert!(params.viewport_height > 0);
        assert!(params.fps > 0.0);
    }

    #[test]
    fn default_has_sane_values() {
        let d = GameParams::default();
        assert!(d.viewport_width > 0);
        assert!(d.viewport_height > 0);
        assert!(d.level_width >= d.viewport_width);
        assert!(d.level_height > d.viewport_height);
        assert!(d.fps > 0.0);
    }

    #[test]
    fn from_str_with_missing_optional_sections() {
        let minimal = r#"
            viewport_width = 100
            viewport_height = 50
            level_width = 100
            level_height = 200
            fps = 30.0
            music_starts_on = true
            render_ship = false
        "#;
        let params: GameParams = minimal.parse().unwrap();
        assert_eq!(params.viewport_width, 100);
        assert_eq!(
            params.particle_system_params.emission_rate,
            ParticleSystemParams::default().emission_rate
        );
        assert_eq!(
            params.ship_params.acceleration,
            ShipParams::default().acceleration
        );
    }

    #[test]
    fn from_str_invalid_toml_returns_error() {
        let bad = "this is not valid toml {{{{";
        assert!(bad.parse::<GameParams>().is_err());
    }
}
