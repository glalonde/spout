//! Top-level render pipeline: blit game view to display resolution, apply CRT
//! post-processing, and composite bloom into the final LDR surface output.

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

    // Composite pass: upscaled HDR + bloom → surface (LDR).
    composite_bgl: wgpu::BindGroupLayout,
    composite_pipeline: wgpu::RenderPipeline,
    composite_bind_group: wgpu::BindGroup,
}

fn make_composite_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    upscaled_view: &wgpu::TextureView,
    bloom_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("composite_sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("composite_bind_group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(upscaled_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(bloom_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    })
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

    // Three texture views are genuinely separate concerns; wrapping them in a struct
    // would not simplify call sites meaningfully.
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        game_params: &crate::game_params::GameParams,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        texture_view: &wgpu::TextureView,  // game view (blit source)
        upscaled_view: &wgpu::TextureView, // blit target = composite hdr source
        bloom_view: &wgpu::TextureView,    // composite bloom source
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

        // Create the blit render pipeline (game_view → upscaled_hdr, camera transform).
        let blit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
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

        // Group 1: game texture (b0), sampler (b1), model-pose uniform (b2).
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
            ],
        });

        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit pipeline layout"),
            bind_group_layouts: &[Some(&camera_bgl), Some(&texture_bgl)],
            immediate_size: 0,
        });

        // Blit pipeline writes to the upscaled HDR texture (Rgba16Float), not the surface.
        let draw_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("draw"),
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: Some("vs_main"),
                buffers: &vertex_buffers,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(crate::bloom::GAME_VIEW_FORMAT.into())],
                compilation_options: Default::default(),
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
            game_params.viewport_width,
            game_params.viewport_height,
        );

        // Aim the camera at the quad to start with.
        Render::reset_camera(&textured_quad, &mut camera);

        // --- Composite pipeline (upscaled HDR + bloom → surface LDR) ---
        let composite_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Composite BGL"),
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
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let composite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom_composite_shader"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("bloom_composite.wgsl")),
        });

        let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Composite pipeline layout"),
            bind_group_layouts: &[Some(&composite_bgl)],
            immediate_size: 0,
        });

        let composite_constants = [
            ("bloom_strength", visual_params.bloom_strength as f64),
            ("crt_strength", visual_params.crt_strength as f64),
        ];

        let composite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("composite"),
            layout: Some(&composite_layout),
            vertex: wgpu::VertexState {
                module: &composite_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &composite_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(config.format.into())],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &composite_constants,
                    ..Default::default()
                },
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

        let composite_bind_group =
            make_composite_bind_group(device, &composite_bgl, upscaled_view, bloom_view);

        Render {
            camera,
            camera_bind_group,
            camera_uniform_buf,
            draw_pipeline,
            model: textured_quad,
            composite_bgl,
            composite_pipeline,
            composite_bind_group,
        }
    }

    pub fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        upscaled_view: &wgpu::TextureView,
        bloom_view: &wgpu::TextureView,
    ) {
        self.camera.screen_size = (config.width, config.height);
        self.composite_bind_group =
            make_composite_bind_group(device, &self.composite_bgl, upscaled_view, bloom_view);
    }

    /// Blit the game view (240×135) into the upscaled HDR texture at surface resolution.
    pub fn blit(
        &mut self,
        upscaled_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
    ) {
        {
            let raw_uniforms = self.camera.to_uniform_data();
            belt.write_buffer(
                encoder,
                &self.camera_uniform_buf,
                0,
                // safe: raw_uniforms always has > 0 elements (camera matrix)
                wgpu::BufferSize::new((raw_uniforms.len() * 4) as wgpu::BufferAddress).unwrap(),
            )
            .copy_from_slice(bytemuck::cast_slice(&raw_uniforms));
        }

        {
            let clear_color = wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            };
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: upscaled_view,
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
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            self.model.render(&mut rpass);
        }
    }

    /// Composite the upscaled HDR + bloom textures onto the surface (LDR output).
    pub fn render(&mut self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let clear_color = wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("composite"),
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
        rpass.set_pipeline(&self.composite_pipeline);
        rpass.set_bind_group(0, &self.composite_bind_group, &[]);
        rpass.draw(0..4, 0..1);
    }
}
