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
