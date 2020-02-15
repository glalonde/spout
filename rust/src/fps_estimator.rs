#[derive(Debug)]
pub struct FpsEstimator {
    iteration_start: std::time::Instant,
    iteration_duration: std::time::Duration,
    fps: f64,
}

static NATIVE_SLEEP_ACCURACY: std::time::Duration = std::time::Duration::from_micros(500);

impl FpsEstimator {
    pub fn new(fps: f64) -> FpsEstimator {
        FpsEstimator {
            iteration_start: std::time::Instant::now(),
            iteration_duration: std::time::Duration::from_secs_f64(1.0 / fps),
            fps: 0.0,
        }
    }

    fn high_resolution_sleep_until(done: &std::time::Instant) {
        let now = std::time::Instant::now();
        let system_sleep_until = done.checked_sub(NATIVE_SLEEP_ACCURACY).unwrap_or(now);
        if now < system_sleep_until {
            std::thread::sleep(system_sleep_until.duration_since(now));
        }
        while *done > std::time::Instant::now() {}
    }

    pub fn fps(&self) -> f64 {
        self.fps
    }

    pub fn tick(&mut self) -> f64 {
        FpsEstimator::high_resolution_sleep_until(
            &(self.iteration_start + self.iteration_duration),
        );
        let dt = self.iteration_start.elapsed().as_secs_f64();
        self.iteration_start = std::time::Instant::now();
        self.fps = 1.0 / dt;
        dt
    }
}

#[cfg(test)]
mod tests {

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
