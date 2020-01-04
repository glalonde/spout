use zerocopy::AsBytes;

pub struct SystemParams {
    pub width: u32,
    pub height: u32,
    pub num_particles: u32,
}

#[repr(C)]
#[derive(Clone, Copy, zerocopy::FromBytes, zerocopy::AsBytes)]
pub struct ComputeUniforms {
    pub num_particles: u32,
    pub dt: f32,
}

pub struct ComputeLocals {
    pub emitter: super::emitter::Emitter,
    pub compute_work_groups: usize,
    pub compute_bind_group: wgpu::BindGroup,
    pub density_texture: wgpu::Texture,
    pub uniform_buf: wgpu::Buffer,
    pub compute_pipeline: wgpu::ComputePipeline,
}

impl ComputeLocals {
    pub fn init(
        device: &wgpu::Device,
        _init_encoder: &mut wgpu::CommandEncoder,
        params: &SystemParams,
    ) -> Self {
        // This sets up the compute stage, which is responsible for updating the
        // particle system and most of the game logic. The output is updated game state
        // and a particle density texture.
        let emitter = super::emitter::Emitter::new(device, params.num_particles, 100.0);

        // This needs to match the layout size in the the particle compute shader. Maybe
        // an equivalent to "specialization constants" will come out and allow us to
        // specify the 512 programmatically.
        let particle_group_size = 512;
        let compute_work_groups =
            (params.num_particles as f64 / particle_group_size as f64).ceil() as usize;
        let texture_extent = wgpu::Extent3d {
            width: params.width,
            height: params.height,
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
            num_particles: params.num_particles,
            dt: 0.0,
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

        let particle_buffer_size = (params.num_particles
            * std::mem::size_of::<super::emitter::Particle>() as u32)
            as wgpu::BufferAddress;

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
        let cs = super::include_shader!("particle_system/shader.comp.spv");
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

    pub fn set_uniforms(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        uniform_buffer: &wgpu::Buffer,
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
            uniform_buffer,
            0,
            uniform_buf_size as wgpu::BufferAddress,
        );
    }
}
