use lazy_static::lazy_static;
use log::info;
// Input path in the source tree, and also the output path in the output
// directory. This needs to match the path in build.rs
// TODO(glalonde) Factor this into a library.
static SHADER_PATH: &str = "shaders";
lazy_static! {
    pub static ref SHADER_OUTPUT_DIR: std::path::PathBuf =
        std::path::Path::new(env!("OUT_DIR")).join(std::path::Path::new(SHADER_PATH));
}
pub fn list_shaders() {
    // Tell the build script to only run again if we change our source shaders.
    // Unfortunately, if a single shader changes, it recompiles everything.
    for entry in walkdir::WalkDir::new(SHADER_OUTPUT_DIR.as_path())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        info!("Found shader: {}", entry.path().display());
    }
}

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
    });
    let temp_buf = device
        .create_buffer_mapped(data.len(), wgpu::BufferUsage::COPY_SRC)
        .fill_from_slice(&data);
    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &temp_buf,
            offset: 0,
            row_pitch: 4 * width,
            image_height: height,
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
        include_bytes!(concat!(env!("OUT_DIR"), "/", "shaders", "/", $shader_name))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        list_shaders();
        let _test_bytes = include_shader!("collatz.comp.spv");
    }
}
