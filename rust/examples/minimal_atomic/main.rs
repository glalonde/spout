use gflags;
use log::info;

gflags::define! {
    --width: u32 = 10 
}
gflags::define! {
    --height: u32 = 1
}

fn run() {
    let width = WIDTH.flag;
    let height = HEIGHT.flag;

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

    let cs = spout::include_shader!("atomics_minimal.comp.spv");
    let cs_module =
        device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&cs[..])).unwrap());

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
    // Clear the texture.
    {
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
    let add_compute_pass = |encoder: &mut wgpu::CommandEncoder| {
        let mut cpass = encoder.begin_compute_pass();
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch(1, 1, 1);
    };
    // Compute shader
    {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        add_compute_pass(&mut encoder);
        queue.submit(&[encoder.finish()]);
    }
    // Retrieve texture
    {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
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

    output_buffer.map_read_async(
        0,
        (width * height) as u64 * std::mem::size_of::<u32>() as u64,
        move |result: wgpu::BufferMapAsyncResult<&[u32]>| {
            if let Ok(mapping) = result {
                for v in mapping.data {
                    info!("val: {}, {:#034b}", v, v);
                }
            }
        },
    );
}

fn main() {
    scrub_log::init().unwrap();
    gflags::parse();
    run();
}
