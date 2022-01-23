use wgpu::util::DeviceExt;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Copy, Clone)]
enum ColorMap {
    Viridis = 0,
    Magma = 1,
    Inferno = 2,
    Plasma = 3,
}

use lazy_static::lazy_static;
lazy_static! {
    static ref COLOR_MAPS: [scarlet::colormap::ListedColorMap; 4] = [
        scarlet::colormap::ListedColorMap::viridis(),
        scarlet::colormap::ListedColorMap::magma(),
        scarlet::colormap::ListedColorMap::inferno(),
        scarlet::colormap::ListedColorMap::plasma(),
    ];
}
pub fn get_color_map_from_index(i: usize) -> &'static scarlet::colormap::ListedColorMap {
    &COLOR_MAPS[i]
}

// Create a particle density color map rgba
// Rust image defaults to row major.
pub fn create_color_map(
    size: u32,
    device: &wgpu::Device,
    cm: &scarlet::colormap::ListedColorMap,
    encoder: &mut wgpu::CommandEncoder,
) -> wgpu::Texture {
    let im = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_fn(size, 1, |x, _y| {
        let parameter = x as f64 / (size - 1) as f64;
        let color_point: scarlet::color::RGBColor =
            scarlet::colormap::ColorMap::transform_single(cm, parameter);
        image::Rgba([
            color_point.int_r(),
            color_point.int_g(),
            color_point.int_b(),
            255,
        ])
    });
    let data = im.into_raw();
    let texture_extent = wgpu::Extent3d {
        width: size,
        height: 1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        // TODO change to 1d texture when supported by Dawn:
        // https://bugs.chromium.org/p/dawn/issues/detail?id=814
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        label: None,
    });

    // Note: we could use queue.write_texture instead, and this is what other
    // examples do, but here we want to show another way to do this.
    let temp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Temporary Buffer"),
        contents: data.as_slice(),
        usage: wgpu::BufferUsages::COPY_SRC,
    });
    encoder.copy_buffer_to_texture(
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

    texture
}
