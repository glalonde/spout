//! Input abstraction: keyboard and touch → unified `InputState`.
//! Supports desktop/mobile native input through winit and mobile web touch through
//! DOM listeners, with touch drag producing an absolute-angle heading.

use crate::game_params::TouchControlScheme;

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
    fn heading_drag_right_jetstream_right_nose_left() {
        // Drag right → jetstream goes right → ship nose points left (π).
        let h = touch_delta_to_target_heading(100.0, 100.0, 130.0, 100.0).unwrap();
        assert!((h.abs() - PI).abs() < 1e-5, "got {h}");
    }

    #[test]
    fn heading_drag_up_jetstream_up_nose_down() {
        // Drag up (screen y decreases) → jetstream up → nose down (-π/2).
        let h = touch_delta_to_target_heading(100.0, 100.0, 100.0, 70.0).unwrap();
        assert!((h - (-FRAC_PI_2)).abs() < 1e-5, "got {h}");
    }

    #[test]
    fn heading_drag_down_jetstream_down_nose_up() {
        // Drag down (screen y increases) → jetstream down → nose up (π/2).
        let h = touch_delta_to_target_heading(100.0, 100.0, 100.0, 130.0).unwrap();
        assert!((h - FRAC_PI_2).abs() < 1e-5, "got {h}");
    }

    #[test]
    fn heading_drag_left_jetstream_left_nose_right() {
        // Drag left → jetstream left → nose right (0).
        let h = touch_delta_to_target_heading(100.0, 100.0, 70.0, 100.0).unwrap();
        assert!((h - 0.0).abs() < 1e-5, "got {h}");
    }

    // --- keyboard -------------------------------------------------------------

    #[test]
    fn keyboard_default_all_zero() {
        let state = InputCollector::default().current_state();
        assert_eq!(state.thrust, 0.0);
        assert_eq!(state.rotate, 0.0);
        assert!(!state.pause);
        assert!(!state.fullscreen);
        assert!(!state.restart);
        assert!(!state.audio_next_track);
        assert!(!state.audio_toggle);
        assert!(!state.menu_up);
        assert!(!state.menu_down);
        assert!(!state.menu_left);
        assert!(!state.menu_right);
        assert!(!state.menu_confirm);
        assert!(!state.menu_cancel);
        assert!(!state.touch_started);
        assert!(state.pointer_pressed.is_none());
        assert!(state.pointer_released.is_none());
    }

    #[test]
    fn input_frame_reports_edges() {
        let current = InputState {
            thrust: 1.0,
            rotate: -1.0,
            restart: true,
            touch_started: true,
            pointer_pressed: Some(PointerPress { x: 12.0, y: 34.0 }),
            pointer_released: Some(PointerPress { x: 56.0, y: 78.0 }),
            help: true,
            audio_next_track: true,
            audio_toggle: true,
            menu_up: true,
            menu_down: true,
            menu_left: true,
            menu_right: true,
            menu_confirm: true,
            menu_cancel: true,
            pause: true,
            fullscreen: true,
            ..Default::default()
        };
        let frame = InputFrame::new(current, InputState::default());

        assert!(frame.pause_pressed());
        assert!(frame.fullscreen_pressed());
        assert!(frame.restart_pressed());
        assert!(frame.help_pressed());
        assert!(frame.audio_next_track_pressed());
        assert!(frame.audio_toggle_pressed());
        assert!(frame.menu_up_pressed());
        assert!(frame.menu_down_pressed());
        assert!(frame.menu_left_pressed());
        assert!(frame.menu_right_pressed());
        assert!(frame.menu_confirm_pressed());
        assert!(frame.menu_cancel_pressed());
        assert!(frame.touch_started());
        assert!(frame.pointer_pressed().is_some());
        assert!(frame.pointer_released().is_some());
        assert!(frame.thrust_started());
        assert!(frame.rotate_started());
    }

    #[test]
    fn input_frame_ignores_held_values() {
        let current = InputState {
            thrust: 1.0,
            rotate: 1.0,
            pause: true,
            fullscreen: true,
            audio_next_track: true,
            audio_toggle: true,
            menu_up: true,
            menu_down: true,
            menu_left: true,
            menu_right: true,
            menu_confirm: true,
            menu_cancel: true,
            ..Default::default()
        };
        let previous = current;
        let frame = InputFrame::new(current, previous);

        assert!(!frame.pause_pressed());
        assert!(!frame.fullscreen_pressed());
        assert!(!frame.audio_next_track_pressed());
        assert!(!frame.audio_toggle_pressed());
        assert!(!frame.menu_up_pressed());
        assert!(!frame.menu_down_pressed());
        assert!(!frame.menu_left_pressed());
        assert!(!frame.menu_right_pressed());
        assert!(!frame.menu_confirm_pressed());
        assert!(!frame.menu_cancel_pressed());
        assert!(!frame.thrust_started());
        assert!(!frame.rotate_started());
    }

    #[test]
    fn keyboard_audio_actions_are_one_shot() {
        let mut c = InputCollector::default();
        c.audio_next_track_requested = true;
        c.audio_toggle_requested = true;

        let first = c.current_state();
        assert!(first.audio_next_track);
        assert!(first.audio_toggle);

        let second = c.current_state();
        assert!(!second.audio_next_track);
        assert!(!second.audio_toggle);
    }

    #[test]
    fn pointer_press_and_release_are_one_shot() {
        let mut c = InputCollector::default();
        c.pointer_press = Some(PointerPress { x: 12.0, y: 34.0 });
        c.pointer_release = Some(PointerPress { x: 56.0, y: 78.0 });

        let first = c.current_state();
        assert_eq!(first.pointer_pressed.map(|point| point.x), Some(12.0));
        assert_eq!(first.pointer_released.map(|point| point.y), Some(78.0));

        let second = c.current_state();
        assert!(second.pointer_pressed.is_none());
        assert!(second.pointer_released.is_none());
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

    #[cfg(not(target_arch = "wasm32"))]
    mod touch_tests {
        use super::*;

        // --- touch ------------------------------------------------------------
        //
        // With surface_width=200: left zone = [0,100), right zone = [100,200].
        // Right zone: center=150, half=50, so left edge (x=100) → +1.0, right (x=200) → -1.0.

        fn touch_collector(width: f32, height: f32) -> InputCollector {
            let mut c = InputCollector::default();
            c.touch.set_surface_width(width);
            c.touch.set_surface_height(height);
            c
        }

        #[test]
        fn touch_left_zone_activates_thrust() {
            let mut c = touch_collector(200.0, 100.0);
            c.touch.started(1, 10.0, 50.0);
            let state = c.current_state();
            assert_eq!(state.thrust, 1.0);
            assert_eq!(state.rotate, 0.0);
        }

        #[test]
        fn touch_end_reports_pointer_release_once() {
            let mut c = touch_collector(200.0, 100.0);
            c.touch.started(1, 10.0, 50.0);
            let _ = c.current_state();
            c.touch.ended(1, 12.0, 52.0);

            let first = c.current_state();
            assert_eq!(first.pointer_released.map(|point| point.x), Some(12.0));

            let second = c.current_state();
            assert!(second.pointer_released.is_none());
        }

        #[test]
        fn touch_rotate_at_anchor_heading_none() {
            // Finger down, not moved → within deadzone → no target heading.
            let mut c = touch_collector(200.0, 100.0);
            c.touch.started(2, 150.0, 50.0);
            let state = c.current_state();
            assert_eq!(state.target_heading, None);
            assert_eq!(state.rotate, 0.0); // touch suppresses keyboard
        }

        #[test]
        fn touch_rotate_drag_right_nose_left() {
            // Drag 30px right → jetstream right → nose left (π).
            let mut c = touch_collector(200.0, 100.0);
            c.touch.started(2, 150.0, 50.0);
            c.touch.moved(2, 180.0, 50.0);
            let h = c.current_state().target_heading.unwrap();
            assert!((h.abs() - PI).abs() < 1e-4, "got {h}");
        }

        #[test]
        fn touch_rotate_drag_up_nose_down() {
            // Drag 30px up (screen y decreases) → jetstream up → nose down (-π/2).
            let mut c = touch_collector(200.0, 100.0);
            c.touch.started(2, 150.0, 50.0);
            c.touch.moved(2, 150.0, 20.0);
            let h = c.current_state().target_heading.unwrap();
            assert!((h - (-FRAC_PI_2)).abs() < 1e-4, "got {h}");
        }

        #[test]
        fn touch_both_zones_independent() {
            // Thrust + target heading simultaneously.
            let mut c = touch_collector(200.0, 100.0);
            c.touch.started(1, 10.0, 50.0);
            c.touch.started(2, 150.0, 50.0);
            c.touch.moved(2, 180.0, 50.0);
            let state = c.current_state();
            assert_eq!(state.thrust, 1.0);
            assert!(state.target_heading.is_some());
        }

        #[test]
        fn touch_thrust_keyboard_rotate_independent() {
            // Touch thrust active, no rotate touch → keyboard rotate still applies.
            let mut c = touch_collector(200.0, 100.0);
            c.held_right = true;
            c.touch.started(1, 10.0, 50.0);
            let state = c.current_state();
            assert_eq!(state.thrust, 1.0);
            assert_eq!(state.rotate, -1.0); // keyboard rotate active, no touch rotate
            assert_eq!(state.target_heading, None);
        }

        #[test]
        fn touch_rotate_suppresses_keyboard_rotate() {
            // Touch in rotate zone (even in deadzone) → keyboard rotate suppressed.
            let mut c = touch_collector(200.0, 100.0);
            c.held_right = true;
            c.touch.started(2, 150.0, 50.0);
            let state = c.current_state();
            assert_eq!(state.rotate, 0.0); // keyboard suppressed
            assert_eq!(state.target_heading, None); // still in deadzone
        }

        #[test]
        fn no_touch_falls_back_to_keyboard() {
            let mut c = touch_collector(200.0, 100.0);
            c.held_thrust = true;
            c.held_left = true;
            // No active touches → keyboard applies on both axes.
            let state = c.current_state();
            assert_eq!(state.thrust, 1.0);
            assert_eq!(state.rotate, 1.0);
        }

        // --- triangle scheme ------------------------------------------------------
        //
        // surface: 400×300. center_x=200. Diagonal from (200,0) to (400,300).
        // CW condition: y * 200 < 300 * (x - 200)
        //
        // Upper-right triangle (CW): e.g. (380, 10) → 10*200=2000 < 300*180=54000 ✓
        // Lower-left triangle (CCW): e.g. (210, 280) → 280*200=56000 ≥ 300*10=3000 ✓

        fn triangle_collector() -> InputCollector {
            let mut c = touch_collector(400.0, 300.0);
            c.touch_scheme = TouchControlScheme::Triangle;
            c
        }

        fn set_right_touch(c: &mut InputCollector, x: f32, y: f32) {
            c.touch.started(1, x, y);
        }

        #[test]
        fn triangle_upper_right_is_cw() {
            // Upper-right area → CW → rotate = -1.0, no target heading.
            let mut c = triangle_collector();
            set_right_touch(&mut c, 380.0, 10.0);
            let state = c.current_state();
            assert_eq!(state.rotate, -1.0);
            assert_eq!(state.target_heading, None);
        }

        #[test]
        fn triangle_lower_left_is_ccw() {
            // Lower-left area of right half → CCW → rotate = +1.0.
            let mut c = triangle_collector();
            set_right_touch(&mut c, 210.0, 280.0);
            let state = c.current_state();
            assert_eq!(state.rotate, 1.0);
            assert_eq!(state.target_heading, None);
        }

        #[test]
        fn triangle_drag_across_diagonal_switches_direction() {
            // Same touch ID; moving from CW zone to CCW zone updates rotate.
            let mut c = triangle_collector();
            set_right_touch(&mut c, 380.0, 10.0); // CW
            assert_eq!(c.current_state().rotate, -1.0);
            c.touch.moved(1, 210.0, 280.0);
            assert_eq!(c.current_state().rotate, 1.0);
        }

        #[test]
        fn triangle_suppresses_keyboard_when_active() {
            // Any right-half touch suppresses keyboard rotation.
            let mut c = triangle_collector();
            set_right_touch(&mut c, 380.0, 10.0);
            c.held_left = true;
            let state = c.current_state();
            assert_eq!(state.rotate, -1.0); // triangle wins, not keyboard
        }

        #[test]
        fn triangle_thrust_independent() {
            // Left-half thrust + right-half CW touch → both active simultaneously.
            let mut c = triangle_collector();
            c.touch.started(2, 10.0, 100.0);
            set_right_touch(&mut c, 380.0, 10.0);
            let state = c.current_state();
            assert_eq!(state.thrust, 1.0);
            assert_eq!(state.rotate, -1.0);
        }

        #[test]
        fn triangle_no_touch_no_rotate() {
            // No right-half touch → no rotation, keyboard applies.
            let mut c = triangle_collector();
            c.held_left = true;
            let state = c.current_state();
            assert_eq!(state.rotate, 1.0); // keyboard
        }
    }
}

/// Minimum drag distance (px) before a touch heading is committed.
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
        // Negate the whole vector: the input direction is where the exhaust/
        // jetstream goes; the ship nose points opposite.
        let game_drag = drag.with_y(-drag.y);
        Some((-game_drag).to_angle())
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct PointerPress {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct InputState {
    pub thrust: f32, // [0.0, 1.0]
    pub rotate: f32, // [-1.0, 1.0]; positive = CCW/left, negative = CW/right (keyboard only)
    /// Absolute target heading in radians from touch input (standard math convention:
    /// 0=right, π/2=up). `None` when touch is not controlling rotation.
    /// When `Some`, the caller should use a bang-bang controller instead of `rotate`.
    pub target_heading: Option<f32>,

    pub restart: bool,
    pub touch_started: bool,
    pub pointer_pressed: Option<PointerPress>,
    pub pointer_released: Option<PointerPress>,
    pub help: bool,
    pub audio_next_track: bool,
    pub audio_toggle: bool,

    pub pause: bool,
    pub fullscreen: bool,

    // Menu controls (keyboard/gamepad-style, edge-triggered by InputFrame):
    pub menu_up: bool,
    pub menu_down: bool,
    pub menu_left: bool,
    pub menu_right: bool,
    pub menu_confirm: bool,
    pub menu_cancel: bool,

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

/// Current + previous input snapshots for edge-triggered actions.
///
/// This keeps consumers from open-coding `current && !previous` checks, and
/// gives future menu/gamepad navigation a single place to grow semantic input
/// intents without changing every screen.
#[derive(Debug, Copy, Clone, Default)]
pub struct InputFrame {
    pub current: InputState,
    pub previous: InputState,
}

impl InputFrame {
    pub fn new(current: InputState, previous: InputState) -> Self {
        Self { current, previous }
    }

    pub fn pause_pressed(&self) -> bool {
        self.current.pause && !self.previous.pause
    }

    pub fn fullscreen_pressed(&self) -> bool {
        self.current.fullscreen && !self.previous.fullscreen
    }

    pub fn restart_pressed(&self) -> bool {
        self.current.restart && !self.previous.restart
    }

    pub fn help_pressed(&self) -> bool {
        self.current.help && !self.previous.help
    }

    pub fn audio_next_track_pressed(&self) -> bool {
        self.current.audio_next_track && !self.previous.audio_next_track
    }

    pub fn audio_toggle_pressed(&self) -> bool {
        self.current.audio_toggle && !self.previous.audio_toggle
    }

    pub fn menu_up_pressed(&self) -> bool {
        self.current.menu_up && !self.previous.menu_up
    }

    pub fn menu_down_pressed(&self) -> bool {
        self.current.menu_down && !self.previous.menu_down
    }

    pub fn menu_left_pressed(&self) -> bool {
        self.current.menu_left && !self.previous.menu_left
    }

    pub fn menu_right_pressed(&self) -> bool {
        self.current.menu_right && !self.previous.menu_right
    }

    pub fn menu_confirm_pressed(&self) -> bool {
        self.current.menu_confirm && !self.previous.menu_confirm
    }

    pub fn menu_cancel_pressed(&self) -> bool {
        self.current.menu_cancel && !self.previous.menu_cancel
    }

    pub fn touch_started(&self) -> bool {
        self.current.touch_started
    }

    pub fn pointer_pressed(&self) -> Option<PointerPress> {
        self.current.pointer_pressed
    }

    pub fn pointer_released(&self) -> Option<PointerPress> {
        self.current.pointer_released
    }

    pub fn thrust_started(&self) -> bool {
        self.current.thrust > 0.0 && self.previous.thrust == 0.0
    }

    pub fn rotate_started(&self) -> bool {
        self.current.rotate.abs() > 0.0 && self.previous.rotate.abs() == 0.0
    }
}

// --- Touch layout -------------------------------------------------------
//
// Screen is split vertically at center (landscape orientation assumed):
//   Left half  → thrust zone: any touch here fires the thruster.
//   Right half → rotate zone (scheme-dependent, see TouchControlScheme):
//     Drag:     drag from anchor sets an absolute target heading.
//     Triangle: diagonal from (W/2,0)→(W,H) splits CW (upper-right) from
//               CCW (lower-left); direction follows current touch position.
//
// Two simultaneous touches (one per zone) are supported so rotation and
// thrust are fully independent.
// ------------------------------------------------------------------------

type TouchId = i64;

#[derive(Debug, Copy, Clone, Default)]
enum TouchRotation {
    #[default]
    Inactive,
    /// A rotate-zone touch is active, but it has not produced a direction yet.
    Neutral,
    Digital(f32),
    Heading(f32),
}

#[derive(Debug, Copy, Clone, Default)]
struct TouchInput {
    thrust: bool,
    rotation: TouchRotation,
}

#[derive(Debug, Default)]
struct TouchTracker {
    surface_width: f32,
    surface_height: f32,
    thrust_id: Option<TouchId>,
    rotate_id: Option<TouchId>,
    rotate_anchor_x: f32,
    rotate_anchor_y: f32,
    rotate_x: f32,
    rotate_y: f32,
    touch_started: bool,
    touch_started_x: f32,
    touch_started_y: f32,
    touch_ended: bool,
    touch_ended_x: f32,
    touch_ended_y: f32,
    /// Sticky "any touch event has occurred this session" flag — used to
    /// detect that the player is on a touch device so we can show the
    /// touch-zone hint. Never resets after the first touch.
    ever_touched: bool,
}

impl TouchTracker {
    fn set_surface_width(&mut self, width: f32) {
        self.surface_width = width;
    }

    fn set_surface_height(&mut self, height: f32) {
        self.surface_height = height;
    }

    fn started(&mut self, id: TouchId, x: f32, y: f32) {
        self.touch_started = true;
        self.touch_started_x = x;
        self.touch_started_y = y;
        self.ever_touched = true;
        if self.surface_width <= 0.0 || self.surface_height <= 0.0 {
            return;
        }

        let center = self.surface_width / 2.0;
        if x < center {
            if self.thrust_id.is_none() {
                self.thrust_id = Some(id);
            }
        } else if self.rotate_id.is_none() {
            self.rotate_id = Some(id);
            self.rotate_anchor_x = x;
            self.rotate_anchor_y = y;
            self.rotate_x = x;
            self.rotate_y = y;
        }
    }

    fn moved(&mut self, id: TouchId, x: f32, y: f32) {
        if Some(id) == self.rotate_id {
            self.rotate_x = x;
            self.rotate_y = y;
        }
    }

    fn ended(&mut self, id: TouchId, x: f32, y: f32) {
        self.touch_ended = true;
        self.touch_ended_x = x;
        self.touch_ended_y = y;
        if Some(id) == self.thrust_id {
            self.thrust_id = None;
        }
        if Some(id) == self.rotate_id {
            self.rotate_id = None;
            self.rotate_anchor_x = 0.0;
            self.rotate_anchor_y = 0.0;
            self.rotate_x = 0.0;
            self.rotate_y = 0.0;
        }
    }

    fn consume_touch_ended(&mut self) -> Option<PointerPress> {
        let ended = self.touch_ended;
        self.touch_ended = false;
        if ended {
            Some(PointerPress {
                x: self.touch_ended_x,
                y: self.touch_ended_y,
            })
        } else {
            None
        }
    }

    fn consume_touch_started(&mut self) -> Option<PointerPress> {
        let started = self.touch_started;
        self.touch_started = false;
        if started {
            Some(PointerPress {
                x: self.touch_started_x,
                y: self.touch_started_y,
            })
        } else {
            None
        }
    }

    fn current_input(&self, scheme: TouchControlScheme) -> TouchInput {
        let thrust = self.thrust_id.is_some();
        let rotation = match scheme {
            TouchControlScheme::Drag => {
                if self.rotate_id.is_some() {
                    touch_delta_to_target_heading(
                        self.rotate_anchor_x,
                        self.rotate_anchor_y,
                        self.rotate_x,
                        self.rotate_y,
                    )
                    .map_or(TouchRotation::Neutral, TouchRotation::Heading)
                } else {
                    TouchRotation::Inactive
                }
            }
            TouchControlScheme::Triangle => {
                if self.rotate_id.is_some() {
                    // Diagonal from (W/2, 0) to (W, H) splits the right half.
                    // A point is CW (upper-right triangle) when:
                    //   y * (W/2) < H * (x - W/2)
                    let rx = self.rotate_x - self.surface_width / 2.0;
                    let is_cw =
                        self.rotate_y * (self.surface_width / 2.0) < self.surface_height * rx;
                    let rotate = if is_cw { -1.0_f32 } else { 1.0_f32 };
                    TouchRotation::Digital(rotate)
                } else {
                    TouchRotation::Inactive
                }
            }
        };

        TouchInput { thrust, rotation }
    }
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
    restart_requested: bool,
    help_requested: bool,
    audio_next_track_requested: bool,
    audio_toggle_requested: bool,
    held_menu_up: bool,
    held_menu_down: bool,
    held_menu_left: bool,
    held_menu_right: bool,
    held_menu_confirm: bool,
    held_menu_cancel: bool,
    pointer_press: Option<PointerPress>,
    pointer_release: Option<PointerPress>,
    cursor_x: f32,
    cursor_y: f32,

    touch_scheme: TouchControlScheme,

    #[cfg(not(target_arch = "wasm32"))]
    touch: TouchTracker,

    // WASM touch (shared with JS closures via Rc; closures are forgotten and kept
    // alive by the DOM for the lifetime of the page).
    #[cfg(target_arch = "wasm32")]
    wasm_touch: std::rc::Rc<std::cell::RefCell<TouchTracker>>,
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
            restart_requested: false,
            help_requested: false,
            audio_next_track_requested: false,
            audio_toggle_requested: false,
            held_menu_up: false,
            held_menu_down: false,
            held_menu_left: false,
            held_menu_right: false,
            held_menu_confirm: false,
            held_menu_cancel: false,
            pointer_press: None,
            pointer_release: None,
            cursor_x: 0.0,
            cursor_y: 0.0,
            touch_scheme: TouchControlScheme::Drag,
            #[cfg(not(target_arch = "wasm32"))]
            touch: TouchTracker::default(),
            #[cfg(target_arch = "wasm32")]
            wasm_touch: std::rc::Rc::new(std::cell::RefCell::new(TouchTracker::default())),
        }
    }
}

impl InputCollector {
    /// Update surface dimensions used for touch zone calculations (native only).
    ///
    /// Call once at init and again on every resize. On WASM the canvas size is
    /// read inside each touch event, so this is not needed there.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_surface_width(&mut self, width: f32) {
        self.touch.set_surface_width(width);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_surface_height(&mut self, height: f32) {
        self.touch.set_surface_height(height);
    }

    pub fn set_touch_scheme(&mut self, scheme: TouchControlScheme) {
        self.touch_scheme = scheme;
    }

    /// True once any touch event has been observed this session. Used by the
    /// renderer to gate touch-only HUD elements so they don't appear on
    /// keyboard-driven desktop or web sessions.
    pub fn has_been_touched(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.touch.ever_touched
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.wasm_touch.borrow().ever_touched
        }
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
                let canvas_height = canvas_ref.client_height() as f32;
                let mut s = state.borrow_mut();
                s.set_surface_width(canvas_width);
                s.set_surface_height(canvas_height);
                let changed = event.changed_touches();
                for i in 0..changed.length() {
                    if let Some(touch) = changed.get(i) {
                        let x = touch.client_x() as f32;
                        let y = touch.client_y() as f32;
                        let id = touch.identifier();
                        s.started(i64::from(id), x, y);
                    }
                }
            });
            // safe: canvas is a valid EventTarget; "touchstart" is a standard event
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
                        s.moved(
                            i64::from(touch.identifier()),
                            touch.client_x() as f32,
                            touch.client_y() as f32,
                        );
                    }
                }
            });
            // safe: canvas is a valid EventTarget; "touchmove" is a standard event
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
                        s.ended(
                            i64::from(touch.identifier()),
                            touch.client_x() as f32,
                            touch.client_y() as f32,
                        );
                    }
                }
            });
            // safe: canvas is a valid EventTarget; "touchend"/"touchcancel" are standard events
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
                KeyCode::KeyR if pressed => self.restart_requested = true,
                KeyCode::KeyH | KeyCode::Slash if pressed => self.help_requested = true,

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
                KeyCode::KeyT if pressed => self.audio_next_track_requested = true,
                KeyCode::KeyY if pressed => self.audio_toggle_requested = true,

                // Menu navigation
                KeyCode::ArrowUp => self.held_menu_up = pressed,
                KeyCode::ArrowDown => self.held_menu_down = pressed,
                KeyCode::ArrowLeft => self.held_menu_left = pressed,
                KeyCode::ArrowRight => self.held_menu_right = pressed,
                KeyCode::Enter | KeyCode::Space => self.held_menu_confirm = pressed,
                KeyCode::Escape => self.held_menu_cancel = pressed,

                _ => {}
            }
        }

        match event {
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.cursor_x = position.x as f32;
                self.cursor_y = position.y as f32;
            }
            winit::event::WindowEvent::MouseInput { state, button, .. }
                if *state == winit::event::ElementState::Pressed
                    && *button == winit::event::MouseButton::Left =>
            {
                self.pointer_press = Some(PointerPress {
                    x: self.cursor_x,
                    y: self.cursor_y,
                });
            }
            winit::event::WindowEvent::MouseInput { state, button, .. }
                if *state == winit::event::ElementState::Released
                    && *button == winit::event::MouseButton::Left =>
            {
                self.pointer_release = Some(PointerPress {
                    x: self.cursor_x,
                    y: self.cursor_y,
                });
            }
            _ => {}
        }

        // Native touch: winit relays WindowEvent::Touch on platforms with touch
        // support (iOS, Android, touchscreen desktops). Not available on WASM —
        // handled via DOM listeners registered in init_touch().
        #[cfg(not(target_arch = "wasm32"))]
        if let winit::event::WindowEvent::Touch(touch) = event {
            use winit::event::TouchPhase;
            let x = touch.location.x as f32;
            let y = touch.location.y as f32;
            let id = touch.id as i64;
            match touch.phase {
                TouchPhase::Started => {
                    self.touch.started(id, x, y);
                }
                TouchPhase::Moved => self.touch.moved(id, x, y),
                TouchPhase::Ended | TouchPhase::Cancelled => self.touch.ended(id, x, y),
            }
        }
    }

    pub fn current_state(&mut self) -> InputState {
        let restart = self.restart_requested;
        self.restart_requested = false;
        let help = self.help_requested;
        self.help_requested = false;
        let audio_next_track = self.audio_next_track_requested;
        self.audio_next_track_requested = false;
        let audio_toggle = self.audio_toggle_requested;
        self.audio_toggle_requested = false;
        let mut pointer_pressed = self.pointer_press.take();
        let mut pointer_released = self.pointer_release.take();

        let keyboard_thrust = if self.held_thrust { 1.0 } else { 0.0 };
        let keyboard_rotate = match (self.held_left, self.held_right) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => 0.0,
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (touch_started, touch_input) = {
            let touch_press = self.touch.consume_touch_started();
            if pointer_pressed.is_none() {
                pointer_pressed = touch_press;
            }
            let touch_release = self.touch.consume_touch_ended();
            if pointer_released.is_none() {
                pointer_released = touch_release;
            }
            let touch_started = touch_press.is_some();
            let touch_input = self.touch.current_input(self.touch_scheme);
            (touch_started, touch_input)
        };

        #[cfg(target_arch = "wasm32")]
        let (touch_started, touch_input) = {
            let mut touch = self.wasm_touch.borrow_mut();
            let touch_press = touch.consume_touch_started();
            if pointer_pressed.is_none() {
                pointer_pressed = touch_press;
            }
            let touch_release = touch.consume_touch_ended();
            if pointer_released.is_none() {
                pointer_released = touch_release;
            }
            let touch_started = touch_press.is_some();
            let touch_input = touch.current_input(self.touch_scheme);
            (touch_started, touch_input)
        };

        // Touch owns its axis entirely; keyboard fills the other.
        let thrust = if touch_input.thrust {
            1.0
        } else {
            keyboard_thrust
        };
        let (rotate, target_heading) = match touch_input.rotation {
            TouchRotation::Inactive => (keyboard_rotate, None),
            TouchRotation::Neutral => (0.0, None),
            TouchRotation::Digital(rotate) => (rotate, None),
            TouchRotation::Heading(heading) => (0.0, Some(heading)),
        };

        InputState {
            thrust,
            rotate,
            target_heading,
            restart,
            touch_started,
            pointer_pressed,
            pointer_released,
            help,
            audio_next_track,
            audio_toggle,
            pause: self.held_pause,
            fullscreen: self.held_fullscreen,
            menu_up: self.held_menu_up,
            menu_down: self.held_menu_down,
            menu_left: self.held_menu_left,
            menu_right: self.held_menu_right,
            menu_confirm: self.held_menu_confirm,
            menu_cancel: self.held_menu_cancel,
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
