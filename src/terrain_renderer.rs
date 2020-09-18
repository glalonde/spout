use wgpu::util::DeviceExt;
use zerocopy::AsBytes;

// Keep track of the rendering members and logic to turn the integer particle
// density texture into a colormapped texture ready to be visualized.
pub struct TerrainRenderer {
    pub render_bind_groups: std::vec::Vec<wgpu::BindGroup>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub uniform_buf: wgpu::Buffer,
}

#[repr(C)]
#[derive(Clone, Copy, zerocopy::FromBytes, zerocopy::AsBytes)]
struct FragmentUniforms {
    pub viewport_width: u32,
    pub viewport_height: u32,

    pub height_of_viewport: i32,
    pub height_of_bottom_buffer: i32,
    pub height_of_top_buffer: i32,
}

impl TerrainRenderer {
    pub fn update_render_state(
        &mut self,
        device: &wgpu::Device,
        game_params: &super::game_params::GameParams,
        level_manager: &super::level_manager::LevelManager,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let uniforms = FragmentUniforms {
            viewport_width: game_params.level_width,
            viewport_height: game_params.viewport_height,
            height_of_viewport: level_manager.height_of_viewport(),
            height_of_bottom_buffer: level_manager.buffer_height(0),
            height_of_top_buffer: level_manager.buffer_height(1),
        };
        self.set_uniforms(device, encoder, &uniforms);
    }

    fn set_uniforms(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        values: &FragmentUniforms,
    ) {
        let bytes: &[u8] = values.as_bytes();
        let uniform_buf_size = std::mem::size_of::<FragmentUniforms>();
        // TODO Can we keep a persistent staging buffer around?
        let temp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Staging buffer"),
            contents: bytes,
            usage: wgpu::BufferUsage::COPY_SRC,
        });
        encoder.copy_buffer_to_buffer(
            &temp_buf,
            0,
            &self.uniform_buf,
            0,
            uniform_buf_size as wgpu::BufferAddress,
        );
    }

    pub fn init(
        device: &wgpu::Device,
        compute_locals: &super::particle_system::ComputeLocals,
        _game_params: &super::game_params::GameParams,
        level_manager: &super::level_manager::LevelManager,
    ) -> Self {
        // Sets up the quad canvas.
        let vs = super::shader_utils::Shaders::get("particle_system/quad.vert.spv").unwrap();
        let vs_module = device.create_shader_module(wgpu::util::make_spirv(&vs));
        // Renders the data texture onto the canvas.
        let fs = super::shader_utils::Shaders::get("particle_system/terrain.frag.spv").unwrap();
        let fs_module = device.create_shader_module(wgpu::util::make_spirv(&fs));

        let fragment_uniforms = FragmentUniforms {
            viewport_width: compute_locals.game_params.viewport_width,
            viewport_height: compute_locals.game_params.viewport_height,
            height_of_viewport: 0,
            height_of_bottom_buffer: 0,
            height_of_top_buffer: 0,
        };
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: &fragment_uniforms.as_bytes(),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        // Create pipeline layout
        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // Bottom terrain buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: true,
                            // TODO find out what min_binding_size should be
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Top terrain buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: true,
                            // TODO find out what min_binding_size should be
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Uniform inputs
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            // TODO find out what min_binding_size should be
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let mut render_bind_groups = vec![];
        for config in level_manager.buffer_configurations() {
            render_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &render_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            level_manager.terrain_buffers()[config[0]].slice(..),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(
                            level_manager.terrain_buffers()[config[1]].slice(..),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
                    },
                ],
                label: None,
            }));
        }

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&render_bind_group_layout],
                label: None,
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: Some(&render_pipeline_layout),
            label: None,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        TerrainRenderer {
            render_bind_groups,
            render_pipeline,
            uniform_buf,
        }
    }

    pub fn render(
        &self,
        level_manager: &super::level_manager::LevelManager,
        output_texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // Render the density texture.
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: output_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(
            0,
            &self.render_bind_groups[level_manager.buffer_config_index()],
            &[],
        );
        rpass.draw(0..4 as u32, 0..1);
    }
}
