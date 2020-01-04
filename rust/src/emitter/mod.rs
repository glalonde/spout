use super::include_shader;
use log::{info, trace};
use zerocopy::AsBytes;

// This should match the struct defined in the relevant compute shader.
#[derive(Copy, Clone, Debug, zerocopy::FromBytes)]
#[repr(C, packed)]
pub struct Particle {
    position: [i32; 2],
    velocity: [i32; 2],
}

struct EmitterParams {
    num_particles: u32,
    emit_period: f32,
}

pub struct Emitter {
    params: EmitterParams,
    time: f32,
    emit_progress: f32,
    write_index: u32,
    compute_work_groups: u32,
    compute_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    pub particle_buffer: wgpu::Buffer,
    compute_pipeline: wgpu::ComputePipeline,
}

// This should match the struct defined in the emitter shader.
#[derive(Copy, Clone, Debug, zerocopy::FromBytes, zerocopy::AsBytes)]
#[repr(C, packed)]
struct EmitterUniforms {
    start_index: u32,
    num_emitted: u32,
    init_position: [i32; 2],
    init_velocity: [i32; 2],
    ttl: f32,
    padding: i32,
}

impl Emitter {
    fn create_particle_buffer(
        device: &wgpu::Device,
        num_particles: u32,
    ) -> (wgpu::BufferAddress, wgpu::Buffer) {
        let buf_size =
            (num_particles * std::mem::size_of::<Particle>() as u32) as wgpu::BufferAddress;
        (
            buf_size,
            device.create_buffer(&wgpu::BufferDescriptor {
                size: buf_size,
                usage: wgpu::BufferUsage::STORAGE,
            }),
        )
    }

    fn create_uniform_buffer(device: &wgpu::Device) -> (wgpu::BufferAddress, wgpu::Buffer) {
        let emitter_uniforms = EmitterUniforms {
            start_index: 0,
            num_emitted: 0,
            init_position: [0, 0],
            init_velocity: [0, 0],
            ttl: 1.0,
            padding: 0,
        };
        (
            std::mem::size_of::<EmitterUniforms>() as wgpu::BufferAddress,
            device
                .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
                .fill_from_slice(&[emitter_uniforms]),
        )
    }

    pub fn new(device: &wgpu::Device, num_particles: u32, emission_frequency: f32) -> Self {
        let (particle_buffer_size, particle_buffer) =
            Emitter::create_particle_buffer(device, num_particles);
        // Initialize the uniform buffer.
        let (uniform_buffer_size, uniform_buffer) = Emitter::create_uniform_buffer(device);

        // This needs to match the layout size in the the particle compute shader. Maybe
        // an equivalent to "specialization constants" will come out and allow us to
        // specify the 512 programmatically.
        let particle_group_size = 512;
        let compute_work_groups = (num_particles as f64 / particle_group_size as f64).ceil() as u32;

        // Setup the shader pipeline stage.
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
                    // Uniform inputs
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
                // Particle storage buffer
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &particle_buffer,
                        range: 0..particle_buffer_size,
                    },
                },
                // Uniforms
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        range: 0..uniform_buffer_size,
                    },
                },
            ],
        });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&compute_bind_group_layout],
            });

        let cs = include_shader!("particle_system/emitter.comp.spv");
        let cs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&cs[..])).unwrap());
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: &compute_pipeline_layout,
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &cs_module,
                entry_point: "main",
            },
        });

        Emitter {
            params: EmitterParams {
                num_particles: num_particles,
                emit_period: 1.0 / emission_frequency,
            },
            time: 0.0,
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
    ) {
        self.time += dt;
        self.emit_progress += dt;
        if self.emit_progress > self.params.emit_period {
            let num_emitted: u32 = (self.emit_progress / self.params.emit_period) as u32;
            self.emit_progress -= (num_emitted as f32) * self.params.emit_period;
            self.emit(device, encoder, num_emitted)
        }
    }
    fn set_uniforms(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        uniform_buffer: &wgpu::Buffer,
        values: &EmitterUniforms,
    ) {
        let bytes: &[u8] = values.as_bytes();
        let uniform_buf_size = std::mem::size_of::<EmitterUniforms>();
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
    fn emit(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        num_emitted: u32,
    ) {
        let emitter_uniforms = EmitterUniforms {
            start_index: self.write_index,
            num_emitted: num_emitted,
            init_position: [1, 1],
            init_velocity: [1, 1],
            ttl: 2.0,
            padding: 0,
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
