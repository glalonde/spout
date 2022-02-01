use wgpu::util::DeviceExt;

pub struct SizedBuffer {
    pub buffer: wgpu::Buffer,
    pub size: wgpu::BufferAddress,
}

pub fn make_buffer(device: &wgpu::Device, width: usize, height: usize, label: &str) -> SizedBuffer {
    let size = (std::mem::size_of::<u32>() * width * height) as wgpu::BufferAddress;
    SizedBuffer {
        buffer: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }),
        size,
    }
}

pub fn make_uniform_buffer<T: bytemuck::Pod>(
    device: &wgpu::Device,
    label: &str,
    data: &T,
) -> SizedBuffer {
    let bytes = bytemuck::bytes_of(data);
    SizedBuffer {
        buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }),
        size: bytes.len() as _,
    }
}

pub fn make_default_uniform_buffer<T: std::default::Default + bytemuck::Pod>(
    device: &wgpu::Device,
    label: &str,
) -> SizedBuffer {
    let uniforms = T::default();
    make_uniform_buffer::<T>(device, label, &uniforms)
}
