#[path = "../examples/framework.rs"]
mod framework;
use log::{info, trace};

gflags::define! {
    --width: u32 = 320
}
gflags::define! {
    --height: u32 = 180
}
gflags::define! {
    --fps: u32 = 60
}
gflags::define! {
    --music_starts_on: bool = false
}

// Parameters that define the game. These don't change at runtime.
#[derive(Debug)]
struct GameParams {
    viewport_width: u32,
    viewport_height: u32,
}

#[derive(Debug, Copy, Clone)]
struct InputState {
    forward: bool,
    left: bool,
    right: bool,
    pause: bool,
}
impl InputState {
    pub fn default() -> Self {
        InputState {
            forward: false,
            left: false,
            right: false,
            pause: false,
        }
    }
}

#[derive(Debug)]
struct GameState {
    input_state: InputState,
    prev_input_state: InputState,
    ship_state: spout::ship::ShipState,
    paused: bool,
}

struct Example {
    game_params: GameParams,
    fps: spout::fps_estimator::FpsEstimator,
    state: GameState,
    compute_locals: spout::particle_system::ComputeLocals,
    particle_renderer: spout::particle_system::ParticleRenderer,
    glow_renderer: spout::glow_pass::GlowRenderer,
    ship_renderer: spout::ship::ShipRenderer,
    level_buffer: spout::level_buffer::LevelBuffer,
    composition: spout::compositor::Composition,
}

impl Example {
    // Update pre-render cpu logic
    fn update_state(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        let input_state = &self.state.input_state;
        let dt = self.fps.tick() as f32;
        if input_state.pause && !self.state.prev_input_state.pause {
            // new pause signal.
            self.state.paused = !self.state.paused;
        }
        if self.state.paused {
            return;
        }

        let width = self.compute_locals.system_params.width;
        let height = self.compute_locals.system_params.height;

        let ship_state = &mut self.state.ship_state;

        // Update "ship"
        let rotation: spout::ship::RotationDirection = match (input_state.left, input_state.right) {
            (true, false) => spout::ship::RotationDirection::CCW,
            (false, true) => spout::ship::RotationDirection::CW,
            _ => spout::ship::RotationDirection::None,
        };
        ship_state.update(dt, input_state.forward, rotation);

        // TODO udpate scrolling state here.

        // Emit particles
        if input_state.forward {
            self.compute_locals.emitter.emit_over_time(
                device,
                encoder,
                dt,
                &ship_state.emit_params,
            );
        }

        // Update simulation
        let starting_height = spout::int_grid::half_outer_grid_size();
        let sim_uniforms = spout::particle_system::ComputeUniforms {
            dt,
            buffer_width: width,
            buffer_height: height,
            bottom_height: starting_height,
            middle_height: starting_height + height,
            top_height: starting_height + height * 2,
        };
        spout::particle_system::ComputeLocals::set_uniforms(
            device,
            encoder,
            &mut self.compute_locals.uniform_buf,
            &sim_uniforms,
        );
    }
}

impl framework::Example for Example {
    fn init(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) -> (Self, Option<wgpu::CommandBuffer>) {
        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        let width = WIDTH.flag;
        let height = HEIGHT.flag;
        let system_params = spout::particle_system::SystemParams {
            width,
            height,
            max_particle_life: 5.0,
        };

        let compute_locals =
            spout::particle_system::ComputeLocals::init(device, &mut init_encoder, &system_params);
        let particle_renderer = spout::particle_system::ParticleRenderer::init(
            device,
            &compute_locals,
            &mut init_encoder,
        );
        let glow_renderer = spout::glow_pass::GlowRenderer::init(
            device,
            &particle_renderer.output_texture_view,
            width,
            height,
        );
        let level_buffer = spout::level_buffer::LevelBuffer::init(
            sc_desc,
            device,
            &glow_renderer.output_texture_view,
            width,
            height,
            &mut init_encoder,
        );

        let ship_position = [
            spout::int_grid::set_values_relative(system_params.width / 4, 0),
            spout::int_grid::set_values_relative(system_params.height / 4, 0),
        ];

        let this = Example {
            game_params: GameParams {
                viewport_width: WIDTH.flag,
                viewport_height: HEIGHT.flag,
            },
            fps: spout::fps_estimator::FpsEstimator::new(FPS.flag as f64),
            state: GameState {
                input_state: InputState::default(),
                prev_input_state: InputState::default(),
                ship_state: spout::ship::ShipState::init_from_flags(ship_position),
                paused: false,
            },
            compute_locals: compute_locals,
            particle_renderer,
            glow_renderer,
            ship_renderer: spout::ship::ShipRenderer::init(
                device,
                system_params.width,
                system_params.height,
            ),
            level_buffer,
            composition: spout::compositor::Composition::init(
                device,
                system_params.width,
                system_params.height,
            ),
        };
        if MUSIC_STARTS_ON.flag {
            spout::music_player::MUSIC_PLAYER.lock().unwrap().play();
        }
        (this, Some(init_encoder.finish()))
    }
    fn handle_event(&mut self, event: winit::event::WindowEvent) {
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
                winit::event::VirtualKeyCode::W => self.state.input_state.forward,
                winit::event::VirtualKeyCode::A => self.state.input_state.left,
                winit::event::VirtualKeyCode::P => self.state.input_state.pause,
                winit::event::VirtualKeyCode::D => self.state.input_state.right),
            _ => (),
        }
    }
    fn resize(
        &mut self,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) -> Option<wgpu::CommandBuffer> {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        let viewport_aspect_ratio =
            self.game_params.viewport_width as f64 / self.game_params.viewport_height as f64;
        let new_window_aspect_ratio = sc_desc.width as f64 / sc_desc.height as f64;
        info!("Resizing: ({}, {})", sc_desc.width, sc_desc.height);
        info!("Game aspect ratio: {}", viewport_aspect_ratio);
        info!("Window aspect ratio: {}", new_window_aspect_ratio);
        self.level_buffer.resize(sc_desc, device, &mut encoder);
        Some(encoder.finish())
    }

    fn render(
        &mut self,
        frame: &wgpu::SwapChainOutput,
        device: &wgpu::Device,
    ) -> wgpu::CommandBuffer {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        self.update_state(device, &mut encoder);
        self.state.prev_input_state = self.state.input_state;

        if !self.state.paused {
            {
                // Clear the density texture.
                self.compute_locals.clear_density(&mut encoder);
            }
            {
                // Update the particles state and density texture.
                self.compute_locals.compute(&mut encoder);
            }
            {
                // Render the density texture.
                self.particle_renderer.render(&mut encoder);
            }
            {
                // Render the particle glow pass.
                self.glow_renderer.render(&mut encoder);
            }
            {
                // Render the ship.
                self.ship_renderer.render(
                    &self.composition.texture_view,
                    device,
                    &self.state.ship_state,
                    &mut encoder,
                );
            }
        }
        {
            // Render the composition texture.
            self.composition.render(
                &device,
                &frame.view,
                &mut encoder,
                self.compute_locals.system_params.width,
                self.compute_locals.system_params.height,
                self.fps.fps(),
            );
        }
        {
            self.level_buffer.render(&frame, &mut encoder);
        }

        encoder.finish()
    }
}

fn main() {
    framework::run::<Example>("Particle System");
}
