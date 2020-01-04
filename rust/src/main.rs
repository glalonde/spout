#[path = "../examples/framework.rs"]
mod framework;
use log::{trace, info};

gflags::define! {
    --num_particles: usize = 500
}
gflags::define! {
    --width: u32 = 500
}
gflags::define! {
    --height: u32 = 500
}

// Create a particle density color map rgba
// Rust image defaults to row major.
fn create_color_map(
    size: u32,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
) -> wgpu::Texture {
    // TODO: get color map name from flag
    let cm = scarlet::colormap::ListedColorMap::viridis();
    let im = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_fn(size, 1, |x, _y| {
        let parameter = x as f64 / (size - 1) as f64;
        let color_point: scarlet::color::RGBColor =
            scarlet::colormap::ColorMap::transform_single(&cm, parameter);
        image::Rgba([
            color_point.int_r(),
            color_point.int_g(),
            color_point.int_b(),
            255,
        ])
    });
    let data = im.into_raw();
    let texture_extent = wgpu::Extent3d {
        width: size,
        height: 1,
        depth: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D1,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED
            | wgpu::TextureUsage::COPY_DST
            | wgpu::TextureUsage::COPY_SRC,
    });
    let temp_buf = device
        .create_buffer_mapped(
            data.len(),
            wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
        )
        .fill_from_slice(&data);
    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &temp_buf,
            offset: 0,
            row_pitch: 4 * size,
            image_height: 1,
        },
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
        texture_extent,
    );

    texture
}

#[repr(C)]
#[derive(Clone, Copy, zerocopy::AsBytes, zerocopy::FromBytes)]
struct ComputeUniforms {
    num_particles: u32,
}

struct ComputeLocals {
    emitter: spout::emitter::Emitter,
    compute_work_groups: usize,
    compute_bind_group: wgpu::BindGroup,
    density_texture: wgpu::Texture,
    uniform_buf: wgpu::Buffer,
    compute_pipeline: wgpu::ComputePipeline,
}

#[derive(Debug)]
struct InputState {
    forward: bool,
    left: bool,
    right: bool,
}

struct Example {
    input: InputState,
    compute_locals: ComputeLocals,
    index_count: usize,
    render_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}
impl ComputeLocals {
    fn init(device: &wgpu::Device, init_encoder: &mut wgpu::CommandEncoder) -> Self {
        // This sets up the compute stage, which is responsible for updating the
        // particle system and most of the game logic. The output is updated game state
        // and a particle density texture.
        let width = WIDTH.flag;
        let height = HEIGHT.flag;
        let num_particles = NUM_PARTICLES.flag as u32;
        let emitter = spout::emitter::Emitter::new(device, num_particles, 100.0);


        // This needs to match the layout size in the the particle compute shader. Maybe
        // an equivalent to "specialization constants" will come out and allow us to
        // specify the 512 programmatically.
        let particle_group_size = 512;
        let compute_work_groups =
            (NUM_PARTICLES.flag as f64 / particle_group_size as f64).ceil() as usize;
        let texture_extent = wgpu::Extent3d {
            width: width,
            height: height,
            depth: 1,
        };
        let density_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::STORAGE
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT
                | wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::SAMPLED,
        });
        let density_texture_view = density_texture.create_default_view();

        let compute_uniform_size = std::mem::size_of::<ComputeUniforms>() as wgpu::BufferAddress;
        let compute_uniforms = ComputeUniforms {
            num_particles: NUM_PARTICLES.flag as u32,
        };
        let uniform_buf = device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM)
            .fill_from_slice(&[compute_uniforms]);

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    // Particle storage buffer
                    wgpu::BindGroupLayoutBinding {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: false,
                        },
                    },
                    // Particle density buffer
                    wgpu::BindGroupLayoutBinding {
                        binding: 1,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    },
                    // Uniform inputs
                    wgpu::BindGroupLayoutBinding {
                        binding: 2,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                ],
            });

        let particle_buffer_size =
            (num_particles * std::mem::size_of::<spout::emitter::Particle>() as u32) as wgpu::BufferAddress;

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute_bind_group_layout,
            bindings: &[
                // Particle storage buffer
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &emitter.particle_buffer,
                        range: 0..particle_buffer_size,
                    },
                },
                // Particle density buffer
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&density_texture_view),
                },
                // Uniforms
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0..compute_uniform_size,
                    },
                },
            ],
        });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&compute_bind_group_layout],
            });
        let cs = spout::include_shader!("particle_system/shader.comp.spv");
        let cs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&cs[..])).unwrap());
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: &compute_pipeline_layout,
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &cs_module,
                entry_point: "main",
            },
        });

        // Copy initial data to GPU
        ComputeLocals {
            emitter,
            compute_work_groups,
            compute_bind_group,
            density_texture,
            uniform_buf,
            compute_pipeline,
        }
    }
}
impl Example {
    // Update pre-render cpu logic
    fn update_state(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        // Emit particles
        if self.input.forward {
            self.compute_locals.emitter.emit_over_time(device, encoder, 1.0 / 60.0 /* dt */);
        }
        // Update simulation
    }
}

impl framework::Example for Example {
    fn init(
        _sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) -> (Self, Option<wgpu::CommandBuffer>) {
        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        let compute_locals = ComputeLocals::init(device, &mut init_encoder);
        // Sets up the quad canvas.
        let vs = spout::include_shader!("particle_system/shader.vert.spv");
        let vs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());
        // Renders the data texture onto the canvas.
        let fs = spout::include_shader!("particle_system/shader.frag.spv");
        let fs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());

        let cm_texture = create_color_map(256, device, &mut init_encoder);

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
        let this = Example {
            input: InputState {
                forward: false,
                left: false,
                right: false,
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
                winit::event::VirtualKeyCode::W => self.input.forward, 
                winit::event::VirtualKeyCode::A => self.input.left, 
                winit::event::VirtualKeyCode::D => self.input.right),
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
