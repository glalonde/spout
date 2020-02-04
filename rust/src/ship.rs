use log::{info, trace};

gflags::define! {
    --ship_acceleration: f32 = 100.0
}

gflags::define! {
    --ship_rotation_rate: f32 = 15.0
}

gflags::define! {
    --ship_emit_velocity: f32 = 100.0
}

gflags::define! {
    --ship_emit_velocity_spread: f32 = 0.5
}

#[repr(i8)]
#[derive(Copy, Clone)]
pub enum RotationDirection {
    CW = -1,
    None = 0,
    CCW = 1,
}

#[derive(Debug)]
pub struct ShipState {
    // This is the state in a kinematics sense, will move to the GPU eventually.
    pub position: [u32; 2],
    pub velocity: [i32; 2],
    pub orientation: f32,

    // The ship's control variables.
    pub rotation_rate: f32,
    pub acceleration: f32,

    // The ships's particle emitter
    pub emit_params: super::emitter::EmitParams,
}

impl ShipState {
    pub fn init(position: [u32; 2]) -> Self {
        ShipState {
            position: position,
            velocity: [0, 0],
            orientation: 0.0,
            rotation_rate: 15.0,
            acceleration: 100.0,
            emit_params: super::emitter::EmitParams::default(),
        }
    }

    pub fn init_from_flags(position: [u32; 2]) -> Self {
        let mut state = ShipState {
            position: position,
            velocity: [0, 0],
            orientation: 0.0,
            rotation_rate: SHIP_ROTATION_RATE.flag,
            acceleration: SHIP_ACCELERATION.flag,
            emit_params: super::emitter::EmitParams::default(),
        };
        state.emit_params.speed_min = SHIP_EMIT_VELOCITY.flag
            * (1.0 - SHIP_EMIT_VELOCITY_SPREAD.flag)
            * super::int_grid::cell_size() as f32;
        state.emit_params.speed_max = SHIP_EMIT_VELOCITY.flag
            * (1.0 + SHIP_EMIT_VELOCITY_SPREAD.flag)
            * super::int_grid::cell_size() as f32;
        state
    }

    pub fn update(&mut self, dt: f32, accelerate: bool, rotation: RotationDirection) {
        // Update position.
        self.emit_params.position_start = [self.position[0], self.position[1]];
        self.position[0] = self.position[0].wrapping_add((dt * self.velocity[0] as f32) as u32);
        self.position[1] = self.position[1].wrapping_add((dt * self.velocity[1] as f32) as u32);
        self.emit_params.position_end = [self.position[0], self.position[1]];
        info!(
            "Position:{}, {}",
            super::int_grid::get_values_relative(self.position[0])[0],
            super::int_grid::get_values_relative(self.position[1])[0]
        );

        // Update velocity.
        self.emit_params.velocity = [self.velocity[0], self.velocity[1]];
        if accelerate {
            info!("accelerationg: {:?}", self.velocity);
            self.velocity[0] += (dt
                * self.acceleration
                * self.orientation.cos()
                * (super::int_grid::cell_size() as f32)) as i32;
            self.velocity[1] += (dt
                * self.acceleration
                * self.orientation.sin()
                * (super::int_grid::cell_size() as f32)) as i32;
        }

        // Update orientation.
        let angle_delta = dt * (rotation as i8 as f32) * self.rotation_rate;
        self.emit_params.angle_start = self.orientation;
        self.orientation += angle_delta;
    }
}
