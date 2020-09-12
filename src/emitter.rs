use log::trace;
use wgpu::util::DeviceExt;
use zerocopy::AsBytes;

// This should match the struct defined in the relevant compute shader.
#[derive(Copy, Clone, Debug, zerocopy::FromBytes)]
#[repr(C, packed)]
pub struct Particle {
    position: [u32; 2],
    velocity: [i32; 2],
    ttl: f32,
    padding: i32,
}

struct EmitterParams {
    num_particles: u32,
    emit_period: f32,
}

pub struct Emitter {
    params: EmitterParams,
    time: f32,
    dt: f32,
    emit_progress: f32,
    write_index: u32,
    compute_work_groups: u32,
    compute_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    pub particle_buffer: wgpu::Buffer,
    compute_pipeline: wgpu::ComputePipeline,
}

// Params for emitting particles in one iteration
#[derive(Copy, Clone, Debug, zerocopy::FromBytes, zerocopy::AsBytes)]
#[repr(C, packed)]
pub struct EmitParams {
    // Boundary values for the emitter base
    pub position_start: [u32; 2],
    pub position_end: [u32; 2],
    pub velocity: [i32; 2],
    pub angle_start: f32,
    pub angle_end: f32,

    // Parameters for the "nozzle"
    pub speed_min: f32,
    pub speed_max: f32,
    pub angle_spread: f32,
    pub ttl_min: f32,
    pub ttl_max: f32,
}

impl EmitParams {
    pub fn default() -> Self {
        EmitParams::stationary(&[0, 0], 0.0, 0.0, 0.0, 0.0, 1.0)
    }

    pub fn moving(
        position_start: &[u32; 2],
        position_end: &[u32; 2],
        speed_mean: f32,
        speed_spread: f32,
        angle_start: f32,
        angle_end: f32,
        angle_spread: f32,
        ttl: f32,
    ) -> Self {
        // TODO fix velocity
        EmitParams {
            position_start: *position_start,
            position_end: *position_end,
            velocity: [0, 0],
            angle_start,
            angle_end,
            speed_min: speed_mean - speed_spread,
            speed_max: speed_mean + speed_spread,
            angle_spread,
            ttl_min: ttl,
            ttl_max: ttl,
        }
    }

    pub fn stationary(
        pos: &[u32; 2],
        speed_mean: f32,
        speed_spread: f32,
        angle: f32,
        angle_spread: f32,
        ttl: f32,
    ) -> Self {
        EmitParams {
            position_start: *pos,
            position_end: *pos,
            velocity: [0, 0],
            angle_start: angle,
            angle_end: angle,
            speed_min: speed_mean - speed_spread,
            speed_max: speed_mean + speed_spread,
            angle_spread,
            ttl_min: ttl,
            ttl_max: ttl,
        }
    }
}

// This should match the struct defined in the emitter shader.
#[derive(Copy, Clone, Debug, zerocopy::FromBytes, zerocopy::AsBytes)]
#[repr(C, packed)]
struct EmitterUniforms {
    start_index: u32,
    num_emitted: u32,
    params: EmitParams,
    time: f32,
    dt: f32,
}

impl Emitter {
    // Returns the total number of particles, including inactive ones.
    pub fn num_particles(&self) -> u32 {
        self.params.num_particles
    }

    fn create_particle_buffer(device: &wgpu::Device, num_particles: u32) -> wgpu::Buffer {
        let buf_size =
            (num_particles * std::mem::size_of::<Particle>() as u32) as wgpu::BufferAddress;

        device.create_buffer(&wgpu::BufferDescriptor {
            size: buf_size,
            usage: wgpu::BufferUsage::STORAGE,
            label: Some("Particle storage"),
            mapped_at_creation: false,
        })
    }

    fn create_uniform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        let emitter_uniforms = EmitterUniforms {
            start_index: 0,
            num_emitted: 0,
            params: EmitParams::default(),
            time: 0.0,
            dt: 0.0,
        };

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Emitter Uniform Buffer"),
            contents: &emitter_uniforms.as_bytes(),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        })
    }

    pub fn new(device: &wgpu::Device, emission_frequency: f32, max_particle_life: f32) -> Self {
        let max_num_particles = (emission_frequency * max_particle_life).ceil() as u32;
        let particle_buffer = Emitter::create_particle_buffer(device, max_num_particles);
        // Initialize the uniform buffer.
        let uniform_buffer = Emitter::create_uniform_buffer(device);

        // This needs to match the layout size in the the particle compute shader. Maybe
        // an equivalent to "specialization constants" will come out and allow us to
        // specify the 512 programmatically.
        let particle_group_size = 512;
        let compute_work_groups =
            (max_num_particles as f64 / particle_group_size as f64).ceil() as u32;

        // Setup the shader pipeline stage.
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
                    // Uniform inputs
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("Particle update layout"),
            });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute_bind_group_layout,
            entries: &[
                // Particle storage buffer
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(particle_buffer.slice(..)),
                },
                // Uniforms
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
                },
            ],
            label: Some("Particle update binding"),
        });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute pipeline layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let cs = super::shader_utils::Shaders::get("particle_system/emitter.comp.spv").unwrap();
        let cs_module = device.create_shader_module(wgpu::util::make_spirv(&cs));
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline descriptor"),
            layout: Some(&compute_pipeline_layout),
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &cs_module,
                entry_point: "main",
            },
        });

        Emitter {
            params: EmitterParams {
                num_particles: max_num_particles,
                emit_period: 1.0 / emission_frequency,
            },
            time: 0.0,
            dt: 0.0,
            emit_progress: 0.0,
            write_index: 0,
            compute_work_groups,
            compute_bind_group,
            uniform_buffer,
            particle_buffer,
            compute_pipeline,
        }
    }
    pub fn emit_over_time(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        dt: f32,
        params: &EmitParams,
    ) {
        self.time += dt;
        self.dt = dt;
        self.emit_progress += dt;
        if self.emit_progress > self.params.emit_period {
            let num_emitted: u32 = (self.emit_progress / self.params.emit_period) as u32;
            self.emit_progress -= (num_emitted as f32) * self.params.emit_period;
            self.emit(device, encoder, num_emitted, params)
        }
    }
    fn set_uniforms(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        uniform_buffer: &wgpu::Buffer,
        values: &EmitterUniforms,
    ) {
        let uniform_buf_size = std::mem::size_of::<EmitterUniforms>();

        let temp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Staging Buffer"),
            contents: &values.as_bytes(),
            usage: wgpu::BufferUsage::COPY_SRC,
        });
        encoder.copy_buffer_to_buffer(
            &temp_buf,
            0,
            uniform_buffer,
            0,
            uniform_buf_size as wgpu::BufferAddress,
        );
    }
    fn emit(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        num_emitted: u32,
        params: &EmitParams,
    ) {
        trace!("Emit params: {:?}", params);
        let emitter_uniforms = EmitterUniforms {
            start_index: self.write_index,
            num_emitted: num_emitted,
            params: *params,
            time: self.time,
            dt: self.dt,
        };
        Emitter::set_uniforms(device, encoder, &self.uniform_buffer, &emitter_uniforms);
        {
            let mut cpass = encoder.begin_compute_pass();
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            trace!("Dispatching {} work groups", self.compute_work_groups);
            cpass.dispatch(self.compute_work_groups, 1, 1);
        }

        self.write_index = (self.write_index + num_emitted) % self.params.num_particles;
    }
}
