use serde::{Deserialize, Serialize};

// Parameters that define the game. These don't change at runtime.
#[derive(Debug, Serialize, Deserialize)]
pub struct GameParams {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub level_width: u32,
    pub level_height: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let params = GameParams {
            viewport_width: 1,
            viewport_height: 2,
            level_width: 3,
            level_height: 4,
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
