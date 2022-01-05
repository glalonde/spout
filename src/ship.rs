use log::trace;
use int_grid;

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
    // pub emit_params: super::emitter::EmitParams,
}
impl Default for ShipState {
    fn default() -> Self {
        ShipState {
            position: [0, 0],
            velocity: [0, 0],
            orientation: 0.0,
            rotation_rate: 0.0,
            acceleration: 0.0,
        }
    }
}

impl ShipState {
    pub fn update(&mut self, dt: f32, accelerate: bool, rotation: RotationDirection) {
        // Update position.
        // self.emit_params.position_start = [self.position[0], self.position[1]];

        // Apparently it is important to cast through i32 before going to u32.
        // So this goes, -5.1 -> -5 -> INT_MAX - 4
        self.position[0] =
            self.position[0].wrapping_add(((dt * self.velocity[0] as f32) as i32) as u32);
        self.position[1] =
            self.position[1].wrapping_add(((dt * self.velocity[1] as f32) as i32) as u32);
        // self.emit_params.position_end = [self.position[0], self.position[1]];

        // Update velocity.
        // self.emit_params.velocity = [self.velocity[0], self.velocity[1]];
        if accelerate {
            let delta_v = [
                (dt * self.acceleration
                    * self.orientation.cos()
                    * (int_grid::cell_size() as f32)) as i32,
                (dt * self.acceleration
                    * self.orientation.sin()
                    * (int_grid::cell_size() as f32)) as i32,
            ];

            trace!("delta_v: {:?}", delta_v);
            trace!("acceleration: {:?}", self.velocity);
            self.velocity[0] += (dt
                * self.acceleration
                * self.orientation.cos()
                * (int_grid::cell_size() as f32)) as i32;
            self.velocity[1] += (dt
                * self.acceleration
                * self.orientation.sin()
                * (int_grid::cell_size() as f32)) as i32;
            trace!("acceleration: {:?}", self.velocity);
        }

        // Update orientation.
        let angle_delta = dt * (rotation as i8 as f32) * self.rotation_rate;
        // self.emit_params.angle_start = self.orientation;
        self.orientation += angle_delta;
        // self.emit_params.angle_end = self.orientation;
    }

}