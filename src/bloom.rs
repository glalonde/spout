use wgpu::util::DeviceExt;

/// Intermediate texture format used for bloom ping-pong buffers.
pub const BLOOM_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// HDR format used for the game view texture (terrain + particles + ship render into this).
/// Float allows bloom to add values back without clipping.
pub const GAME_VIEW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// Threshold + separable Gaussian bloom pass.
///
/// Pipeline:
///   game_view → [threshold] → bright_tex
///   bright_tex → [h_blur]   → blur_tex
///   blur_tex   → [v_blur]   → bright_tex  (final bloom output)
///
/// `bloom_view()` exposes the final bloom result for compositing in the blit pass.
pub struct Bloom {
    threshold_pipeline: wgpu::RenderPipeline,
    threshold_bind_group: wgpu::BindGroup,

    blur_pipeline: wgpu::RenderPipeline,
    /// bright_tex → blur_tex (horizontal pass)
    h_blur_bind_group: wgpu::BindGroup,
    /// blur_tex → bright_tex (vertical pass)
    v_blur_bind_group: wgpu::BindGroup,

    /// Receives threshold output; also the final bloom result after v_blur overwrites it.
    _bright_texture: wgpu::Texture,
    bright_view: wgpu::TextureView,

    /// Intermediate buffer for horizontal blur output.
    _blur_texture: wgpu::Texture,
    blur_view: wgpu::TextureView,
}

impl Bloom {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        game_view: &wgpu::TextureView,
    ) -> Self {
        let make_tex = |label| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: BLOOM_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
        };
        let bright_texture = make_tex("bloom_bright");
        let bright_view = bright_texture.create_view(&Default::default());
        let blur_texture = make_tex("bloom_blur");
        let blur_view = blur_texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("bloom_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        // --- Threshold pipeline ---
        let threshold_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom_threshold_bgl"),
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
            ],
        });
        let threshold_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom_threshold_bg"),
            layout: &threshold_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(game_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let threshold_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom_threshold_shader"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("bloom_threshold.wgsl")),
        });
        let threshold_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bloom_threshold_layout"),
            bind_group_layouts: &[Some(&threshold_bgl)],
            immediate_size: 0,
        });
        let threshold_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bloom_threshold"),
            layout: Some(&threshold_layout),
            vertex: wgpu::VertexState {
                module: &threshold_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &threshold_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: BLOOM_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        // --- Blur pipeline ---
        let blur_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom_blur_bgl"),
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
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(16),
                    },
                    count: None,
                },
            ],
        });

        // Direction uniforms: step in UV space (2 texels wide for a more visible radius).
        let step_scale = 2.0_f32;
        let h_dir: [f32; 4] = [step_scale / width as f32, 0.0, 0.0, 0.0];
        let v_dir: [f32; 4] = [0.0, step_scale / height as f32, 0.0, 0.0];

        let h_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bloom_h_uniform"),
            contents: bytemuck::cast_slice(&h_dir),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let v_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bloom_v_uniform"),
            contents: bytemuck::cast_slice(&v_dir),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let h_blur_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom_h_blur_bg"),
            layout: &blur_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bright_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: h_uniform.as_entire_binding(),
                },
            ],
        });
        let v_blur_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom_v_blur_bg"),
            layout: &blur_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: v_uniform.as_entire_binding(),
                },
            ],
        });

        let blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom_blur_shader"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("bloom_blur.wgsl")),
        });
        let blur_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bloom_blur_layout"),
            bind_group_layouts: &[Some(&blur_bgl)],
            immediate_size: 0,
        });
        let blur_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bloom_blur"),
            layout: Some(&blur_layout),
            vertex: wgpu::VertexState {
                module: &blur_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &blur_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: BLOOM_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        Bloom {
            threshold_pipeline,
            threshold_bind_group,
            blur_pipeline,
            h_blur_bind_group,
            v_blur_bind_group,
            _bright_texture: bright_texture,
            bright_view,
            _blur_texture: blur_texture,
            blur_view,
        }
    }

    /// Run all bloom passes. Call after all game renders (terrain, particles, ship) are done,
    /// and before the final blit.
    pub fn render(&self, encoder: &mut wgpu::CommandEncoder) {
        let clear = wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
        };

        // Pass 1: threshold — game_view → bright_tex
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom_threshold"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.bright_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: clear,
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.threshold_pipeline);
            pass.set_bind_group(0, &self.threshold_bind_group, &[]);
            pass.draw(0..4, 0..1);
        }

        // Pass 2: horizontal blur — bright_tex → blur_tex
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom_h_blur"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.blur_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: clear,
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.blur_pipeline);
            pass.set_bind_group(0, &self.h_blur_bind_group, &[]);
            pass.draw(0..4, 0..1);
        }

        // Pass 3: vertical blur — blur_tex → bright_tex (final bloom output)
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom_v_blur"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.bright_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: clear,
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.blur_pipeline);
            pass.set_bind_group(0, &self.v_blur_bind_group, &[]);
            pass.draw(0..4, 0..1);
        }
    }

    /// The final bloom texture, ready to be additively composited onto the game view.
    pub fn bloom_view(&self) -> &wgpu::TextureView {
        &self.bright_view
    }
}
