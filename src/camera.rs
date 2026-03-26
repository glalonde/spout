//! Orbital camera with orthographic projection for the game viewport.

use std::f32::consts::PI;

#[allow(clippy::duplicate_mod)]
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
    pub center: glam::Vec3,

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
            center: glam::Vec3::ZERO,
            phi: -PI / 2.0,
            theta: 0.0,
            radius: 1500.0,
            ortho: None,
            perspective: None,
        }
    }
}

impl CameraState {
    pub fn pos(&self) -> glam::Vec3 {
        glam::Vec3::new(
            self.radius * self.phi.cos() * self.theta.sin(),
            self.radius * self.phi.sin() * self.theta.sin(),
            self.radius * self.theta.cos(),
        ) + self.center
    }

    pub fn up(&self) -> glam::Vec3 {
        // Using spherical coordinates compute the vector in the global frame that corresponds to up in the camera's frame.
        let up_theta = self.theta - PI / 2.0;
        glam::Vec3::new(
            self.phi.cos() * up_theta.sin(),
            self.phi.sin() * up_theta.sin(),
            up_theta.cos(),
        )
    }

    pub fn update(
        &mut self,
        dt: f32,
        input_state: &crate::input::InputState,
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
        self.theta = self.theta.clamp(0.0, PI);
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
                glam::Mat4::orthographic_rh_gl(
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
                glam::Mat4::orthographic_rh_gl(
                    new_left,
                    new_right,
                    ortho_state.bottom,
                    ortho_state.top,
                    1e-6,
                    10000.0,
                )
            }
        } else if let Some(perspective_state) = &self.state.perspective {
            glam::Mat4::perspective_rh_gl(perspective_state.fov, aspect, 1e-6, 10000.0)
        } else {
            glam::Mat4::perspective_rh_gl(45f32.to_radians(), aspect, 1e-6, 10000.0)
        };

        // camera_pose_world
        let cam_pos = self.state.pos();
        let cam_up = self.state.up();
        let mx_view = glam::Mat4::look_at_rh(cam_pos, self.state.center, cam_up);
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
        self.state.center = glam::Vec3::new(center[0], center[1], 0.0);
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
        self.state.center = glam::Vec3::new(center[0], center[1], 0.0);
    }

    pub fn update_state(&mut self, dt: f32, input_state: &crate::input::InputState) {
        self.state.update(dt, input_state, &self.motion_params);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    // pos() relative to center — same as the position vector from center to camera.
    fn pos_vec(state: &CameraState) -> glam::Vec3 {
        state.pos() - state.center
    }

    #[test]
    fn pos_at_north_pole() {
        // theta=0 is the "north pole" in spherical coords: camera sits directly above center on +Z axis.
        let state = CameraState {
            phi: 0.0,
            theta: 0.0,
            radius: 100.0,
            center: glam::Vec3::ZERO,
            ortho: None,
            perspective: None,
        };
        let pos = state.pos();
        // sin(0) = 0, cos(0) = 1, so x=y=0, z=radius
        assert!(approx_eq(pos.x, 0.0), "x={}", pos.x);
        assert!(approx_eq(pos.y, 0.0), "y={}", pos.y);
        assert!(approx_eq(pos.z, 100.0), "z={}", pos.z);
    }

    #[test]
    fn pos_along_positive_x_axis() {
        // phi=0, theta=PI/2: camera on +X axis, looking at origin
        let state = CameraState {
            phi: 0.0,
            theta: PI / 2.0,
            radius: 100.0,
            center: glam::Vec3::ZERO,
            ortho: None,
            perspective: None,
        };
        let pos = state.pos();
        assert!(approx_eq(pos.x, 100.0), "x={}", pos.x);
        assert!(approx_eq(pos.y, 0.0), "y={}", pos.y);
        assert!(approx_eq(pos.z, 0.0), "z={}", pos.z);
    }

    #[test]
    fn pos_radius_is_preserved() {
        // At any angle, |pos - center| should equal radius.
        for &(phi, theta) in &[
            (0.0_f32, 0.1_f32),
            (1.0, 1.2),
            (-PI / 3.0, PI / 4.0),
            (PI, PI / 6.0),
        ] {
            let state = CameraState {
                phi,
                theta,
                radius: 250.0,
                center: glam::Vec3::new(10.0, -5.0, 0.0),
                ortho: None,
                perspective: None,
            };
            let len = pos_vec(&state).length();
            // Use relative tolerance: trig at f32 precision accumulates ~1e-6 relative error.
            assert!(
                (len - 250.0).abs() < 250.0 * 1e-5,
                "phi={phi} theta={theta}: |pos-center|={len}"
            );
        }
    }

    #[test]
    fn up_is_unit_vector() {
        for &(phi, theta) in &[
            (0.0_f32, 0.1_f32),
            (1.0, 1.2),
            (-PI / 3.0, PI / 4.0),
            (PI / 2.0, PI / 3.0),
        ] {
            let state = CameraState {
                phi,
                theta,
                ..Default::default()
            };
            let len = state.up().length();
            assert!(approx_eq(len, 1.0), "phi={phi} theta={theta}: |up|={len}");
        }
    }

    #[test]
    fn up_is_orthogonal_to_look_direction() {
        // The up vector should be perpendicular to the view direction (pos→center).
        for &(phi, theta) in &[
            (0.0_f32, 0.1_f32),
            (1.0, 1.2),
            (-PI / 3.0, PI / 4.0),
            (PI / 2.0, PI / 3.0),
        ] {
            let state = CameraState {
                phi,
                theta,
                radius: 100.0,
                center: glam::Vec3::ZERO,
                ortho: None,
                perspective: None,
            };
            // Look direction = center - pos
            let look = -pos_vec(&state);
            let dot = look.dot(state.up());
            assert!(
                approx_eq(dot, 0.0),
                "phi={phi} theta={theta}: look·up={dot} (expected ~0)"
            );
        }
    }

    #[test]
    fn to_uniform_data_no_nan_or_inf() {
        // Smoke test: ortho and perspective modes both produce finite matrix data.
        let mut cam = Camera::default();
        cam.ortho_look_at([0.0, 0.0], 1920.0, 1080.0, true);
        let data = cam.to_uniform_data();
        assert!(
            data.iter().all(|v| v.is_finite()),
            "ortho mode produced non-finite values"
        );

        let mut cam = Camera::default();
        cam.perspective_look_at([0.0, 0.0], 1920.0, 1080.0, true);
        let data = cam.to_uniform_data();
        assert!(
            data.iter().all(|v| v.is_finite()),
            "perspective mode produced non-finite values"
        );
    }

    #[test]
    fn ortho_look_at_sets_center() {
        let mut cam = Camera::default();
        cam.ortho_look_at([100.0, 200.0], 1920.0, 1080.0, true);
        assert!(
            approx_eq(cam.state.center.x, 100.0),
            "x={}",
            cam.state.center.x
        );
        assert!(
            approx_eq(cam.state.center.y, 200.0),
            "y={}",
            cam.state.center.y
        );
        assert!(
            approx_eq(cam.state.center.z, 0.0),
            "z={}",
            cam.state.center.z
        );
        assert!(cam.state.ortho.is_some());
        assert!(cam.state.perspective.is_none());
    }
}
