#[derive(Debug, Copy, Clone, Default)]
pub struct InputState {
    pub forward: bool,
    pub left: bool,
    pub right: bool,
    pub pause: bool,

    // Camera controls:
    pub cam_in: bool,
    pub cam_out: bool,
    pub cam_up: bool,
    pub cam_down: bool,
    pub cam_left: bool,
    pub cam_right: bool,

    pub cam_perspective: bool,
    pub cam_reset: bool,

    pub fullscreen: bool,
}
