use std::f32::consts::PI;

#[path = "../examples/framework.rs"]
mod framework;

pub struct CameraMotion {
    pub angular_speed: f32,
    pub linear_speed: f32,
}

impl Default for CameraMotion {
    fn default() -> CameraMotion {
        CameraMotion {
            angular_speed: 1.0,
            linear_speed: 1.0,
        }
    }
}

pub struct CameraState {
    // Camera in spherical coordinates.
    pub phi: f32,    // Longitude.
    pub theta: f32,  // Latitude.
    pub radius: f32, // Radial distance.
    pub ortho: bool, // Orthographic projection.
}

impl Default for CameraState {
    fn default() -> CameraState {
        CameraState {
            phi: 0.0,
            theta: 0.0,
            radius: 5.0,
            ortho: true,
        }
    }
}

impl CameraState {
    pub fn pos(&self) -> cgmath::Point3<f32> {
        cgmath::Point3::new(
            self.radius * self.phi.cos() * self.theta.sin(),
            self.radius * self.phi.sin() * self.theta.sin(),
            self.radius * self.theta.cos(),
        )
    }

    pub fn up(&self) -> cgmath::Vector3<f32> {
        // Using spherical coordinates compute the vector in the global frame that corresponds to up in the camera's frame.
        let up_theta = self.theta - PI / 2.0;
        cgmath::Vector3::new(
            self.phi.cos() * up_theta.sin(),
            self.phi.sin() * up_theta.sin(),
            up_theta.cos(),
        )
    }

    pub fn update(
        &mut self,
        dt: f32,
        input_state: &crate::InputState,
        motion_params: &CameraMotion,
    ) {
        if input_state.cam_in && !input_state.cam_out {
            self.radius -= motion_params.linear_speed * dt;
        } else if !input_state.cam_in && input_state.cam_out {
            self.radius += motion_params.linear_speed * dt;
        }
        if input_state.cam_up && !input_state.cam_down {
            self.theta += motion_params.angular_speed * dt;
        } else if !input_state.cam_up && input_state.cam_down {
            self.theta -= motion_params.angular_speed * dt;
        }
        if input_state.cam_left && !input_state.cam_right {
            self.phi += motion_params.angular_speed * dt;
        } else if !input_state.cam_left && input_state.cam_right {
            self.phi -= motion_params.angular_speed * dt;
        }
        self.phi %= 2.0 * PI;
        if self.theta > PI {
            self.theta = PI;
        } else if self.theta < 0.0 {
            self.theta = 0.0;
        }
    }
}

pub struct Camera {
    pub motion_params: CameraMotion,
    pub screen_size: (u32, u32),
    pub state: CameraState,

    pub reset_pushed: bool,
    pub perspective_pushed: bool,
}

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            motion_params: CameraMotion::default(),
            screen_size: (640, 360),
            state: CameraState::default(),
            reset_pushed: bool::default(),
            perspective_pushed: bool::default(),
        }
    }
}

impl Camera {
    pub fn reset(&mut self) {
        self.state = CameraState::default();
    }

    /// Outputs 4x4 projection matrix and 4x4 view matrix
    pub fn to_uniform_data(&self) -> [f32; 16 * 2] {
        let aspect = self.screen_size.0 as f32 / self.screen_size.1 as f32;

        // pixel_pose_camera
        let mx_projection = if self.state.ortho {
            if aspect > 1.0 {
                // Wider than tall, fill height.
                cgmath::ortho(-aspect, aspect as _, -1.0, 1.0 as _, 1e-6, 500.0)
            } else {
                // Taller than wide, fill width.
                cgmath::ortho(-1.0, 1.0, -1.0 / aspect, 1.0 / aspect, 1e-6, 500.0)
            }
        } else {
            cgmath::perspective(cgmath::Deg(45f32), aspect, 1e-6, 500.0)
        };

        // camera_pose_world
        let cam_pos = self.state.pos();
        let cam_up = self.state.up();
        let mx_view =
            cgmath::Matrix4::look_at_rh(cam_pos, cgmath::Point3::new(0f32, 0.0, 0.0), cam_up);
        let proj = framework::OPENGL_TO_WGPU_MATRIX * mx_projection;
        let view = framework::OPENGL_TO_WGPU_MATRIX * mx_view;

        let mut raw = [0f32; 16 * 2];
        raw[..16].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj)[..]);
        raw[16..32].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&view)[..]);
        raw
    }

    pub fn update_state(&mut self, dt: f32, input_state: &crate::InputState) {
        self.state.update(dt, input_state, &self.motion_params);
        if input_state.cam_reset && !self.reset_pushed {
            self.reset();
        }
        self.reset_pushed = input_state.cam_reset;
        if input_state.cam_perspective && !self.perspective_pushed {
            self.state.ortho = !self.state.ortho;
            log::info!("Changing to ortho: {}", self.state.ortho);
        }
        self.perspective_pushed = input_state.cam_perspective;
    }
}
