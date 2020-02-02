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
    pub velocity: nalgebra::Vector2<i32>,
    pub orientation: nalgebra::UnitComplex<f32>,

    // The ship's control variables.
    pub rotation_rate: f32,
    pub acceleration: f32,

    // The ships's particle emitter
    pub emit_params: super::emitter::EmitParams,
}

impl ShipState {
    pub fn update(&mut self, dt: f32, accelerate: bool, rotation: RotationDirection) {
        let angle_delta = dt * (rotation as i8 as f32) * self.rotation_rate;
        self.orientation *= nalgebra::Rotation2::new(angle_delta);
        let dv: nalgebra::Vector2<i32> = nalgebra::try_convert(
            nalgebra::Vector2::<f32>::new(
                self.orientation.cos_angle(),
                self.orientation.sin_angle(),
            ) * dt
                * self.acceleration
                * (accelerate as i32 as f32),
        )
        .unwrap_or(nalgebra::Vector2::<i32>::new(0, 0));
        self.velocity += dv;
    }
}
