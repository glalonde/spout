//! Pure scoring, timer, and level-progression math.

use std::time::Duration;

use crate::game_params;

const TIME_BONUS_SCORE_PER_SECOND: i32 = 10;

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

pub fn time_bonus_score(remaining: Duration) -> i32 {
    let seconds = ceil_duration_seconds(remaining).min(i32::MAX as u64) as i32;
    seconds.saturating_mul(TIME_BONUS_SCORE_PER_SECOND)
}

pub fn level_index_for_progress(progress_height: i32, level_height: u32) -> i32 {
    if level_height == 0 {
        0
    } else {
        progress_height.max(0) / level_height as i32
    }
}

pub fn combined_score(progress_height: i32, time_bonus_score: i32) -> i32 {
    progress_height.saturating_add(time_bonus_score)
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
    fn time_bonus_uses_remaining_seconds_ceiling() {
        assert_eq!(ceil_duration_seconds(Duration::from_millis(1)), 1);
        assert_eq!(time_bonus_score(Duration::from_millis(1)), 10);
        assert_eq!(time_bonus_score(Duration::from_secs(42)), 420);
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
    fn level_index_uses_progress_height_not_bonus_score() {
        assert_eq!(level_index_for_progress(-10, 100), 0);
        assert_eq!(level_index_for_progress(99, 100), 0);
        assert_eq!(level_index_for_progress(100, 100), 1);
        assert_eq!(level_index_for_progress(250, 100), 2);
        assert_eq!(combined_score(105, 900), 1005);
    }
}
