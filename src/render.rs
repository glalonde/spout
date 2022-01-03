use crate::camera;
use crate::textured_quad;

use std::{borrow::Cow, mem};
use wgpu::util::DeviceExt;

pub struct Render {
    camera: camera::Camera,
    camera_bind_group: wgpu::BindGroup,
    camera_uniform_buf: wgpu::Buffer,

    draw_pipeline: wgpu::RenderPipeline,

    model: textured_quad::TexturedQuad,

    frame_num: i64,
    staging_belt: wgpu::util::StagingBelt,
}

impl Render {
    pub fn update_state(&mut self, dt: f32, input_state: &crate::InputState) {
        self.camera.update_state(dt, input_state);
    }

    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let camera = camera::Camera {
            motion_params: camera::CameraMotion {
                angular_speed: 1.0,
                vertical_speed: 1.0,
            },
            screen_size: (config.width, config.height),
            radius: 5.0,
            phi: 0.0,
            height: 3.0,
        };
        let raw_uniforms = camera.to_uniform_data();
        let camera_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&raw_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create the vertex and index buffers
        let vertex_size = mem::size_of::<textured_quad::Vertex>();
        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Vertex position.
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                // Texture position.
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];

        // Create the render pipeline
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("textured_model.wgsl"))),
        });

        let draw_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("draw"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[config.format.into()],
            }),
            primitive: wgpu::PrimitiveState {
                // cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Create bind group
        let camera_bind_group_layout = draw_pipeline.get_bind_group_layout(0);
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buf.as_entire_binding(),
            }],
            label: None,
        });

        let textured_quad = textured_quad::TexturedQuad::init(
            device,
            draw_pipeline.get_bind_group_layout(1),
            &mut init_encoder,
        );

        queue.submit(Some(init_encoder.finish()));

        Render {
            camera,
            camera_bind_group,
            camera_uniform_buf,
            draw_pipeline,
            model: textured_quad,

            frame_num: 0,
            staging_belt: wgpu::util::StagingBelt::new(0x100),
        }
    }

    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration) {
        self.camera.screen_size = (config.width, config.height);
    }

    pub fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let raw_uniforms = self.camera.to_uniform_data();
            self.staging_belt
                .write_buffer(
                    &mut encoder,
                    &self.camera_uniform_buf,
                    0,
                    wgpu::BufferSize::new((raw_uniforms.len() * 4) as wgpu::BufferAddress).unwrap(),
                    device,
                )
                .copy_from_slice(bytemuck::cast_slice(&raw_uniforms));

            self.staging_belt.finish();
        }

        {
            let clear_color = wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            };
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(clear_color),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });
                rpass.set_pipeline(&self.draw_pipeline);

                // Bind camera data.
                rpass.set_bind_group(0, &self.camera_bind_group, &[]);

                self.model.render(&mut rpass);
            }
        }

        self.frame_num += 1;

        queue.submit(Some(encoder.finish()));
    }
}
