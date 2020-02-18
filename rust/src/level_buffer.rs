gflags::define! {
    --test_texture_file: &str = "assets/coords.png"
}
// TODO:
// orthographic projection
// movable viewport
// world space coordinates for the level canvas

use zerocopy::{AsBytes, FromBytes};

#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(unused)]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, -1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes)]
struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

fn vertex(pos: [i8; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1, -1, 0], [0, 0]),
        vertex([1, -1, 0], [1, 0]),
        vertex([1, 1, 0], [1, 1]),
        vertex([-1, 1, 0], [0, 1]),
    ];
    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
    ];
    (vertex_data.to_vec(), index_data.to_vec())
}

// Keeps track of a level including terrain and particles.
pub struct LevelBuffer {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl LevelBuffer {
    fn generate_matrix(aspect_ratio: f32) -> cgmath::Matrix4<f32> {
        let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 10.0);
        let mx_view = cgmath::Matrix4::look_at(
            cgmath::Point3::new(1.5f32, -5.0, 3.0),
            cgmath::Point3::new(0f32, 0.0, 0.0),
            cgmath::Vector3::unit_z(),
        );
        let mx_correction = OPENGL_TO_WGPU_MATRIX;
        mx_correction * mx_projection * mx_view
    }

    pub fn init(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        init_encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        // Sets up the quad canvas.
        let vs = super::include_shader!("particle_system/perspective.vert.spv");
        let vs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());
        // Renders the data texture onto the canvas.
        let fs = super::include_shader!("particle_system/perspective.frag.spv");
        let fs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());

        let texture_view =
            super::shader_utils::load_png_to_texture(device, TEST_TEXTURE_FILE.flag, init_encoder);

        // Create the vertex and index buffers
        let vertex_size = std::mem::size_of::<Vertex>();
        let (vertex_data, index_data) = create_vertices();
        let vertex_buf = device
            .create_buffer_mapped(vertex_data.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertex_data);

        let index_buf = device
            .create_buffer_mapped(index_data.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&index_data);

        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        // Create other resources
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });
        let mx_total = Self::generate_matrix(sc_desc.width as f32 / sc_desc.height as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        let uniform_buf = device
            .create_buffer_mapped(16, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
            .fill_from_slice(mx_ref);

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0..64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
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
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: vertex_size as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float4,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float2,
                        offset: 4 * 4,
                        shader_location: 1,
                    },
                ],
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        LevelBuffer {
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            bind_group,
            uniform_buf,
            pipeline,
        }
    }

    pub fn resize(
        &mut self,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mx_total = Self::generate_matrix(sc_desc.width as f32 / sc_desc.height as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        let temp_buf = device
            .create_buffer_mapped(16, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(mx_ref);
        encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.uniform_buf, 0, 64);
    }

    pub fn render(&self, frame: &wgpu::SwapChainOutput, encoder: &mut wgpu::CommandEncoder) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Load,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_index_buffer(&self.index_buf, 0);
        rpass.set_vertex_buffers(0, &[(&self.vertex_buf, 0)]);
        rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
    }
}
