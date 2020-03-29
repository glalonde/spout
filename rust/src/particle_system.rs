use zerocopy::AsBytes;

gflags::define! {
    --emission_rate: f32 = 10000.0
}

gflags::define! {
    --damage_rate: f32 = 0.00001
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
    // It is assumed with viewport width is the same as the level width.
    pub viewport_height: u32,
    pub viewport_bottom_height: u32,
    pub damage_rate: f32,
}

pub struct ComputeLocals {
    pub system_params: SystemParams,
    pub emitter: super::emitter::Emitter,
    pub compute_work_groups: usize,
    pub compute_bind_group_a: wgpu::BindGroup,
    pub compute_bind_group_b: wgpu::BindGroup,
    pub terrain_buffer_a: wgpu::Buffer,
    pub terrain_buffer_a_size: wgpu::BufferAddress,
    pub terrain_buffer_b: wgpu::Buffer,
    pub terrain_buffer_b_size: wgpu::BufferAddress,
    pub density_buffer: wgpu::Buffer,
    pub density_buffer_size: wgpu::BufferAddress,
    pub uniform_buf: wgpu::Buffer,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub clear_work_groups: usize,
    pub clear_bind_group: wgpu::BindGroup,
    pub clear_pipeline: wgpu::ComputePipeline,
}

impl ComputeLocals {
    fn fill_level_buffer(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        buffer: &wgpu::Buffer,
        width: u32,
        height: u32,
        level_num: u32,
    ) {
        let im =
            image::ImageBuffer::<image::Luma<i32>, Vec<i32>>::from_fn(width, height, |x, y| {
                let (index, _) = match level_num % 2 {
                    0 => (x, width),
                    1 => (y, height),
                    _ => panic!(),
                };
                match index % 5 {
                    0 => image::Luma::<i32>([1000]),
                    _ => image::Luma::<i32>([0]),
                }
            });
        let data = im.into_raw();
        let temp_buf = device
            .create_buffer_mapped(
                data.len(),
                wgpu::BufferUsage::COPY_SRC
                    | wgpu::BufferUsage::COPY_DST
                    | wgpu::BufferUsage::MAP_READ,
            )
            .fill_from_slice(&data);
        encoder.copy_buffer_to_buffer(
            &temp_buf,
            0,
            &buffer,
            0,
            (width * height * std::mem::size_of::<i32>() as u32) as u64,
        );
    }

    fn make_density_buffer(
        device: &wgpu::Device,
        width: usize,
        height: usize,
    ) -> (wgpu::Buffer, wgpu::BufferAddress) {
        let size = (std::mem::size_of::<u32>() * width * height) as wgpu::BufferAddress;
        (
            device.create_buffer(&wgpu::BufferDescriptor {
                size,
                usage: wgpu::BufferUsage::STORAGE
                    | wgpu::BufferUsage::COPY_DST
                    | wgpu::BufferUsage::COPY_SRC
                    | wgpu::BufferUsage::STORAGE_READ,
            }),
            size,
        )
    }

    fn make_terrain_buffer(
        device: &wgpu::Device,
        width: usize,
        height: usize,
    ) -> (wgpu::Buffer, wgpu::BufferAddress) {
        let size = (std::mem::size_of::<i32>() * width * height) as wgpu::BufferAddress;
        (
            device.create_buffer(&wgpu::BufferDescriptor {
                size,
                usage: wgpu::BufferUsage::STORAGE
                    | wgpu::BufferUsage::COPY_DST
                    | wgpu::BufferUsage::COPY_SRC
                    | wgpu::BufferUsage::STORAGE_READ,
            }),
            size,
        )
    }

    pub fn init(
        device: &wgpu::Device,
        init_encoder: &mut wgpu::CommandEncoder,
        params: &SystemParams,
        game_params: &super::game_params::GameParams,
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

        let (terrain_buffer_a, terrain_buffer_a_size) = ComputeLocals::make_terrain_buffer(
            device,
            params.width as usize,
            params.height as usize,
        );
        ComputeLocals::fill_level_buffer(
            device,
            init_encoder,
            &terrain_buffer_a,
            params.width,
            params.height,
            0,
        );

        let (terrain_buffer_b, terrain_buffer_b_size) = ComputeLocals::make_terrain_buffer(
            device,
            params.width as usize,
            params.height as usize,
        );
        ComputeLocals::fill_level_buffer(
            device,
            init_encoder,
            &terrain_buffer_b,
            params.width,
            params.height,
            1,
        );

        let (density_buffer, density_buffer_size) = ComputeLocals::make_density_buffer(
            device,
            params.width as usize,
            params.height as usize,
        );

        let compute_uniform_size = std::mem::size_of::<ComputeUniforms>() as wgpu::BufferAddress;
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
        };
        let uniform_buf = device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
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
                    // Bottom terrain buffer
                    wgpu::BindGroupLayoutBinding {
                        binding: 1,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: false,
                        },
                    },
                    // Top terrain buffer
                    wgpu::BindGroupLayoutBinding {
                        binding: 2,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: false,
                        },
                    },
                    // Particle density buffer
                    wgpu::BindGroupLayoutBinding {
                        binding: 3,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: false,
                        },
                    },
                    // Uniform inputs
                    wgpu::BindGroupLayoutBinding {
                        binding: 4,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                ],
            });

        let particle_buffer_size = (num_particles
            * std::mem::size_of::<super::emitter::Particle>() as u32)
            as wgpu::BufferAddress;

        let compute_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                // Bottom level buffer(A)
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &terrain_buffer_a,
                        range: 0..terrain_buffer_a_size,
                    },
                },
                // Top level buffer(B)
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &terrain_buffer_b,
                        range: 0..terrain_buffer_b_size,
                    },
                },
                // Particle density buffer
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &density_buffer,
                        range: 0..density_buffer_size,
                    },
                },
                // Uniforms
                wgpu::Binding {
                    binding: 4,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0..compute_uniform_size,
                    },
                },
            ],
        });
        let compute_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                // Bottom level buffer(B)
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &terrain_buffer_b,
                        range: 0..terrain_buffer_b_size,
                    },
                },
                // Top level buffer(A)
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &terrain_buffer_a,
                        range: 0..terrain_buffer_a_size,
                    },
                },
                // Particle density buffer
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &density_buffer,
                        range: 0..density_buffer_size,
                    },
                },
                // Uniforms
                wgpu::Binding {
                    binding: 4,
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
        let cs = super::include_shader!("particle_system/particles.comp.spv");
        let cs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&cs[..])).unwrap());
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: &compute_pipeline_layout,
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &cs_module,
                entry_point: "main",
            },
        });

        let (clear_work_groups, clear_bind_group, clear_pipeline) = {
            let clear_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    bindings: &[
                        // Particle density buffer
                        wgpu::BindGroupLayoutBinding {
                            binding: 0,
                            visibility: wgpu::ShaderStage::COMPUTE,
                            ty: wgpu::BindingType::StorageBuffer {
                                dynamic: false,
                                readonly: false,
                            },
                        },
                    ],
                });
            let clear_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &clear_bind_group_layout,
                bindings: &[
                    // Particle density buffer
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &density_buffer,
                            range: 0..density_buffer_size,
                        },
                    },
                ],
            });

            let clear_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[&clear_bind_group_layout],
                });
            let cs = super::include_shader!("particle_system/clear_ssbo.comp.spv");
            let cs_module = device
                .create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&cs[..])).unwrap());
            let clear_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                layout: &clear_pipeline_layout,
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
            compute_bind_group_a,
            compute_bind_group_b,
            terrain_buffer_a,
            terrain_buffer_a_size,
            terrain_buffer_b,
            terrain_buffer_b_size,
            density_buffer,
            density_buffer_size,
            uniform_buf,
            compute_pipeline,
            clear_work_groups,
            clear_bind_group,
            clear_pipeline,
        }
    }

    pub fn update_uniforms(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        dt: f32,
        game_params: &super::game_params::GameParams,
    ) {
        // Update simulation
        let compute_uniforms = ComputeUniforms {
            dt,
            level_width: game_params.level_width,
            level_height: game_params.level_height,
            bottom_level_height: 0,
            middle_level_height: game_params.level_height,
            top_level_height: game_params.level_height * 2,
            viewport_height: game_params.viewport_height,
            viewport_bottom_height: 0,
            damage_rate: DAMAGE_RATE.flag,
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
        let temp_buf = device
            .create_buffer_mapped(uniform_buf_size, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(bytes);
        encoder.copy_buffer_to_buffer(
            &temp_buf,
            0,
            &self.uniform_buf,
            0,
            uniform_buf_size as wgpu::BufferAddress,
        );
    }

    pub fn compute(&self, encoder: &mut wgpu::CommandEncoder) {
        // Update the particles state and density texture.
        // TODO also allow bind group b
        let mut cpass = encoder.begin_compute_pass();
        cpass.set_pipeline(&self.compute_pipeline);
        cpass.set_bind_group(0, &self.compute_bind_group_a, &[]);
        cpass.dispatch(self.compute_work_groups as u32, 1, 1);
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
        let vs = super::include_shader!("particle_system/quad.vert.spv");
        let vs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());
        // Renders the data texture onto the canvas.
        let fs = super::include_shader!("particle_system/particles.frag.spv");
        let fs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());

        let cm_texture = super::color_maps::create_color_map(
            256,
            device,
            super::color_maps::get_color_map_from_flag(),
            init_encoder,
        );

        let fragment_uniform_size = std::mem::size_of::<FragmentUniforms>() as wgpu::BufferAddress;
        let fragment_uniforms = FragmentUniforms {
            width: compute_locals.system_params.width,
            height: compute_locals.system_params.height,
        };
        let uniform_buf = device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM)
            .fill_from_slice(&[fragment_uniforms]);

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
                    // Particle density buffer
                    wgpu::BindGroupLayoutBinding {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: true,
                        },
                    },
                    // Color map.
                    wgpu::BindGroupLayoutBinding {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D1,
                        },
                    },
                    // Color map sampler.
                    wgpu::BindGroupLayoutBinding {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler,
                    },
                    // Uniform inputs
                    wgpu::BindGroupLayoutBinding {
                        binding: 3,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                ],
            });
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &compute_locals.density_buffer,
                        range: 0..compute_locals.density_buffer_size,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cm_texture.create_default_view()),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&color_map_sampler),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0..fragment_uniform_size,
                    },
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
                load_op: wgpu::LoadOp::Load,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::BLACK,
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.render_bind_group, &[]);
        rpass.draw(0..4 as u32, 0..1);
    }
}
