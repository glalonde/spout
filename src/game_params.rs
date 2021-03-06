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

    #[serde(default)]
    pub particle_system_params: ParticleSystemParams,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ParticleSystemParams {
    pub emission_rate: f32,
    pub max_particle_life: f32,
    pub damage_rate: f32,
    pub gravity: f32,
    pub elasticity: f32,
}

impl Default for ParticleSystemParams {
    fn default() -> Self {
        ParticleSystemParams {
            emission_rate: 100000.0,
            max_particle_life: 2.0,
            damage_rate: 0.00001,
            gravity: -5.0,
            elasticity: 0.5,
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
            particle_system_params: ParticleSystemParams::default(),
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
            particle_system_params: ParticleSystemParams::default(),
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
