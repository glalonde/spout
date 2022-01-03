mod camera;
#[path = "../examples/framework.rs"]
mod framework;
mod game_params;
mod int_grid;
mod render;
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
    renderer: render::Render,
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

    fn update_state(&mut self) {
        self.update_paused();

        // let target_duration = std::time::Duration::from_secs_f64(1.0 / self.game_params.fps);
        let dt = self.tick();

        self.renderer.update_state(dt, &self.state.input_state);

        // Process input state integrated over passage of time.
        self.update_ship(dt);

        // Finished processing input, set previous input state.
        self.state.prev_input_state = self.state.input_state;
    }
}

impl framework::Example for Spout {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::PIPELINE_STATISTICS_QUERY
    }

    fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let game_params = game_params::get_game_config_from_default_file();
        let game_state = GameState::default();
        let renderer = render::Render::init(config, adapter, device, queue);

        Spout {
            game_params,
            state: game_state,
            game_time: std::time::Duration::default(),
            iteration_start: instant::Instant::now(),
            renderer,
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
        _spawner: &framework::Spawner,
    ) {
        self.update_state();
        // Run compute pipeline.
        // TODO

        // Run render pipeline.
        self.renderer.render(view, device, queue);
    }
}

fn main() {
    framework::run::<Spout>("Spout");
}
