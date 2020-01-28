#[derive(Debug)]
pub struct FpsEstimator {
    last_tick_time: std::time::Instant,
    iteration_duration: std::time::Duration,
}

static NATIVE_SLEEP_ACCURACY: std::time::Duration = std::time::Duration::from_micros(500);

impl FpsEstimator {
    pub fn new(fps: f64) -> FpsEstimator {
        FpsEstimator {
            last_tick_time: std::time::Instant::now(),
            iteration_duration: std::time::Duration::from_secs_f64(1.0 / fps),
        }
    }

    fn high_resolution_sleep_for(duration: &std::time::Duration) {
        // Accuracy of rust sleep on my machine is .0005 max oversleep and no
        // undersleep.
        if duration > &NATIVE_SLEEP_ACCURACY {
            let start = std::time::Instant::now();
            std::thread::sleep(*duration - NATIVE_SLEEP_ACCURACY);
            while &start.elapsed() < duration {}
        }
    }

    pub fn tick(&mut self) -> f64 {
        let elapsed = self.last_tick_time.elapsed();
        let maybe_delta = self.iteration_duration.checked_sub(elapsed);
        if maybe_delta.is_some() {
            FpsEstimator::high_resolution_sleep_for(&maybe_delta.unwrap());
        }
        let dt = self.last_tick_time.elapsed();
        self.last_tick_time = std::time::Instant::now();
        dt.as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Get some stats about std::thread::sleep
    #[test]
    fn sleep_test() {
        let target: f64 = 1.0 / 60.0;
        let mut max_err: f64 = 0.0;
        let mut avg_overshoot_err: f64 = 0.0;
        let mut avg_undershoot_err: f64 = 0.0;
        let mut overshoot = 0;
        let mut undershoot = 0;
        for _ in 0..1000 {
            let now = std::time::Instant::now();
            std::thread::sleep(std::time::Duration::from_secs_f64(target));
            let elapsed = now.elapsed();
            let actual = elapsed.as_secs_f64();
            let err = actual - target;
            if err >= 0.0 {
                avg_overshoot_err += err.abs();
                overshoot += 1;
            } else {
                avg_undershoot_err += err.abs();
                undershoot += 1;
            }
            if err > max_err {
                max_err = err;
            }
        }
        avg_overshoot_err = avg_overshoot_err / (overshoot as f64);
        avg_undershoot_err = avg_undershoot_err / (undershoot as f64);
        println!(
            "Max: {}, Avg Over: {}, Avg Under: {}",
            max_err, avg_overshoot_err, avg_undershoot_err
        );
    }
}
