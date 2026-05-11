//! Game configuration structs deserialized from `game_config.toml`.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TouchControlScheme {
    /// Drag-to-aim: touching the right half and dragging sets an absolute target
    /// heading via a bang-bang controller. Original scheme.
    #[default]
    Drag,
    /// Triangle split: the right half is divided by a diagonal from top-center to
    /// bottom-right. Current touch position determines direction — above/right of
    /// the diagonal → rotate CW, below/left → rotate CCW. Drag across the diagonal
    /// to switch directions instantly, no dead zone.
    Triangle,
}

// Parameters that define the game. These don't change at runtime.
#[derive(Debug)]
pub enum GameParamsError {
    Parse(toml::de::Error),
    Invalid(String),
}

impl fmt::Display for GameParamsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameParamsError::Parse(err) => write!(f, "{err}"),
            GameParamsError::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for GameParamsError {}

impl From<toml::de::Error> for GameParamsError {
    fn from(value: toml::de::Error) -> Self {
        GameParamsError::Parse(value)
    }
}

// Particle counts are calculated in f32 before being cast to u32 for buffer
// allocation. Keep validation inside f32's exact integer range so the limit
// does not depend on rounded large values.
const MAX_EXACT_PARTICLE_COUNT_F32: f32 = (1_u32 << 24) as f32;

// Parameters that define the game. These don't change at runtime.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct GameParams {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub level_width: u32,
    pub level_height: u32,

    pub fps: f64,
    pub music_starts_on: bool,
    pub render_ship: bool,

    #[serde(default)]
    pub touch_control_scheme: TouchControlScheme,

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
#[serde(deny_unknown_fields)]
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

    /// Depth of the dual-filter bloom mip pyramid. More levels = wider halo.
    /// Each level halves dimensions; the smallest mip is 2^N times smaller per axis.
    /// 4 = tight halo, 6 = typical, 8 = very wide. Clamped to what the surface allows.
    #[serde(default = "default_bloom_mip_levels", alias = "bloom_passes")]
    pub bloom_mip_levels: u32,

    /// CRT post-process intensity. 0.0 = bypass, 1.0 = full effect.
    /// Applies barrel distortion, chromatic aberration, phosphor mask,
    /// scanlines, and vignette.
    #[serde(default)]
    pub crt_strength: f32,

    /// Density-to-color scaling. Controls how many particles per cell are needed
    /// to reach full saturation. Lower = saturates faster, higher = needs more
    /// particles. The raw count is multiplied by 1/density_scale before the
    /// sigmoid. Default 100 means ~100 particles per cell ≈ half saturation.
    #[serde(default = "default_density_scale")]
    pub density_scale: f32,

    /// Sigmoid exponent for density mapping. Values > 1 make the curve steeper
    /// (sharper transition from dim to bright), values < 1 make it gentler.
    /// Default 1.0 = standard sigmoid.
    #[serde(default = "default_density_exponent")]
    pub density_exponent: f32,
}

fn default_density_scale() -> f32 {
    100.0
}

fn default_level_time_limit_seconds() -> f32 {
    120.0
}

fn default_bloom_mip_levels() -> u32 {
    6
}

fn default_density_exponent() -> f32 {
    1.0
}

impl Default for VisualParams {
    fn default() -> Self {
        VisualParams {
            color_map: 0,
            bloom_threshold: 0.6,
            bloom_strength: 1.0,
            bloom_mip_levels: default_bloom_mip_levels(),
            crt_strength: 0.0,
            density_scale: default_density_scale(),
            density_exponent: default_density_exponent(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct LevelParams {
    pub starting_terrain_health: i32,
    #[serde(default = "default_level_time_limit_seconds")]
    pub level_time_limit_seconds: f32,
}

impl Default for LevelParams {
    fn default() -> Self {
        LevelParams {
            starting_terrain_health: 100000,
            level_time_limit_seconds: default_level_time_limit_seconds(),
        }
    }
}

impl std::str::FromStr for GameParams {
    type Err = GameParamsError;
    fn from_str(serialized: &str) -> Result<Self, Self::Err> {
        let params: GameParams = toml::from_str(serialized)?;
        params.validate()?;
        Ok(params)
    }
}

impl GameParams {
    pub fn validate(&self) -> Result<(), GameParamsError> {
        ensure_positive_u32("viewport_width", self.viewport_width)?;
        ensure_positive_u32("viewport_height", self.viewport_height)?;
        ensure_positive_u32("level_width", self.level_width)?;
        ensure_positive_u32("level_height", self.level_height)?;
        // Width is currently fixed to the viewport because particle/terrain
        // shaders still conflate terrain stride, view width, and density width.
        // Height may exceed the viewport because levels scroll vertically.
        ensure(
            self.level_width == self.viewport_width,
            "level_width must equal viewport_width until terrain/view/density widths are split",
        )?;
        ensure(
            self.level_height >= self.viewport_height,
            "level_height must be at least viewport_height",
        )?;
        ensure_positive_f64("fps", self.fps)?;

        let particle_count = self.particle_system_params.emission_rate
            * self.particle_system_params.max_particle_life;
        ensure(
            particle_count.is_finite()
                && particle_count > 0.0
                && particle_count <= MAX_EXACT_PARTICLE_COUNT_F32,
            "particle_system_params.emission_rate * max_particle_life must be finite, positive, and no more than 16,777,216",
        )?;
        ensure_positive_f32(
            "particle_system_params.emission_rate",
            self.particle_system_params.emission_rate,
        )?;
        ensure_positive_f32(
            "particle_system_params.emission_speed",
            self.particle_system_params.emission_speed,
        )?;
        ensure_positive_f32(
            "particle_system_params.max_particle_life",
            self.particle_system_params.max_particle_life,
        )?;
        ensure_non_negative_f32(
            "particle_system_params.damage_rate",
            self.particle_system_params.damage_rate,
        )?;
        ensure_finite_f32(
            "particle_system_params.gravity",
            self.particle_system_params.gravity,
        )?;
        ensure_non_negative_f32(
            "particle_system_params.elasticity",
            self.particle_system_params.elasticity,
        )?;

        ensure_non_negative_f32("ship_params.acceleration", self.ship_params.acceleration)?;
        ensure_non_negative_f32("ship_params.rotation_rate", self.ship_params.rotation_rate)?;
        ensure_positive_f32("ship_params.max_speed", self.ship_params.max_speed)?;

        ensure(
            self.level_params.starting_terrain_health > 0,
            "level_params.starting_terrain_health must be positive",
        )?;
        ensure_positive_f32(
            "level_params.level_time_limit_seconds",
            self.level_params.level_time_limit_seconds,
        )?;

        ensure(
            self.visual_params.color_map >= 0
                && crate::color_maps::has_color_map_index(self.visual_params.color_map as usize),
            "visual_params.color_map must name an existing color map",
        )?;
        ensure_non_negative_f32(
            "visual_params.bloom_threshold",
            self.visual_params.bloom_threshold,
        )?;
        ensure_non_negative_f32(
            "visual_params.bloom_strength",
            self.visual_params.bloom_strength,
        )?;
        ensure_positive_u32(
            "visual_params.bloom_mip_levels",
            self.visual_params.bloom_mip_levels,
        )?;
        ensure_non_negative_f32(
            "visual_params.crt_strength",
            self.visual_params.crt_strength,
        )?;
        ensure_positive_f32(
            "visual_params.density_scale",
            self.visual_params.density_scale,
        )?;
        ensure_positive_f32(
            "visual_params.density_exponent",
            self.visual_params.density_exponent,
        )?;

        Ok(())
    }
}

fn ensure(condition: bool, message: &'static str) -> Result<(), GameParamsError> {
    if condition {
        Ok(())
    } else {
        Err(GameParamsError::Invalid(message.to_owned()))
    }
}

fn ensure_positive_u32(name: &'static str, value: u32) -> Result<(), GameParamsError> {
    if value == 0 {
        return Err(GameParamsError::Invalid(format!(
            "{name} must be greater than zero"
        )));
    }

    Ok(())
}

fn ensure_finite_f32(name: &'static str, value: f32) -> Result<(), GameParamsError> {
    if !value.is_finite() {
        return Err(GameParamsError::Invalid(format!("{name} must be finite")));
    }

    Ok(())
}

fn ensure_positive_f32(name: &'static str, value: f32) -> Result<(), GameParamsError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(GameParamsError::Invalid(format!(
            "{name} must be finite and greater than zero"
        )));
    }

    Ok(())
}

fn ensure_non_negative_f32(name: &'static str, value: f32) -> Result<(), GameParamsError> {
    if !value.is_finite() || value < 0.0 {
        return Err(GameParamsError::Invalid(format!(
            "{name} must be finite and non-negative"
        )));
    }

    Ok(())
}

fn ensure_positive_f64(name: &'static str, value: f64) -> Result<(), GameParamsError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(GameParamsError::Invalid(format!(
            "{name} must be finite and greater than zero"
        )));
    }

    Ok(())
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
            touch_control_scheme: TouchControlScheme::Drag,
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
    let params = match load_params_from_file() {
        Ok(params) => params,
        Err(err) => {
            panic!("embedded game_config.toml is invalid: {err}");
        }
    };
    // iOS has no keyboard, so music must default to on (no M-key to toggle).
    #[cfg(target_os = "ios")]
    let params = GameParams {
        music_starts_on: true,
        ..params
    };
    params
}

fn load_params_from_file() -> Result<GameParams, GameParamsError> {
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(disk_config) = std::fs::read_to_string("game_config.toml") {
        match disk_config.parse() {
            Ok(params) => {
                log::info!("Loaded config from disk: game_config.toml");
                return Ok(params);
            }
            Err(e) => {
                log::warn!(
                    "Failed to parse game_config.toml from disk: {e}; using embedded default"
                );
            }
        }
    }

    parse_embedded_config()
}

pub fn parse_embedded_config() -> Result<GameParams, GameParamsError> {
    EMBEDDED_CONFIG.parse()
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
            touch_control_scheme: TouchControlScheme::Drag,
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
        let params = match parse_embedded_config() {
            Ok(params) => params,
            Err(err) => panic!("embedded game_config.toml should parse and validate: {err}"),
        };
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
        assert!(d.level_params.level_time_limit_seconds > 0.0);
        assert!(d.validate().is_ok());
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
        assert_eq!(
            params.level_params.level_time_limit_seconds,
            default_level_time_limit_seconds()
        );
    }

    #[test]
    fn from_str_invalid_toml_returns_error() {
        let bad = "this is not valid toml {{{{";
        assert!(bad.parse::<GameParams>().is_err());
    }

    #[test]
    fn from_str_unknown_field_returns_error() {
        let bad = r#"
            viewport_width = 100
            viewport_height = 50
            level_width = 100
            level_height = 200
            fps = 30.0
            music_starts_on = true
            render_ship = false
            typo_that_should_not_be_ignored = true
        "#;
        assert!(bad.parse::<GameParams>().is_err());
    }

    #[test]
    fn width_mismatch_is_invalid_until_wider_levels_are_split() {
        let mut params = GameParams::default();
        params.level_width = params.viewport_width + 1;
        assert!(params.validate().is_err());
    }

    #[test]
    fn invalid_visual_params_are_rejected() {
        let mut params = GameParams::default();
        params.visual_params.color_map = i32::MAX;
        assert!(params.validate().is_err());

        params = GameParams::default();
        params.visual_params.density_scale = 0.0;
        assert!(params.validate().is_err());

        params = GameParams::default();
        params.visual_params.density_exponent = f32::NAN;
        assert!(params.validate().is_err());
    }

    #[test]
    fn invalid_particle_counts_are_rejected() {
        let mut params = GameParams::default();
        params.particle_system_params.emission_rate = 0.0;
        assert!(params.validate().is_err());

        params = GameParams::default();
        params.particle_system_params.max_particle_life = f32::INFINITY;
        assert!(params.validate().is_err());

        params = GameParams::default();
        params.particle_system_params.emission_rate = MAX_EXACT_PARTICLE_COUNT_F32 * 2.0;
        params.particle_system_params.max_particle_life = 1.0;
        assert!(params.validate().is_err());
    }
}
