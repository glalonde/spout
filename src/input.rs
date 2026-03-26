#[cfg(test)]
mod tests {
    use super::*;

    // --- touch_delta_to_target_heading ----------------------------------------

    const PI: f32 = std::f32::consts::PI;
    const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;

    #[test]
    fn heading_no_drag_is_none() {
        // Finger hasn't moved from anchor → no heading.
        assert_eq!(
            touch_delta_to_target_heading(100.0, 100.0, 100.0, 100.0),
            None
        );
    }

    #[test]
    fn heading_small_drag_is_none() {
        // 5px drag — below MIN_DRAG_PX (8px) → no heading.
        assert_eq!(
            touch_delta_to_target_heading(100.0, 100.0, 105.0, 100.0),
            None
        );
    }

    #[test]
    fn heading_drag_right_is_zero() {
        // Drag right → angle 0.
        let h = touch_delta_to_target_heading(100.0, 100.0, 130.0, 100.0).unwrap();
        assert!((h - 0.0).abs() < 1e-5, "got {h}");
    }

    #[test]
    fn heading_drag_up_is_pi_over_2() {
        // Drag up in screen (y decreases) → angle π/2 in game coords.
        let h = touch_delta_to_target_heading(100.0, 100.0, 100.0, 70.0).unwrap();
        assert!((h - FRAC_PI_2).abs() < 1e-5, "got {h}");
    }

    #[test]
    fn heading_drag_down_is_neg_pi_over_2() {
        // Drag down in screen (y increases) → angle -π/2 in game coords.
        let h = touch_delta_to_target_heading(100.0, 100.0, 100.0, 130.0).unwrap();
        assert!((h - (-FRAC_PI_2)).abs() < 1e-5, "got {h}");
    }

    #[test]
    fn heading_drag_left_is_pi() {
        // Drag left → angle π.
        let h = touch_delta_to_target_heading(100.0, 100.0, 70.0, 100.0).unwrap();
        assert!((h.abs() - PI).abs() < 1e-5, "got {h}");
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
    fn touch_rotate_at_anchor_heading_none() {
        // Finger down, not moved → within deadzone → no target heading.
        let mut c = InputCollector::default();
        c.surface_width = 200.0;
        c.rotate_id = Some(2);
        c.rotate_anchor_x = 150.0;
        c.rotate_anchor_y = 100.0;
        c.rotate_x = 150.0;
        c.rotate_y = 100.0;
        let state = c.current_state();
        assert_eq!(state.target_heading, None);
        assert_eq!(state.rotate, 0.0); // touch suppresses keyboard
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn touch_rotate_drag_right_heading_zero() {
        // Drag 30px right → target heading ~0 (pointing right).
        let mut c = InputCollector::default();
        c.surface_width = 200.0;
        c.rotate_id = Some(2);
        c.rotate_anchor_x = 150.0;
        c.rotate_anchor_y = 100.0;
        c.rotate_x = 180.0;
        c.rotate_y = 100.0;
        let h = c.current_state().target_heading.unwrap();
        assert!((h - 0.0).abs() < 1e-4, "got {h}");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn touch_rotate_drag_up_heading_pi_over_2() {
        // Drag 30px up (screen y decreases) → target heading π/2 (game up).
        let mut c = InputCollector::default();
        c.surface_width = 200.0;
        c.rotate_id = Some(2);
        c.rotate_anchor_x = 150.0;
        c.rotate_anchor_y = 100.0;
        c.rotate_x = 150.0;
        c.rotate_y = 70.0;
        let h = c.current_state().target_heading.unwrap();
        assert!((h - FRAC_PI_2).abs() < 1e-4, "got {h}");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn touch_both_zones_independent() {
        // Thrust + target heading simultaneously.
        let mut c = InputCollector::default();
        c.surface_width = 200.0;
        c.thrust_id = Some(1);
        c.rotate_id = Some(2);
        c.rotate_anchor_x = 150.0;
        c.rotate_anchor_y = 100.0;
        c.rotate_x = 180.0; // 30px right → heading ~0
        c.rotate_y = 100.0;
        let state = c.current_state();
        assert_eq!(state.thrust, 1.0);
        assert!(state.target_heading.is_some());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn touch_thrust_keyboard_rotate_independent() {
        // Touch thrust active, no rotate touch → keyboard rotate still applies.
        let mut c = InputCollector::default();
        c.surface_width = 200.0;
        c.held_right = true;
        c.thrust_id = Some(1);
        let state = c.current_state();
        assert_eq!(state.thrust, 1.0);
        assert_eq!(state.rotate, -1.0); // keyboard rotate active, no touch rotate
        assert_eq!(state.target_heading, None);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn touch_rotate_suppresses_keyboard_rotate() {
        // Touch in rotate zone (even in deadzone) → keyboard rotate suppressed.
        let mut c = InputCollector::default();
        c.surface_width = 200.0;
        c.held_right = true;
        c.rotate_id = Some(2);
        c.rotate_anchor_x = 150.0;
        c.rotate_anchor_y = 100.0;
        c.rotate_x = 150.0; // no drag → heading None
        c.rotate_y = 100.0;
        let state = c.current_state();
        assert_eq!(state.rotate, 0.0); // keyboard suppressed
        assert_eq!(state.target_heading, None); // still in deadzone
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

/// Minimum drag distance (px) before a touch heading is committed.
/// Also used to detect "taps" (touchend with displacement below this threshold).
const MIN_DRAG_PX: f32 = 8.0;

/// Converts a 2-D touch drag into an absolute target heading in radians.
///
/// Returns `None` when the drag is too small to reliably determine a direction
/// (prevents jitter immediately after touch-down).  Screen y is negated before
/// `atan2` because screen coordinates increase downward while game-world y
/// increases upward.
///
/// Result uses the standard math convention:
///   0 = right,  π/2 = up,  ±π = left,  -π/2 = down.
fn touch_delta_to_target_heading(
    anchor_x: f32,
    anchor_y: f32,
    current_x: f32,
    current_y: f32,
) -> Option<f32> {
    let drag = glam::Vec2::new(current_x - anchor_x, current_y - anchor_y);
    if drag.length_squared() < MIN_DRAG_PX * MIN_DRAG_PX {
        None
    } else {
        // Negate y: screen y increases downward, game y increases upward.
        Some(drag.with_y(-drag.y).to_angle())
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct InputState {
    pub thrust: f32, // [0.0, 1.0]
    pub rotate: f32, // [-1.0, 1.0]; positive = CCW/left, negative = CW/right (keyboard only)
    /// Absolute target heading in radians from touch input (standard math convention:
    /// 0=right, π/2=up). `None` when touch is not controlling rotation.
    /// When `Some`, the caller should use a bang-bang controller instead of `rotate`.
    pub target_heading: Option<f32>,

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
    rotate_anchor_x: f32, // x at touchstart
    rotate_anchor_y: f32, // y at touchstart
    rotate_x: f32,        // x at latest touchmove/touchstart
    rotate_y: f32,        // y at latest touchmove/touchstart

    // Accelerometer state (updated by deviceorientation listener)
    last_gamma: f32,         // latest DeviceOrientationEvent.gamma (degrees)
    last_beta: f32,          // latest DeviceOrientationEvent.beta (degrees)
    accel_offset_gamma: f32, // calibration offset (set on right-side tap)
    accel_offset_beta: f32,
    accel_heading: Option<f32>, // computed heading from accel tilt
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
    rotate_anchor_x: f32, // x at touchstart
    #[cfg(not(target_arch = "wasm32"))]
    rotate_anchor_y: f32, // y at touchstart
    #[cfg(not(target_arch = "wasm32"))]
    rotate_x: f32, // x at latest touchmove/touchstart
    #[cfg(not(target_arch = "wasm32"))]
    rotate_y: f32, // y at latest touchmove/touchstart

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
            rotate_anchor_y: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            rotate_x: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            rotate_y: 0.0,
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
                            let y = touch.client_y() as f32;
                            s.rotate_id = Some(id);
                            s.rotate_anchor_x = x;
                            s.rotate_anchor_y = y;
                            s.rotate_x = x;
                            s.rotate_y = y;
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
                            s.rotate_y = touch.client_y() as f32;
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
                            // Tap detection: if finger lifted without dragging,
                            // reset the accelerometer calibration offset.
                            let drag = glam::Vec2::new(
                                s.rotate_x - s.rotate_anchor_x,
                                s.rotate_y - s.rotate_anchor_y,
                            );
                            if drag.length_squared() < MIN_DRAG_PX * MIN_DRAG_PX {
                                s.accel_offset_gamma = s.last_gamma;
                                s.accel_offset_beta = s.last_beta;
                            }
                            s.rotate_id = None;
                            s.rotate_anchor_x = 0.0;
                            s.rotate_anchor_y = 0.0;
                            s.rotate_x = 0.0;
                            s.rotate_y = 0.0;
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

        // deviceorientation: map phone tilt to target heading.
        // Fires on Android/desktop without permission; requires explicit
        // requestPermission() on iOS 13+ (not yet implemented — touch still works).
        {
            let state = std::rc::Rc::clone(&self.wasm_touch);
            let cb = Closure::<dyn FnMut(_)>::new(move |event: web_sys::DeviceOrientationEvent| {
                let gamma_deg = event.gamma().unwrap_or(0.0) as f32;
                let beta_deg = event.beta().unwrap_or(0.0) as f32;
                let mut s = state.borrow_mut();
                s.last_gamma = gamma_deg;
                s.last_beta = beta_deg;

                // Subtract calibration offset, convert to radians.
                let gamma_rad = (gamma_deg - s.accel_offset_gamma).to_radians();
                let beta_rad = (beta_deg - s.accel_offset_beta).to_radians();

                // Landscape-left mapping: game-right = beta, game-up = gamma.
                let tilt = glam::Vec2::new(beta_rad, gamma_rad);
                const MIN_TILT_RAD: f32 = 0.1; // ~6° deadzone when nearly flat
                s.accel_heading = if tilt.length_squared() < MIN_TILT_RAD * MIN_TILT_RAD {
                    None
                } else {
                    Some(tilt.to_angle())
                };
            });
            if let Some(window) = web_sys::window() {
                // The unwrap is safe: Window always implements EventTarget.
                window
                    .add_event_listener_with_callback(
                        "deviceorientation",
                        cb.as_ref().unchecked_ref(),
                    )
                    .unwrap();
            }
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
                    let y = touch.location.y as f32;
                    if x < center {
                        if self.thrust_id.is_none() {
                            self.thrust_id = Some(touch.id);
                        }
                    } else if self.rotate_id.is_none() {
                        self.rotate_id = Some(touch.id);
                        self.rotate_anchor_x = x;
                        self.rotate_anchor_y = y;
                        self.rotate_x = x;
                        self.rotate_y = y;
                    }
                }
                TouchPhase::Moved => {
                    if Some(touch.id) == self.rotate_id {
                        self.rotate_x = x;
                        self.rotate_y = touch.location.y as f32;
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if Some(touch.id) == self.thrust_id {
                        self.thrust_id = None;
                    }
                    if Some(touch.id) == self.rotate_id {
                        self.rotate_id = None;
                        self.rotate_anchor_x = 0.0;
                        self.rotate_anchor_y = 0.0;
                        self.rotate_x = 0.0;
                        self.rotate_y = 0.0;
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
        let (touch_thrust, touch_has_rotate, touch_heading) = {
            let thrust = self.thrust_id.is_some();
            let (has_rotate, heading) = if self.rotate_id.is_some() {
                let h = touch_delta_to_target_heading(
                    self.rotate_anchor_x,
                    self.rotate_anchor_y,
                    self.rotate_x,
                    self.rotate_y,
                );
                (true, h)
            } else {
                (false, None)
            };
            (thrust, has_rotate, heading)
        };

        #[cfg(target_arch = "wasm32")]
        let (touch_thrust, touch_has_rotate, touch_heading) = {
            let s = self.wasm_touch.borrow();
            let thrust = s.thrust_id.is_some();
            let (has_rotate, heading) = if s.rotate_id.is_some() {
                // Touch drag takes highest priority.
                let h = touch_delta_to_target_heading(
                    s.rotate_anchor_x,
                    s.rotate_anchor_y,
                    s.rotate_x,
                    s.rotate_y,
                );
                (true, h)
            } else if s.accel_heading.is_some() {
                // Accelerometer provides heading when no rotate touch is active.
                (true, s.accel_heading)
            } else {
                (false, None)
            };
            (thrust, has_rotate, heading)
        };

        // Touch owns its axis entirely; keyboard fills the other.
        let thrust = if touch_thrust { 1.0 } else { keyboard_thrust };
        // When touch is in the rotate zone (even within the drag deadzone), it
        // suppresses keyboard rotation so the two don't fight.
        let rotate = if touch_has_rotate {
            0.0
        } else {
            keyboard_rotate
        };
        let target_heading = touch_heading;

        InputState {
            thrust,
            rotate,
            target_heading,
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
