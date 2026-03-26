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
    audio: audio::AudioPlayer,
    staging_belt: wgpu::util::StagingBelt,
    #[cfg(feature = "profiling")]
    gpu_profiler: wgpu_profiler::GpuProfiler,
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
        self.state
            .ship_state
            .update(dt, input_state.thrust, input_state.rotate);
    }

    fn update_particle_system(&mut self, dt: f32, prev_ship: &ship::ShipState) {
        let current_ship = &self.state.ship_state;
        let maybe_motion = if self.state.input_state.thrust > 0.0 {
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
        #[cfg(feature = "profiling")]
        puffin::profile_function!();

        {
            #[cfg(feature = "profiling")]
            puffin::profile_scope!("audio_poll");
            self.audio.poll();
        }

        // Snapshot all input sources into logical InputState for this frame.
        self.state.prev_input_state = self.state.input_state;
        self.state.input_state = self.collector.current_state();

        self.update_paused();

        {
            #[cfg(feature = "profiling")]
            puffin::profile_scope!("level_gen");
            self.level_manager
                .level_maker
                .work_until(Instant::now() + LEVEL_BUDGET);
        }

        let (game_dt, wall_dt) = self.tick();

        {
            #[cfg(feature = "profiling")]
            puffin::profile_scope!("physics");
            let prev_ship = self.state.ship_state;
            self.update_ship(game_dt);
            self.update_viewport_height();
            self.update_particle_system(game_dt, &prev_ship);
        }

        {
            #[cfg(feature = "profiling")]
            puffin::profile_scope!("camera_update");
            self.renderer.update_state(
                wall_dt,
                &self.state.input_state,
                &self.state.prev_input_state,
            );
        }
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
    fn optional_features() -> wgpu::Features {
        if cfg!(feature = "profiling") {
            wgpu::Features::TIMESTAMP_QUERY
                | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS
                | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES
        } else {
            wgpu::Features::empty()
        }
    }

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

        staging_belt.finish();
        queue.submit(Some(init_encoder.finish()));
        staging_belt.recall();

        let audio = if game_params.music_starts_on {
            audio::AudioPlayer::new()
        } else {
            audio::AudioPlayer::disabled()
        };

        #[cfg(feature = "profiling")]
        let gpu_profiler = wgpu_profiler::GpuProfiler::new(device, wgpu_profiler::GpuProfilerSettings {
            enable_timer_queries: device
                .features()
                .contains(wgpu::Features::TIMESTAMP_QUERY),
            enable_debug_groups: true,
            ..Default::default()
        })
        .expect("Failed to create GPU profiler"); // safe: settings are valid

        #[cfg(feature = "profiling")]
        {
            let server_addr = format!("0.0.0.0:{}", puffin_http::DEFAULT_PORT);
            let _server = puffin_http::Server::new(&server_addr)
                .expect("Failed to start puffin server"); // safe: port should be free
            log::info!(
                "Puffin profiling server started on {}. Run `puffin_viewer` to connect.",
                server_addr
            );
            puffin::set_scopes_on(true);
            // Leak the server handle so it lives for the program's lifetime.
            std::mem::forget(_server);

            // Initialize the global frame view so it starts collecting frames.
            // GlobalFrameView::default() registers its own sink with the profiler.
            std::sync::LazyLock::force(&PROFILE_FRAMES);
        }

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
            audio,
            staging_belt,
            #[cfg(feature = "profiling")]
            gpu_profiler,
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
                    _ => {}
                }
            }
        }
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
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
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _spawner: &framework::Spawner,
        window: &winit::window::Window,
    ) {
        #[cfg(feature = "profiling")]
        puffin::profile_function!();

        if self.state.reset_requested {
            self.reset(device, queue);
        }

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

        // --- GPU-profiled passes ---
        // Each scope wraps encoder operations; Scope derefs to CommandEncoder
        // so existing methods work unchanged.

        {
            #[cfg(feature = "profiling")]
            let mut encoder =
                self.gpu_profiler
                    .scope("sync_height", &mut encoder);
            self.level_manager.sync_height(
                device,
                self.state.viewport_offset,
                &mut encoder,
                &self.game_params,
                &mut self.staging_belt,
            );
        }

        {
            #[cfg(feature = "profiling")]
            let mut encoder =
                self.gpu_profiler
                    .scope("compose_tiles", &mut encoder);
            self.level_manager.compose_tiles(&mut encoder);
        }

        {
            #[cfg(feature = "profiling")]
            let mut encoder =
                self.gpu_profiler
                    .scope("particle_compute", &mut encoder);
            self.particle_system
                .run_compute(&self.level_manager, &mut encoder, &mut self.staging_belt);
        }

        {
            #[cfg(feature = "profiling")]
            let mut encoder =
                self.gpu_profiler
                    .scope("terrain_render", &mut encoder);
            self.level_manager
                .terrain_renderer
                .render(&self.game_view_texture, &mut encoder);
        }

        {
            #[cfg(feature = "profiling")]
            let mut encoder =
                self.gpu_profiler
                    .scope("particle_render", &mut encoder);
            self.particle_system
                .render(&self.game_view_texture, &mut encoder);
        }

        if self.game_params.render_ship {
            #[cfg(feature = "profiling")]
            let mut encoder =
                self.gpu_profiler.scope("ship_render", &mut encoder);
            self.ship_renderer.render(
                &self.state.ship_state,
                &self.game_params,
                self.state.viewport_offset,
                &self.game_view_texture,
                &mut encoder,
                &mut self.staging_belt,
            );
        }

        {
            #[cfg(feature = "profiling")]
            let mut encoder =
                self.gpu_profiler.scope("blit", &mut encoder);
            self.renderer
                .blit(&self.upscaled_view, &mut encoder, &mut self.staging_belt);
        }

        {
            #[cfg(feature = "profiling")]
            let mut encoder =
                self.gpu_profiler.scope("bloom", &mut encoder);
            self.bloom.render(&mut encoder);
        }

        {
            #[cfg(feature = "profiling")]
            let mut encoder =
                self.gpu_profiler
                    .scope("composite", &mut encoder);
            self.renderer.render(view, &mut encoder);
        }

        {
            #[cfg(feature = "profiling")]
            let mut encoder =
                self.gpu_profiler
                    .scope("decompose_tiles", &mut encoder);
            self.level_manager.decompose_tiles(&mut encoder);
        }

        {
            #[cfg(feature = "profiling")]
            puffin::profile_scope!("submit");
            self.staging_belt.finish();

            #[cfg(feature = "profiling")]
            self.gpu_profiler.resolve_queries(&mut encoder);

            queue.submit(Some(encoder.finish()));
            self.staging_belt.recall();
        }

        #[cfg(feature = "profiling")]
        self.gpu_profiler.end_frame().unwrap(); // safe: begin/end always paired

        #[cfg(feature = "profiling")]
        if let Some(results) = self.gpu_profiler.process_finished_frame(queue.get_timestamp_period()) {
            puffin_wgpu_log(&results, 0);
        }

        #[cfg(feature = "profiling")]
        puffin::GlobalProfiler::lock().new_frame();

        {
            #[cfg(feature = "profiling")]
            puffin::profile_scope!("level_gen_post");
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

#[cfg(feature = "profiling")]
static PROFILE_FRAMES: std::sync::LazyLock<puffin::GlobalFrameView> =
    std::sync::LazyLock::new(puffin::GlobalFrameView::default);

#[cfg(feature = "profiling")]
fn save_puffin_profile() {
    let path = "profile.puffin";
    let view = PROFILE_FRAMES.lock();
    match std::fs::File::create(path) {
        Ok(mut f) => match view.write(&mut f) {
            Ok(()) => log::info!("Saved profiling data to {path}"),
            Err(e) => log::error!("Failed to write profiling data: {e}"),
        },
        Err(e) => log::error!("Failed to create {path}: {e}"),
    }
}

#[cfg(feature = "profiling")]
fn puffin_wgpu_log(results: &[wgpu_profiler::GpuTimerQueryResult], depth: usize) {
    for r in results {
        if let Some(ref time) = r.time {
            let dur_ms = (time.end - time.start) * 1000.0;
            let indent = "  ".repeat(depth);
            log::info!("{indent}[GPU] {}: {dur_ms:.3}ms", r.label);
        }
        puffin_wgpu_log(&r.nested_queries, depth + 1);
    }
}

fn main() {
    framework::run::<Spout>("Spout");

    #[cfg(feature = "profiling")]
    save_puffin_profile();
}
