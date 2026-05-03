//! Spout entry point: game loop, state machine, and winit event handling.

mod audio;
#[path = "../examples/framework.rs"]
mod framework;

use web_time::Instant;

use spout::background;
use spout::bloom;
use spout::collision;
use spout::game_params;
use spout::input::{InputCollector, InputState};
use spout::level_manager;
use spout::particles;
use spout::render;
use spout::ship;
use spout::text;

/// Shortest signed angular distance from `current` to `target`, in [-π, π].
fn angle_diff(target: f32, current: f32) -> f32 {
    let d = glam::Vec2::from_angle(target - current);
    d.y.atan2(d.x) // equivalent to wrapping (target-current) to [-π, π]
}

/// Time budget per frame for background level generation (≈ 1/300 s).
const LEVEL_BUDGET: std::time::Duration = std::time::Duration::from_nanos(3_333_333);

/// Maximum physics step. Caps dt so that GPU stalls or level-loading pauses
/// don't cause the ship and particles to simulate a huge time jump.
const MAX_FRAME_DT: std::time::Duration = std::time::Duration::from_millis(50);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum GameMode {
    /// Title screen: "SPOUT" rendered as terrain, eroded by a fixed emitter.
    #[default]
    Title,
    /// Normal gameplay.
    Playing,
}

#[derive(Debug, Default)]
struct GameState {
    mode: GameMode,
    input_state: InputState,
    prev_input_state: InputState,
    ship_state: ship::ShipState,
    prev_ship_state: ship::ShipState,
    viewport_offset: i32,
    score: i32,
    paused: bool,
    reset_requested: bool,
    dead: bool,
    explosion_pending: bool,
}

struct Spout {
    game_params: game_params::GameParams,
    state: GameState,
    collector: InputCollector,
    level_manager: level_manager::LevelManager,
    game_time: std::time::Duration,
    iteration_start: Instant,
    game_view_texture: wgpu::TextureView,
    upscaled_view: wgpu::TextureView,
    bloom: bloom::Bloom,
    renderer: render::Render,
    particle_system: particles::ParticleSystem,
    ship_renderer: ship::ShipRenderer,
    collision_detector: collision::CollisionDetector,
    background: background::BackgroundRenderer,
    /// Renders into the game view (240x135) — pixel-perfect with terrain/particles.
    game_text: text::TextRenderer,
    /// Renders at display resolution on top of everything — for debug info.
    overlay_text: text::TextRenderer,
    audio: audio::AudioPlayer,
    staging_belt: wgpu::util::StagingBelt,
    show_debug_overlay: bool,
    frame_times: Vec<f32>,
    tick_wall_dt: f32,
}

impl Spout {
    fn enter_title(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // Preserve input state across transitions to avoid false edges
        // (e.g. fullscreen toggle firing because prev_input was cleared).
        let prev_input = self.state.prev_input_state;
        let cur_input = self.state.input_state;
        self.state = GameState {
            mode: GameMode::Title,
            prev_input_state: prev_input,
            input_state: cur_input,
            ..Default::default()
        };
        self.game_time = std::time::Duration::default();
        self.iteration_start = Instant::now();

        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.level_manager = level_manager::LevelManager::init_title(
            device,
            &self.game_params,
            0,
            &mut init_encoder,
            &mut self.staging_belt,
        );

        self.particle_system = particles::ParticleSystem::new(
            device,
            &self.game_params,
            &mut init_encoder,
            &self.level_manager,
        );

        self.collision_detector.result = collision::CollisionResult::default();

        self.particle_system.set_nozzle_speed(75.0, 75.0);

        self.staging_belt.finish();
        queue.submit(Some(init_encoder.finish()));
        self.staging_belt.recall();

        log::info!("Entered title screen");
    }

    fn start_game(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let prev_input = self.state.prev_input_state;
        let cur_input = self.state.input_state;
        self.state = GameState {
            mode: GameMode::Playing,
            prev_input_state: prev_input,
            input_state: cur_input,
            ship_state: ship::ShipState::init(
                &self.game_params.ship_params,
                [
                    (self.game_params.viewport_width / 2) as f32 + 0.5,
                    (self.game_params.viewport_height / 2) as f32 + 0.5,
                ],
            ),
            ..Default::default()
        };
        self.game_time = std::time::Duration::default();
        self.iteration_start = Instant::now();

        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.level_manager = level_manager::LevelManager::init(
            device,
            &self.game_params,
            0,
            &mut init_encoder,
            &mut self.staging_belt,
        );

        self.particle_system = particles::ParticleSystem::new(
            device,
            &self.game_params,
            &mut init_encoder,
            &self.level_manager,
        );

        self.collision_detector.result = collision::CollisionResult::default();

        // Restore normal emission speed for gameplay.
        let base_speed = self.game_params.particle_system_params.emission_speed;
        self.particle_system
            .set_nozzle_speed(base_speed, base_speed);

        self.staging_belt.finish();
        queue.submit(Some(init_encoder.finish()));
        self.staging_belt.recall();

        log::info!("Game started");
    }

    fn tick(&mut self) -> (f32, f32) {
        let now = Instant::now();
        let delta_t = (now - self.iteration_start).min(MAX_FRAME_DT);
        self.iteration_start = now;

        if self.state.paused {
            (0.0, delta_t.as_secs_f32())
        } else {
            self.game_time += delta_t;
            (delta_t.as_secs_f32(), delta_t.as_secs_f32())
        }
    }

    fn update_paused(&mut self) {
        if self.state.input_state.pause && !self.state.prev_input_state.pause {
            // new pause signal.
            self.state.paused = !self.state.paused;
            if self.state.paused {
                log::info!("Paused game at t={:#?}", self.game_time);
            } else {
                log::info!("Unpaused game at t={:#?}", self.game_time);
            }
        }
    }

    fn update_ship(&mut self, dt: f32) {
        let gravity = self.game_params.particle_system_params.gravity;

        if self.state.dead {
            // Dead: no thrust or rotation, just gravity pulls it down.
            self.state.ship_state.update(dt, 0.0, 0.0, gravity);
            return;
        }

        let input_state = self.state.input_state;
        let rotate = if let Some(target) = input_state.target_heading {
            // Bang-bang controller: rotate at full speed toward target heading,
            // stop when within one frame's worth of rotation to avoid oscillation.
            let current = self.state.ship_state.orientation;
            let error = angle_diff(target, current);
            let dead = self.state.ship_state.rotation_rate * dt;
            if error.abs() <= dead {
                0.0
            } else {
                error.signum()
            }
        } else {
            input_state.rotate
        };
        self.state
            .ship_state
            .update(dt, input_state.thrust, rotate, gravity);

        // Kill the ship if it flies off the horizontal edges.
        let x = self.state.ship_state.position[0];
        if x < 0.0 || x >= self.game_params.level_width as f32 {
            self.state.dead = true;
            self.state.explosion_pending = true;
        }
    }

    fn title_emitter_motion(&self) -> particles::EmitterMotion {
        // Emit from the right side above the title, angled slightly up-left.
        let w = self.game_params.viewport_width as f32;
        let h = self.game_params.viewport_height as f32;
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

    fn update_particle_system(&mut self, dt: f32, prev_ship: &ship::ShipState) {
        let maybe_motion = match self.state.mode {
            GameMode::Title => {
                // Always emit from the fixed title emitter.
                Some(self.title_emitter_motion())
            }
            GameMode::Playing => {
                let current_ship = &self.state.ship_state;
                if self.state.input_state.thrust > 0.0 && !self.state.dead {
                    let start_emitter = prev_ship.get_emitter_state();
                    let end_emitter = current_ship.get_emitter_state();
                    Some(particles::EmitterMotion {
                        position_start: start_emitter.0,
                        position_end: end_emitter.0,
                        velocity_start: prev_ship.velocity,
                        velocity_end: current_ship.velocity,
                        angle_start: start_emitter.1,
                        angle_end: end_emitter.1,
                        ..Default::default()
                    })
                } else {
                    None
                }
            }
        };

        // Updates state, but doesn't run GPU just yet.
        self.particle_system
            .update_state(dt, self.state.viewport_offset, maybe_motion);
    }

    fn update_viewport_height(&mut self) {
        let ship_height = self.state.ship_state.position[1] as i32;
        self.state.score = std::cmp::max(ship_height, self.state.score);
        self.state.viewport_offset =
            self.state.score - (self.game_params.viewport_height / 2) as i32;
    }

    /// Mostly responsible for updating superficial state based on new inputs.
    fn update_state(&mut self) {
        self.audio.poll();

        // Snapshot all input sources into logical InputState for this frame.
        self.state.prev_input_state = self.state.input_state;
        self.state.input_state = self.collector.current_state();

        self.update_paused();

        self.level_manager
            .level_maker
            .work_until(Instant::now() + LEVEL_BUDGET);

        let (game_dt, wall_dt) = self.tick();
        self.tick_wall_dt = wall_dt;

        match self.state.mode {
            GameMode::Title => {
                // No ship physics or collision in title mode.
                // Just run particles to erode the title text.
                let prev_ship = self.state.ship_state;
                self.update_particle_system(game_dt, &prev_ship);
            }
            GameMode::Playing => {
                // Process input state integrated over passage of time.
                self.state.prev_ship_state = self.state.ship_state;
                let prev_ship = self.state.ship_state;
                self.update_ship(game_dt);

                // Poll GPU collision result from last frame (1-frame latency).
                if !self.state.dead && self.collision_detector.result.hit {
                    self.state.dead = true;
                    self.state.explosion_pending = true;
                    log::info!(
                        "Ship collided with terrain at ({:.0}, {:.0})",
                        self.state.ship_state.position[0],
                        self.state.ship_state.position[1]
                    );
                }

                if !self.state.dead {
                    self.update_viewport_height();
                }

                self.update_particle_system(game_dt, &prev_ship);
            }
        }

        // Update camera state.
        self.renderer.update_state(
            wall_dt,
            &self.state.input_state,
            &self.state.prev_input_state,
        );
    }
}

impl framework::Example for Spout {
    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::downlevel_defaults()
    }

    fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::COMPUTE_SHADERS,
            ..Default::default()
        }
    }

    fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: &winit::window::Window,
    ) -> Self {
        window.set_cursor_visible(false);
        let game_params = game_params::get_game_config_from_default_file();
        // Start in title mode — no ship needed yet.
        let game_state = GameState::default();

        let game_view_texture = make_texture(
            device,
            game_params.viewport_width,
            game_params.viewport_height,
        );

        let upscaled_view = make_texture(device, config.width, config.height);

        let bloom = bloom::Bloom::new(
            device,
            config.width,
            config.height,
            &upscaled_view,
            &game_params.visual_params,
        );

        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Chunk size covers one terrain tile upload; the belt grows as needed.
        let mut staging_belt = wgpu::util::StagingBelt::new(
            device.clone(),
            (game_params.level_width * game_params.level_height * 4) as u64,
        );

        // Use a placeholder level manager for construction; enter_title() will
        // replace it immediately after.
        let level_manager = level_manager::LevelManager::init(
            device,
            &game_params,
            0,
            &mut init_encoder,
            &mut staging_belt,
        );

        let renderer = render::Render::init(
            config,
            &game_params,
            adapter,
            device,
            queue,
            &game_view_texture,
            &upscaled_view,
            bloom.bloom_view(),
        );

        let particle_system =
            particles::ParticleSystem::new(device, &game_params, &mut init_encoder, &level_manager);

        let ship_renderer = ship::ShipRenderer::init(device);
        let collision_detector = collision::CollisionDetector::init(device);
        let background = background::BackgroundRenderer::init(device, queue);

        let game_text = text::TextRenderer::init(
            device,
            queue,
            bloom::GAME_VIEW_FORMAT,
            game_params.viewport_width,
            game_params.viewport_height,
            text::YDirection::Up,
            text::Font::O4b11,
        );
        let overlay_text = text::TextRenderer::init(
            device,
            queue,
            config.format,
            config.width,
            config.height,
            text::YDirection::Down,
            text::Font::O4b11,
        );

        staging_belt.finish();
        queue.submit(Some(init_encoder.finish()));
        staging_belt.recall();

        let audio = if game_params.music_starts_on {
            audio::AudioPlayer::new()
        } else {
            audio::AudioPlayer::disabled()
        };

        let mut collector = InputCollector::default();

        #[cfg(not(target_arch = "wasm32"))]
        collector.set_surface_width(config.width as f32);

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            if let Some(canvas) = window.canvas() {
                collector.init_touch(canvas);
            }
        }

        let mut spout = Spout {
            game_params,
            state: game_state,
            collector,
            level_manager,
            game_time: std::time::Duration::default(),
            iteration_start: Instant::now(),
            game_view_texture,
            upscaled_view,
            bloom,
            renderer,
            particle_system,
            ship_renderer,
            collision_detector,
            background,
            game_text,
            overlay_text,
            audio,
            staging_belt,
            show_debug_overlay: true,
            frame_times: Vec::with_capacity(60),
            tick_wall_dt: 0.0,
        };
        spout.enter_title(device, queue);
        spout
    }

    fn update(&mut self, event: winit::event::WindowEvent) {
        self.collector.handle_winit_event(&event);

        // Touch-to-restart: any tap while the ship is dead restarts the game.
        #[cfg(not(target_arch = "wasm32"))]
        if let winit::event::WindowEvent::Touch(touch) = &event {
            use winit::event::TouchPhase;
            if touch.phase == TouchPhase::Started && self.state.dead {
                self.state.reset_requested = true;
            }
        }

        // One-shot audio actions are handled here directly since they are
        // immediate commands, not held state.
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
            if state == winit::event::ElementState::Pressed {
                match key {
                    KeyCode::KeyT => self.audio.next_track(),
                    KeyCode::KeyY => self.audio.toggle(),
                    KeyCode::KeyR => self.state.reset_requested = true,
                    KeyCode::F3 => self.show_debug_overlay = !self.show_debug_overlay,
                    _ => {}
                }
            }
        }
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        self.collector.set_surface_width(config.width as f32);

        let new_upscaled = make_texture(device, config.width, config.height);
        self.bloom = bloom::Bloom::new(
            device,
            config.width,
            config.height,
            &new_upscaled,
            &self.game_params.visual_params,
        );
        self.renderer
            .resize(config, device, &new_upscaled, self.bloom.bloom_view());
        self.upscaled_view = new_upscaled;
        self.overlay_text.resize(queue, config.width, config.height);
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _spawner: &framework::Spawner,
        window: &winit::window::Window,
    ) {
        // Read back GPU collision result from previous frame.
        if self.state.mode == GameMode::Playing {
            self.collision_detector.poll_result();
        }

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Update input and game state first, so transitions see correct prev/current.
        self.update_state();

        // Fullscreen toggle (edge-triggered, independent of game state).
        if !self.state.prev_input_state.fullscreen && self.state.input_state.fullscreen {
            if window.fullscreen().is_some() {
                log::info!("Setting windowed mode.");
                window.set_fullscreen(None);
            } else {
                log::info!("Setting borderless fullscreen.");
                window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            }
        }

        // Handle mode transitions after input is updated.
        if self.state.reset_requested {
            self.state.reset_requested = false;
            self.enter_title(device, queue);
        } else if self.state.mode == GameMode::Title {
            // Start game on a NEW press of thrust or rotate (edge, not held).
            let input = &self.state.input_state;
            let prev = &self.state.prev_input_state;
            let new_thrust = input.thrust > 0.0 && prev.thrust == 0.0;
            let new_rotate = input.rotate.abs() > 0.0 && prev.rotate.abs() == 0.0;
            if new_thrust || new_rotate {
                self.start_game(device, queue);
            }
        }

        self.level_manager.sync_height(
            device,
            self.state.viewport_offset,
            &mut encoder,
            &self.game_params,
            &mut self.staging_belt,
        );

        // Ship explosion burst — write particles before compute runs.
        if self.state.explosion_pending {
            self.state.explosion_pending = false;
            let ship = &self.state.ship_state;
            self.particle_system.emit_burst(
                &mut encoder,
                &mut self.staging_belt,
                ship.position,
                ship.velocity,
                50000, // burst count
                self.game_params.particle_system_params.emission_speed,
                self.game_params.particle_system_params.max_particle_life,
            );
        }

        // Run compute pipeline(s).
        self.level_manager.compose_tiles(&mut encoder);
        self.particle_system
            .run_compute(&self.level_manager, &mut encoder, &mut self.staging_belt);

        // Collision detection — only during gameplay.
        if self.state.mode == GameMode::Playing {
            self.collision_detector.dispatch(
                device,
                &mut encoder,
                &mut self.staging_belt,
                &self.state.ship_state,
                &self.state.prev_ship_state,
                self.level_manager.terrain_buffer(),
                self.game_params.level_width,
            );
        }

        // Render background (clears and draws tiled background).
        self.background.update_state(
            &self.game_params,
            self.state.viewport_offset,
            &mut encoder,
            &mut self.staging_belt,
        );
        self.background
            .render(&self.game_view_texture, &mut encoder);

        // Render terrain (on top of background).
        self.level_manager
            .terrain_renderer
            .render(&self.game_view_texture, &mut encoder);

        // Render particles.
        self.particle_system
            .render(&self.game_view_texture, &mut encoder);

        // Render ship — only during gameplay, and not when dead.
        if self.state.mode == GameMode::Playing && self.game_params.render_ship && !self.state.dead
        {
            self.ship_renderer.render(
                &self.state.ship_state,
                &self.game_params,
                self.state.viewport_offset,
                &self.game_view_texture,
                &mut encoder,
                &mut self.staging_belt,
            );
        }

        // In-game HUD — only during gameplay.
        if self.state.mode == GameMode::Playing {
            let current_level = self.state.score / self.game_params.level_height as i32 + 1;
            let score_text = format!("{}", self.state.score);
            let level_text = format!("LV{}", current_level);
            let white = [1.0, 1.0, 1.0, 1.0];
            let level_x =
                self.game_text.surface_width - self.game_text.text_width(&level_text, 1.0) - 2.0;
            self.game_text.draw(
                device,
                &mut encoder,
                &self.game_view_texture,
                &[
                    (&score_text, 2.0, 2.0, 1.0, white),
                    (&level_text, level_x, 2.0, 1.0, white),
                ],
            );

            // Game over overlay.
            if self.state.dead {
                let w = self.game_text.surface_width;
                let h = self.game_text.surface_height;
                let white = [1.0, 1.0, 1.0, 1.0];
                let dim = [0.6, 0.6, 0.6, 1.0];

                let go = "GAME OVER";
                let go_x = (w - self.game_text.text_width(go, 1.0)) / 2.0;
                let go_y = h / 2.0 - 10.0;

                let sc = format!("SCORE {}", self.state.score);
                let sc_x = (w - self.game_text.text_width(&sc, 1.0)) / 2.0;
                let sc_y = go_y - 18.0;

                #[cfg(target_os = "ios")]
                let restart = "TAP TO RESTART";
                #[cfg(not(target_os = "ios"))]
                let restart = "R TO RESTART";
                let r_x = (w - self.game_text.text_width(restart, 1.0)) / 2.0;
                let r_y = sc_y - 18.0;

                self.game_text.draw(
                    device,
                    &mut encoder,
                    &self.game_view_texture,
                    &[
                        (go, go_x, go_y, 1.0, white),
                        (&sc, sc_x, sc_y, 1.0, white),
                        (restart, r_x, r_y, 1.0, dim),
                    ],
                );
            }
        }

        // Blit game view (240×135) → upscaled HDR (surface resolution).
        self.renderer
            .blit(&self.upscaled_view, &mut encoder, &mut self.staging_belt);

        // Run bloom post-process at full surface resolution (threshold + blur).
        self.bloom.render(&mut encoder);

        // Composite upscaled HDR + bloom → surface (LDR).
        self.renderer.render(view, &mut encoder);
        self.level_manager.decompose_tiles(&mut encoder);

        // Debug overlay (FPS) — renders at display resolution on top of everything.
        if self.show_debug_overlay {
            let dt = self.tick_wall_dt;
            if self.frame_times.len() >= 60 {
                self.frame_times.remove(0);
            }
            self.frame_times.push(dt);

            let avg_dt: f32 = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            let fps = if avg_dt > 0.0 { 1.0 / avg_dt } else { 0.0 };
            let fps_text = format!("FPS: {:.0}", fps);

            let white = [1.0, 1.0, 1.0, 1.0];
            self.overlay_text.draw(
                device,
                &mut encoder,
                view,
                &[(fps_text.as_str(), 8.0, 8.0, 1.0, white)],
            );
        }

        self.staging_belt.finish();
        queue.submit(Some(encoder.finish()));
        // Drain wgpu callbacks from the previous frame's completed GPU work (non-blocking).
        // This fires the map_async callback set by start_readback() last frame, so that
        // poll_result() at the top of the next frame sees map_ready == true without stalling.
        #[cfg(not(target_arch = "wasm32"))]
        device.poll(wgpu::PollType::Poll).ok();
        self.staging_belt.recall();

        // Initiate async readback of collision result now that GPU work is submitted.
        // On native the callback fires during the next poll(); on WASM it fires
        // asynchronously before the next frame.
        self.collision_detector.start_readback();

        {
            // After rendering, do some "async" work:
            let deadline = self.iteration_start + LEVEL_BUDGET;
            self.level_manager.level_maker.work_until(deadline);
        }
    }
}

fn make_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: bloom::GAME_VIEW_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

fn main() {
    framework::run::<Spout>("Spout");
}

#[cfg(test)]
mod tests {
    use super::angle_diff;
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
}
