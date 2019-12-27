// TODO: uniforms
#[path = "../framework.rs"]
mod framework;
use log::info;
use spout::shader_utils::load_png_to_texture;

gflags::define! {
    --num_particles: usize = 500
}

#[repr(C)]
#[derive(Clone, Copy, zerocopy::AsBytes, zerocopy::FromBytes)]
struct ComputeUniforms {
    num_particles: u32,
}

struct Example {
    compute_work_groups: usize,
    compute_bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
    index_count: usize,
    render_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buf: wgpu::Buffer,
}

impl framework::Example for Example {
    fn init(
        _sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) -> (Self, Option<wgpu::CommandBuffer>) {
        // This sets up the compute stage.
        // Computes data for the main texture.
        // This needs to match the layout size in the the particle compute shader. Maybe an equivalent to "specialization constants" will come out and allow us to specify the 512 programmatically.
        let particle_group_size = 512;
        let num_work_groups =
            (NUM_PARTICLES.flag as f64 / particle_group_size as f64).ceil() as usize;
        let texture_extent = wgpu::Extent3d {
            width: 30,
            height: 40,
            depth: 1,
        };
        let texture1 = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::STORAGE
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT
                | wgpu::TextureUsage::SAMPLED,
        });
        let texture_view1 = texture1.create_default_view();

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
                    wgpu::BindGroupLayoutBinding {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    },
                    wgpu::BindGroupLayoutBinding {
                        binding: 1,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                ],
            });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view1),
                },
                wgpu::Binding {
                    binding: 1,
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
        let cs = spout::include_shader!("image_viewer/shader.comp.spv");
        let cs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&cs[..])).unwrap());
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: &compute_pipeline_layout,
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &cs_module,
                entry_point: "main",
            },
        });
        // Sets up the quad canvas.
        let vs = spout::include_shader!("image_viewer/shader.vert.spv");
        let vs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());
        // Renders the data texture onto the canvas.
        let fs = spout::include_shader!("image_viewer/shader.frag.spv");
        let fs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());

        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        let texture_view2 = load_png_to_texture(device, "assets/coords.png", &mut init_encoder);
        // The render pipeline renders data into this texture

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
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
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    },
                    wgpu::BindGroupLayoutBinding {
                        binding: 2,
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
                    resource: wgpu::BindingResource::TextureView(&texture_view1),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view2),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
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
            compute_work_groups: num_work_groups,
            compute_pipeline: compute_pipeline,
            compute_bind_group: compute_bind_group,
            index_count: 4,
            render_bind_group: render_bind_group,
            render_pipeline: render_pipeline,
            uniform_buf: uniform_buf,
        };
        (this, Some(init_encoder.finish()))
    }
    fn update(&mut self, _event: winit::event::WindowEvent) {
        //empty
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
        {
            let mut cpass = encoder.begin_compute_pass();
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            info!("Dispatching {} work groups", self.compute_work_groups);
            cpass.dispatch(self.compute_work_groups as u32, 1, 1);
        }
        info!("Moving on to render.");
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::GREEN,
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
    framework::run::<Example>("Image Viewer");
}
