#[path = "../examples/framework.rs"]
mod framework;

pub struct CameraMotion {
    pub angular_speed: f32,
    pub vertical_speed: f32,
}

pub struct Camera {
    pub motion_params: CameraMotion,
    pub screen_size: (u32, u32),
    // Camera in cylindrical coordinates.
    pub phi: f32,
    pub radius: f32,
    pub height: f32,
}

impl Camera {
    /// Outputs 4x4 projection matrix and 4x4 view matrix
    pub fn to_uniform_data(&self) -> [f32; 16 * 2] {
        let aspect = self.screen_size.0 as f32 / self.screen_size.1 as f32;

        // pixel_pose_camera
        let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect, 1.0, 50.0);

        let cam_pos = cgmath::Point3::new(
            self.phi.cos() * self.radius,
            self.phi.sin() * self.radius,
            self.height,
        );

        // camera_pose_world
        let mx_view = cgmath::Matrix4::look_at_rh(
            cam_pos,
            cgmath::Point3::new(0f32, 0.0, 0.0),
            cgmath::Vector3::unit_z(),
        );
        let proj = framework::OPENGL_TO_WGPU_MATRIX * mx_projection;
        let view = framework::OPENGL_TO_WGPU_MATRIX * mx_view;

        let mut raw = [0f32; 16 * 2];
        raw[..16].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj)[..]);
        raw[16..32].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&view)[..]);
        raw
    }

    pub fn update_state(&mut self, dt: f32, input_state: &crate::InputState) {
        if input_state.cam_up && !input_state.cam_down {
            self.height += self.motion_params.vertical_speed * dt;
        } else if !input_state.cam_up && input_state.cam_down {
            self.height -= self.motion_params.vertical_speed * dt;
        }
        if input_state.cam_left && !input_state.cam_right {
            self.phi += self.motion_params.angular_speed * dt;
        } else if !input_state.cam_left && input_state.cam_right {
            self.phi -= self.motion_params.angular_speed * dt;
        }
        // self.radius
    }
}
