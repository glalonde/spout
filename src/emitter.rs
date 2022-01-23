use wgpu::util::DeviceExt;

// This should match the struct defined in the relevant compute shader.
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Particle {
    position: [f32; 2],
    velocity: [f32; 2],
    ttl: f32,
    _padding: i32,
}

struct EmitterParams {
    num_particles: u32,
    emit_period: f32,
    nozzle: NozzleParams,
}

pub struct Emitter {
    params: EmitterParams,

    // State
    time: f32,
    dt: f32,
    emit_progress: f32,
    write_index: u32,

    // This holds the state of the current update's emit, in between update and compute.
    emit_params: Option<EmitParams>,

    // GPU interface cruft
    compute_work_groups: u32,
    compute_bind_group: wgpu::BindGroup,
    uniform_buffer: SizedBuffer,
    pub particle_buffer: SizedBuffer,
    compute_pipeline: wgpu::ComputePipeline,
    staging_belt: wgpu::util::StagingBelt,
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct EmitterMotion {
    // Boundary values for the emitter base
    pub position_start: [f32; 2],
    pub position_end: [f32; 2],
    pub velocity: [f32; 2],
    pub angle_start: f32,
    pub angle_end: f32,
}
impl Default for EmitterMotion {
    fn default() -> Self {
        EmitterMotion {
            position_start: [0.0, 0.0],
            position_end: [0.0, 0.0],
            velocity: [0.0, 0.0],
            angle_start: 0.0,
            angle_end: 0.0,
        }
    }
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct NozzleParams {
    // Boundary values for the emitter base
    pub speed_min: f32,
    pub speed_max: f32,
    pub angle_spread: f32,
    pub ttl_min: f32,
    pub ttl_max: f32,
}
impl Default for NozzleParams {
    fn default() -> Self {
        NozzleParams {
            speed_min: 10.0,
            speed_max: 100.0,
            angle_spread: 0.0,
            ttl_min: 0.0,
            ttl_max: 0.0,
        }
    }
}

// Params for emitting particles in one iteration
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct EmitParams {
    pub start_index: u32,
    pub num_emitted: u32,
    pub time: f32,
    pub dt: f32,

    pub motion: EmitterMotion,
    pub nozzle: NozzleParams,
    _padding: u32,
}

impl Default for EmitParams {
    fn default() -> Self {
        EmitParams {
            start_index: 0,
            num_emitted: 0,
            time: 0.0,
            dt: 0.0,
            motion: EmitterMotion::default(),
            nozzle: NozzleParams::default(),
            _padding: 0,
        }
    }
}

pub struct SizedBuffer {
    buffer: wgpu::Buffer,
    size: wgpu::BufferAddress,
}

impl Emitter {
    fn num_particles(&self) -> u32 {
        return self.params.num_particles;
    }

    fn create_particle_buffer(device: &wgpu::Device, num_particles: u32) -> SizedBuffer {
        let buf_size =
            (num_particles * std::mem::size_of::<Particle>() as u32) as wgpu::BufferAddress;
        SizedBuffer {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                size: buf_size,
                usage: wgpu::BufferUsages::STORAGE,
                label: Some("Particle storage"),
                mapped_at_creation: false,
            }),
            size: buf_size,
        }
    }

    pub fn new(device: &wgpu::Device, emission_frequency: f32, max_particle_life: f32) -> Self {
        let max_num_particles = (emission_frequency * max_particle_life).ceil() as u32;
        log::info!("Num particles: {}", max_num_particles);
        let particle_buffer = Emitter::create_particle_buffer(device, max_num_particles);
        // Initialize the uniform buffer.
        let uniform_buffer =
            make_default_uniform_buffer::<EmitParams>(device, "Emitter Uniform Buffer");

        // This needs to match the layout size in the the particle compute shader. Maybe
        // an equivalent to "specialization constants" will come out and allow us to
        // specify the 512 programmatically.
        let particle_group_size = 256;
        let compute_work_groups =
            (max_num_particles as f64 / particle_group_size as f64).ceil() as u32;
        log::info!(
            "Work groups: {}, Size: {}",
            compute_work_groups,
            particle_group_size
        );

        // Loads the shader from WGSL
        let cs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Emitter shader module"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("emitter.wgsl")),
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(uniform_buffer.size as _),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(particle_buffer.size as _),
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Emitter pipeline layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        // Instantiates the pipeline.
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Emitter pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &cs_module,
            entry_point: "main",
        });

        // Instantiates the bind group, once again specifying the binding of buffers.
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particle_buffer.buffer.as_entire_binding(),
                },
            ],
        });

        let staging_belt = wgpu::util::StagingBelt::new(uniform_buffer.size);

        Emitter {
            params: EmitterParams {
                num_particles: max_num_particles,
                emit_period: 1.0 / emission_frequency,
                nozzle: NozzleParams::default(),
            },
            time: 0.0,
            dt: 0.0,
            emit_progress: 0.0,
            write_index: 0,
            emit_params: None,

            compute_work_groups,
            compute_bind_group,
            uniform_buffer,
            particle_buffer,
            compute_pipeline,
            staging_belt,
        }
    }

    pub fn emit_for_period(&mut self, dt: f32, emitter_motion: EmitterMotion) {
        // Update the emitter state and prepare all the necessary inputs to run compute, but don't actually run the compute yet.
        self.time += dt;
        self.dt = dt;
        self.emit_progress += dt;
        if self.emit_progress > self.params.emit_period {
            let num_emitted: u32 = (self.emit_progress / self.params.emit_period) as u32;
            log::info!("Emitting {} particles", num_emitted);
            self.emit_progress -= (num_emitted as f32) * self.params.emit_period;
            self.emit_params = Some(EmitParams {
                start_index: self.write_index,
                num_emitted,
                time: self.time,
                dt,
                motion: emitter_motion,
                nozzle: self.params.nozzle,
                _padding: 0,
            });

            self.write_index = (self.write_index + num_emitted) % self.params.num_particles;
        } else {
            self.emit_params = None;
        }
    }

    pub fn run_compute(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        if let Some(emit_params) = &self.emit_params {
            // Update uniforms
            // TODO reference https://toji.github.io/webgpu-best-practices/buffer-uploads.html
            self.staging_belt
                .write_buffer(
                    encoder,
                    &self.uniform_buffer.buffer,
                    0,
                    wgpu::BufferSize::new(self.uniform_buffer.size as _).unwrap(),
                    device,
                )
                .copy_from_slice(bytemuck::bytes_of(emit_params));
            self.staging_belt.finish();

            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Particle Emitter"),
                });
                cpass.set_pipeline(&self.compute_pipeline);
                cpass.set_bind_group(0, &self.compute_bind_group, &[]);
                log::info!(
                    "Emitter dispatching {} work groups, emit angle: {}, {}",
                    self.compute_work_groups,
                    emit_params.motion.angle_start,
                    emit_params.motion.angle_end
                );
                cpass.dispatch(self.compute_work_groups, 1, 1);
            }

            // Reset emit, since we processed the compute part.
            self.emit_params = None;
        }
    }

    pub fn after_queue_submission(&mut self, spawner: &crate::framework::Spawner) {
        let belt_future = self.staging_belt.recall();
        spawner.spawn_local(belt_future);
    }
}

pub struct ParticleSystem {
    emitter: Emitter,
    uniform_values: ParticleSystemUniforms,

    // GPU interface cruft
    uniform_buffer: SizedBuffer,
    staging_belt: wgpu::util::StagingBelt,

    update_particles_work_groups: u32,
    update_particles_pipeline: wgpu::ComputePipeline,
    update_particles_bind_group: wgpu::BindGroup,

    clear_work_groups: u32,
    clear_pipeline: wgpu::ComputePipeline,
    clear_bind_group: wgpu::BindGroup,

    renderer: ParticleRenderer,
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct ParticleSystemUniforms {
    dt: f32,
    viewport_width: u32,
    viewport_height: u32,
    viewport_bottom_height: i32,
}
impl Default for ParticleSystemUniforms {
    fn default() -> Self {
        ParticleSystemUniforms {
            dt: 0.0,
            viewport_width: 0,
            viewport_height: 0,
            viewport_bottom_height: 0,
        }
    }
}

impl ParticleSystem {
    fn make_density_buffer(device: &wgpu::Device, width: usize, height: usize) -> SizedBuffer {
        let size = (std::mem::size_of::<u32>() * width * height) as wgpu::BufferAddress;
        SizedBuffer {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Density buffer"),
                size,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
            size,
        }
    }

    pub fn update_state(&mut self, dt: f32, motion: Option<EmitterMotion>) {
        if let Some(motion) = motion {
            self.emitter.emit_for_period(dt, motion);
        }

        self.uniform_values.dt = dt;
    }

    fn init_update_particles_pipeline(
        device: &wgpu::Device,
        uniform_buffer: &SizedBuffer,
        density_buffer: &SizedBuffer,
        emitter: &Emitter,
    ) -> (u32, wgpu::ComputePipeline, wgpu::BindGroup) {
        let compute_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Uniform inputs
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(uniform_buffer.size as _),
                    },
                    count: None,
                },
                // Particle storage buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(emitter.particle_buffer.size as _),
                    },
                    count: None,
                },
                // Particle density buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(density_buffer.size as _),
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle system pipeline layout"),
                bind_group_layouts: &[&compute_bgl],
                push_constant_ranges: &[],
            });

        // Loads the shader from WGSL
        let cs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Particle system shader module"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("particles.wgsl")),
        });

        // Instantiates the pipeline.
        let update_particles_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Particle system pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &cs_module,
                entry_point: "main",
            });

        // Instantiates the bind group, once again specifying the binding of buffers.
        let update_particles_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: emitter.particle_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: density_buffer.buffer.as_entire_binding(),
                },
            ],
        });

        // TODO keep this in sync with shader.
        let num_particles = emitter.num_particles();
        let particle_group_size = 256;
        let update_particles_work_groups =
            (num_particles as f64 / particle_group_size as f64).ceil() as u32;
        (
            update_particles_work_groups,
            update_particles_pipeline,
            update_particles_bind_group,
        )
    }

    fn init_clear_buffer_pipeline(
        device: &wgpu::Device,
        density_buffer: &SizedBuffer,
    ) -> (u32, wgpu::ComputePipeline, wgpu::BindGroup) {
        let compute_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Particle density buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(density_buffer.size as _),
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Clear density buffer pipeline layout"),
                bind_group_layouts: &[&compute_bgl],
                push_constant_ranges: &[],
            });

        // Loads the shader from WGSL
        let cs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Clear density buffer shader module"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("clear_density_buffer.wgsl")),
        });

        // Instantiates the pipeline.
        let clear_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Clear density buffer pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &cs_module,
            entry_point: "main",
        });

        // Instantiates the bind group, once again specifying the binding of buffers.
        let clear_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: density_buffer.buffer.as_entire_binding(),
            }],
        });

        // TODO keep this in sync with shader.
        let clear_group_size = 256;
        let num_density_cells = density_buffer.size / (std::mem::size_of::<u32>() as u64);
        let clear_work_groups = (num_density_cells as f64 / clear_group_size as f64).ceil() as u32;
        (clear_work_groups, clear_pipeline, clear_bind_group)
    }

    pub fn new(
        device: &wgpu::Device,
        game_params: &crate::game_params::GameParams,
        init_encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let uniform_values = ParticleSystemUniforms {
            dt: 0.0,
            viewport_width: game_params.level_width,
            viewport_height: game_params.viewport_height,
            viewport_bottom_height: 0,
        };
        let uniform_buffer = make_uniform_buffer::<ParticleSystemUniforms>(
            device,
            "Particle System Uniform Buffer",
            &uniform_values,
        );

        let density_buffer = ParticleSystem::make_density_buffer(
            device,
            game_params.viewport_width as usize,
            game_params.viewport_height as usize,
        );

        let emitter = Emitter::new(
            device,
            game_params.particle_system_params.emission_rate,
            game_params.particle_system_params.max_particle_life,
        );

        let staging_belt = wgpu::util::StagingBelt::new(uniform_buffer.size);
        let renderer = ParticleRenderer::init(device, game_params, &density_buffer, init_encoder);

        // Set up all the clear density buffer compute pass.
        let (clear_work_groups, clear_pipeline, clear_bind_group) =
            ParticleSystem::init_clear_buffer_pipeline(device, &density_buffer);

        // Set up all the stuff for the particle update compute pass.
        let (update_particles_work_groups, update_particles_pipeline, update_particles_bind_group) =
            ParticleSystem::init_update_particles_pipeline(
                device,
                &uniform_buffer,
                &density_buffer,
                &emitter,
            );

        ParticleSystem {
            emitter,
            uniform_values,
            uniform_buffer,
            staging_belt,

            update_particles_work_groups,
            update_particles_pipeline,
            update_particles_bind_group,

            clear_work_groups,
            clear_pipeline,
            clear_bind_group,

            renderer,
        }
    }

    pub fn run_compute(
        &mut self,
        device: &wgpu::Device,
        game_view_texture: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.emitter.run_compute(device, encoder);

        // Clear density buffer.
        // See https://docs.rs/wgpu/latest/wgpu/struct.CommandEncoder.html#method.clear_buffer
        // encoder.clear_buffer(&self.density_buffer.buffer, 0 as wgpu::BufferAddress, None);
        // Can't use `clear_buffer` on wasm yet:
        // see todo: https://github.com/gfx-rs/wgpu/blob/master/wgpu/src/backend/web.rs#L2079
        {
            // So use a compute shader instead.
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Clear density buffer"),
            });
            cpass.set_pipeline(&self.clear_pipeline);
            cpass.set_bind_group(0, &self.clear_bind_group, &[]);
            cpass.dispatch(self.clear_work_groups, 1, 1);
        }

        {
            // Update uniforms
            self.staging_belt
                .write_buffer(
                    encoder,
                    &self.uniform_buffer.buffer,
                    0,
                    wgpu::BufferSize::new(self.uniform_buffer.size as _).unwrap(),
                    device,
                )
                .copy_from_slice(bytemuck::bytes_of(&self.uniform_values));
            self.staging_belt.finish();

            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Particle Emitter"),
            });
            cpass.set_pipeline(&self.update_particles_pipeline);
            cpass.set_bind_group(0, &self.update_particles_bind_group, &[]);
            cpass.dispatch(self.update_particles_work_groups, 1, 1);
        }

        {
            self.renderer.render(encoder, game_view_texture);
        }
    }

    pub fn after_queue_submission(&mut self, spawner: &crate::framework::Spawner) {
        self.emitter.after_queue_submission(spawner);

        let belt_future = self.staging_belt.recall();
        spawner.spawn_local(belt_future);
    }
}

struct ParticleRenderer {
    pub render_bind_group: wgpu::BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl ParticleRenderer {
    fn init(
        device: &wgpu::Device,
        game_params: &crate::game_params::GameParams,
        density_buffer: &SizedBuffer,
        init_encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("render_particles.wgsl")),
        });

        let cm_texture = crate::color_maps::create_color_map(
            256,
            device,
            super::color_maps::get_color_map_from_index(0),
            init_encoder,
        );

        let fragment_uniforms = ParticleRendererUniforms {
            width: game_params.viewport_width,
            height: game_params.viewport_height,
        };
        let uniform_buffer = make_uniform_buffer::<ParticleRendererUniforms>(
            device,
            "Particle Renderer Uniform Buffer",
            &fragment_uniforms,
        );
        // Create other resources
        let color_map_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Particle render sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create pipeline layout
        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // Uniform inputs
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(uniform_buffer.size as _),
                        },
                        count: None,
                    },
                    // Particle density buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Color map.
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D1,
                        },
                        count: None,
                    },
                    // Color map sampler.
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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
                        uniform_buffer.buffer.as_entire_buffer_binding(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(
                        density_buffer.buffer.as_entire_buffer_binding(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &cm_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&color_map_sampler),
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
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::all(),
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                // cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
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
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: output_texture_view,
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ParticleRendererUniforms {
    pub width: u32,
    pub height: u32,
}

fn make_default_uniform_buffer<T: std::default::Default + bytemuck::Pod>(
    device: &wgpu::Device,
    label: &str,
) -> SizedBuffer {
    let uniforms = T::default();
    make_uniform_buffer::<T>(device, label, &uniforms)
}

fn make_uniform_buffer<T: bytemuck::Pod>(
    device: &wgpu::Device,
    label: &str,
    data: &T,
) -> SizedBuffer {
    let bytes = bytemuck::bytes_of(data);
    SizedBuffer {
        buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }),
        size: bytes.len() as _,
    }
}
