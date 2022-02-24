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
    pub enable_glow_pass: bool,
    pub render_ship: bool,
    pub color_map: i32,

    #[serde(default)]
    pub particle_system_params: ParticleSystemParams,

    #[serde(default)]
    pub ship_params: ShipParams,

    #[serde(default)]
    pub level_params: LevelParams,
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
            enable_glow_pass: true,
            render_ship: false,
            color_map: 0,
            particle_system_params: ParticleSystemParams::default(),
            ship_params: ShipParams::default(),
            level_params: LevelParams::default(),
        }
    }
}

pub fn get_game_config_from_default_file() -> GameParams {
    let config_data = include_str!("../game_config.toml");
    match config_data.parse() {
        Ok(params) => params,
        Err(e) => {
            log::error!(
                "Failed to parse config file({}): {:?}",
                "../game_config.toml",
                e
            );
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
            enable_glow_pass: false,
            render_ship: true,
            color_map: 0,
            particle_system_params: ParticleSystemParams::default(),
            ship_params: ShipParams::default(),
            level_params: LevelParams::default(),
        };
        let serialized = toml::to_string(&params).unwrap();
        println!("serialized = {}", serialized);
        let deserialized: GameParams = toml::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        assert_eq!(params.viewport_width, deserialized.viewport_width);
        assert_eq!(params.viewport_height, deserialized.viewport_height);
        assert_eq!(params.level_width, deserialized.level_width);
        assert_eq!(params.level_height, deserialized.level_height);
    }
}
