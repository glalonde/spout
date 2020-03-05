#[path = "../examples/framework.rs"]
mod framework;
use log::info;

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
    game_params: spout::game_params::GameParams,
    fps: spout::fps_estimator::FpsEstimator,
    state: GameState,
    compute_locals: spout::particle_system::ComputeLocals,
    pre_glow_texture: wgpu::TextureView,
    post_glow_texture: wgpu::TextureView,
    terrain_renderer: spout::terrain_renderer::TerrainRenderer,
    particle_renderer: spout::particle_system::ParticleRenderer,
    glow_renderer: spout::glow_pass::GlowRenderer,
    ship_renderer: spout::ship::ShipRenderer,
    viewport: spout::viewport::Viewport,
    debug_overlay: spout::debug_overlay::DebugOverlay,
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

        let ship_state = &mut self.state.ship_state;

        // Update "ship"
        let rotation: spout::ship::RotationDirection = match (input_state.left, input_state.right) {
            (true, false) => spout::ship::RotationDirection::CCW,
            (false, true) => spout::ship::RotationDirection::CW,
            _ => spout::ship::RotationDirection::None,
        };
        ship_state.update(dt, input_state.forward, rotation);

        // TODO update scrolling state here.

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
        self.compute_locals
            .update_uniforms(device, encoder, dt, &self.game_params);
    }
}

impl framework::Example for Example {
    fn init(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) -> (Self, Option<wgpu::CommandBuffer>) {
        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        let game_params = spout::game_params::GameParams {
            viewport_width: WIDTH.flag,
            viewport_height: HEIGHT.flag,
            level_width: WIDTH.flag,
            level_height: HEIGHT.flag * 3,
        };
        let width = WIDTH.flag;
        let height = HEIGHT.flag;
        let system_params = spout::particle_system::SystemParams {
            width,
            height,
            max_particle_life: 5.0,
        };

        let compute_locals = spout::particle_system::ComputeLocals::init(
            device,
            &mut init_encoder,
            &system_params,
            &game_params,
        );
        let pre_glow_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: game_params.viewport_width,
                height: game_params.viewport_height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let pre_glow_texture_view = pre_glow_texture.create_default_view();
        let post_glow_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: game_params.viewport_width,
                height: game_params.viewport_height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let post_glow_texture_view = post_glow_texture.create_default_view();

        let terrain_renderer =
            spout::terrain_renderer::TerrainRenderer::init(device, &compute_locals);

        let particle_renderer = spout::particle_system::ParticleRenderer::init(
            device,
            &compute_locals,
            &mut init_encoder,
        );
        let glow_renderer = spout::glow_pass::GlowRenderer::init(device, &pre_glow_texture_view);
        let viewport = spout::viewport::Viewport::init(
            sc_desc,
            device,
            &post_glow_texture_view,
            width,
            height,
            &mut init_encoder,
        );

        let ship_position = [
            spout::int_grid::set_values_relative(system_params.width / 4, 0),
            spout::int_grid::set_values_relative(system_params.height / 4, 0),
        ];

        let this = Example {
            game_params,
            fps: spout::fps_estimator::FpsEstimator::new(FPS.flag as f64),
            state: GameState {
                input_state: InputState::default(),
                prev_input_state: InputState::default(),
                ship_state: spout::ship::ShipState::init_from_flags(ship_position),
                paused: false,
            },
            compute_locals: compute_locals,
            pre_glow_texture: pre_glow_texture_view,
            post_glow_texture: post_glow_texture_view,
            terrain_renderer,
            particle_renderer,
            glow_renderer,
            ship_renderer: spout::ship::ShipRenderer::init(
                device,
                system_params.width,
                system_params.height,
            ),
            viewport,
            debug_overlay: spout::debug_overlay::DebugOverlay::init(device, sc_desc),
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
        self.viewport.resize(sc_desc, device, &mut encoder);
        self.debug_overlay.resize(sc_desc);
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
                // Clear the pre-glow pass
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &self.pre_glow_texture,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color::BLACK,
                    }],
                    depth_stencil_attachment: None,
                });
            }
            {
                // Render the terrain
                self.terrain_renderer
                    .render(&mut encoder, &self.pre_glow_texture);
            }
            {
                // Render the density texture.
                self.particle_renderer
                    .render(&mut encoder, &self.pre_glow_texture);
            }
            {
                // Render the particle glow pass.
                self.glow_renderer
                    .render(&mut encoder, &self.post_glow_texture);
            }
            {
                // Render the ship.
                self.ship_renderer.render(
                    &self.post_glow_texture,
                    device,
                    &self.state.ship_state,
                    &mut encoder,
                );
            }
        }
        {
            self.viewport.render(&frame, &mut encoder);
        }
        {
            self.debug_overlay
                .render(&device, &frame.view, &mut encoder, self.fps.fps());
        }

        encoder.finish()
    }
}

fn main() {
    framework::run::<Example>("Particle System");
}
