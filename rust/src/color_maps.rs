use gflags::custom::{Arg, Error, Result, Value};

gflags::define! {
    --color_map: ColorMap = ColorMap::Inferno
}

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

impl Value for ColorMap {
    fn parse(arg: Arg) -> Result<Self> {
        match arg.get_str() {
            "viridis" => Ok(ColorMap::Viridis),
            "magma" => Ok(ColorMap::Magma),
            "inferno" => Ok(ColorMap::Inferno),
            "plasma" => Ok(ColorMap::Plasma),
            _ => Err(Error::new("Invalid ColorMap")),
        }
    }
}

pub fn get_color_map_from_flag() -> &'static scarlet::colormap::ListedColorMap {
    &COLOR_MAPS[COLOR_MAP.flag as usize]
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
        depth: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D1,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED
            | wgpu::TextureUsage::COPY_DST
            | wgpu::TextureUsage::COPY_SRC,
    });
    let temp_buf = device
        .create_buffer_mapped(
            data.len(),
            wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
        )
        .fill_from_slice(&data);
    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &temp_buf,
            offset: 0,
            row_pitch: 4 * size,
            image_height: 1,
        },
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
        texture_extent,
    );

    texture
}
