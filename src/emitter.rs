use std::borrow::Cow;
use wgpu::util::DeviceExt;

// This should match the struct defined in the relevant compute shader.
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Particle {
    position: [u32; 2],
    velocity: [i32; 2],
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
    pub position_start: [u32; 2],
    pub position_end: [u32; 2],
    pub velocity: [i32; 2],
    pub angle_start: f32,
    pub angle_end: f32,
}
impl Default for EmitterMotion {
    fn default() -> Self {
        EmitterMotion {
            position_start: [0, 0],
            position_end: [0, 0],
            velocity: [0, 0],
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
            speed_min: 0.0,
            speed_max: 0.0,
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
        }
    }
}

pub struct SizedBuffer {
    buffer: wgpu::Buffer,
    size: wgpu::BufferAddress,
}

impl Emitter {
    // Returns the total number of particles, including inactive ones.
    pub fn num_particles(&self) -> u32 {
        self.params.num_particles
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

    fn create_uniform_buffer(device: &wgpu::Device) -> SizedBuffer {
        let emitter_uniforms = EmitParams::default();
        let bytes = bytemuck::bytes_of(&emitter_uniforms);
        SizedBuffer {
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Emitter Uniform Buffer"),
                contents: bytes,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            size: bytes.len() as _,
        }
    }

    pub fn new(device: &wgpu::Device, emission_frequency: f32, max_particle_life: f32) -> Self {
        let max_num_particles = (emission_frequency * max_particle_life).ceil() as u32;
        log::info!("Num particles: {}", max_num_particles);
        let particle_buffer = Emitter::create_particle_buffer(device, max_num_particles);
        // Initialize the uniform buffer.
        let uniform_buffer = Emitter::create_uniform_buffer(device);

        // This needs to match the layout size in the the particle compute shader. Maybe
        // an equivalent to "specialization constants" will come out and allow us to
        // specify the 512 programmatically.
        let particle_group_size = 512;
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
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(crate::include_shader!("emitter.wgsl"))),
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

    pub fn update(&mut self, dt: f32, emitter_motion: EmitterMotion) {
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
                log::info!("Dispatching {} work groups", self.compute_work_groups);
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
