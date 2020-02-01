#[path = "../examples/framework.rs"]
mod framework;
use log::{info, trace};

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
struct ShipState {
    // Params for the emit functionality
    // TODO maybe factor out
    position: [u32; 2],
    angle: f32,
    angle_spread: f32,
    emit_speed: f32,
    emit_speed_spread: f32,
    ttl: f32,

    // Params for the ship's motion
    rotation_rate: f32,
    acceleration: f32,
}

#[derive(Debug)]
struct GameState {
    input_state: InputState,
    ship_state: ShipState,
}

struct Example {
    fps: spout::fps_estimator::FpsEstimator,
    state: GameState,
    compute_locals: spout::particle_system::ComputeLocals,
    index_count: usize,
    render_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl Example {
    // Update pre-render cpu logic
    fn update_state(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        let input_state = &self.state.input_state;

        let width = self.compute_locals.system_params.width;
        let height = self.compute_locals.system_params.height;
        let dt = self.fps.tick() as f32;
        info!("fps: {}", 1.0 / dt);

        let ship_state = &mut self.state.ship_state;

        // Update "ship"
        let angle_start = ship_state.angle;
        if input_state.left && !input_state.right {
            // Rotate ccw
            ship_state.angle -= dt * ship_state.rotation_rate;
        } else if !input_state.left && input_state.right {
            // Rotate cw
            ship_state.angle += dt * ship_state.rotation_rate;
        }

        let emit_params = spout::emitter::EmitParams::moving(
            &ship_state.position,
            &ship_state.position,
            ship_state.emit_speed,
            ship_state.emit_speed_spread,
            angle_start,
            ship_state.angle,
            ship_state.angle_spread,
            ship_state.ttl,
        );

        // Emit particles
        if input_state.forward {
            self.compute_locals
                .emitter
                .emit_over_time(device, encoder, dt, &emit_params);
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
        // Sets up the quad canvas.
        let vs = spout::include_shader!("particle_system/shader.vert.spv");
        let vs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());
        // Renders the data texture onto the canvas.
        let fs = spout::include_shader!("particle_system/shader.frag.spv");
        let fs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());

        let cm_texture = spout::color_maps::create_color_map(
            256,
            device,
            spout::color_maps::get_color_map_from_flag(),
            &mut init_encoder,
        );

        // The render pipeline renders data into this texture
        let density_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        // The render pipeline renders data into this texture
        let color_map_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        // Create pipeline layout
        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutBinding {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    },
                    wgpu::BindGroupLayoutBinding {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler,
                    },
                    wgpu::BindGroupLayoutBinding {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D1,
                        },
                    },
                    wgpu::BindGroupLayoutBinding {
                        binding: 3,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler,
                    },
                ],
            });
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &compute_locals.density_texture.create_default_view(),
                    ),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&density_sampler),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&cm_texture.create_default_view()),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&color_map_sampler),
                },
            ],
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&render_bind_group_layout],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        let ship_position = [
            spout::int_grid::set_values_relative(system_params.width / 2, 0),
            spout::int_grid::set_values_relative(system_params.height / 2, 0),
        ];

        let this = Example {
            fps: spout::fps_estimator::FpsEstimator::new(60.0),
            state: GameState {
                input_state: InputState {
                    left: false,
                    forward: false,
                    right: false,
                },
                ship_state: ShipState {
                    position: ship_position,
                    angle: 0.0,
                    angle_spread: 1.0,
                    emit_speed: 100.0 * (spout::int_grid::cell_size() as f32),
                    emit_speed_spread: 25.0 * (spout::int_grid::cell_size() as f32),
                    ttl: 5.0,
                    rotation_rate: 15.0,
                    acceleration: 1.0,
                },
            },
            compute_locals: compute_locals,
            index_count: 4,
            render_bind_group: render_bind_group,
            render_pipeline: render_pipeline,
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
            // Clear the density texture
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
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.render_bind_group, &[]);
            rpass.draw(0..self.index_count as u32, 0..1);
        }

        encoder.finish()
    }
}

fn main() {
    framework::run::<Example>("Particle System");
}
