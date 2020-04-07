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
        let temp_buf = device.create_buffer_with_data(bytes, wgpu::BufferUsage::COPY_SRC);
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
        game_params: &super::game_params::GameParams,
        level_manager: &super::level_manager::LevelManager,
    ) -> Self {
        // Sets up the quad canvas.
        let vs = super::shader_utils::Shaders::get("particle_system/quad.vert.spv").unwrap();
        let vs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());
        // Renders the data texture onto the canvas.
        let fs = super::shader_utils::Shaders::get("particle_system/terrain.frag.spv").unwrap();
        let fs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());

        let fragment_uniform_size = std::mem::size_of::<FragmentUniforms>() as wgpu::BufferAddress;
        let fragment_uniforms = FragmentUniforms {
            viewport_width: compute_locals.system_params.width,
            viewport_height: compute_locals.system_params.height,
            height_of_viewport: 0,
            height_of_bottom_buffer: 0,
            height_of_top_buffer: 0,
        };
        let uniform_buf = device.create_buffer_with_data(
            &fragment_uniforms.as_bytes(),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        // Create pipeline layout
        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    // Bottom terrain buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: true,
                        },
                    },
                    // Top terrain buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: true,
                        },
                    },
                    // Uniform inputs
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                ],
                label: None,
            });

        let mut render_bind_groups = vec![];
        let terrain_buffer_size = level_manager.terrain_buffer_size();
        for config in level_manager.buffer_configurations() {
            render_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &render_bind_group_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &level_manager.terrain_buffers()[config[0]],
                            range: 0..terrain_buffer_size as wgpu::BufferAddress,
                        },
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &level_manager.terrain_buffers()[config[1]],
                            range: 0..terrain_buffer_size as wgpu::BufferAddress,
                        },
                    },
                    wgpu::Binding {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &uniform_buf,
                            range: 0..fragment_uniform_size,
                        },
                    },
                ],
                label: None,
            }));
        }

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&render_bind_group_layout],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
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
                load_op: wgpu::LoadOp::Load,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::BLACK,
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
