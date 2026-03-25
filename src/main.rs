mod audio;
#[path = "../examples/framework.rs"]
mod framework;

use web_time::Instant;

use spout::bloom;
use spout::game_params;
use spout::input::InputState;
use spout::level_manager;
use spout::particles;
use spout::render;
use spout::ship;

/// Time budget per frame for background level generation (≈ 1/300 s).
const LEVEL_BUDGET: std::time::Duration = std::time::Duration::from_nanos(3_333_333);

#[derive(Debug, Default)]
struct GameState {
    input_state: InputState,
    prev_input_state: InputState,
    ship_state: ship::ShipState,
    viewport_offset: i32,
    score: i32,
    paused: bool,
}

struct Spout {
    game_params: game_params::GameParams,
    state: GameState,
    level_manager: level_manager::LevelManager,
    game_time: std::time::Duration,
    iteration_start: Instant,
    game_view_texture: wgpu::TextureView,
    bloom: bloom::Bloom,
    renderer: render::Render,
    particle_system: particles::ParticleSystem,
    ship_renderer: ship::ShipRenderer,
    audio: audio::AudioPlayer,
}

impl Spout {
    fn tick(&mut self) -> (f32, f32) {
        let now = Instant::now();
        let delta_t = now - self.iteration_start;
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
        let ship_state = &mut self.state.ship_state;

        // Update "ship"
        let rotation: ship::RotationDirection = match (input_state.left, input_state.right) {
            (true, false) => ship::RotationDirection::CCW,
            (false, true) => ship::RotationDirection::CW,
            _ => ship::RotationDirection::None,
        };
        ship_state.update(dt, input_state.forward, rotation);
    }

    fn update_particle_system(&mut self, dt: f32, prev_ship: &ship::ShipState) {
        let current_ship = &self.state.ship_state;
        let maybe_motion = if self.state.input_state.forward {
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
        self.update_paused();

        self.level_manager
            .level_maker
            .work_until(Instant::now() + LEVEL_BUDGET);

        let (game_dt, wall_dt) = self.tick();

        // Process input state integrated over passage of time.
        let prev_ship = self.state.ship_state;
        self.update_ship(game_dt);

        self.update_viewport_height();

        self.update_particle_system(game_dt, &prev_ship);

        // Update camera state.
        self.renderer.update_state(
            wall_dt,
            &self.state.input_state,
            &self.state.prev_input_state,
        );

        // Finished processing input, set previous input state.
        self.state.prev_input_state = self.state.input_state;
    }

    fn select_fullscreen_video_mode(
        window: &winit::window::Window,
    ) -> Option<winit::monitor::VideoModeHandle> {
        let mut video_mode: Option<winit::monitor::VideoModeHandle> = None;
        match window.primary_monitor() {
            Some(monitor) => {
                for mode in monitor.video_modes() {
                    if let Some(best_mode) = &video_mode {
                        match mode
                            .refresh_rate_millihertz()
                            .cmp(&best_mode.refresh_rate_millihertz())
                        {
                            std::cmp::Ordering::Greater => {
                                video_mode = Some(mode);
                            }
                            std::cmp::Ordering::Equal => {
                                let best_area = best_mode.size().width * best_mode.size().height;
                                let current_area = mode.size().width * mode.size().height;
                                if best_area < current_area {
                                    video_mode = Some(mode);
                                }
                            }
                            std::cmp::Ordering::Less => {}
                        }
                    } else {
                        video_mode = Some(mode);
                    }
                }
            }
            None => {
                log::info!("No primary monitor detected.");
            }
        };
        video_mode
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

        let bloom = bloom::Bloom::new(
            device,
            game_params.viewport_width,
            game_params.viewport_height,
            &game_view_texture,
        );

        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let level_manager =
            level_manager::LevelManager::init(device, &game_params, 0, &mut init_encoder);

        let renderer = render::Render::init(
            config,
            &game_params,
            adapter,
            device,
            queue,
            &game_view_texture,
            bloom.bloom_view(),
        );

        let particle_system =
            particles::ParticleSystem::new(device, &game_params, &mut init_encoder, &level_manager);

        let ship_renderer = ship::ShipRenderer::init(device);

        queue.submit(Some(init_encoder.finish()));

        let audio = if game_params.music_starts_on {
            audio::AudioPlayer::new()
        } else {
            audio::AudioPlayer::disabled()
        };

        Spout {
            game_params,
            state: game_state,
            level_manager,
            game_time: std::time::Duration::default(),
            iteration_start: Instant::now(),
            game_view_texture,
            bloom,
            renderer,
            particle_system,
            ship_renderer,
            audio,
        }
    }

    fn update(&mut self, event: winit::event::WindowEvent) {
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
            let pressed = state == winit::event::ElementState::Pressed;
            match key {
                // Ship motion bindings
                KeyCode::KeyW => self.state.input_state.forward = pressed,
                KeyCode::KeyA => self.state.input_state.left = pressed,
                KeyCode::KeyP => self.state.input_state.pause = pressed,
                KeyCode::KeyD => self.state.input_state.right = pressed,

                // Camera bindings
                KeyCode::KeyU => self.state.input_state.cam_in = pressed,
                KeyCode::KeyO => self.state.input_state.cam_out = pressed,
                KeyCode::KeyI => self.state.input_state.cam_up = pressed,
                KeyCode::KeyK => self.state.input_state.cam_down = pressed,
                KeyCode::KeyJ => self.state.input_state.cam_left = pressed,
                KeyCode::KeyL => self.state.input_state.cam_right = pressed,
                KeyCode::KeyN => self.state.input_state.cam_perspective = pressed,
                KeyCode::KeyM => self.state.input_state.cam_reset = pressed,

                // Full screen
                KeyCode::KeyF => self.state.input_state.fullscreen = pressed,

                // Skip to next music track (on key-down only)
                KeyCode::KeyT => {
                    if pressed {
                        self.audio.next_track();
                    }
                }

                _ => {}
            }
        }
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.renderer.resize(config);
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _spawner: &framework::Spawner,
        window: &winit::window::Window,
    ) {
        {
            if !self.state.prev_input_state.fullscreen && self.state.input_state.fullscreen {
                if window.fullscreen().is_some() {
                    // Set unfullscreen.
                    log::info!("Setting windowed mode.");
                    window.set_fullscreen(None);
                } else if let Some(best_mode) = Spout::select_fullscreen_video_mode(window) {
                    log::info!("Setting exclusive fullscreen with mode: {}", best_mode);
                    window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(best_mode)));
                } else {
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
        );

        // Run compute pipeline(s).
        self.level_manager.compose_tiles(&mut encoder);
        self.particle_system
            .run_compute(&self.level_manager, &mut encoder);

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
            );
        }

        // Run bloom post-process (threshold + blur).
        self.bloom.render(&mut encoder);

        // Render the game view quad.
        self.renderer.render(view, &mut encoder);
        self.level_manager.decompose_tiles(&mut encoder);

        queue.submit(Some(encoder.finish()));
        self.ship_renderer.after_queue_submission();
        self.particle_system.after_queue_submission();
        self.renderer.after_queue_submission();
        self.level_manager.after_queue_submission();

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
