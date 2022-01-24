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

pub struct OrthoState {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

pub struct PerspectiveState {
    fov: f32,
}

pub struct CameraState {
    pub center: cgmath::Point3<f32>,

    // Camera in spherical coordinates.
    pub phi: f32,                              // Longitude.
    pub theta: f32,                            // Latitude.
    pub radius: f32,                           // Radial distance.
    pub ortho: Option<OrthoState>,             // Orthographic projection.
    pub perspective: Option<PerspectiveState>, // Orthographic projection.
}

impl Default for CameraState {
    fn default() -> CameraState {
        CameraState {
            center: cgmath::Point3::<f32>::new(0.0, 0.0, 0.0),
            phi: -PI / 2.0,
            theta: 0.0,
            radius: 1500.0,
            ortho: None,
            perspective: None,
        }
    }
}

impl CameraState {
    pub fn pos(&self) -> cgmath::Point3<f32> {
        cgmath::Point3::new(
            self.radius * self.phi.cos() * self.theta.sin(),
            self.radius * self.phi.sin() * self.theta.sin(),
            self.radius * self.theta.cos(),
        ) + cgmath::Vector3::<f32>::new(self.center.x, self.center.y, self.center.z)
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
            self.radius -= self.radius * (motion_params.linear_speed * dt);
        } else if !input_state.cam_in && input_state.cam_out {
            self.radius += self.radius * (motion_params.linear_speed * dt);
        }
        if input_state.cam_up && !input_state.cam_down {
            self.theta -= motion_params.angular_speed * dt;
        } else if !input_state.cam_up && input_state.cam_down {
            self.theta += motion_params.angular_speed * dt;
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
}

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            motion_params: CameraMotion::default(),

            // The size of the user's window output.
            screen_size: (640, 360),
            state: CameraState::default(),
        }
    }
}

impl Camera {
    /// Outputs 4x4 projection matrix and 4x4 view matrix
    pub fn to_uniform_data(&self) -> [f32; 16 * 2] {
        let aspect = self.screen_size.0 as f32 / self.screen_size.1 as f32;

        // pixel_pose_camera
        let mx_projection = if let Some(ortho_state) = &self.state.ortho {
            let target_width = ortho_state.right - ortho_state.left;
            let target_height = ortho_state.top - ortho_state.bottom;
            let target_aspect = target_width / target_height;
            if target_aspect > aspect {
                // Desired view is wider than actual, letter box on top and bottom.
                let required_height = target_width / aspect;
                let new_bottom = -required_height / 2.0;
                let new_top = required_height / 2.0;
                cgmath::ortho(
                    ortho_state.left,
                    ortho_state.right,
                    new_bottom,
                    new_top,
                    1e-6,
                    10000.0,
                )
            } else {
                // Desired view is taller than actual, letter box on left and right.
                let required_width = aspect * target_height;
                let new_left = -required_width / 2.0;
                let new_right = required_width / 2.0;
                cgmath::ortho(
                    new_left,
                    new_right,
                    ortho_state.bottom,
                    ortho_state.top,
                    1e-6,
                    10000.0,
                )
            }
        } else if let Some(perspective_state) = &self.state.perspective {
            cgmath::perspective(cgmath::Rad(perspective_state.fov), aspect, 1e-6, 10000.0)
        } else {
            cgmath::perspective(cgmath::Deg(45f32), aspect, 1e-6, 10000.0)
        };

        // camera_pose_world
        let cam_pos = self.state.pos();
        let cam_up = self.state.up();
        let mx_view = cgmath::Matrix4::look_at_rh(cam_pos, self.state.center, cam_up);
        let proj = framework::OPENGL_TO_WGPU_MATRIX * mx_projection;
        let view = framework::OPENGL_TO_WGPU_MATRIX * mx_view;

        let mut raw = [0f32; 16 * 2];
        raw[..16].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj)[..]);
        raw[16..32].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&view)[..]);
        raw
    }

    pub fn reset_orientation(&mut self) {
        self.state.phi = -PI / 2.0;
        self.state.theta = 0.0;
    }

    pub fn ortho_look_at(
        &mut self,
        center: [f32; 2],
        width: f32,
        height: f32,
        reset_orientation: bool,
    ) {
        let ortho_state = OrthoState {
            left: -width / 2.0,
            right: width / 2.0,
            bottom: -height / 2.0,
            top: height / 2.0,
        };

        // Phi is the position of the camera. The camera is oriented towards the origin, and thus is opposite of phi.
        // So to view with the Positive Y axis in the 'up' direction, we position the camera in the negative Y axis.
        if reset_orientation {
            self.reset_orientation();
        }
        self.state.ortho = Some(ortho_state);
        self.state.perspective = None;
        self.state.center = cgmath::Point3::<f32>::new(center[0], center[1], 0.0);
    }

    pub fn perspective_look_at(
        &mut self,
        center: [f32; 2],
        _width: f32,
        _height: f32,
        reset_orientation: bool,
    ) {
        let perspective_state = PerspectiveState { fov: PI / 4.0 };

        // TODO use the FOV to compute how far away we need to position the camera (radius) in order to fit the width/height into view.
        if reset_orientation {
            self.reset_orientation();
        }

        self.state.perspective = Some(perspective_state);
        self.state.ortho = None;
        self.state.center = cgmath::Point3::<f32>::new(center[0], center[1], 0.0);
    }

    pub fn update_state(&mut self, dt: f32, input_state: &crate::InputState) {
        self.state.update(dt, input_state, &self.motion_params);
    }
}
