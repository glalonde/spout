use log::info;
use zerocopy::AsBytes;

#[derive(rust_embed::RustEmbed)]
#[folder = "$OUT_DIR/shaders"]
pub struct Shaders;

pub fn create_default_texture(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    width: u32,
    height: u32,
    data: &Vec<u8>,
) -> wgpu::TextureView {
    let texture_extent = wgpu::Extent3d {
        width: width,
        height: height,
        depth: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        label: None,
    });
    let temp_buf = device.create_buffer_with_data(data.as_bytes(), wgpu::BufferUsage::COPY_SRC);
    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &temp_buf,
            offset: 0,
            bytes_per_row: 4 * width,
            rows_per_image: height,
        },
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        texture_extent,
    );
    texture.create_default_view()
}

// Read an image file, create a command to copy it to a texture, and return a
// view.
pub fn load_png_to_texture(
    device: &wgpu::Device,
    path: &str,
    encoder: &mut wgpu::CommandEncoder,
) -> wgpu::TextureView {
    let dyn_image = image::open(path).unwrap();
    let image = dyn_image.to_rgba();
    let width = image.width();
    let height = image.height();
    info!(
        "Loading image with (width, height) = ({}, {})",
        width, height
    );
    let data = image.into_raw();
    create_default_texture(device, encoder, width, height, &data)
}

// Include precompiled shader bytes by specifying a path relative to the shader
// source directory.
#[macro_export]
macro_rules! include_shader {
    ( $shader_name:expr ) => {
        super::shader_utils::Shaders::get($shader_name).unwrap();
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        for entry in Shaders::iter() {
            println!("Found shader: {:?}", entry);
        }
        let _test_bytes = Shaders::get("collatz.comp.spv").unwrap();
    }
}
