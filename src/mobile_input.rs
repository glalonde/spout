//! Mobile input bridge for WASM builds.
//!
//! Exports three `wasm_bindgen` functions that JavaScript calls to feed
//! touch / DeviceOrientation events into the game loop:
//!
//!   set_mobile_forward(bool)  – thrust
//!   set_mobile_left(bool)     – rotate CCW
//!   set_mobile_right(bool)    – rotate CW
//!
//! The game loop reads back the current state via the `get_*` helpers,
//! which are plain Rust (not exported to JS).

use std::sync::atomic::{AtomicBool, Ordering};
use wasm_bindgen::prelude::*;

static FORWARD: AtomicBool = AtomicBool::new(false);
static LEFT: AtomicBool = AtomicBool::new(false);
static RIGHT: AtomicBool = AtomicBool::new(false);

/// Set by JS: hold = thrust.
#[wasm_bindgen]
pub fn set_mobile_forward(val: bool) {
    FORWARD.store(val, Ordering::Relaxed);
}

/// Set by JS: tilt / button left = rotate CCW.
#[wasm_bindgen]
pub fn set_mobile_left(val: bool) {
    LEFT.store(val, Ordering::Relaxed);
}

/// Set by JS: tilt / button right = rotate CW.
#[wasm_bindgen]
pub fn set_mobile_right(val: bool) {
    RIGHT.store(val, Ordering::Relaxed);
}

pub fn get_forward() -> bool {
    FORWARD.load(Ordering::Relaxed)
}

pub fn get_left() -> bool {
    LEFT.load(Ordering::Relaxed)
}

pub fn get_right() -> bool {
    RIGHT.load(Ordering::Relaxed)
}
