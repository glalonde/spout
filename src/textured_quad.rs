use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

fn vertex(pos: [u32; 3], tc: [u32; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

fn create_vertices(width: u32, height: u32) -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // Bottom left on the quad. We want this to be the same as the "top left" in UV coordinates.
        vertex([0, 0, 0], [0, 0]),
        // Bottom right on the quad. We want this to be the same as the "top right" in UV coordinates.
        vertex([width, 0, 0], [1, 0]),
        // Top right on the quad. We want this to be the same as the "bottom right" in UV coordinates.
        vertex([width, height, 0], [1, 1]),
        // Top left on the quad. We want this to be the same as the "bottom left" in UV coordinates.
        vertex([0, height, 0], [0, 1]),
    ];
    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
    ];
    (vertex_data.to_vec(), index_data.to_vec())
}

pub struct TexturedQuad {
    // Geometry for the canvas.
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    bind_group: wgpu::BindGroup,

    pub width: u32,
    pub height: u32,
}

impl TexturedQuad {
    pub fn init(
        device: &wgpu::Device,
        bind_group_layout: wgpu::BindGroupLayout,
        texture_view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Self {
        // Create the vertex and index buffers
        let (vertex_data, index_data) = create_vertices(width, height);
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create other resources
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Model pose uniform
        let model_pose = cgmath::Matrix4::from_translation(cgmath::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        let pose_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&AsRef::<[f32; 16]>::as_ref(&model_pose)[..]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group for the non-camera inputs.
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: pose_uniform_buf.as_entire_binding(),
                },
            ],
            label: None,
        });

        TexturedQuad {
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            bind_group,
            width,
            height,
        }
    }

    pub fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        // Assume camera bind group (0) has already been set by outer context.

        // Set the texture data
        rpass.set_bind_group(1, &self.bind_group, &[]);

        // Set the mesh data
        rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
        rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));

        // Pose the geometry pose data
        // rpass.set_bind_group(2, ...)

        rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
    }
}
