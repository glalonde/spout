//! Particle color map palettes (Viridis, Magma, Inferno, Plasma) backed by
//! the `scarlet` crate. Generates 1D GPU textures for the particle shader.

use wgpu::util::DeviceExt;

/// Named color map palettes. The discriminant matches the index into the
/// internal palette array, so `ColorMap::Magma as usize` works as expected.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ColorMap {
    Viridis = 0,
    Magma = 1,
    Inferno = 2,
    Plasma = 3,
    Parula = 4,
}

use std::sync::OnceLock;

/// MATLAB Parula colormap — 64 evenly-spaced RGB control points (0.0–1.0).
/// Source: <https://github.com/BIDS/colormap/blob/master/parula.py>
#[rustfmt::skip]
const PARULA_DATA: [[f64; 3]; 64] = [
    [0.2081, 0.1663, 0.5292], [0.2116, 0.1898, 0.5777],
    [0.2123, 0.2138, 0.6270], [0.2081, 0.2386, 0.6771],
    [0.1959, 0.2645, 0.7279], [0.1707, 0.2919, 0.7792],
    [0.1253, 0.3242, 0.8303], [0.0591, 0.3598, 0.8683],
    [0.0117, 0.3875, 0.8820], [0.0060, 0.4086, 0.8828],
    [0.0165, 0.4266, 0.8786], [0.0329, 0.4430, 0.8720],
    [0.0498, 0.4586, 0.8641], [0.0629, 0.4737, 0.8554],
    [0.0723, 0.4887, 0.8467], [0.0779, 0.5040, 0.8384],
    [0.0793, 0.5200, 0.8312], [0.0749, 0.5375, 0.8263],
    [0.0641, 0.5570, 0.8240], [0.0488, 0.5772, 0.8228],
    [0.0343, 0.5966, 0.8199], [0.0265, 0.6137, 0.8135],
    [0.0239, 0.6287, 0.8038], [0.0231, 0.6418, 0.7913],
    [0.0228, 0.6535, 0.7768], [0.0267, 0.6642, 0.7607],
    [0.0384, 0.6743, 0.7436], [0.0590, 0.6838, 0.7254],
    [0.0843, 0.6928, 0.7062], [0.1133, 0.7015, 0.6859],
    [0.1453, 0.7098, 0.6646], [0.1801, 0.7177, 0.6424],
    [0.2178, 0.7250, 0.6193], [0.2586, 0.7317, 0.5954],
    [0.3022, 0.7376, 0.5712], [0.3482, 0.7424, 0.5473],
    [0.3953, 0.7459, 0.5244], [0.4420, 0.7481, 0.5033],
    [0.4871, 0.7491, 0.4840], [0.5300, 0.7491, 0.4661],
    [0.5709, 0.7485, 0.4494], [0.6099, 0.7473, 0.4337],
    [0.6473, 0.7456, 0.4188], [0.6834, 0.7435, 0.4044],
    [0.7184, 0.7411, 0.3905], [0.7525, 0.7384, 0.3768],
    [0.7858, 0.7356, 0.3633], [0.8185, 0.7327, 0.3498],
    [0.8507, 0.7299, 0.3360], [0.8824, 0.7274, 0.3217],
    [0.9139, 0.7258, 0.3063], [0.9450, 0.7261, 0.2886],
    [0.9739, 0.7314, 0.2666], [0.9938, 0.7455, 0.2403],
    [0.9990, 0.7653, 0.2164], [0.9955, 0.7861, 0.1967],
    [0.9880, 0.8066, 0.1794], [0.9789, 0.8271, 0.1633],
    [0.9697, 0.8481, 0.1475], [0.9626, 0.8705, 0.1309],
    [0.9589, 0.8949, 0.1132], [0.9598, 0.9218, 0.0948],
    [0.9661, 0.9514, 0.0755], [0.9763, 0.9831, 0.0538],
];

fn parula() -> scarlet::colormap::ListedColorMap {
    scarlet::colormap::ListedColorMap::new(PARULA_DATA.iter().copied())
}

static COLOR_MAPS: OnceLock<[scarlet::colormap::ListedColorMap; 5]> = OnceLock::new();

fn color_maps() -> &'static [scarlet::colormap::ListedColorMap; 5] {
    COLOR_MAPS.get_or_init(|| {
        [
            scarlet::colormap::ListedColorMap::viridis(),
            scarlet::colormap::ListedColorMap::magma(),
            scarlet::colormap::ListedColorMap::inferno(),
            scarlet::colormap::ListedColorMap::plasma(),
            parula(),
        ]
    })
}

pub fn get_color_map_from_index(i: usize) -> &'static scarlet::colormap::ListedColorMap {
    &color_maps()[i]
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
        view_formats: &[],
    });

    // Note: we could use queue.write_texture instead, and this is what other
    // examples do, but here we want to show another way to do this.
    let temp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Temporary Buffer"),
        contents: data.as_slice(),
        usage: wgpu::BufferUsages::COPY_SRC,
    });
    encoder.copy_buffer_to_texture(
        wgpu::TexelCopyBufferInfo {
            buffer: &temp_buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size),
                rows_per_image: None,
            },
        },
        texture.as_image_copy(),
        texture_extent,
    );

    texture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_color_maps_accessible() {
        for i in 0..5 {
            let _cm = get_color_map_from_index(i);
        }
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_panics() {
        let _cm = get_color_map_from_index(5);
    }
}
