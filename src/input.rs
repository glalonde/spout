#[cfg(test)]
mod tests {
    use super::*;

    // --- touch_delta_to_rotate ------------------------------------------------

    #[test]
    fn rotate_no_movement_is_zero() {
        // Finger has not moved from anchor → zero output.
        assert_eq!(touch_delta_to_rotate(100.0, 100.0), 0.0);
    }

    #[test]
    fn rotate_within_deadzone_is_zero() {
        // 3px left — below DEADZONE_PX (4px) → zero.
        assert_eq!(touch_delta_to_rotate(100.0, 97.0), 0.0);
        // 3px right
        assert_eq!(touch_delta_to_rotate(100.0, 103.0), 0.0);
    }

    #[test]
    fn rotate_left_is_ccw_positive() {
        // 30px left of anchor → full CCW (+1.0).
        assert_eq!(touch_delta_to_rotate(100.0, 70.0), 1.0);
    }

    #[test]
    fn rotate_right_is_cw_negative() {
        // 30px right of anchor → full CW (-1.0).
        assert_eq!(touch_delta_to_rotate(100.0, 130.0), -1.0);
    }

    #[test]
    fn rotate_beyond_full_range_clamped() {
        // Way beyond 30px → still clamped to ±1.0.
        assert_eq!(touch_delta_to_rotate(100.0, -999.0), 1.0);
        assert_eq!(touch_delta_to_rotate(100.0, 999.0), -1.0);
    }

    #[test]
    fn rotate_halfway_is_between_zero_and_one() {
        // ~17px left ≈ halfway through active range → between 0 and 1.
        let result = touch_delta_to_rotate(100.0, 83.0);
        assert!(result > 0.0 && result < 1.0, "got {result}");
    }

    // --- keyboard -------------------------------------------------------------

    #[test]
    fn keyboard_default_all_zero() {
        let state = InputCollector::default().current_state();
        assert_eq!(state.thrust, 0.0);
        assert_eq!(state.rotate, 0.0);
        assert!(!state.pause);
        assert!(!state.fullscreen);
    }

    #[test]
    fn keyboard_thrust() {
        let mut c = InputCollector::default();
        c.held_thrust = true;
        let state = c.current_state();
        assert_eq!(state.thrust, 1.0);
        assert_eq!(state.rotate, 0.0);
    }

    #[test]
    fn keyboard_rotate_left() {
        let mut c = InputCollector::default();
        c.held_left = true;
        let state = c.current_state();
        assert_eq!(state.thrust, 0.0);
        assert_eq!(state.rotate, 1.0);
    }

    #[test]
    fn keyboard_rotate_right() {
        let mut c = InputCollector::default();
        c.held_right = true;
        assert_eq!(c.current_state().rotate, -1.0);
    }

    #[test]
    fn keyboard_left_and_right_cancel() {
        let mut c = InputCollector::default();
        c.held_left = true;
        c.held_right = true;
        assert_eq!(c.current_state().rotate, 0.0);
    }

    #[test]
    fn keyboard_thrust_and_rotate_independent() {
        let mut c = InputCollector::default();
        c.held_thrust = true;
        c.held_left = true;
        let state = c.current_state();
        assert_eq!(state.thrust, 1.0);
        assert_eq!(state.rotate, 1.0);
    }

    // --- native touch (non-WASM only) ----------------------------------------
    //
    // With surface_width=200: left zone = [0,100), right zone = [100,200].
    // Right zone: center=150, half=50, so left edge (x=100) → +1.0, right (x=200) → -1.0.

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn touch_left_zone_activates_thrust() {
        let mut c = InputCollector::default();
        c.surface_width = 200.0;
        c.thrust_id = Some(1);
        let state = c.current_state();
        assert_eq!(state.thrust, 1.0);
        assert_eq!(state.rotate, 0.0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn touch_rotate_at_anchor_is_zero() {
        // Finger down, not moved → anchor == current → zero rotation.
        let mut c = InputCollector::default();
        c.surface_width = 200.0;
        c.rotate_id = Some(2);
        c.rotate_anchor_x = 150.0;
        c.rotate_x = 150.0;
        assert_eq!(c.current_state().rotate, 0.0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn touch_rotate_left_full_ccw() {
        // 30px left of anchor → full CCW (+1.0).
        let mut c = InputCollector::default();
        c.surface_width = 200.0;
        c.rotate_id = Some(2);
        c.rotate_anchor_x = 150.0;
        c.rotate_x = 120.0;
        assert_eq!(c.current_state().rotate, 1.0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn touch_rotate_right_full_cw() {
        // 30px right of anchor → full CW (-1.0).
        let mut c = InputCollector::default();
        c.surface_width = 200.0;
        c.rotate_id = Some(2);
        c.rotate_anchor_x = 150.0;
        c.rotate_x = 180.0;
        assert_eq!(c.current_state().rotate, -1.0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn touch_both_zones_independent() {
        // Thrust from left zone + full CCW rotation simultaneously.
        let mut c = InputCollector::default();
        c.surface_width = 200.0;
        c.thrust_id = Some(1);
        c.rotate_id = Some(2);
        c.rotate_anchor_x = 150.0;
        c.rotate_x = 120.0; // 30px left → full CCW
        let state = c.current_state();
        assert_eq!(state.thrust, 1.0);
        assert_eq!(state.rotate, 1.0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn touch_overrides_keyboard_per_axis() {
        let mut c = InputCollector::default();
        c.surface_width = 200.0;
        // Keyboard: rotate right, no thrust.
        c.held_right = true;
        // Touch: thrust active, no rotate touch.
        c.thrust_id = Some(1);
        let state = c.current_state();
        // Touch wins thrust, keyboard wins rotate.
        assert_eq!(state.thrust, 1.0);
        assert_eq!(state.rotate, -1.0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn no_touch_falls_back_to_keyboard() {
        let mut c = InputCollector::default();
        c.surface_width = 200.0;
        c.held_thrust = true;
        c.held_left = true;
        // No active touches → keyboard applies on both axes.
        let state = c.current_state();
        assert_eq!(state.thrust, 1.0);
        assert_eq!(state.rotate, 1.0);
    }
}

/// Maps touch displacement from the anchor (touch-down point) to a rotation scalar.
///
/// Moving left (negative delta) = CCW = positive; moving right = CW = negative.
/// `FULL_ROTATION_PX` pixels of travel reaches ±1.0. A small pixel deadzone
/// prevents drift when the finger is stationary.
fn touch_delta_to_rotate(anchor_x: f32, current_x: f32) -> f32 {
    /// Pixels of travel from anchor → full rotation speed.
    const FULL_ROTATION_PX: f32 = 30.0;
    /// Pixels below which output is zero (prevents drift at rest).
    const DEADZONE_PX: f32 = 4.0;
    let delta = anchor_x - current_x; // left of anchor → positive → CCW
    let abs = delta.abs();
    if abs < DEADZONE_PX {
        0.0
    } else {
        (delta.signum() * (abs - DEADZONE_PX) / (FULL_ROTATION_PX - DEADZONE_PX)).clamp(-1.0, 1.0)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct InputState {
    pub thrust: f32, // [0.0, 1.0]
    pub rotate: f32, // [-1.0, 1.0]; positive = CCW/left, negative = CW/right

    pub pause: bool,
    pub fullscreen: bool,

    // Camera controls (debug, keyboard-only):
    pub cam_in: bool,
    pub cam_out: bool,
    pub cam_up: bool,
    pub cam_down: bool,
    pub cam_left: bool,
    pub cam_right: bool,
    pub cam_perspective: bool,
    pub cam_reset: bool,
}

// --- Touch layout -------------------------------------------------------
//
// Screen is split vertically at center (landscape orientation assumed):
//   Left half  → thrust zone: any touch here fires the thruster.
//   Right half → rotate zone: horizontal position within the half controls
//                rotation rate (left edge of half = full CCW, right = full CW,
//                center of half = 0 with a small deadzone).
//
// Two simultaneous touches (one per zone) are supported so rotation and thrust
// are fully independent.
// ------------------------------------------------------------------------

/// Touch state shared between JS event listeners and the game loop (WASM only).
#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct WasmTouch {
    canvas_width: f32, // CSS px, refreshed each event
    thrust_id: Option<i32>,
    rotate_id: Option<i32>,
    rotate_anchor_x: f32, // x at touchstart — the zero point for delta steering
    rotate_x: f32,        // x at latest touchmove
}

/// Accumulates raw platform events and produces a logical [`InputState`] each frame.
///
/// Keyboard is handled via `handle_winit_event` on all platforms.
/// Touch is handled via `handle_winit_event` on native (winit relays
/// `WindowEvent::Touch`) and via DOM listeners registered in `init_touch` on WASM.
pub struct InputCollector {
    // Keyboard held-key state
    held_thrust: bool,
    held_left: bool,
    held_right: bool,
    held_pause: bool,
    held_fullscreen: bool,
    held_cam_in: bool,
    held_cam_out: bool,
    held_cam_up: bool,
    held_cam_down: bool,
    held_cam_left: bool,
    held_cam_right: bool,
    held_cam_perspective: bool,
    held_cam_reset: bool,

    // Native touch (updated via winit WindowEvent::Touch; IDs are winit's u64).
    // surface_width (physical px) must be kept current via set_surface_width().
    #[cfg(not(target_arch = "wasm32"))]
    surface_width: f32,
    #[cfg(not(target_arch = "wasm32"))]
    thrust_id: Option<u64>,
    #[cfg(not(target_arch = "wasm32"))]
    rotate_id: Option<u64>,
    #[cfg(not(target_arch = "wasm32"))]
    rotate_anchor_x: f32, // x at touchstart — the zero point for delta steering
    #[cfg(not(target_arch = "wasm32"))]
    rotate_x: f32,

    // WASM touch (shared with JS closures via Rc; closures are forgotten and kept
    // alive by the DOM for the lifetime of the page).
    #[cfg(target_arch = "wasm32")]
    wasm_touch: std::rc::Rc<std::cell::RefCell<WasmTouch>>,
}

impl Default for InputCollector {
    fn default() -> Self {
        InputCollector {
            held_thrust: false,
            held_left: false,
            held_right: false,
            held_pause: false,
            held_fullscreen: false,
            held_cam_in: false,
            held_cam_out: false,
            held_cam_up: false,
            held_cam_down: false,
            held_cam_left: false,
            held_cam_right: false,
            held_cam_perspective: false,
            held_cam_reset: false,
            #[cfg(not(target_arch = "wasm32"))]
            surface_width: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            thrust_id: None,
            #[cfg(not(target_arch = "wasm32"))]
            rotate_id: None,
            #[cfg(not(target_arch = "wasm32"))]
            rotate_anchor_x: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            rotate_x: 0.0,
            #[cfg(target_arch = "wasm32")]
            wasm_touch: std::rc::Rc::new(std::cell::RefCell::new(WasmTouch::default())),
        }
    }
}

impl InputCollector {
    /// Update the surface width used for zone-split calculation (native only).
    ///
    /// Call once at init and again on every resize. On WASM the canvas width is
    /// read inside each touch event, so this is not needed there.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_surface_width(&mut self, width: f32) {
        self.surface_width = width;
    }

    /// Register DOM touch event listeners on the game canvas (WASM only).
    ///
    /// Must be called once after the canvas element is available. Closures are
    /// forgotten (kept alive by the DOM) for the lifetime of the page — this is
    /// intentional for a single-page game.
    #[cfg(target_arch = "wasm32")]
    pub fn init_touch(&mut self, canvas: web_sys::HtmlCanvasElement) {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        // touchstart: claim each new touch for whichever zone it lands in.
        {
            let state = std::rc::Rc::clone(&self.wasm_touch);
            let canvas_ref = canvas.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |event: web_sys::TouchEvent| {
                event.prevent_default();
                let canvas_width = canvas_ref.client_width() as f32;
                if canvas_width <= 0.0 {
                    return;
                }
                let center = canvas_width / 2.0;
                let mut s = state.borrow_mut();
                s.canvas_width = canvas_width;
                let changed = event.changed_touches();
                for i in 0..changed.length() {
                    if let Some(touch) = changed.get(i) {
                        let x = touch.client_x() as f32;
                        let id = touch.identifier();
                        if x < center {
                            if s.thrust_id.is_none() {
                                s.thrust_id = Some(id);
                            }
                        } else if s.rotate_id.is_none() {
                            s.rotate_id = Some(id);
                            s.rotate_anchor_x = x;
                            s.rotate_x = x;
                        }
                    }
                }
            });
            canvas
                .add_event_listener_with_callback("touchstart", cb.as_ref().unchecked_ref())
                .unwrap();
            cb.forget();
        }

        // touchmove: update rotate_x if the rotate touch moved.
        {
            let state = std::rc::Rc::clone(&self.wasm_touch);
            let cb = Closure::<dyn FnMut(_)>::new(move |event: web_sys::TouchEvent| {
                event.prevent_default();
                let mut s = state.borrow_mut();
                let changed = event.changed_touches();
                for i in 0..changed.length() {
                    if let Some(touch) = changed.get(i) {
                        if Some(touch.identifier()) == s.rotate_id {
                            s.rotate_x = touch.client_x() as f32;
                        }
                    }
                }
            });
            canvas
                .add_event_listener_with_callback("touchmove", cb.as_ref().unchecked_ref())
                .unwrap();
            cb.forget();
        }

        // touchend + touchcancel: release whichever zone(s) ended.
        {
            let state = std::rc::Rc::clone(&self.wasm_touch);
            let cb = Closure::<dyn FnMut(_)>::new(move |event: web_sys::TouchEvent| {
                event.prevent_default();
                let mut s = state.borrow_mut();
                let changed = event.changed_touches();
                for i in 0..changed.length() {
                    if let Some(touch) = changed.get(i) {
                        let id = touch.identifier();
                        if Some(id) == s.thrust_id {
                            s.thrust_id = None;
                        }
                        if Some(id) == s.rotate_id {
                            s.rotate_id = None;
                            s.rotate_anchor_x = 0.0;
                            s.rotate_x = 0.0;
                        }
                    }
                }
            });
            let f = cb.as_ref().unchecked_ref();
            canvas
                .add_event_listener_with_callback("touchend", f)
                .unwrap();
            canvas
                .add_event_listener_with_callback("touchcancel", f)
                .unwrap();
            cb.forget();
        }
    }

    pub fn handle_winit_event(&mut self, event: &winit::event::WindowEvent) {
        use winit::keyboard::{KeyCode, PhysicalKey};
        if let winit::event::WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    physical_key: PhysicalKey::Code(key),
                    state,
                    ..
                },
            ..
        } = event
        {
            let pressed = *state == winit::event::ElementState::Pressed;
            match key {
                // Ship motion
                KeyCode::KeyW => self.held_thrust = pressed,
                KeyCode::KeyA => self.held_left = pressed,
                KeyCode::KeyD => self.held_right = pressed,
                KeyCode::KeyP => self.held_pause = pressed,

                // Camera
                KeyCode::KeyU => self.held_cam_in = pressed,
                KeyCode::KeyO => self.held_cam_out = pressed,
                KeyCode::KeyI => self.held_cam_up = pressed,
                KeyCode::KeyK => self.held_cam_down = pressed,
                KeyCode::KeyJ => self.held_cam_left = pressed,
                KeyCode::KeyL => self.held_cam_right = pressed,
                KeyCode::KeyN => self.held_cam_perspective = pressed,
                KeyCode::KeyM => self.held_cam_reset = pressed,

                // Misc
                KeyCode::KeyF => self.held_fullscreen = pressed,

                _ => {}
            }
        }

        // Native touch: winit relays WindowEvent::Touch on platforms with touch
        // support (iOS, Android, touchscreen desktops). Not available on WASM —
        // handled via DOM listeners registered in init_touch().
        #[cfg(not(target_arch = "wasm32"))]
        if let winit::event::WindowEvent::Touch(touch) = event {
            use winit::event::TouchPhase;
            let center = self.surface_width / 2.0;
            let x = touch.location.x as f32;
            match touch.phase {
                TouchPhase::Started => {
                    if x < center {
                        if self.thrust_id.is_none() {
                            self.thrust_id = Some(touch.id);
                        }
                    } else if self.rotate_id.is_none() {
                        self.rotate_id = Some(touch.id);
                        self.rotate_anchor_x = x;
                        self.rotate_x = x;
                    }
                }
                TouchPhase::Moved => {
                    if Some(touch.id) == self.rotate_id {
                        self.rotate_x = x;
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if Some(touch.id) == self.thrust_id {
                        self.thrust_id = None;
                    }
                    if Some(touch.id) == self.rotate_id {
                        self.rotate_id = None;
                        self.rotate_anchor_x = 0.0;
                        self.rotate_x = 0.0;
                    }
                }
            }
        }
    }

    pub fn current_state(&self) -> InputState {
        let keyboard_thrust = if self.held_thrust { 1.0 } else { 0.0 };
        let keyboard_rotate = match (self.held_left, self.held_right) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => 0.0,
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (touch_thrust, touch_rotate) = {
            let thrust = self.thrust_id.is_some();
            let rotate = self
                .rotate_id
                .map(|_| touch_delta_to_rotate(self.rotate_anchor_x, self.rotate_x));
            (thrust, rotate)
        };

        #[cfg(target_arch = "wasm32")]
        let (touch_thrust, touch_rotate) = {
            let s = self.wasm_touch.borrow();
            let thrust = s.thrust_id.is_some();
            let rotate = s
                .rotate_id
                .map(|_| touch_delta_to_rotate(s.rotate_anchor_x, s.rotate_x));
            (thrust, rotate)
        };

        // Touch and keyboard are independent per axis: touch takes priority on
        // whichever axis has an active touch; keyboard fills the other.
        let thrust = if touch_thrust { 1.0 } else { keyboard_thrust };
        let rotate = touch_rotate.unwrap_or(keyboard_rotate);

        InputState {
            thrust,
            rotate,
            pause: self.held_pause,
            fullscreen: self.held_fullscreen,
            cam_in: self.held_cam_in,
            cam_out: self.held_cam_out,
            cam_up: self.held_cam_up,
            cam_down: self.held_cam_down,
            cam_left: self.held_cam_left,
            cam_right: self.held_cam_right,
            cam_perspective: self.held_cam_perspective,
            cam_reset: self.held_cam_reset,
        }
    }
}
