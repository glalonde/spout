//! Pure scoring, timer, and level-progression math.

use std::time::Duration;

use crate::game_params;

pub fn level_time_limit_duration(params: &game_params::GameParams) -> Duration {
    let seconds = params.level_params.level_time_limit_seconds;
    if seconds.is_finite() && seconds > 0.0 {
        Duration::from_secs_f32(seconds)
    } else {
        Duration::from_secs_f32(game_params::LevelParams::default().level_time_limit_seconds)
    }
}

pub fn ceil_duration_seconds(duration: Duration) -> u64 {
    duration
        .as_secs()
        .saturating_add(u64::from(duration.subsec_nanos() > 0))
}

pub fn format_level_timer(remaining: Duration) -> String {
    let total_seconds = ceil_duration_seconds(remaining);
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes}:{seconds:02}")
}

pub fn level_time_award(params: &game_params::GameParams, levels_crossed: i32) -> Duration {
    let levels_crossed = levels_crossed.max(0) as u32;
    level_time_limit_duration(params)
        .checked_mul(levels_crossed)
        .unwrap_or(Duration::MAX)
}

pub fn level_index_for_progress(progress_height: i32, level_height: u32) -> i32 {
    if level_height == 0 {
        0
    } else {
        progress_height.max(0) / level_height as i32
    }
}

pub fn height_score(progress_height: i32) -> i32 {
    progress_height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_timer_display_rounds_up() {
        assert_eq!(format_level_timer(Duration::from_secs(120)), "2:00");
        assert_eq!(format_level_timer(Duration::from_millis(61_001)), "1:02");
        assert_eq!(format_level_timer(Duration::from_millis(1)), "0:01");
        assert_eq!(format_level_timer(Duration::ZERO), "0:00");
    }

    #[test]
    fn timer_display_uses_remaining_seconds_ceiling() {
        assert_eq!(ceil_duration_seconds(Duration::from_millis(1)), 1);
    }

    #[test]
    fn invalid_level_time_limit_falls_back_to_default() {
        let mut params = game_params::GameParams::default();
        params.level_params.level_time_limit_seconds = f32::NAN;
        assert_eq!(level_time_limit_duration(&params), Duration::from_secs(120));

        params.level_params.level_time_limit_seconds = -5.0;
        assert_eq!(level_time_limit_duration(&params), Duration::from_secs(120));
    }

    #[test]
    fn level_time_award_multiplies_configured_level_time() {
        let mut params = game_params::GameParams::default();
        params.level_params.level_time_limit_seconds = 30.0;

        assert_eq!(level_time_award(&params, 0), Duration::ZERO);
        assert_eq!(level_time_award(&params, 2), Duration::from_secs(60));
    }

    #[test]
    fn scoring_uses_progress_height() {
        assert_eq!(level_index_for_progress(-10, 100), 0);
        assert_eq!(level_index_for_progress(99, 100), 0);
        assert_eq!(level_index_for_progress(100, 100), 1);
        assert_eq!(level_index_for_progress(250, 100), 2);
        assert_eq!(height_score(105), 105);
    }
}
