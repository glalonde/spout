use gflags;
use log::info;
use rand::{Rng, SeedableRng};

gflags::define! {
    --num_particles: usize = 500
}
gflags::define! {
    --num_iterations: usize = 1
}
gflags::define! {
    --width: u32 = 500
}
gflags::define! {
    --height: u32 = 500
}

#[derive(Copy, Clone, Debug, zerocopy::FromBytes)]
#[repr(C, packed)]
struct Particle {
    position: [i32; 2],
    velocity: [i32; 2],
}

fn fill_with_random_particles(
    x_range: &[i32; 2],
    y_range: &[i32; 2],
    velocity_range: &[i32; 2],
    rng: &mut rand::rngs::SmallRng,
    particles: &mut Vec<Particle>,
) {
    for _ in particles.len()..particles.capacity() {
        particles.push(Particle {
            position: [
                rng.gen_range(x_range[0], x_range[1]),
                rng.gen_range(y_range[0], y_range[1]),
            ],
            velocity: [
                rng.gen_range(velocity_range[0], velocity_range[1]),
                rng.gen_range(velocity_range[0], velocity_range[1]),
            ],
        });
    }
}

fn run() {
    let width = WIDTH.flag;
    let height = HEIGHT.flag;

    // Create the particles
    let mut particle_buf: Vec<Particle> = Vec::with_capacity(NUM_PARTICLES.flag);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(10);
    fill_with_random_particles(
        &[0, width as i32],
        &[0, height as i32],
        &[-5, 5],
        &mut rng,
        &mut particle_buf,
    );

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

    // This needs to match the layout size in the the particle compute shader. Maybe
    // an equivalent to "specialization constants" will come out and allow us to
    // specify the 512 programmatically.
    let particle_group_size = 512;
    let num_work_groups = (NUM_PARTICLES.flag as f64 / particle_group_size as f64).ceil() as u32;
    let cs = spout::shader_utils::Shaders::get("atomics.comp.spv").unwrap();
    let cs_module =
        device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&cs[..])).unwrap());

    // This buffer is used to transfer to the GPU-only buffer.
    let staging_buffer = device
        .create_buffer_mapped(
            particle_buf.len(),
            wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
        )
        .fill_from_slice(&particle_buf);

    // The GPU-only buffer
    let particle_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        size: buf_size,
        usage: wgpu::BufferUsage::STORAGE
            | wgpu::BufferUsage::COPY_DST
            | wgpu::BufferUsage::COPY_SRC,
    });

    // The render pipeline renders data into this texture
    let texture_extent = wgpu::Extent3d {
        width: width as u32,
        height: height as u32,
        depth: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Uint,
        usage: wgpu::TextureUsage::COPY_SRC
            | wgpu::TextureUsage::STORAGE
            | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    });
    let texture_view = texture.create_default_view();

    // The output buffer lets us retrieve the texture data as an array
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        size: (width * height) as u64 * std::mem::size_of::<u32>() as u64,
        usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
    });

    info!("Creating bind group layout");
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            wgpu::BindGroupLayoutBinding {
                binding: 0,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::StorageBuffer {
                    dynamic: false,
                    readonly: false,
                },
            },
            wgpu::BindGroupLayoutBinding {
                binding: 1,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    dimension: wgpu::TextureViewDimension::D2,
                },
            },
        ],
    });

    info!("Creating bind group");
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[
            wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &particle_storage_buffer,
                    range: 0..buf_size,
                },
            },
            wgpu::Binding {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
        ],
    });

    info!("Creating pipeline layout");
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    info!("Creating compute pipeline");
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
        cpass.dispatch(num_work_groups, 1, 1);
    };
    let copy_buffer_to_buffer =
        |from: &wgpu::Buffer, to: &wgpu::Buffer, queue: &mut wgpu::Queue| {
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
            encoder.copy_buffer_to_buffer(&from, 0, &to, 0, buf_size);
            queue.submit(&[encoder.finish()]);
        };

    // Copy initial data to GPU
    copy_buffer_to_buffer(&staging_buffer, &particle_storage_buffer, &mut queue);
    {
        // Clear the texture.
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &texture_view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::BLACK,
            }],
            depth_stencil_attachment: None,
        });
        queue.submit(&[encoder.finish()]);
    }
    for _ in 0..NUM_ITERATIONS.flag {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        add_particle_update(&mut encoder);
        queue.submit(&[encoder.finish()]);
    }
    {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        // Particle data
        encoder.copy_buffer_to_buffer(&particle_storage_buffer, 0, &staging_buffer, 0, buf_size);
        // Particle density
        encoder.copy_texture_to_buffer(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::BufferCopyView {
                buffer: &output_buffer,
                offset: 0,
                row_pitch: std::mem::size_of::<u32>() as u32 * width as u32,
                image_height: width as u32,
            },
            texture_extent,
        );
        queue.submit(&[encoder.finish()]);
    }

    // This reads the particle data.
    staging_buffer.map_read_async(
        0,
        buf_size,
        |result: wgpu::BufferMapAsyncResult<&[Particle]>| {
            if let Ok(mapping) = result {
                info!("Particles: {:?}", mapping.data);
            }
        },
    );

    output_buffer.map_read_async(
        0,
        (width * height) as u64 * std::mem::size_of::<u32>() as u64,
        move |result: wgpu::BufferMapAsyncResult<&[u32]>| {
            if let Ok(mapping) = result {
                let mut sum: i64 = 0;
                for v in mapping.data {
                    // info!("val: {}, {:#034b}, {:b}", v, v, 1);
                    sum += *v as i64;
                }
                info!("Num particles: {}", sum);
            }
        },
    );
}

fn main() {
    scrub_log::init().unwrap();
    gflags::parse();
    run();
}
