use crate::camera;
use crate::textured_quad;

use std::mem;
use wgpu::util::DeviceExt;

pub struct Render {
    camera: camera::Camera,
    camera_bind_group: wgpu::BindGroup,
    camera_uniform_buf: wgpu::Buffer,

    draw_pipeline: wgpu::RenderPipeline,

    pub show_demo_texture: bool,
    model: textured_quad::TexturedQuad,
    demo_model: Option<textured_quad::TexturedQuad>,

    frame_num: i64,
    staging_belt: wgpu::util::StagingBelt,
}

impl Render {
    pub fn reset_camera(
        target: &crate::textured_quad::TexturedQuad,
        camera: &mut crate::camera::Camera,
    ) {
        let target_width: f32 = target.width as _;
        let target_height: f32 = target.height as _;
        let center_point = [target_width / 2.0, target_height / 2.0];
        camera.ortho_look_at(center_point, target_width, target_height, true);
    }

    pub fn update_state(
        &mut self,
        dt: f32,
        input_state: &crate::InputState,
        prev_input_state: &crate::InputState,
    ) {
        let target_width: f32 = self.model.width as _;
        let target_height: f32 = self.model.height as _;
        self.camera.update_state(dt, input_state);
        if input_state.cam_perspective && !prev_input_state.cam_perspective {
            let center_point = [target_width / 2.0, target_height / 2.0];
            // Toggle cam perspective:
            if self.camera.state.ortho.is_none() {
                self.camera
                    .ortho_look_at(center_point, target_width, target_height, false);
            } else {
                self.camera
                    .perspective_look_at(center_point, target_width, target_height, false);
            }
        }

        if input_state.cam_reset {
            Render::reset_camera(&self.model, &mut self.camera);
        }
    }

    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        game_params: &crate::game_params::GameParams,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_view: &wgpu::TextureView,
    ) -> Self {
        let mut camera = camera::Camera {
            screen_size: (config.width, config.height),
            ..Default::default()
        };
        let raw_uniforms = camera.to_uniform_data();
        let camera_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
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
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("textured_model.wgsl")),
        });

        let draw_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("draw"),
            // TODO convert this to explicit pipeline layout.
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
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
            texture_view,
            game_params.viewport_width,
            game_params.viewport_height,
        );

        let maybe_demo_texture = crate::load_image::load_image_to_texture(device, queue);
        let maybe_demo_quad = match maybe_demo_texture {
            Ok(demo_texture) => Some(textured_quad::TexturedQuad::init(
                device,
                draw_pipeline.get_bind_group_layout(1),
                &demo_texture,
                game_params.viewport_width,
                game_params.viewport_height,
            )),
            Err(e) => {
                log::error!("Couldn't load demo texture: {:?}", e);
                None
            }
        };

        // Aim the camera at the quad to start with.
        Render::reset_camera(&textured_quad, &mut camera);

        Render {
            camera,
            camera_bind_group,
            camera_uniform_buf,
            draw_pipeline,

            show_demo_texture: false,
            model: textured_quad,
            demo_model: maybe_demo_quad,

            frame_num: 0,
            staging_belt: wgpu::util::StagingBelt::new(0x100),
        }
    }

    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration) {
        self.camera.screen_size = (config.width, config.height);
    }

    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        {
            let raw_uniforms = self.camera.to_uniform_data();
            self.staging_belt
                .write_buffer(
                    encoder,
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
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            };
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(clear_color),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });
                rpass.set_pipeline(&self.draw_pipeline);

                // Bind camera data.
                rpass.set_bind_group(0, &self.camera_bind_group, &[]);

                // Show the quad.
                if !self.show_demo_texture {
                    self.model.render(&mut rpass);
                } else {
                    if let Some(quad) = &self.demo_model {
                        quad.render(&mut rpass);
                    } else {
                        self.model.render(&mut rpass);
                    }
                }
            }
        }

        self.frame_num += 1;
    }

    pub fn after_queue_submission(&mut self) {
        self.staging_belt.recall();
    }
}
