use wgpu::util::DeviceExt;
use zerocopy::AsBytes;

gflags::define! {
    --emission_rate: f32 = 100000.0
}

gflags::define! {
    --damage_rate: f32 = 0.00001
}

gflags::define! {
    --gravity: f32 = -9.81
}
gflags::define! {
    --elasticity: f32 = 0.9
}

#[derive(Clone, Copy)]
pub struct SystemParams {
    pub width: u32,
    pub height: u32,
    pub max_particle_life: f32,
}

#[repr(C)]
#[derive(Clone, Copy, zerocopy::FromBytes, zerocopy::AsBytes)]
pub struct ComputeUniforms {
    pub dt: f32,
    pub level_width: u32,
    pub level_height: u32,
    pub bottom_level_height: u32,
    pub middle_level_height: u32,
    pub top_level_height: u32,
    // It is assumed that viewport width is the same as the level width.
    pub viewport_height: u32,
    pub viewport_bottom_height: u32,
    pub damage_rate: f32,
    pub gravity: f32,
    pub elasticity: f32,
}

pub struct ComputeLocals {
    pub system_params: SystemParams,
    pub emitter: super::emitter::Emitter,
    pub compute_work_groups: usize,
    pub density_buffer: wgpu::Buffer,
    pub density_buffer_size: wgpu::BufferAddress,
    pub staging_buffer: wgpu::Buffer,
    pub uniform_buf: wgpu::Buffer,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_bind_groups: std::vec::Vec<wgpu::BindGroup>,
    pub clear_work_groups: usize,
    pub clear_bind_group: wgpu::BindGroup,
    pub clear_pipeline: wgpu::ComputePipeline,
}

impl ComputeLocals {
    fn make_density_buffer(
        device: &wgpu::Device,
        width: usize,
        height: usize,
    ) -> (wgpu::Buffer, wgpu::BufferAddress) {
        let size = (std::mem::size_of::<u32>() * width * height) as wgpu::BufferAddress;
        (
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Density buffer"),
                size,
                usage: wgpu::BufferUsage::STORAGE
                    | wgpu::BufferUsage::COPY_DST
                    | wgpu::BufferUsage::COPY_SRC,
                mapped_at_creation: false,
            }),
            size,
        )
    }

    pub fn init(
        device: &wgpu::Device,
        params: &SystemParams,
        game_params: &super::game_params::GameParams,
        level_manager: &super::level_manager::LevelManager,
        _init_encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        // This sets up the compute stage, which is responsible for updating the
        // particle system and most of the game logic. The output is updated game state
        // and a particle density texture.
        let emitter =
            super::emitter::Emitter::new(device, EMISSION_RATE.flag, params.max_particle_life);
        let num_particles = emitter.num_particles();

        // This needs to match the layout size in the the particle compute shader. Maybe
        // an equivalent to "specialization constants" will come out and allow us to
        // specify the 512 programmatically.
        let particle_group_size = 512;
        let compute_work_groups =
            (num_particles as f64 / particle_group_size as f64).ceil() as usize;

        let (density_buffer, density_buffer_size) = ComputeLocals::make_density_buffer(
            device,
            params.width as usize,
            params.height as usize,
        );
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: density_buffer_size,
            usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
            label: None,
            mapped_at_creation: false,
        });

        let compute_uniforms = ComputeUniforms {
            dt: 0.0,
            level_width: params.width,
            level_height: params.height,
            bottom_level_height: 0,
            middle_level_height: params.height,
            top_level_height: params.height * 2,
            viewport_height: game_params.viewport_height,
            viewport_bottom_height: 0,
            damage_rate: DAMAGE_RATE.flag,
            gravity: GRAVITY.flag,
            elasticity: ELASTICITY.flag,
        };
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle uniforms"),
            contents: &compute_uniforms.as_bytes(),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // Particle storage buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Bottom terrain buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Top terrain buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Particle density buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Uniform inputs
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute pipeline layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let mut compute_bind_groups = vec![];
        for config in level_manager.buffer_configurations() {
            compute_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    // Particle storage buffer
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(emitter.particle_buffer.slice(..)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(
                            level_manager.terrain_buffers()[config[0]].slice(..),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(
                            level_manager.terrain_buffers()[config[1]].slice(..),
                        ),
                    },
                    // Particle density buffer
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Buffer(density_buffer.slice(..)),
                    },
                    // Uniforms
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
                    },
                ],
                label: None,
            }));
        }

        let cs = super::shader_utils::Shaders::get("particle_system/particles.comp.spv").unwrap();
        let cs_module = device.create_shader_module(wgpu::util::make_spirv(&cs));
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute particles"),
            layout: Some(&compute_pipeline_layout),
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &cs_module,
                entry_point: "main",
            },
        });

        let (clear_work_groups, clear_bind_group, clear_pipeline) = {
            let clear_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        // Particle density buffer
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStage::COMPUTE,
                            ty: wgpu::BindingType::StorageBuffer {
                                dynamic: false,
                                readonly: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                    label: None,
                });
            let clear_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &clear_bind_group_layout,
                entries: &[
                    // Particle density buffer
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(density_buffer.slice(..)),
                    },
                ],
                label: None,
            });

            let clear_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Clear pipeline layout"),
                    bind_group_layouts: &[&clear_bind_group_layout],
                    push_constant_ranges: &[],
                });
            let cs =
                super::shader_utils::Shaders::get("particle_system/clear_ssbo.comp.spv").unwrap();
            let cs_module = device.create_shader_module(wgpu::util::make_spirv(&cs));
            let clear_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Clear pipeline"),
                layout: Some(&clear_pipeline_layout),
                compute_stage: wgpu::ProgrammableStageDescriptor {
                    module: &cs_module,
                    entry_point: "main",
                },
            });

            let clear_group_size = 512;
            let clear_work_groups =
                ((params.width * params.height) as f64 / clear_group_size as f64).ceil() as usize;
            (clear_work_groups, clear_bind_group, clear_pipeline)
        };

        // Copy initial data to GPU
        ComputeLocals {
            system_params: *params,
            emitter,
            compute_work_groups,
            density_buffer,
            density_buffer_size,
            staging_buffer,
            uniform_buf,
            compute_pipeline,
            compute_bind_groups,
            clear_work_groups,
            clear_bind_group,
            clear_pipeline,
        }
    }

    pub fn update_state(
        &mut self,
        device: &wgpu::Device,
        game_params: &super::game_params::GameParams,
        level_manager: &super::level_manager::LevelManager,
        dt: f32,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let compute_uniforms = ComputeUniforms {
            dt,
            level_width: game_params.level_width,
            level_height: game_params.level_height,
            bottom_level_height: level_manager.buffer_height(0) as u32,
            middle_level_height: level_manager.buffer_height(1) as u32,
            top_level_height: level_manager.buffer_height(1) as u32 + game_params.level_height,
            viewport_height: game_params.viewport_height,
            viewport_bottom_height: level_manager.height_of_viewport() as u32,
            damage_rate: DAMAGE_RATE.flag,
            gravity: GRAVITY.flag,
            elasticity: ELASTICITY.flag,
        };
        self.set_uniforms(device, encoder, &compute_uniforms);
    }

    fn set_uniforms(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        values: &ComputeUniforms,
    ) {
        let bytes: &[u8] = values.as_bytes();
        let uniform_buf_size = std::mem::size_of::<ComputeUniforms>();
        let temp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Staging buffer"),
            contents: bytes,
            usage: wgpu::BufferUsage::COPY_SRC,
        });
        encoder.copy_buffer_to_buffer(
            &temp_buf,
            0,
            &self.uniform_buf,
            0,
            uniform_buf_size as wgpu::BufferAddress,
        );
    }

    pub fn compute(
        &self,
        level_manager: &super::level_manager::LevelManager,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        {
            // Add the heavy computation
            let mut cpass = encoder.begin_compute_pass();
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(
                0,
                &self.compute_bind_groups[level_manager.buffer_config_index()],
                &[],
            );
            cpass.dispatch(self.compute_work_groups as u32, 1, 1);
        }
        {
            // Copy the results to the staging buffer so we can tell when this computation is over.
            encoder.copy_buffer_to_buffer(
                &self.density_buffer,
                0,
                &self.staging_buffer,
                0,
                self.density_buffer_size,
            );
        }
    }

    pub fn clear_density(&self, encoder: &mut wgpu::CommandEncoder) {
        // Clear the density buffer.
        let mut cpass = encoder.begin_compute_pass();
        cpass.set_pipeline(&self.clear_pipeline);
        cpass.set_bind_group(0, &self.clear_bind_group, &[]);
        cpass.dispatch(self.clear_work_groups as u32, 1, 1);
    }
}

// Keep track of the rendering members and logic to turn the integer particle
// density texture into a colormapped texture ready to be visualized.
pub struct ParticleRenderer {
    pub render_bind_group: wgpu::BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
}

#[repr(C)]
#[derive(Clone, Copy, zerocopy::FromBytes, zerocopy::AsBytes)]
struct FragmentUniforms {
    pub width: u32,
    pub height: u32,
}

impl ParticleRenderer {
    pub fn init(
        device: &wgpu::Device,
        compute_locals: &ComputeLocals,
        init_encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        // Sets up the quad canvas.
        let vs = super::shader_utils::Shaders::get("particle_system/quad.vert.spv").unwrap();
        let vs_module = device.create_shader_module(wgpu::util::make_spirv(&vs));
        // Renders the data texture onto the canvas.
        let fs = super::shader_utils::Shaders::get("particle_system/particles.frag.spv").unwrap();
        let fs_module = device.create_shader_module(wgpu::util::make_spirv(&fs));

        let cm_texture = super::color_maps::create_color_map(
            256,
            device,
            super::color_maps::get_color_map_from_flag(),
            init_encoder,
        );

        let fragment_uniforms = FragmentUniforms {
            width: compute_locals.system_params.width,
            height: compute_locals.system_params.height,
        };
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle render uniforms"),
            contents: &fragment_uniforms.as_bytes(),
            usage: wgpu::BufferUsage::UNIFORM,
        });

        // The render pipeline renders data into this texture
        let color_map_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Particle render sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
        });

        // Create pipeline layout
        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // Particle density buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: true,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Color map.
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            component_type: wgpu::TextureComponentType::Float,
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D1,
                        },
                        count: None,
                    },
                    // Color map sampler.
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                    // Uniform inputs
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: None,
            });
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        compute_locals.density_buffer.slice(..),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &cm_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&color_map_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
                },
            ],
            label: None,
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle render pipeline layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle render pipeline"),
            layout: Some(&render_pipeline_layout),
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
                clamp_depth: false,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            depth_stencil_state: None,
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        ParticleRenderer {
            render_bind_group,
            render_pipeline,
        }
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        output_texture_view: &wgpu::TextureView,
    ) {
        // Render the density texture.
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: output_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.render_bind_group, &[]);
        rpass.draw(0..4 as u32, 0..1);
    }
}
