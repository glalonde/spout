#[repr(i8)]
#[derive(Copy, Clone)]
pub enum RotationDirection {
    CW = -1,
    None = 0,
    CCW = 1,
}

#[derive(Debug, Copy, Clone)]
pub struct ShipState {
    // This is the state in a kinematics sense, will move to the GPU eventually.
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub orientation: f32,

    // The ship's control variables.
    pub rotation_rate: f32,
    pub acceleration: f32,
}
impl Default for ShipState {
    fn default() -> Self {
        ShipState {
            position: [0.0, 0.0],
            velocity: [0.0, 0.0],
            orientation: 0.0,
            rotation_rate: 15.0,
            acceleration: 1.0,
        }
    }
}

impl ShipState {
    pub fn update(&mut self, dt: f32, accelerate: bool, rotation: RotationDirection) {
        self.position[0] += dt * self.velocity[0];
        self.position[1] += dt * self.velocity[1];

        if accelerate {
            self.velocity[0] += dt * self.acceleration * self.orientation.cos();
            self.velocity[1] += dt * self.acceleration * self.orientation.sin();
        }

        let angle_delta = dt * (rotation as i8 as f32) * self.rotation_rate;
        self.orientation += angle_delta;
    }
}
