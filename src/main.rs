mod camera;
mod color_maps;
mod emitter;
#[path = "../examples/framework.rs"]
mod framework;
mod game_params;
mod render;
mod shader_util;
mod ship;
mod textured_quad;

#[derive(Debug, Copy, Clone)]
pub struct InputState {
    forward: bool,
    left: bool,
    right: bool,
    pause: bool,

    // Camera controls:
    cam_up: bool,
    cam_down: bool,
    cam_left: bool,
    cam_right: bool,
}
impl Default for InputState {
    fn default() -> Self {
        InputState {
            forward: false,
            left: false,
            right: false,
            pause: false,

            cam_up: false,
            cam_down: false,
            cam_left: false,
            cam_right: false,
        }
    }
}

#[derive(Debug)]
struct GameState {
    input_state: InputState,
    prev_input_state: InputState,
    ship_state: ship::ShipState,
    score: i32,
    paused: bool,
}
impl Default for GameState {
    fn default() -> Self {
        GameState {
            input_state: InputState::default(),
            prev_input_state: InputState::default(),
            ship_state: ship::ShipState::default(),
            score: 0,
            paused: false,
        }
    }
}

struct Spout {
    game_params: game_params::GameParams,
    state: GameState,
    // level_manager: level_manager::LevelManager,
    game_time: std::time::Duration,
    iteration_start: instant::Instant,
    game_view_texture: wgpu::TextureView,
    renderer: render::Render,
    // emitter: emitter::Emitter,
    particle_system: emitter::ParticleSystem,
    // staging_belt: wgpu::util::StagingBelt,

    // fps: fps_estimator::FpsEstimator,
    /*
    compute_locals: super::particle_system::ComputeLocals,
    pre_glow_texture: wgpu::TextureView,
    post_glow_texture: wgpu::TextureView,
    game_view_texture: wgpu::TextureView,
    terrain_renderer: super::terrain_renderer::TerrainRenderer,
    particle_renderer: super::particle_system::ParticleRenderer,
    glow_renderer: super::glow_pass::GlowRenderer,
    ship_renderer: super::ship::ShipRenderer,
    viewport: super::viewport::Viewport,
    debug_overlay: super::debug_overlay::DebugOverlay,
    text_renderer: super::text_renderer::TextRenderer,
    game_viewport: super::game_viewport::GameViewport,
    */
}

impl Spout {
    fn tick(&mut self) -> f32 {
        let now = instant::Instant::now();
        let delta_t = now - self.iteration_start;
        self.iteration_start = now;

        if self.state.paused {
            return 0.0;
        } else {
            self.game_time += delta_t;
            return delta_t.as_secs_f32();
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

    fn update_particle_system(&mut self, dt: f32) {
        let do_emit = self.state.input_state.forward;
        self.particle_system.update_state(dt, do_emit);
    }

    /// Mostly responsible for updating superficial state based on new inputs.
    fn update_state(&mut self) {
        self.update_paused();

        // let target_duration = std::time::Duration::from_secs_f64(1.0 / self.game_params.fps);
        let dt = self.tick();

        // Process input state integrated over passage of time.
        self.update_ship(dt);

        self.update_particle_system(dt);

        // Update camera state.
        self.renderer.update_state(dt, &self.state.input_state);

        // Finished processing input, set previous input state.
        self.state.prev_input_state = self.state.input_state;
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
    ) -> Self {
        let game_params = game_params::get_game_config_from_default_file();
        let game_state = GameState::default();

        let game_view_texture = make_texture(
            device,
            game_params.viewport_width,
            game_params.viewport_height,
        );

        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let renderer = render::Render::init(config, adapter, device, &game_view_texture);

        // TODO load params from config.
        // TODO render to game texture view.
        let particle_system = emitter::ParticleSystem::new(device, &game_params, &mut init_encoder);

        queue.submit(Some(init_encoder.finish()));

        Spout {
            game_params,
            state: game_state,
            game_time: std::time::Duration::default(),
            iteration_start: instant::Instant::now(),
            game_view_texture,
            renderer,
            particle_system,
        }
    }

    fn update(&mut self, event: winit::event::WindowEvent) {
        // Update inpute state
        macro_rules! bind_keys {
            ($input:expr, $($pat:pat => $result:expr),*) => (
                            match $input {
                                    $(
                            winit::event::KeyboardInput {
                                virtual_keycode: Some($pat),
                                state,
                                ..
                            } => match state {
                                winit::event::ElementState::Pressed =>  $result = true,
                                winit::event::ElementState::Released => $result = false,
                            }
                        ),*
                    _ => (),
                }
            );
        }
        match event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => bind_keys!(input,
                // Ship motion bindings
                winit::event::VirtualKeyCode::W => self.state.input_state.forward,
                winit::event::VirtualKeyCode::A => self.state.input_state.left,
                winit::event::VirtualKeyCode::P => self.state.input_state.pause,
                winit::event::VirtualKeyCode::D => self.state.input_state.right,

                // Camera bindings
                winit::event::VirtualKeyCode::I => self.state.input_state.cam_up,
                winit::event::VirtualKeyCode::K => self.state.input_state.cam_down,
                winit::event::VirtualKeyCode::J => self.state.input_state.cam_left,
                winit::event::VirtualKeyCode::L => self.state.input_state.cam_right
            ),
            _ => (),
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
        spawner: &framework::Spawner,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.update_state();

        // Run compute pipeline(s).
        self.particle_system
            .run_compute(device, &self.game_view_texture, &mut encoder);

        // Run render pipeline.
        self.renderer.render(view, device, &mut encoder);

        queue.submit(Some(encoder.finish()));
        self.particle_system.after_queue_submission(spawner);
        self.renderer.after_queue_submission(spawner);
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
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

fn main() {
    framework::run::<Spout>("Spout");
}
