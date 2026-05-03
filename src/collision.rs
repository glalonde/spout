//! GPU-based ship-terrain collision detection with contact normal.
//!
//! Runs a compute shader that Bresenham-walks each hull vertex from its
//! previous to current position, checking the GPU terrain buffer. Returns
//! a hit flag and the axis-aligned contact normal for bouncing.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
    /// Whether a readback copy has been queued (dispatch called).
    pending_readback: bool,
    /// Whether `map_async` has been initiated for the current readback.
    mapping_started: bool,
    /// Set to `true` by the `map_async` callback when the mapping completes.
    map_ready: Arc<AtomicBool>,
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
            mapping_started: false,
            map_ready: Arc::new(AtomicBool::new(false)),
            result: CollisionResult::default(),
        }
    }

    /// Dispatch the collision compute shader.
    /// Skips if a previous readback is still in flight (the staging buffer
    /// can't be used as a copy destination while a map is pending).
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
        if self.pending_readback {
            return;
        }
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

    /// Initiate async mapping of the staging buffer. Call after `queue.submit()`
    /// so the GPU copy has been submitted. On native, the callback fires during
    /// the next `device.poll()`; on WASM it fires on the next microtask.
    pub fn start_readback(&mut self) {
        if !self.pending_readback || self.mapping_started {
            return;
        }
        let slice = self.staging_buffer.slice(..);
        let ready = Arc::clone(&self.map_ready);
        ready.store(false, Ordering::Release);
        slice.map_async(wgpu::MapMode::Read, move |_| {
            ready.store(true, Ordering::Release);
        });
        self.mapping_started = true;
    }

    /// Poll for the async readback result. Updates `self.result`.
    /// Returns immediately if the mapping hasn't completed yet (WASM-safe).
    /// Caller is responsible for driving wgpu callbacks (device.poll) before
    /// calling this — see render() in main.rs.
    pub fn poll_result(&mut self) {
        if !self.pending_readback || !self.mapping_started {
            return;
        }

        if !self.map_ready.load(Ordering::Acquire) {
            return; // Not ready yet — will check again next frame.
        }

        let data = self.staging_buffer.slice(..).get_mapped_range();
        let values: &[u32] = bytemuck::cast_slice(&data[..12]);
        self.result = CollisionResult {
            hit: values[0] != 0,
            normal: [f32::from_bits(values[1]), f32::from_bits(values[2])],
        };
        drop(data);
        self.staging_buffer.unmap();

        self.pending_readback = false;
        self.mapping_started = false;
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::gpu_test_utils as gpu;

    /// Run the full dispatch → submit → start_readback → poll_result cycle.
    /// This catches the WASM bug where poll_result tried to read the staging
    /// buffer before the map_async callback had fired.
    #[test]
    fn test_collision_readback_lifecycle() {
        let Some((device, queue)) = gpu::try_create_headless_device() else {
            eprintln!("No GPU adapter available — skipping test_collision_readback_lifecycle");
            return;
        };

        let mut detector = CollisionDetector::init(&device);

        // Create a minimal empty terrain buffer (ship in empty space = no collision).
        let terrain_width = 64u32;
        let terrain_height = 64u32;
        let terrain_data = vec![0i32; (terrain_width * terrain_height) as usize];
        let terrain_buffer = crate::buffer_util::SizedBuffer {
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Test terrain"),
                contents: bytemuck::cast_slice(&terrain_data),
                usage: wgpu::BufferUsages::STORAGE,
            }),
            size: (terrain_data.len() * 4) as u64,
        };
        let terrain_tile = crate::level_manager::TerrainTile {
            shape: crate::level_manager::Interval {
                start: 0,
                end: terrain_height as i32,
            },
            buffer: terrain_buffer,
        };

        let ship = crate::ship::ShipState {
            position: [32.0, 32.0],
            ..Default::default()
        };

        // Phase 1: dispatch (queues compute + copy).
        let mut belt = wgpu::util::StagingBelt::new(device.clone(), 256);
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        detector.dispatch(
            &device,
            &mut encoder,
            &mut belt,
            &ship,
            &ship,
            &terrain_tile,
            terrain_width,
        );

        // Phase 2: submit GPU work.
        belt.finish();
        queue.submit(Some(encoder.finish()));
        belt.recall();

        // Phase 3: start_readback (initiates map_async with callback).
        detector.start_readback();
        assert!(detector.mapping_started);

        // Phase 4: poll_result (waits for callback, reads data).
        // This is the step that panicked on WASM before the fix.
        // Drive callbacks before polling — in production this happens via the
        // device.poll(Poll) call in the render loop after queue.submit().
        device.poll(wgpu::PollType::wait_indefinitely()).ok();
        detector.poll_result();
        assert!(!detector.pending_readback, "Readback should have completed");

        // Ship in empty terrain → no collision.
        assert!(
            !detector.result.hit,
            "No collision expected in empty terrain"
        );
    }
}
