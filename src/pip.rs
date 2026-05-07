//! Picture-in-Picture overlay: shows a miniature ship + glowing frame at the
//! bottom of the game view when the ship has scrolled outside the visible area.

use crate::{bloom, buffer_util::SizedBuffer, game_params};

/// Half-width / half-height of the panel in game-view pixels.
pub const PIP_HW: f32 = 15.0;
pub const PIP_HH: f32 = 11.0;
/// Fixed Y coordinate of the panel centre (game-view, Y-up).
pub const PIP_CENTER_Y: f32 = 14.0;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PipUniforms {
    pip_center: [f32; 2],
    ship_orientation: f32,
    viewport_width: u32,
    viewport_height: u32,
    _pad: u32,
}

pub struct PipRenderer {
    uniform_buffer: SizedBuffer,
    bind_group: wgpu::BindGroup,
    bg_pipeline: wgpu::RenderPipeline,
    brackets_pipeline: wgpu::RenderPipeline,
    ship_pipeline: wgpu::RenderPipeline,
    outline_pipeline: wgpu::RenderPipeline,
    flame_pipeline: wgpu::RenderPipeline,
}

impl PipRenderer {
    pub fn init(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("pip_shader"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("pip.wgsl")),
        });

        let init_uniforms = PipUniforms {
            pip_center: [0.0, PIP_CENTER_Y],
            ship_orientation: 0.0,
            viewport_width: 0,
            viewport_height: 0,
            _pad: 0,
        };
        let uniform_buffer = crate::buffer_util::make_uniform_buffer(
            device,
            "PIP Uniform Buffer",
            &init_uniforms,
        );

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("PIP BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(uniform_buffer.size as _),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PIP BG"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.buffer.as_entire_binding(),
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PIP pipeline layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });

        let target = wgpu::ColorTargetState {
            format: bloom::GAME_VIEW_FORMAT,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let bg_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pip_bg"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_bg"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_bg"),
                targets: &[Some(target.clone())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        let brackets_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pip_brackets"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_brackets"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_brackets"),
                targets: &[Some(target.clone())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        let ship_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pip_ship"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_ship"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_ship"),
                targets: &[Some(target.clone())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        let outline_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pip_outline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_outline"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_outline"),
                targets: &[Some(target.clone())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        let flame_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pip_flame"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_flame"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_flame"),
                targets: &[Some(target)],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        PipRenderer {
            uniform_buffer,
            bind_group,
            bg_pipeline,
            brackets_pipeline,
            ship_pipeline,
            outline_pipeline,
            flame_pipeline,
        }
    }

    /// Returns true when the ship's Y world coordinate is outside the visible viewport.
    pub fn is_ship_offscreen(ship_y: f32, viewport_offset: i32, viewport_height: u32) -> bool {
        let vp_bot = viewport_offset as f32;
        let vp_top = vp_bot + viewport_height as f32;
        ship_y < vp_bot || ship_y > vp_top
    }

    /// The horizontal centre of the PIP panel, clamped so the box stays on screen.
    pub fn pip_center_x(ship_x: f32, viewport_width: u32) -> f32 {
        let margin = PIP_HW + 2.0;
        ship_x.clamp(margin, viewport_width as f32 - margin)
    }

    // All arguments are distinct GPU/game concerns; grouping them would not
    // simplify call sites.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        ship_pos: [f32; 2],
        ship_orientation: f32,
        is_thrusting: bool,
        game_params: &game_params::GameParams,
        output: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
    ) {
        let pip_cx = Self::pip_center_x(ship_pos[0], game_params.viewport_width);

        let uniforms = PipUniforms {
            pip_center: [pip_cx, PIP_CENTER_Y],
            ship_orientation,
            viewport_width: game_params.viewport_width,
            viewport_height: game_params.viewport_height,
            _pad: 0,
        };
        belt.write_buffer(
            encoder,
            &self.uniform_buffer.buffer,
            0,
            // safe: buffer was created with this exact size
            wgpu::BufferSize::new(self.uniform_buffer.size as _).unwrap(),
        )
        .copy_from_slice(bytemuck::bytes_of(&uniforms));

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("pip"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        rpass.set_bind_group(0, &self.bind_group, &[]);

        rpass.set_pipeline(&self.bg_pipeline);
        rpass.draw(0..6, 0..1);

        rpass.set_pipeline(&self.brackets_pipeline);
        rpass.draw(0..16, 0..1);

        // Flame behind the ship — drawn before the ship fill so the hull occludes the base.
        if is_thrusting {
            rpass.set_pipeline(&self.flame_pipeline);
            rpass.draw(0..3, 0..1);
        }

        rpass.set_pipeline(&self.ship_pipeline);
        rpass.draw(0..6, 0..1);

        rpass.set_pipeline(&self.outline_pipeline);
        rpass.draw(0..5, 0..1);
    }
}
