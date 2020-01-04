use log::info;

struct EmitterParams {
    num_particles: u32,
    emit_period: f32,
}

pub struct Emitter {
    params: EmitterParams,
    time: f32,
    emit_progress: f32,
    write_index: u32,
}

impl Emitter {
    pub fn new(num_particles: u32, emission_frequency: f32) -> Self {
        Emitter {
            params: EmitterParams {
                num_particles: num_particles,
                emit_period: 1.0 / emission_frequency,
            },
            time: 0.0,
            emit_progress: 0.0,
            write_index: 0,
        }
    }
    pub fn emit_over_time(&mut self, dt: f32) {
        self.time += dt;
        self.emit_progress += dt;
        if self.emit_progress > self.params.emit_period {
            let num_emitted: u32 = (self.emit_progress / self.params.emit_period) as u32;
            self.emit_progress -= (num_emitted as f32) * self.params.emit_period;
            self.emit(num_emitted);
        }
    }

    fn emit(&mut self, num_emitted: u32) {
        info!("Emitting {} particles", num_emitted);
        self.write_index = (self.write_index + num_emitted) % self.params.num_particles;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emitter_test() {
        scrub_log::init().unwrap();
        let mut e = Emitter::new(100000, 30.0);
        e.emit_over_time(1.0 / 60.0);
    }
}
