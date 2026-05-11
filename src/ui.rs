//! Tiny game-space UI primitives for menu screens.
//!
//! This is deliberately smaller than a GUI framework: screens own their state
//! and actions, while this module provides shared rect math, buttons, and a
//! low-resolution rectangle renderer.

use wgpu::util::DeviceExt;

use crate::input::PointerPress;
use crate::text::YDirection;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl UiRect {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }

    pub fn centered(width: f32, height: f32, center_x: f32, center_y: f32) -> Self {
        Self {
            x: center_x - width / 2.0,
            y: center_y - height / 2.0,
            w: width,
            h: height,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UiButton<A> {
    pub action: A,
    pub label: &'static str,
    pub rect: UiRect,
}

#[derive(Debug, Clone, Copy)]
pub struct RectStyle {
    pub fill_color: [f32; 4],
    pub outline_color: [f32; 4],
    pub outline_px: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenUniform {
    size: [f32; 2],
    y_dir: f32,
    _pad: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct RectInstance {
    pos: [f32; 2],
    size: [f32; 2],
    fill_color: [f32; 4],
    outline_color: [f32; 4],
    outline_px: f32,
    _pad: [f32; 3],
}

pub struct UiRenderer {
    pipeline: wgpu::RenderPipeline,
    _screen_uniform_buf: wgpu::Buffer,
    screen_bind_group: wgpu::BindGroup,
}

impl UiRenderer {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        surface_width: u32,
        surface_height: u32,
        y_direction: YDirection,
    ) -> Self {
        let y_dir = match y_direction {
            YDirection::Down => 1.0,
            YDirection::Up => -1.0,
        };
        let screen_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Rect Screen Uniform"),
            contents: bytemuck::bytes_of(&ScreenUniform {
                size: [surface_width as f32, surface_height as f32],
                y_dir,
                _pad: 0.0,
            }),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let screen_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("UI Rect Screen BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<ScreenUniform>() as u64
                    ),
                },
                count: None,
            }],
        });

        let screen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("UI Rect Screen BG"),
            layout: &screen_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_uniform_buf.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Rect Shader"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("ui_rect.wgsl")),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Rect Pipeline Layout"),
            bind_group_layouts: &[Some(&screen_bgl)],
            immediate_size: 0,
        });

        let vertex_size = std::mem::size_of::<RectInstance>() as wgpu::BufferAddress;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Rect Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_size,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 16,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        wgpu::VertexAttribute {
                            offset: 32,
                            shader_location: 3,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        wgpu::VertexAttribute {
                            offset: 48,
                            shader_location: 4,
                            format: wgpu::VertexFormat::Float32,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        Self {
            pipeline,
            _screen_uniform_buf: screen_uniform_buf,
            screen_bind_group,
        }
    }

    pub fn draw_rects(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        rects: &[(UiRect, RectStyle)],
    ) {
        if rects.is_empty() {
            return;
        }

        let instances: Vec<RectInstance> = rects
            .iter()
            .map(|(rect, style)| RectInstance {
                pos: [rect.x.round(), rect.y.round()],
                size: [rect.w.round().max(1.0), rect.h.round().max(1.0)],
                fill_color: style.fill_color,
                outline_color: style.outline_color,
                outline_px: style.outline_px.round().max(0.0),
                _pad: [0.0; 3],
            })
            .collect();
        let instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Rect Instances"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("UI Rect Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.screen_bind_group, &[]);
        pass.set_vertex_buffer(0, instance_buf.slice(..));
        pass.draw(0..4, 0..instances.len() as u32);
    }
}

pub fn surface_to_game_point(
    point: PointerPress,
    viewport_width: u32,
    viewport_height: u32,
    surface_width: u32,
    surface_height: u32,
) -> Option<(f32, f32)> {
    if surface_width == 0 || surface_height == 0 {
        return None;
    }

    let game_w = viewport_width as f32;
    let game_h = viewport_height as f32;
    let surface_w = surface_width as f32;
    let surface_h = surface_height as f32;
    let scale = (surface_w / game_w)
        .min(surface_h / game_h)
        .floor()
        .max(1.0);
    let draw_w = game_w * scale;
    let draw_h = game_h * scale;
    let offset_x = ((surface_w - draw_w) * 0.5).floor();
    let offset_y = ((surface_h - draw_h) * 0.5).floor();

    if point.x < offset_x
        || point.x > offset_x + draw_w
        || point.y < offset_y
        || point.y > offset_y + draw_h
    {
        return None;
    }

    let game_x = (point.x - offset_x) / draw_w * game_w;
    let game_y = (point.y - offset_y) / draw_h * game_h;
    Some((game_x, game_y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains_edges() {
        let rect = UiRect {
            x: 10.0,
            y: 20.0,
            w: 30.0,
            h: 40.0,
        };

        assert!(rect.contains(10.0, 20.0));
        assert!(rect.contains(40.0, 60.0));
        assert!(!rect.contains(9.9, 20.0));
        assert!(!rect.contains(40.1, 60.0));
    }

    #[test]
    fn maps_letterboxed_surface_point_to_game_space() {
        let point = PointerPress { x: 200.0, y: 100.0 };
        let mapped = surface_to_game_point(point, 100, 50, 400, 300).expect("inside game area");

        assert_eq!(mapped, (50.0, 12.5));
        assert!(
            surface_to_game_point(PointerPress { x: 200.0, y: 20.0 }, 100, 50, 400, 300).is_none()
        );
    }

    #[test]
    fn ui_rect_renderer_constructs_headless() {
        let Some((device, _queue)) = crate::gpu_test_utils::try_create_headless_device() else {
            eprintln!("No headless GPU adapter available; skipping ui_rect_renderer_constructs");
            return;
        };

        let _renderer = UiRenderer::new(
            &device,
            crate::bloom::GAME_VIEW_FORMAT,
            64,
            32,
            YDirection::Up,
        );
    }
}
