//! Spout entry point: game loop, state machine, and winit event handling.

mod audio;
#[path = "../examples/framework.rs"]
mod framework;

use web_time::Instant;

use spout::bloom;
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

#[derive(Debug, Default)]
struct GameState {
    input_state: InputState,
    prev_input_state: InputState,
    ship_state: ship::ShipState,
    viewport_offset: i32,
    score: i32,
    paused: bool,
    reset_requested: bool,
    dead: bool,
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
    fn reset(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.state = GameState {
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

        self.staging_belt.finish();
        queue.submit(Some(init_encoder.finish()));
        self.staging_belt.recall();

        log::info!("Game reset");
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
        self.state.ship_state.update(dt, input_state.thrust, rotate);
    }

    fn update_particle_system(&mut self, dt: f32, prev_ship: &ship::ShipState) {
        let current_ship = &self.state.ship_state;
        let maybe_motion = if self.state.input_state.thrust > 0.0 && !self.state.dead {
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

        // Process input state integrated over passage of time.
        let prev_ship = self.state.ship_state;
        if !self.state.dead {
            self.update_ship(game_dt);
        }

        // Check collision with terrain (CPU-side initial data).
        if !self.state.dead
            && self
                .level_manager
                .check_ship_collision(&self.state.ship_state)
        {
            self.state.dead = true;
            log::info!(
                "Ship collided with terrain at ({:.0}, {:.0})",
                self.state.ship_state.position[0],
                self.state.ship_state.position[1]
            );
        }

        self.update_viewport_height();

        self.update_particle_system(game_dt, &prev_ship);

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
        let game_state = GameState {
            ship_state: ship::ShipState::init(
                &game_params.ship_params,
                [
                    (game_params.viewport_width / 2) as f32 + 0.5,
                    (game_params.viewport_height / 2) as f32 + 0.5,
                ],
            ),
            ..Default::default()
        };

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

        Spout {
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
            game_text,
            overlay_text,
            audio,
            staging_belt,
            show_debug_overlay: true,
            frame_times: Vec::with_capacity(60),
            tick_wall_dt: 0.0,
        }
    }

    fn update(&mut self, event: winit::event::WindowEvent) {
        self.collector.handle_winit_event(&event);

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
        if self.state.reset_requested {
            self.reset(device, queue);
        }

        {
            if !self.state.prev_input_state.fullscreen && self.state.input_state.fullscreen {
                if window.fullscreen().is_some() {
                    log::info!("Setting windowed mode.");
                    window.set_fullscreen(None);
                } else {
                    // Borderless fullscreen — instant, no display mode switch or
                    // macOS Space animation. Just covers the screen.
                    log::info!("Setting borderless fullscreen.");
                    window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                }
            }
        }

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.update_state();

        self.level_manager.sync_height(
            device,
            self.state.viewport_offset,
            &mut encoder,
            &self.game_params,
            &mut self.staging_belt,
        );

        // Run compute pipeline(s).
        self.level_manager.compose_tiles(&mut encoder);
        self.particle_system
            .run_compute(&self.level_manager, &mut encoder, &mut self.staging_belt);

        // Render terrain.
        self.level_manager
            .terrain_renderer
            .render(&self.game_view_texture, &mut encoder);

        // Render particles.
        self.particle_system
            .render(&self.game_view_texture, &mut encoder);

        // Render ship
        if self.game_params.render_ship {
            self.ship_renderer.render(
                &self.state.ship_state,
                &self.game_params,
                self.state.viewport_offset,
                &self.game_view_texture,
                &mut encoder,
                &mut self.staging_belt,
            );
        }

        // In-game HUD — renders into game view at native pixel resolution.
        // Goes through bloom + CRT with everything else.
        {
            let score_text = format!("{}", self.state.score);
            let white = [1.0, 1.0, 1.0, 1.0];
            let score_y = 2.0;
            self.game_text.draw(
                device,
                &mut encoder,
                &self.game_view_texture,
                &[(&score_text, 2.0, score_y, 1.0, white)],
            );
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
        self.staging_belt.recall();

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
