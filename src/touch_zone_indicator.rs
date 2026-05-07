//! Faint diagonal hint line for the Triangle touch-control scheme.
//!
//! Drawn after the bloom composite, directly onto the surface, only when the
//! Triangle scheme is active and the game is in `Playing` mode. Renders as a
//! thin alpha-blended rectangle — visible enough to teach the player where
//! the CW/CCW split is, faint enough to not compete with the game.

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    surface_size: [f32; 2],
    thickness_px: f32,
    _pad: f32,
}

pub struct TouchZoneIndicator {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
}

impl TouchZoneIndicator {
    /// Line thickness in surface pixels. Small enough to read as a hint, not
    /// a UI element.
    const THICKNESS_PX: f32 = 2.0;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("touch_zone_indicator_uniform"),
            contents: bytemuck::bytes_of(&Uniforms {
                surface_size: [width as f32, height as f32],
                thickness_px: Self::THICKNESS_PX,
                _pad: 0.0,
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let _ = queue;

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("touch_zone_indicator_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Uniforms>() as u64),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("touch_zone_indicator_bg"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("touch_zone_indicator_shader"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("touch_zone_indicator.wgsl")),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("touch_zone_indicator_layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });

        let blend = wgpu::BlendState::ALPHA_BLENDING;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("touch_zone_indicator"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        TouchZoneIndicator {
            pipeline,
            bind_group,
            uniform_buf,
        }
    }

    pub fn resize(&self, queue: &wgpu::Queue, width: u32, height: u32) {
        queue.write_buffer(
            &self.uniform_buf,
            0,
            bytemuck::bytes_of(&Uniforms {
                surface_size: [width as f32, height as f32],
                thickness_px: Self::THICKNESS_PX,
                _pad: 0.0,
            }),
        );
    }

    /// Append the indicator render pass to `encoder`. Loads the existing
    /// surface contents and alpha-blends the line on top.
    pub fn render(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("touch_zone_indicator"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..4, 0..1);
    }
}
