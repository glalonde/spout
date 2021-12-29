use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

fn vertex(pos: [i8; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

fn create_texels(size: usize, cx: f32, cy: f32) -> Vec<u8> {
    use std::iter;

    (0..size * size)
        .flat_map(|id| {
            // get high five for recognizing this ;)
            let mut x = 4.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let mut y = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let mut count = 0;
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            iter::once(0xFF - (count * 2) as u8)
                .chain(iter::once(0xFF - (count * 5) as u8))
                .chain(iter::once(0xFF - (count * 13) as u8))
                .chain(iter::once(std::u8::MAX))
        })
        .collect()
}

fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1, -1, 0], [0, 1]),
        vertex([1, -1, 0], [1, 1]),
        vertex([1, 1, 0], [1, 0]),
        vertex([-1, 1, 0], [0, 0]),
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
}

impl TexturedQuad {
    pub fn init(
        device: &wgpu::Device,
        bind_group_layout: wgpu::BindGroupLayout,
        init_encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        // Create the texture
        let size: u32 = 1 << 9;
        let texels = create_texels(size as usize, -0.8, 0.156);
        let texture_extent = wgpu::Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: None,
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        //Note: we could use queue.write_texture instead, and this is what other
        // examples do, but here we want to show another way to do this.
        let temp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Temporary Buffer"),
            contents: texels.as_slice(),
            usage: wgpu::BufferUsages::COPY_SRC,
        });
        init_encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &temp_buf,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(std::num::NonZeroU32::new(4 * size).unwrap()),
                    rows_per_image: None,
                },
            },
            texture.as_image_copy(),
            texture_extent,
        );

        // Create the vertex and index buffers
        let (vertex_data, index_data) = create_vertices();
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
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Model pose uniform
        let model_pose = cgmath::Matrix4::from_translation(cgmath::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
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
                    resource: wgpu::BindingResource::TextureView(&texture_view),
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
