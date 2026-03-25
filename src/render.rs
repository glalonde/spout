use crate::camera;
use crate::textured_quad;

use std::mem;
use wgpu::util::DeviceExt;

pub struct Render {
    camera: camera::Camera,
    camera_bind_group: wgpu::BindGroup,
    camera_uniform_buf: wgpu::Buffer,

    draw_pipeline: wgpu::RenderPipeline,

    model: textured_quad::TexturedQuad,
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
        input_state: &crate::input::InputState,
        prev_input_state: &crate::input::InputState,
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
        _queue: &wgpu::Queue,
        texture_view: &wgpu::TextureView,
        bloom_view: &wgpu::TextureView,
    ) -> Self {
        let visual_params = &game_params.visual_params;
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

        // Group 0: camera uniforms (ViewData = projection + view, 2 × mat4x4<f32> = 128 bytes).
        let camera_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(128),
                },
                count: None,
            }],
        });

        // Group 1: game texture (b0), sampler (b1), model-pose uniform (b2), bloom texture (b3), bloom sampler (b4).
        let texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Draw pipeline layout"),
            bind_group_layouts: &[Some(&camera_bgl), Some(&texture_bgl)],
            immediate_size: 0,
        });

        let draw_constants = [("bloom_strength", visual_params.bloom_strength as f64)];

        let draw_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("draw"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &vertex_buffers,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(config.format.into())],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &draw_constants,
                    ..Default::default()
                },
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        // Create bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buf.as_entire_binding(),
            }],
            label: None,
        });

        let textured_quad = textured_quad::TexturedQuad::init(
            device,
            texture_bgl,
            texture_view,
            bloom_view,
            game_params.viewport_width,
            game_params.viewport_height,
        );

        // Aim the camera at the quad to start with.
        Render::reset_camera(&textured_quad, &mut camera);

        Render {
            camera,
            camera_bind_group,
            camera_uniform_buf,
            draw_pipeline,

            model: textured_quad,
            staging_belt: wgpu::util::StagingBelt::new(device.clone(), 0x100),
        }
    }

    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration) {
        self.camera.screen_size = (config.width, config.height);
    }

    pub fn render(&mut self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        {
            let raw_uniforms = self.camera.to_uniform_data();
            self.staging_belt
                .write_buffer(
                    encoder,
                    &self.camera_uniform_buf,
                    0,
                    wgpu::BufferSize::new((raw_uniforms.len() * 4) as wgpu::BufferAddress).unwrap(),
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
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(clear_color),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                rpass.set_pipeline(&self.draw_pipeline);

                // Bind camera data.
                rpass.set_bind_group(0, &self.camera_bind_group, &[]);

                // Show the quad.
                self.model.render(&mut rpass);
            }
        }
    }

    pub fn after_queue_submission(&mut self) {
        self.staging_belt.recall();
    }
}
