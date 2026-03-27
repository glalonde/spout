//! GPU-based ship-terrain collision detection with contact normal.
//!
//! Runs a compute shader that Bresenham-walks each hull vertex from its
//! previous to current position, checking the GPU terrain buffer. Returns
//! a hit flag and the axis-aligned contact normal for bouncing.

use wgpu::util::DeviceExt;

use crate::buffer_util::SizedBuffer;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CollisionUniforms {
    ship_x: f32,
    ship_y: f32,
    prev_ship_x: f32,
    prev_ship_y: f32,
    ship_orientation: f32,
    prev_ship_orientation: f32,
    terrain_buffer_offset: i32,
    terrain_width: u32,
    terrain_buffer_height: u32,
    _pad: u32,
}

/// Collision result read back from the GPU.
#[derive(Debug, Clone, Copy, Default)]
pub struct CollisionResult {
    pub hit: bool,
    /// Axis-aligned contact normal: e.g. (1,0) = hit from the left,
    /// (0,-1) = hit from above.
    pub normal: [f32; 2],
}

pub struct CollisionDetector {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: SizedBuffer,
    /// GPU-side result buffer (3 x u32). Written by the compute shader.
    result_buffer: wgpu::Buffer,
    /// CPU-readable staging buffer for async readback.
    staging_buffer: wgpu::Buffer,
    /// Whether a readback is in flight.
    pending_readback: bool,
    /// Last collision result read from the GPU.
    pub result: CollisionResult,
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
            prev_ship_x: 0.0,
            prev_ship_y: 0.0,
            ship_orientation: 0.0,
            prev_ship_orientation: 0.0,
            terrain_buffer_offset: 0,
            terrain_width: 0,
            terrain_buffer_height: 0,
            _pad: 0,
        };
        let uniform_buffer =
            crate::buffer_util::make_uniform_buffer(device, "Collision Uniform Buffer", &uniforms);

        // 3 x u32: [hit, normal_x_bits, normal_y_bits]
        let result_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Collision Result"),
            contents: bytemuck::cast_slice(&[0u32; 3]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Collision Staging"),
            size: 12, // 3 x u32
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
            result: CollisionResult::default(),
        }
    }

    /// Dispatch the collision compute shader.
    #[allow(clippy::too_many_arguments)]
    pub fn dispatch(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        ship: &crate::ship::ShipState,
        prev_ship: &crate::ship::ShipState,
        terrain: &crate::level_manager::TerrainTile,
        terrain_width: u32,
    ) {
        let uniforms = CollisionUniforms {
            ship_x: ship.position[0],
            ship_y: ship.position[1],
            prev_ship_x: prev_ship.position[0],
            prev_ship_y: prev_ship.position[1],
            ship_orientation: ship.orientation,
            prev_ship_orientation: prev_ship.orientation,
            terrain_buffer_offset: terrain.shape.start,
            terrain_width,
            terrain_buffer_height: terrain.shape.size() as u32,
            _pad: 0,
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

        encoder.copy_buffer_to_buffer(&self.result_buffer, 0, &self.staging_buffer, 0, 12);
        self.pending_readback = true;
    }

    /// Poll for the async readback result. Updates `self.result`.
    pub fn poll_result(&mut self, device: &wgpu::Device) {
        if !self.pending_readback {
            return;
        }

        let slice = self.staging_buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        // safe: wait_indefinitely always resolves after GPU work completes
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

        let data = slice.get_mapped_range();
        let values: &[u32] = bytemuck::cast_slice(&data[..12]);
        self.result = CollisionResult {
            hit: values[0] != 0,
            normal: [f32::from_bits(values[1]), f32::from_bits(values[2])],
        };
        drop(data);
        self.staging_buffer.unmap();

        self.pending_readback = false;
    }
}
