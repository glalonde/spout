use gflags;
use log::info;
use rand::{rngs::SmallRng, Rng, SeedableRng};

gflags::define! {
    --num_particles: usize = 500
}
gflags::define! {
    --num_iterations: usize = 1
}

#[derive(Copy, Clone, Debug, zerocopy::FromBytes)]
#[repr(C, packed)]
struct Particle {
    position: [i32; 2],
    velocity: [i32; 2],
}

fn fill_with_random_particles(
    position_range: &[i32; 2],
    velocity_range: &[i32; 2],
    rng: &mut rand::rngs::SmallRng,
    particles: &mut Vec<Particle>,
) {
    for _ in particles.len()..particles.capacity() {
        particles.push(Particle {
            position: [
                rng.gen_range(position_range[0], position_range[1]),
                rng.gen_range(position_range[0], position_range[1]),
            ],
            velocity: [
                rng.gen_range(velocity_range[0], velocity_range[1]),
                rng.gen_range(velocity_range[0], velocity_range[1]),
            ],
        });
    }
}

fn main() {
    scrub_log::init().unwrap();
    gflags::parse();

    // Create the particles
    let mut particle_buf: Vec<Particle> = Vec::with_capacity(NUM_PARTICLES.flag);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(10);
    fill_with_random_particles(&[0, 100], &[-5, 5], &mut rng, &mut particle_buf);

    let buf_size = (particle_buf.len() * std::mem::size_of::<Particle>()) as wgpu::BufferAddress;

    let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::Default,
        backends: wgpu::BackendBit::PRIMARY,
    })
    .unwrap();

    let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    });

    let cs = spout::include_shader!("atomics.comp.spv");
    let cs_module =
        device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&cs[..])).unwrap());

    // This buffer is used to transfer to the GPU-only buffer.
    let staging_buffer = device
        .create_buffer_mapped(
            particle_buf.len(),
            wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
        )
        .fill_from_slice(&particle_buf);

    // I think this is the GPU-only buffer
    let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        size: buf_size,
        usage: wgpu::BufferUsage::STORAGE
            | wgpu::BufferUsage::COPY_DST
            | wgpu::BufferUsage::COPY_SRC,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[wgpu::BindGroupLayoutBinding {
            binding: 0,
            visibility: wgpu::ShaderStage::COMPUTE,
            ty: wgpu::BindingType::StorageBuffer {
                dynamic: false,
                readonly: false,
            },
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[wgpu::Binding {
            binding: 0,
            resource: wgpu::BindingResource::Buffer {
                buffer: &storage_buffer,
                range: 0..buf_size,
            },
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        layout: &pipeline_layout,
        compute_stage: wgpu::ProgrammableStageDescriptor {
            module: &cs_module,
            entry_point: "main",
        },
    });
    let add_particle_update = |encoder: &mut wgpu::CommandEncoder| {
        let mut cpass = encoder.begin_compute_pass();
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch(particle_buf.len() as u32, 1, 1);
    };
    let copy_buffer_to_buffer =
        |from: &wgpu::Buffer, to: &wgpu::Buffer, queue: &mut wgpu::Queue| {
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
            encoder.copy_buffer_to_buffer(&from, 0, &to, 0, buf_size);
            queue.submit(&[encoder.finish()]);
        };

    // Copy initial data to GPU
    copy_buffer_to_buffer(&staging_buffer, &storage_buffer, &mut queue);
    for _ in 0..NUM_ITERATIONS.flag {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        add_particle_update(&mut encoder);
        queue.submit(&[encoder.finish()]);
    }
    // Copy finished data from GPU
    copy_buffer_to_buffer(&storage_buffer, &staging_buffer, &mut queue);

    staging_buffer.map_read_async(
        0,
        buf_size,
        |result: wgpu::BufferMapAsyncResult<&[Particle]>| {
            if let Ok(mapping) = result {
                info!("Particles: {:?}", mapping.data);
            }
        },
    );
}
