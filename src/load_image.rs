
// Loads a fixed demo texture to show UV coordinates.
pub fn load_image_to_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Result<wgpu::TextureView, image::ImageError> {
    let image = image::load_from_memory(include_bytes!("../assets/texture_coordinates2.png"))?;
    let image = image.into_bgra8();
    let texture_extent = wgpu::Extent3d {
        width: image.width(),
        height: image.height(),
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: None,
    });

    let nonzero_width = core::num::NonZeroU32::new(4 * image.width());
    if let Some(nonzero_width) = nonzero_width {
        let data_layout = wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(nonzero_width),
            rows_per_image: None,
        };
        queue.write_texture(
            texture.as_image_copy(),
            &image.to_vec(),
            data_layout,
            texture_extent,
        );
    }
    Ok(texture.create_view(&wgpu::TextureViewDescriptor::default()))
}
