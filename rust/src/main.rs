#[path = "../examples/framework.rs"]
mod framework;
use log::trace;

gflags::define! {
    --width: u32 = 320
}
gflags::define! {
    --height: u32 = 180
}

#[derive(Debug)]
struct InputState {
    forward: bool,
    left: bool,
    right: bool,
}

#[derive(Debug)]
struct GameState {
    input_state: InputState,
    ship_state: spout::ship::ShipState,
}

struct Example {
    fps: spout::fps_estimator::FpsEstimator,
    state: GameState,
    compute_locals: spout::particle_system::ComputeLocals,
    particle_renderer: spout::particle_system::ParticleRenderer,
    ship_renderer: spout::ship::ShipRenderer,
    composition: spout::compositor::Composition,
}

impl Example {
    // Update pre-render cpu logic
    fn update_state(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        let input_state = &self.state.input_state;

        let width = self.compute_locals.system_params.width;
        let height = self.compute_locals.system_params.height;
        let dt = self.fps.tick() as f32;

        let ship_state = &mut self.state.ship_state;

        // Update "ship"
        let rotation: spout::ship::RotationDirection = match (input_state.left, input_state.right) {
            (true, false) => spout::ship::RotationDirection::CCW,
            (false, true) => spout::ship::RotationDirection::CW,
            _ => spout::ship::RotationDirection::None,
        };
        ship_state.update(dt, input_state.forward, rotation);

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
        let sim_uniforms = spout::particle_system::ComputeUniforms {
            dt,
            buffer_width: width,
            buffer_height: height,
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
        _sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) -> (Self, Option<wgpu::CommandBuffer>) {
        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        let system_params = spout::particle_system::SystemParams {
            width: WIDTH.flag,
            height: HEIGHT.flag,
            max_particle_life: 5.0,
        };

        let compute_locals =
            spout::particle_system::ComputeLocals::init(device, &mut init_encoder, &system_params);
        let particle_renderer = spout::particle_system::ParticleRenderer::init(
            device,
            &compute_locals,
            &mut init_encoder,
        );

        let ship_position = [
            spout::int_grid::set_values_relative(system_params.width / 4, 0),
            spout::int_grid::set_values_relative(system_params.height / 4, 0),
        ];

        let this = Example {
            fps: spout::fps_estimator::FpsEstimator::new(60.0),
            state: GameState {
                input_state: InputState {
                    left: false,
                    forward: false,
                    right: false,
                },
                ship_state: spout::ship::ShipState::init_from_flags(ship_position),
            },
            compute_locals: compute_locals,
            particle_renderer,
            ship_renderer: spout::ship::ShipRenderer::init(
                device,
                system_params.width,
                system_params.height,
            ),
            composition: spout::compositor::Composition::init(
                device,
                system_params.width,
                system_params.height,
            ),
        };
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
                                winit::event::ElementState::Pressed => $result = true,
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
                winit::event::VirtualKeyCode::D => self.state.input_state.right),
            _ => (),
        }
    }
    fn resize(
        &mut self,
        _sc_desc: &wgpu::SwapChainDescriptor,
        _device: &wgpu::Device,
    ) -> Option<wgpu::CommandBuffer> {
        None
    }

    fn render(
        &mut self,
        frame: &wgpu::SwapChainOutput,
        device: &wgpu::Device,
    ) -> wgpu::CommandBuffer {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        self.update_state(device, &mut encoder);

        {
            // Clear the density texture.
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &self.compute_locals.density_texture.create_default_view(),
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });
        }
        {
            // Update the particles state and density texture.
            let mut cpass = encoder.begin_compute_pass();
            cpass.set_pipeline(&self.compute_locals.compute_pipeline);
            cpass.set_bind_group(0, &self.compute_locals.compute_bind_group, &[]);
            trace!(
                "Dispatching {} work groups",
                self.compute_locals.compute_work_groups
            );
            cpass.dispatch(self.compute_locals.compute_work_groups as u32, 1, 1);
        }
        {
            // Render the density texture.
            self.particle_renderer
                .render(&self.composition.texture_view, &mut encoder);
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

        encoder.finish()
    }
}

fn main() {
    framework::run::<Example>("Particle System");
}
