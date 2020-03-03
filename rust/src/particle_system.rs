use zerocopy::AsBytes;

gflags::define! {
    --emission_rate: f32 = 10000.0
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
}

pub struct ComputeLocals {
    pub system_params: SystemParams,
    pub emitter: super::emitter::Emitter,
    pub compute_work_groups: usize,
    pub compute_bind_group_a: wgpu::BindGroup,
    pub compute_bind_group_b: wgpu::BindGroup,
    pub terrain_texture_a_view: wgpu::TextureView,
    pub terrain_texture_b_view: wgpu::TextureView,
    pub density_texture_view: wgpu::TextureView,
    pub uniform_buf: wgpu::Buffer,
    pub compute_pipeline: wgpu::ComputePipeline,
}

impl ComputeLocals {
    fn fill_level_buffer(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::Texture,
        texture_extent: &wgpu::Extent3d,
        level_num: u32,
    ) {
        let width = texture_extent.width;
        let height = texture_extent.height;
        let im = image::ImageBuffer::<image::Luma<u8>, Vec<u8>>::from_fn(width, height, |x, y| {
            let (index, extent) = match level_num % 2 {
                0 => (x, width),
                1 => (y, height),
                _ => panic!(),
            };
            let parameter = index as f64 / extent as f64;
            image::Luma([(parameter * 255.0).floor() as u8])
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
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                row_pitch: 1 * width,
                image_height: height,
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
            *texture_extent,
        );
    }

    fn make_density_texture(
        device: &wgpu::Device,
        texture_extent: &wgpu::Extent3d,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            size: *texture_extent,
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
        })
    }

    fn make_terrain_texture(
        device: &wgpu::Device,
        texture_extent: &wgpu::Extent3d,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            size: *texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::STORAGE
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT
                | wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::SAMPLED,
        })
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
        let texture_extent = wgpu::Extent3d {
            width: params.width,
            height: params.height,
            depth: 1,
        };
        let terrain_texture_a = ComputeLocals::make_terrain_texture(device, &texture_extent);
        let terrain_texture_a_view = terrain_texture_a.create_default_view();
        ComputeLocals::fill_level_buffer(
            device,
            init_encoder,
            &terrain_texture_a,
            &texture_extent,
            0,
        );

        let terrain_texture_b = ComputeLocals::make_terrain_texture(device, &texture_extent);
        let terrain_texture_b_view = terrain_texture_b.create_default_view();
        ComputeLocals::fill_level_buffer(
            device,
            init_encoder,
            &terrain_texture_b,
            &texture_extent,
            1,
        );

        let density_texture = ComputeLocals::make_density_texture(device, &texture_extent);
        let density_texture_view = density_texture.create_default_view();

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
                        ty: wgpu::BindingType::StorageTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    },
                    // Top terrain buffer
                    wgpu::BindGroupLayoutBinding {
                        binding: 2,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    },
                    // Particle density buffer
                    wgpu::BindGroupLayoutBinding {
                        binding: 3,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            dimension: wgpu::TextureViewDimension::D2,
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
                // Bottom level buffer
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&terrain_texture_a_view),
                },
                // Top level buffer
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&terrain_texture_b_view),
                },
                // Particle density buffer
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&density_texture_view),
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
                // Particle density buffer
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&terrain_texture_b_view),
                },
                // Particle density buffer
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&terrain_texture_a_view),
                },
                // Particle density buffer
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&density_texture_view),
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

        // Copy initial data to GPU
        ComputeLocals {
            system_params: *params,
            emitter,
            compute_work_groups,
            compute_bind_group_a,
            compute_bind_group_b,
            terrain_texture_a_view,
            terrain_texture_b_view,
            density_texture_view,
            uniform_buf,
            compute_pipeline,
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
        // Clear the density texture.
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &self.density_texture_view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::BLACK,
            }],
            depth_stencil_attachment: None,
        });
    }
}

// Keep track of the rendering members and logic to turn the integer particle
// density texture into a colormapped texture ready to be visualized.
pub struct ParticleRenderer {
    pub render_bind_group: wgpu::BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
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
                    // Particle density texture.
                    wgpu::BindGroupLayoutBinding {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    },
                    // Particle density texture sampler.
                    wgpu::BindGroupLayoutBinding {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler,
                    },
                    // Color map.
                    wgpu::BindGroupLayoutBinding {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D1,
                        },
                    },
                    // Color map sampler.
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
                        &compute_locals.density_texture_view,
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
