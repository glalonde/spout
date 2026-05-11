//! Spout entry point: game loop, state machine, and winit event handling.

mod app;
mod audio;
mod graphics;
mod screens;

use std::time::Duration;

use web_time::Instant;

use spout::collision;
use spout::game_params;
use spout::input::{InputCollector, InputFrame, InputState};
use spout::level_manager;
use spout::particles;
use spout::scoring;
use spout::ship;

use graphics::Graphics;
use screens::title::{TitleAction, TitleRenderFlags, TitleScreen, TitleUiRenderContext};

/// Shortest signed angular distance from `current` to `target`, in [-π, π].
fn angle_diff(target: f32, current: f32) -> f32 {
    let d = glam::Vec2::from_angle(target - current);
    d.y.atan2(d.x) // equivalent to wrapping (target-current) to [-π, π]
}

/// Time budget per frame for background level generation (≈ 1/300 s).
const LEVEL_BUDGET: Duration = Duration::from_nanos(3_333_333);

/// Maximum physics step. Caps dt so that GPU stalls or level-loading pauses
/// don't cause the ship and particles to simulate a huge time jump.
const MAX_FRAME_DT: Duration = Duration::from_millis(50);

fn restart_prompt() -> &'static str {
    if tap_restart_prompt() {
        "TAP TO RESTART"
    } else {
        "R TO RESTART"
    }
}

#[cfg(any(target_os = "ios", target_os = "android"))]
fn tap_restart_prompt() -> bool {
    true
}

#[cfg(all(
    target_arch = "wasm32",
    not(any(target_os = "ios", target_os = "android"))
))]
fn tap_restart_prompt() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };

    let touch_event = js_sys::Reflect::has(
        window.as_ref(),
        &wasm_bindgen::JsValue::from_str("ontouchstart"),
    )
    .unwrap_or(false);
    let max_touch_points = js_sys::Reflect::get(
        window.as_ref(),
        &wasm_bindgen::JsValue::from_str("navigator"),
    )
    .ok()
    .and_then(|navigator| {
        js_sys::Reflect::get(
            &navigator,
            &wasm_bindgen::JsValue::from_str("maxTouchPoints"),
        )
        .ok()
    })
    .and_then(|value| value.as_f64())
    .unwrap_or(0.0);

    touch_event || max_touch_points > 0.0
}

#[cfg(not(any(target_os = "ios", target_os = "android", target_arch = "wasm32")))]
fn tap_restart_prompt() -> bool {
    false
}

/// How the player died — drives the GAME OVER / TIMES UP overlay text and
/// makes future causes (hazards, etc.) easy to add.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeathCause {
    Collided,
    FellOff,
    TimeExpired,
}

/// GPU-bound state transitions queued up by `update_phase` to be applied
/// after the simulation step (they need to rebuild `level_manager` and
/// `particle_system`, which submit their own initialization encoder).
/// Lightweight transitions (`ToGameOver`, pause toggle, title-instructions)
/// are applied inline during `update_phase` instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingTransition {
    ToTitle,
    ToPlay,
}

/// In-game session data, shared across Playing / Paused / GameOver.
///
/// Lives inside the relevant `AppState` variants. Methods on `Play` are pure
/// game logic — they take `&GameParams`/`&InputState` rather than touching any
/// GPU state, so the simulation step is testable without wgpu.
#[derive(Debug, Default)]
struct Play {
    ship_state: ship::ShipState,
    prev_ship_state: ship::ShipState,
    viewport_offset: i32,
    progress_height: i32,
    time_bonus_score: i32,
    score: i32,
    current_level_index: i32,
    level_elapsed: Duration,
    pending_collision_segment: Option<PendingCollisionSegment>,
    in_flight_collision_segment: Option<PendingCollisionSegment>,
}

/// Top-level state machine. Exhaustive — every screen / mode is a variant,
/// and impossible combinations (e.g. dead-on-title) cannot be represented.
#[derive(Debug)]
enum AppState {
    Title(TitleScreen),
    /// Placeholder for the upcoming settings screen. Not yet constructed.
    #[allow(dead_code)]
    Settings,
    /// Placeholder for the upcoming leaderboard screen. Not yet constructed.
    #[allow(dead_code)]
    Leaderboard,
    Playing(Play),
    Paused(Play),
    GameOver {
        play: Play,
        cause: DeathCause,
    },
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Title(TitleScreen::default())
    }
}

impl AppState {
    /// Camera offset for level/background rendering. Title/Settings/Leaderboard
    /// have no notion of a viewport, so report 0 (level 0 origin).
    fn viewport_offset(&self) -> i32 {
        match self {
            AppState::Playing(p) | AppState::Paused(p) => p.viewport_offset,
            AppState::GameOver { play, .. } => play.viewport_offset,
            AppState::Title(_) | AppState::Settings | AppState::Leaderboard => 0,
        }
    }

    fn is_playing(&self) -> bool {
        matches!(self, AppState::Playing(_))
    }

    fn resolve_collision_result(
        &mut self,
        params: &game_params::GameParams,
        result: collision::CollisionResult,
    ) -> Option<DeathCause> {
        match self {
            AppState::Playing(play) | AppState::Paused(play) => {
                play.resolve_collision_result(params, result)
            }
            AppState::Title(_)
            | AppState::Settings
            | AppState::Leaderboard
            | AppState::GameOver { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PendingCollisionSegment {
    prev_ship: ship::ShipState,
    next_ship: ship::ShipState,
}

impl PendingCollisionSegment {
    fn ship_at(&self, t: f32) -> ship::ShipState {
        let t = t.clamp(0.0, 1.0);
        let mut ship = self.next_ship;
        ship.position = [
            self.prev_ship.position[0]
                + (self.next_ship.position[0] - self.prev_ship.position[0]) * t,
            self.prev_ship.position[1]
                + (self.next_ship.position[1] - self.prev_ship.position[1]) * t,
        ];
        ship
    }
}

fn record_collision_motion(
    pending_segment: &mut Option<PendingCollisionSegment>,
    prev_ship: ship::ShipState,
    next_ship: ship::ShipState,
) {
    if let Some(segment) = pending_segment {
        segment.next_ship = next_ship;
    } else {
        *pending_segment = Some(PendingCollisionSegment {
            prev_ship,
            next_ship,
        });
    }
}

/// Position + velocity captured at the moment of death, used to spawn the
/// explosion particle burst on the next render.
#[derive(Debug, Clone, Copy)]
struct ExplosionRequest {
    position: [f32; 2],
    velocity: [f32; 2],
}

/// Particle emitter motion for the title screen: a fixed emitter angled
/// up-left from the right edge of the viewport.
fn title_emitter_motion(params: &game_params::GameParams) -> particles::EmitterMotion {
    let w = params.viewport_width as f32;
    let h = params.viewport_height as f32;
    let x = w - 10.0;
    let y = h * 0.75;
    // ~170° — mostly left with a slight upward angle.
    let angle = std::f32::consts::PI - 0.2;
    particles::EmitterMotion {
        position_start: [x, y],
        position_end: [x, y],
        velocity_start: [0.0, 0.0],
        velocity_end: [0.0, 0.0],
        angle_start: angle,
        angle_end: angle,
        ..Default::default()
    }
}

/// Particle emitter motion driven by the ship's thrust. Returns `None` when
/// thrust is off or the ship is no longer alive.
fn ship_emitter_motion(
    prev_ship: &ship::ShipState,
    cur_ship: &ship::ShipState,
    thrust: f32,
) -> Option<particles::EmitterMotion> {
    if thrust <= 0.0 {
        return None;
    }
    let start = prev_ship.get_emitter_state();
    let end = cur_ship.get_emitter_state();
    Some(particles::EmitterMotion {
        position_start: start.0,
        position_end: end.0,
        velocity_start: prev_ship.velocity,
        velocity_end: cur_ship.velocity,
        angle_start: start.1,
        angle_end: end.1,
        ..Default::default()
    })
}

impl Play {
    fn new(params: &game_params::GameParams) -> Self {
        let ship_state = ship::ShipState::init(
            &params.ship_params,
            [
                (params.viewport_width / 2) as f32 + 0.5,
                (params.viewport_height / 2) as f32 + 0.5,
            ],
        );
        Self {
            ship_state,
            prev_ship_state: ship_state,
            ..Default::default()
        }
    }

    fn level_timer_remaining(&self, params: &game_params::GameParams) -> Duration {
        scoring::level_time_limit_duration(params).saturating_sub(self.level_elapsed)
    }

    fn commit_progress_height(&mut self, params: &game_params::GameParams, height: f32) {
        let confirmed_height = height.floor() as i32;
        self.progress_height = std::cmp::max(confirmed_height, self.progress_height);

        let next_level_index =
            scoring::level_index_for_progress(self.progress_height, params.level_height);
        if next_level_index > self.current_level_index {
            let levels_crossed = next_level_index - self.current_level_index;
            let bonus = scoring::time_bonus_score(self.level_timer_remaining(params))
                .saturating_mul(levels_crossed);
            self.time_bonus_score = self.time_bonus_score.saturating_add(bonus);
            self.current_level_index = next_level_index;
            self.level_elapsed = Duration::ZERO;
            log::info!(
                "Entered level {} with {} time bonus points",
                self.current_level_index + 1,
                bonus
            );
        }

        self.score = scoring::combined_score(self.progress_height, self.time_bonus_score);
    }

    fn update_camera(&mut self, params: &game_params::GameParams) {
        let live_height = self.ship_state.position[1].floor() as i32;
        let camera_height = std::cmp::max(live_height, self.progress_height);
        self.viewport_offset = camera_height - (params.viewport_height / 2) as i32;
    }

    /// Advance ship physics one frame. Returns the death cause if the ship
    /// flew off the playfield this step.
    fn update_ship(
        &mut self,
        params: &game_params::GameParams,
        input: &InputState,
        dt: f32,
    ) -> Option<DeathCause> {
        let gravity = params.particle_system_params.gravity;

        let rotate = if let Some(target) = input.target_heading {
            // Bang-bang controller: rotate at full speed toward target heading,
            // stop when within one frame's worth of rotation to avoid oscillation.
            let current = self.ship_state.orientation;
            let error = angle_diff(target, current);
            let dead_zone = self.ship_state.rotation_rate * dt;
            if error.abs() <= dead_zone {
                0.0
            } else {
                error.signum()
            }
        } else {
            input.rotate
        };
        self.ship_state.update(dt, input.thrust, rotate, gravity);

        // Kill if the ship flies off the horizontal edges.
        let x = self.ship_state.position[0];
        if x < 0.0 || x >= params.level_width as f32 {
            return Some(DeathCause::FellOff);
        }

        // Kill if the ship falls more than one viewport-height below the
        // bottom of the visible window. At default gravity and viewport
        // height, that is ~2.8 s of free-fall from rest from the viewport
        // bottom — enough to recover from a stall, but not from a sustained
        // fall.
        let bottom = self.viewport_offset as f32;
        let viewport_h = params.viewport_height as f32;
        if self.ship_state.position[1] < bottom - viewport_h {
            return Some(DeathCause::FellOff);
        }
        None
    }

    /// Advance one frame of gameplay. Returns the death cause if any check
    /// killed the ship this step (timer expiry, out-of-bounds).
    fn update(
        &mut self,
        params: &game_params::GameParams,
        input: &InputState,
        game_dt: f32,
        game_dt_duration: Duration,
    ) -> Option<DeathCause> {
        self.level_elapsed = self.level_elapsed.saturating_add(game_dt_duration);
        if self.level_elapsed >= scoring::level_time_limit_duration(params) {
            log::info!(
                "Level timer expired on level {}",
                self.current_level_index + 1,
            );
            return Some(DeathCause::TimeExpired);
        }

        if game_dt <= 0.0 {
            return None;
        }

        let prev_ship = self.ship_state;
        self.prev_ship_state = prev_ship;
        if let Some(cause) = self.update_ship(params, input, game_dt) {
            return Some(cause);
        }
        self.update_camera(params);
        record_collision_motion(
            &mut self.pending_collision_segment,
            prev_ship,
            self.ship_state,
        );
        None
    }

    /// Apply a GPU collision-detection readback to current play state.
    /// Returns `Some(DeathCause::Collided)` if the ship hit terrain.
    fn resolve_collision_result(
        &mut self,
        params: &game_params::GameParams,
        result: collision::CollisionResult,
    ) -> Option<DeathCause> {
        let segment = self.in_flight_collision_segment.take()?;
        if result.hit {
            let impact_ship = segment.ship_at(result.impact_t);
            self.commit_progress_height(params, impact_ship.position[1]);
            self.prev_ship_state = segment.prev_ship;
            self.ship_state = impact_ship;
            self.pending_collision_segment = None;
            log::info!(
                "Ship collided with terrain at ({:.0}, {:.0}) t={:.3}",
                impact_ship.position[0],
                impact_ship.position[1],
                result.impact_t
            );
            Some(DeathCause::Collided)
        } else {
            self.commit_progress_height(params, segment.next_ship.position[1]);
            None
        }
    }
}

struct Spout {
    game_params: game_params::GameParams,
    state: AppState,
    /// Input snapshot for this frame. Lives on `Spout` (not in `AppState`) so
    /// it carries across screen transitions and edge-detection isn't broken
    /// when state changes.
    input_state: InputState,
    prev_input_state: InputState,
    /// One-shot explosion to emit on the next render. Set when transitioning
    /// into `GameOver`; consumed by the render pass.
    pending_explosion: Option<ExplosionRequest>,
    collector: InputCollector,
    level_manager: level_manager::LevelManager,
    particle_system: particles::ParticleSystem,
    collision_detector: collision::CollisionDetector,
    audio: audio::AudioPlayer,
    game_time: Duration,
    iteration_start: Instant,
    graphics: Graphics,
    frame_times: Vec<f32>,
    cpu_times: Vec<f32>,
    tick_wall_dt: f32,
    frame_log_count: u32,
}

impl Spout {
    /// Reset to the title screen. Used at startup and after death/restart.
    fn transition_to_title(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.state = AppState::Title(TitleScreen::default());
        self.game_time = Duration::default();
        self.iteration_start = Instant::now();

        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.level_manager = level_manager::LevelManager::init_title(
            device,
            &self.game_params,
            0,
            &mut init_encoder,
            &mut self.graphics.staging_belt,
        );
        self.particle_system = particles::ParticleSystem::new(
            device,
            &self.game_params,
            &mut init_encoder,
            &self.level_manager,
        );
        self.collision_detector.result = collision::CollisionResult::default();
        self.particle_system.set_nozzle_speed(75.0, 75.0);

        self.graphics.staging_belt.finish();
        queue.submit(Some(init_encoder.finish()));
        self.graphics.staging_belt.recall();

        log::info!("Entered title screen");
    }

    /// Start a fresh game from the title (or after game-over).
    fn transition_to_play(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.state = AppState::Playing(Play::new(&self.game_params));
        self.game_time = Duration::default();
        self.iteration_start = Instant::now();

        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.level_manager = level_manager::LevelManager::init(
            device,
            &self.game_params,
            0,
            &mut init_encoder,
            &mut self.graphics.staging_belt,
        );
        self.particle_system = particles::ParticleSystem::new(
            device,
            &self.game_params,
            &mut init_encoder,
            &self.level_manager,
        );
        self.collision_detector.result = collision::CollisionResult::default();
        let base_speed = self.game_params.particle_system_params.emission_speed;
        self.particle_system
            .set_nozzle_speed(base_speed, base_speed);

        self.graphics.staging_belt.finish();
        queue.submit(Some(init_encoder.finish()));
        self.graphics.staging_belt.recall();

        log::info!("Game started");
    }

    /// Take the current `Play` session and move it into `AppState::GameOver`,
    /// queueing the explosion burst for the next frame.
    fn transition_to_game_over(&mut self, cause: DeathCause) {
        let prev = std::mem::take(&mut self.state);
        let mut play = match prev {
            AppState::Playing(p) | AppState::Paused(p) => p,
            other => {
                // Shouldn't happen — only call this from an active play session.
                self.state = other;
                return;
            }
        };
        // Drop any in-flight collision pipelining; the result no longer matters.
        play.pending_collision_segment = None;
        play.in_flight_collision_segment = None;
        self.pending_explosion = Some(ExplosionRequest {
            position: play.ship_state.position,
            velocity: play.ship_state.velocity,
        });
        log::info!(
            "Game over (cause: {:?}) at ({:.0}, {:.0})",
            cause,
            play.ship_state.position[0],
            play.ship_state.position[1]
        );
        self.state = AppState::GameOver { play, cause };
    }

    /// Toggle Playing ↔ Paused. No-op in other states.
    fn toggle_pause(&mut self) {
        let prev = std::mem::take(&mut self.state);
        self.state = match prev {
            AppState::Playing(play) => {
                log::info!("Paused game at t={:#?}", self.game_time);
                AppState::Paused(play)
            }
            AppState::Paused(play) => {
                log::info!("Unpaused game at t={:#?}", self.game_time);
                AppState::Playing(play)
            }
            other => other,
        };
    }

    /// Compute the per-frame game / wall delta times. Wall time always
    /// advances; game time freezes while paused.
    fn tick_time(&mut self) -> (Duration, Duration) {
        let now = Instant::now();
        let delta_t = (now - self.iteration_start).min(MAX_FRAME_DT);
        self.iteration_start = now;

        let paused = matches!(self.state, AppState::Paused(_));
        if paused {
            (Duration::ZERO, delta_t)
        } else {
            self.game_time += delta_t;
            (delta_t, delta_t)
        }
    }

    /// Poll last frame's GPU collision readback. Results still resolve while
    /// paused so in-flight collision segments do not block future dispatches
    /// after unpausing.
    fn resolve_pending_collision(&mut self) {
        let Some(result) = self.collision_detector.poll_result() else {
            return;
        };

        let collision_death = self
            .state
            .resolve_collision_result(&self.game_params, result);
        if let Some(cause) = collision_death {
            self.transition_to_game_over(cause);
        }
    }

    /// Window-level / state-agnostic input edges: pause toggle, fullscreen.
    /// Neither needs GPU access.
    fn handle_global_input(&mut self, window: &winit::window::Window, input: InputFrame) {
        if input.pause_pressed() {
            self.toggle_pause();
        }

        if input.fullscreen_pressed() {
            if window.fullscreen().is_some() {
                log::info!("Setting windowed mode.");
                window.set_fullscreen(None);
            } else {
                log::info!("Setting borderless fullscreen.");
                window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            }
        }
    }

    /// Audio input edges are global commands, but they still flow through
    /// `InputCollector` so window-event handling stays platform-agnostic.
    fn handle_audio_input(&mut self, input: InputFrame) {
        if input.audio_next_track_pressed() {
            self.audio.next_track();
        }

        if input.audio_toggle_pressed() {
            self.audio.toggle();
        }
    }

    /// Drive the simulation for the active state. Returns a death cause if
    /// the sim killed the ship this frame (timer expiry, fell off the
    /// playfield).
    fn update_simulation(
        &mut self,
        game_dt: f32,
        game_dt_duration: Duration,
    ) -> Option<DeathCause> {
        match &mut self.state {
            AppState::Title(_) => {
                self.particle_system.update_state(
                    game_dt,
                    0,
                    Some(title_emitter_motion(&self.game_params)),
                );
                None
            }
            AppState::Playing(play) => {
                let prev_ship = play.ship_state;
                let cause = play.update(
                    &self.game_params,
                    &self.input_state,
                    game_dt,
                    game_dt_duration,
                );
                let motion = if cause.is_none() {
                    ship_emitter_motion(&prev_ship, &play.ship_state, self.input_state.thrust)
                } else {
                    None
                };
                self.particle_system
                    .update_state(game_dt, play.viewport_offset, motion);
                cause
            }
            AppState::Paused(_)
            | AppState::GameOver { .. }
            | AppState::Settings
            | AppState::Leaderboard => {
                // No simulation step. Particles continue to animate without
                // a new emitter motion (drifting from previous frame's state).
                let offset = self.state.viewport_offset();
                self.particle_system.update_state(game_dt, offset, None);
                None
            }
        }
    }

    /// Title-screen input. The screen owns its UI sub-state and reports
    /// high-level actions back to `Spout`.
    fn process_title_input(
        &mut self,
        input: InputFrame,
        surface_w: u32,
        surface_h: u32,
    ) -> Option<PendingTransition> {
        let AppState::Title(title) = &mut self.state else {
            return None;
        };

        match title.update(
            input,
            &self.game_params,
            &self.graphics.game_text,
            self.audio.is_playing(),
            (surface_w, surface_h),
        ) {
            Some(TitleAction::StartGame) => Some(PendingTransition::ToPlay),
            Some(TitleAction::ToggleMusic) => {
                self.audio.toggle();
                None
            }
            None => None,
        }
    }

    /// Pure-CPU update phase. Snapshots input, ticks time, drives simulation,
    /// applies non-GPU transitions inline (pause toggle, GameOver), and
    /// returns a GPU-bound transition intent (ToTitle / ToPlay) if any.
    fn update_phase(
        &mut self,
        window: &winit::window::Window,
        surface_size: (u32, u32),
    ) -> Option<PendingTransition> {
        self.audio.poll();
        self.resolve_pending_collision();

        self.prev_input_state = self.input_state;
        self.input_state = self.collector.current_state();
        let input = InputFrame::new(self.input_state, self.prev_input_state);

        self.handle_global_input(window, input);
        self.handle_audio_input(input);

        self.level_manager
            .level_maker
            .work_until(Instant::now() + LEVEL_BUDGET);

        let (game_dt_duration, wall_dt_duration) = self.tick_time();
        let game_dt = game_dt_duration.as_secs_f32();
        let wall_dt = wall_dt_duration.as_secs_f32();
        self.tick_wall_dt = wall_dt;

        if let Some(cause) = self.update_simulation(game_dt, game_dt_duration) {
            self.transition_to_game_over(cause);
        }

        self.graphics
            .renderer
            .update_state(wall_dt, &input.current, &input.previous);

        // Restart from game-over: explicit restart key, or any tap.
        let restart_pressed = input.restart_pressed()
            || (matches!(self.state, AppState::GameOver { .. }) && input.touch_started());
        if restart_pressed {
            return Some(PendingTransition::ToTitle);
        }

        self.process_title_input(input, surface_size.0, surface_size.1)
    }

    fn apply_transition(
        &mut self,
        t: PendingTransition,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        match t {
            PendingTransition::ToTitle => self.transition_to_title(device, queue),
            PendingTransition::ToPlay => self.transition_to_play(device, queue),
        }
    }
}

impl Spout {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: &winit::window::Window,
    ) -> Self {
        window.set_cursor_visible(false);
        let game_params = game_params::get_game_config_from_default_file();
        let mut graphics = Graphics::new(config, adapter, device, queue, &game_params);

        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Use a placeholder level manager for construction; transition_to_title()
        // below will replace it immediately after.
        let level_manager = level_manager::LevelManager::init(
            device,
            &game_params,
            0,
            &mut init_encoder,
            &mut graphics.staging_belt,
        );

        let particle_system =
            particles::ParticleSystem::new(device, &game_params, &mut init_encoder, &level_manager);

        let collision_detector = collision::CollisionDetector::init(device);

        graphics.staging_belt.finish();
        queue.submit(Some(init_encoder.finish()));
        graphics.staging_belt.recall();

        let audio = if game_params.music_starts_on {
            audio::AudioPlayer::new()
        } else {
            audio::AudioPlayer::disabled()
        };

        let mut collector = InputCollector::default();
        collector.set_touch_scheme(game_params.touch_control_scheme);

        #[cfg(not(target_arch = "wasm32"))]
        {
            collector.set_surface_width(config.width as f32);
            collector.set_surface_height(config.height as f32);
        }

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            if let Some(canvas) = window.canvas() {
                collector.init_touch(canvas);
            }
        }

        let mut spout = Spout {
            game_params,
            state: AppState::default(),
            input_state: InputState::default(),
            prev_input_state: InputState::default(),
            pending_explosion: None,
            collector,
            level_manager,
            game_time: Duration::default(),
            iteration_start: Instant::now(),
            particle_system,
            collision_detector,
            audio,
            graphics,
            frame_times: Vec::with_capacity(60),
            cpu_times: Vec::with_capacity(60),
            tick_wall_dt: 0.0,
            frame_log_count: 0,
        };
        spout.transition_to_title(device, queue);
        spout
    }

    fn update(&mut self, event: winit::event::WindowEvent) {
        self.collector.handle_winit_event(&event);
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.collector.set_surface_width(config.width as f32);
            self.collector.set_surface_height(config.height as f32);
        }

        self.graphics
            .resize(config, device, queue, &self.game_params);
    }

    /// Top-level frame entry point. Splits into four phases:
    ///
    /// 1. `update_phase` — pure-CPU simulation + input handling.
    /// 2. `apply_transition` — GPU-bound state transitions (rebuild level
    ///    and particle resources). Submits its own encoder.
    /// 3. `draw_phase` — render the current state. Owns the main encoder.
    /// 4. `post_phase` — frame-time bookkeeping + spare-budget level work.
    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: &winit::window::Window,
        surface_size: (u32, u32),
    ) {
        let cpu_start = Instant::now();
        let pending = self.update_phase(window, surface_size);
        if let Some(t) = pending {
            self.apply_transition(t, device, queue);
        }
        self.draw_phase(view, device, queue, window);
        self.post_phase(cpu_start, window);
    }

    /// Run all GPU work for the current frame and submit it. Owns the render
    /// encoder; doesn't mutate game-logic state (only consumes `pending_explosion`
    /// and dispatches queued collision work).
    fn draw_phase(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: &winit::window::Window,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let viewport_offset = self.state.viewport_offset();

        self.level_manager.sync_height(
            device,
            viewport_offset,
            &mut encoder,
            &self.game_params,
            &mut self.graphics.staging_belt,
        );

        // Ship explosion burst — write particles before compute runs.
        if let Some(explosion) = self.pending_explosion.take() {
            self.particle_system.emit_burst(
                &mut encoder,
                &mut self.graphics.staging_belt,
                explosion.position,
                explosion.velocity,
                50000, // burst count
                self.game_params.particle_system_params.emission_speed,
                self.game_params.particle_system_params.max_particle_life,
            );
        }

        // Compute pipelines.
        self.level_manager.compose_tiles(&mut encoder);
        self.particle_system.run_compute(
            &self.level_manager,
            &mut encoder,
            &mut self.graphics.staging_belt,
        );

        // Collision detection — only during active gameplay.
        if let AppState::Playing(play) = &mut self.state {
            if play.in_flight_collision_segment.is_none() {
                if let Some(segment) = play.pending_collision_segment.take() {
                    let dispatched = self.collision_detector.dispatch(
                        device,
                        &mut encoder,
                        &mut self.graphics.staging_belt,
                        &segment.next_ship,
                        &segment.prev_ship,
                        self.level_manager.terrain_buffer(),
                        self.game_params.level_width,
                    );
                    if dispatched {
                        play.in_flight_collision_segment = Some(segment);
                    } else {
                        play.pending_collision_segment = Some(segment);
                    }
                }
            }
        }

        // Background → terrain → particles into the game view.
        self.graphics.background.update_state(
            &self.game_params,
            viewport_offset,
            &mut encoder,
            &mut self.graphics.staging_belt,
        );
        self.graphics
            .background
            .render(&self.graphics.game_view_texture, &mut encoder);
        self.level_manager
            .terrain_renderer
            .render(&self.graphics.game_view_texture, &mut encoder);
        self.particle_system
            .render(&self.graphics.game_view_texture, &mut encoder);

        // Ship — only during active gameplay or pause.
        if self.game_params.render_ship {
            let active_play = match &self.state {
                AppState::Playing(p) | AppState::Paused(p) => Some(p),
                _ => None,
            };
            if let Some(play) = active_play {
                self.graphics.ship_renderer.render(
                    &play.ship_state,
                    &self.game_params,
                    play.viewport_offset,
                    &self.graphics.game_view_texture,
                    &mut encoder,
                    &mut self.graphics.staging_belt,
                );
            }
        }

        self.draw_hud(device, &mut encoder);

        // Blit the game scene, bloom it, then composite to the surface.
        self.graphics.renderer.blit(
            &self.graphics.upscaled_view,
            &mut encoder,
            &mut self.graphics.staging_belt,
        );
        self.graphics.bloom.render(&mut encoder);

        if let AppState::Title(title) = &self.state {
            title.prepare_ui(TitleUiRenderContext {
                device,
                encoder: &mut encoder,
                target_view: &self.graphics.title_ui_view,
                ui: &self.graphics.ui,
                params: &self.game_params,
                text: &self.graphics.game_text,
                flags: TitleRenderFlags {
                    music_playing: self.audio.is_playing(),
                    using_touch: self.collector.has_been_touched() || tap_restart_prompt(),
                },
            });
        }

        self.graphics.renderer.render(view, &mut encoder);
        if matches!(self.state, AppState::Title(_)) {
            self.graphics.title_overlay.render(view, &mut encoder);
        }

        // Touch-zone diagonal hint only during active gameplay, on Triangle
        // scheme, after the player has actually used touch this session.
        if matches!(
            self.game_params.touch_control_scheme,
            game_params::TouchControlScheme::Triangle
        ) && self.collector.has_been_touched()
            && self.state.is_playing()
        {
            self.graphics
                .touch_zone_indicator
                .render(view, &mut encoder);
        }

        self.level_manager.decompose_tiles(&mut encoder);

        // FPS overlay (debug builds). Computes avg_dt now so the on-screen
        // text uses up-to-date numbers; the same avg_dt is reused for the
        // periodic frame log in `post_phase`.
        let dt = self.tick_wall_dt;
        if self.frame_times.len() >= 60 {
            self.frame_times.remove(0);
        }
        self.frame_times.push(dt);

        #[cfg(debug_assertions)]
        {
            let avg_dt = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            let fps = if avg_dt > 0.0 { 1.0 / avg_dt } else { 0.0 };
            let win_size = window.inner_size();
            let fps_text = format!("FPS:{:.0} {}x{}", fps, win_size.width, win_size.height);
            let white = [1.0, 1.0, 1.0, 1.0];
            self.graphics.overlay_text.draw(
                device,
                &mut encoder,
                view,
                &[(&fps_text, 8.0, 8.0, 1.0, white)],
            );
        }
        #[cfg(not(debug_assertions))]
        let _ = window;

        self.graphics.staging_belt.finish();
        queue.submit(Some(encoder.finish()));
        // Drain wgpu callbacks from the previous frame's completed GPU work
        // (non-blocking). Fires the map_async callback set by start_readback()
        // last frame, so poll_result() at the top of the next frame sees
        // map_ready == true without stalling.
        #[cfg(not(target_arch = "wasm32"))]
        device.poll(wgpu::PollType::Poll).ok();
        self.graphics.staging_belt.recall();

        // Initiate async readback of collision result now that GPU work is
        // submitted. On native the callback fires during the next poll(); on
        // WASM it fires asynchronously before the next frame.
        self.collision_detector.start_readback();
    }

    /// Score / level / timer text + game-over overlay. Drawn whenever a `Play`
    /// session exists (Playing, Paused, GameOver).
    fn draw_hud(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        let hud_play_and_cause: Option<(&Play, Option<DeathCause>)> = match &self.state {
            AppState::Playing(p) | AppState::Paused(p) => Some((p, None)),
            AppState::GameOver { play, cause } => Some((play, Some(*cause))),
            _ => None,
        };
        let Some((play, cause)) = hud_play_and_cause else {
            return;
        };

        let score_text = format!("{}", play.score);
        let level_text = format!("LV{}", play.current_level_index + 1);
        let timer_text = scoring::format_level_timer(play.level_timer_remaining(&self.game_params));
        // Held below 1.0 so bloom contribution stays well under what the
        // particle/ship neon hits when stacked. With bloom_threshold=0.4 and
        // soft-knee, value v → bloom contribution v*(v-0.4)/v = v-0.4.
        let text_color = [0.7, 0.7, 0.7, 1.0];
        let timer_color =
            if play.level_timer_remaining(&self.game_params) <= Duration::from_secs(10) {
                [0.95, 0.45, 0.45, 1.0]
            } else {
                text_color
            };
        let timer_x = (self.graphics.game_text.surface_width
            - self.graphics.game_text.text_width(&timer_text, 1.0))
            / 2.0;
        let level_x = self.graphics.game_text.surface_width
            - self.graphics.game_text.text_width(&level_text, 1.0)
            - 2.0;
        self.graphics.game_text.draw(
            device,
            encoder,
            &self.graphics.game_view_texture,
            &[
                (&score_text, 2.0, 2.0, 1.0, text_color),
                (&timer_text, timer_x, 2.0, 1.0, timer_color),
                (&level_text, level_x, 2.0, 1.0, text_color),
            ],
        );

        if let Some(cause) = cause {
            let w = self.graphics.game_text.surface_width;
            let h = self.graphics.game_text.surface_height;
            let status = match cause {
                DeathCause::TimeExpired => "TIMES UP",
                DeathCause::Collided | DeathCause::FellOff => "GAME OVER",
            };
            let status_x = (w - self.graphics.game_text.text_width(status, 1.0)) / 2.0;
            let status_y = h * 0.28;

            let score = format!("SCORE: {}", play.score);
            let score_x = (w - self.graphics.game_text.text_width(&score, 1.0)) / 2.0;
            let score_y = h * 0.5;

            let restart = restart_prompt();
            let restart_x = (w - self.graphics.game_text.text_width(restart, 1.0)) / 2.0;
            let restart_y = h * 0.68;

            self.graphics.game_text.draw(
                device,
                encoder,
                &self.graphics.game_view_texture,
                &[
                    (status, status_x, status_y, 1.0, text_color),
                    (&score, score_x, score_y, 1.0, text_color),
                    (restart, restart_x, restart_y, 1.0, text_color),
                ],
            );
        }
    }

    /// Frame-time accounting + periodic log + spare-budget level work. No
    /// rendering, no state mutation.
    fn post_phase(&mut self, cpu_start: Instant, window: &winit::window::Window) {
        let cpu_dt = cpu_start.elapsed().as_secs_f32();
        if self.cpu_times.len() >= 60 {
            self.cpu_times.remove(0);
        }
        self.cpu_times.push(cpu_dt);
        let avg_dt = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        let avg_cpu = self.cpu_times.iter().sum::<f32>() / self.cpu_times.len() as f32;
        let fps = if avg_dt > 0.0 { 1.0 / avg_dt } else { 0.0 };

        self.frame_log_count = self.frame_log_count.wrapping_add(1);
        if self.frame_log_count.is_multiple_of(60) {
            let win_size = window.inner_size();
            log::info!(
                "frame avg_dt={:.2}ms cpu={:.2}ms fps={:.1} surface={}x{} bloom_mips={}",
                avg_dt * 1000.0,
                avg_cpu * 1000.0,
                fps,
                win_size.width,
                win_size.height,
                self.game_params.visual_params.bloom_mip_levels,
            );
        }

        // Spare budget for background level generation.
        let deadline = self.iteration_start + LEVEL_BUDGET;
        self.level_manager.level_maker.work_until(deadline);
    }
}

fn main() {
    app::run("Spout");
}

#[cfg(test)]
mod tests {
    use super::{angle_diff, record_collision_motion, AppState, PendingCollisionSegment, Play};
    use std::f32::consts::PI;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn same_angle_is_zero() {
        assert!(approx(angle_diff(1.0, 1.0), 0.0));
    }

    #[test]
    fn small_positive_difference() {
        assert!(approx(angle_diff(1.0, 0.5), 0.5));
    }

    #[test]
    fn small_negative_difference() {
        assert!(approx(angle_diff(0.5, 1.0), -0.5));
    }

    #[test]
    fn wraps_across_pi_boundary() {
        // From just below π to just above π should be a small positive step,
        // not a large negative one.
        let diff = angle_diff(PI - 0.1, -(PI - 0.1));
        assert!(diff.abs() < 0.3, "should wrap short way, got {}", diff);
    }

    #[test]
    fn wraps_negative_direction() {
        // From just above -π to just below -π.
        let diff = angle_diff(-(PI - 0.1), PI - 0.1);
        assert!(diff.abs() < 0.3, "should wrap short way, got {}", diff);
    }

    #[test]
    fn opposite_directions_magnitude_is_pi() {
        let diff = angle_diff(0.0, PI);
        assert!(
            approx(diff.abs(), PI),
            "opposite angles should differ by π, got {}",
            diff
        );
    }

    #[test]
    fn result_always_in_minus_pi_to_pi() {
        for i in 0..100 {
            let target = (i as f32) * 0.13 - 3.0;
            let current = (i as f32) * 0.07 - 5.0;
            let diff = angle_diff(target, current);
            assert!(
                diff >= -PI - 1e-5 && diff <= PI + 1e-5,
                "angle_diff({}, {}) = {} out of range",
                target,
                current,
                diff
            );
        }
    }

    #[test]
    fn collision_segment_interpolates_position() {
        let mut prev_ship = spout::ship::ShipState::default();
        prev_ship.position = [10.0, 20.0];
        let mut next_ship = prev_ship;
        next_ship.position = [18.0, 36.0];

        let segment = PendingCollisionSegment {
            prev_ship,
            next_ship,
        };
        let ship = segment.ship_at(0.25);

        assert!(approx(ship.position[0], 12.0));
        assert!(approx(ship.position[1], 24.0));
    }

    #[test]
    fn collision_segment_clamps_impact_time() {
        let mut prev_ship = spout::ship::ShipState::default();
        prev_ship.position = [1.0, 2.0];
        let mut next_ship = prev_ship;
        next_ship.position = [3.0, 4.0];

        let segment = PendingCollisionSegment {
            prev_ship,
            next_ship,
        };

        assert!(approx(segment.ship_at(-1.0).position[0], 1.0));
        assert!(approx(segment.ship_at(2.0).position[1], 4.0));
    }

    #[test]
    fn pending_collision_segment_extends_to_latest_motion() {
        let mut a = spout::ship::ShipState::default();
        a.position = [0.0, 0.0];
        let mut b = a;
        b.position = [1.0, 1.0];
        let mut c = b;
        c.position = [2.0, 3.0];
        let mut d = c;
        d.position = [4.0, 8.0];

        let mut pending = None;
        record_collision_motion(&mut pending, a, b);
        record_collision_motion(&mut pending, b, c);
        record_collision_motion(&mut pending, c, d);

        let segment = pending.expect("segment");

        assert!(approx(segment.prev_ship.position[0], 0.0));
        assert!(approx(segment.prev_ship.position[1], 0.0));
        assert!(approx(segment.next_ship.position[0], 4.0));
        assert!(approx(segment.next_ship.position[1], 8.0));
    }

    #[test]
    fn paused_state_resolves_in_flight_collision_result() {
        let params = spout::game_params::GameParams::default();
        let mut prev_ship = spout::ship::ShipState::default();
        prev_ship.position = [10.0, 20.0];
        let mut next_ship = prev_ship;
        next_ship.position = [10.0, 42.0];

        let mut state = AppState::Paused(Play {
            ship_state: next_ship,
            prev_ship_state: prev_ship,
            in_flight_collision_segment: Some(PendingCollisionSegment {
                prev_ship,
                next_ship,
            }),
            ..Default::default()
        });

        let death = state.resolve_collision_result(
            &params,
            spout::collision::CollisionResult {
                hit: false,
                normal: [0.0, 0.0],
                impact_t: 1.0,
            },
        );

        assert_eq!(death, None);
        let AppState::Paused(play) = state else {
            panic!("state should remain paused after a clean collision result");
        };
        assert!(play.in_flight_collision_segment.is_none());
        assert_eq!(play.progress_height, 42);
    }
}
