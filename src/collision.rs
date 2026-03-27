//! GPU-based ship-terrain collision detection. Runs a compute shader that
//! checks terrain health at the ship position against the GPU terrain buffer
//! (which reflects particle erosion). The result is read back asynchronously
//! with one frame of latency.

use wgpu::util::DeviceExt;

use crate::buffer_util::SizedBuffer;

const SHIP_COLLISION_RADIUS: f32 = 4.0;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CollisionUniforms {
    ship_x: f32,
    ship_y: f32,
    terrain_buffer_offset: i32,
    terrain_width: u32,
    terrain_buffer_height: u32,
    collision_radius: f32,
}

pub struct CollisionDetector {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: SizedBuffer,
    /// GPU-side result buffer (1 x u32). Written by the compute shader.
    result_buffer: wgpu::Buffer,
    /// CPU-readable staging buffer for async readback.
    staging_buffer: wgpu::Buffer,
    /// Whether a readback is in flight (waiting for map_async).
    pending_readback: bool,
    /// Last collision result read from the GPU.
    pub colliding: bool,
}

impl CollisionDetector {
    pub fn init(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Collision Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/collision.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Collision BGL"),
            entries: &[
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Terrain buffer (read-only)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Result buffer (read-write)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Collision Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Collision Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let uniforms = CollisionUniforms {
            ship_x: 0.0,
            ship_y: 0.0,
            terrain_buffer_offset: 0,
            terrain_width: 0,
            terrain_buffer_height: 0,
            collision_radius: SHIP_COLLISION_RADIUS,
        };
        let uniform_buffer =
            crate::buffer_util::make_uniform_buffer(device, "Collision Uniform Buffer", &uniforms);

        let result_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Collision Result"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Collision Staging"),
            size: 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        CollisionDetector {
            pipeline,
            bind_group_layout,
            uniform_buffer,
            result_buffer,
            staging_buffer,
            pending_readback: false,
            colliding: false,
        }
    }

    /// Dispatch the collision compute shader and copy the result to the staging
    /// buffer. Call `poll_result` next frame to read it back.
    pub fn dispatch(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        ship: &crate::ship::ShipState,
        terrain: &crate::level_manager::TerrainTile,
        terrain_width: u32,
    ) {
        let uniforms = CollisionUniforms {
            ship_x: ship.position[0],
            ship_y: ship.position[1],
            terrain_buffer_offset: terrain.shape.start,
            terrain_width,
            terrain_buffer_height: terrain.shape.size() as u32,
            collision_radius: SHIP_COLLISION_RADIUS,
        };

        // safe: uniform_buffer.size is always > 0 (set at GPU buffer creation)
        belt.write_buffer(
            encoder,
            &self.uniform_buffer.buffer,
            0,
            wgpu::BufferSize::new(self.uniform_buffer.size as _).unwrap(),
        )
        .copy_from_slice(bytemuck::bytes_of(&uniforms));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Collision BG"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: terrain.buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.result_buffer.as_entire_binding(),
                },
            ],
        });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Collision"),
                ..Default::default()
            });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(1, 1, 1);
        }

        // Copy result to staging buffer for CPU readback.
        encoder.copy_buffer_to_buffer(&self.result_buffer, 0, &self.staging_buffer, 0, 4);
        self.pending_readback = true;
    }

    /// Poll for the async readback result. Call once per frame after
    /// `queue.submit`. Updates `self.colliding`.
    pub fn poll_result(&mut self, device: &wgpu::Device) {
        if !self.pending_readback {
            return;
        }

        let slice = self.staging_buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        // Poll the device to drive the map operation. This is non-blocking
        // if the GPU has already finished the copy (which it has, since we
        // submitted last frame).
        // safe: wait_indefinitely always resolves after GPU work completes
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

        let data = slice.get_mapped_range();
        let value: u32 = *bytemuck::from_bytes(&data[..4]);
        drop(data);
        self.staging_buffer.unmap();

        self.colliding = value != 0;
        self.pending_readback = false;
    }
}
