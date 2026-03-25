//! Shared GPU test utilities. Compiled only under `#[cfg(test)]`.

/// Try to create a wgpu device and queue without a surface (headless). Returns `None` if no
/// adapter is available (e.g. no GPU and no software renderer installed).
pub fn try_create_headless_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .ok()?;
        let info = adapter.get_info();
        log::info!(
            "GPU test adapter: {:?} (vendor 0x{:x})",
            info.name,
            info.vendor
        );
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .ok()?;
        Some((device, queue))
    })
}

/// Offscreen BGRA render target — texture + default view.
pub struct OffscreenTarget {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

/// Create an offscreen render target of the given size and format.
///
/// For `Bgra8UnormSrgb` or `Rgba16Float`, width must be 64 so that
/// `bytes_per_row` satisfies `COPY_BYTES_PER_ROW_ALIGNMENT` exactly.
pub fn create_offscreen_target(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> OffscreenTarget {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Offscreen render target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    OffscreenTarget { texture, view }
}

/// Create a staging buffer for CPU readback of a texture with the given dimensions.
///
/// `bytes_per_pixel` must match the texture format: 4 for `Bgra8UnormSrgb`, 8 for `Rgba16Float`.
pub fn create_readback_buffer(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    bytes_per_pixel: u32,
) -> wgpu::Buffer {
    let size = (width * bytes_per_pixel * height) as wgpu::BufferAddress;
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Readback staging buffer"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

/// Encode a copy from `texture` into `staging_buffer`. Call before `queue.submit()`.
///
/// `bytes_per_pixel` must match the texture format: 4 for `Bgra8UnormSrgb`, 8 for `Rgba16Float`.
pub fn encode_texture_readback(
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    staging_buffer: &wgpu::Buffer,
    width: u32,
    height: u32,
    bytes_per_pixel: u32,
) {
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: staging_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * bytes_per_pixel),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

/// Map `staging_buffer`, copy pixels to a `Vec<u8>`, then unmap. Call after `queue.submit()`.
pub fn readback_pixels(device: &wgpu::Device, staging_buffer: &wgpu::Buffer) -> Vec<u8> {
    let buffer_slice = staging_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let raw = buffer_slice.get_mapped_range();
    let pixels = raw.to_vec();
    drop(raw);
    staging_buffer.unmap();
    pixels
}

/// Encode a render pass that clears `view` to opaque black.
///
/// Required before rendering with `LoadOp::Load` on a freshly created texture,
/// which would otherwise contain undefined data.
pub fn encode_clear_texture(encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Clear pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}

/// Convert raw `Rgba16Float` bytes (8 bytes/pixel, little-endian f16) to 8-bit RGBA.
///
/// Each f16 channel is decoded, clamped to [0, 1], and scaled to [0, 255].
pub fn rgba16f_to_rgba8(raw: &[u8], width: u32, height: u32) -> Vec<u8> {
    let num_pixels = (width * height) as usize;
    let mut out = vec![0u8; num_pixels * 4];
    for i in 0..num_pixels {
        for ch in 0..4usize {
            let offset = (i * 4 + ch) * 2;
            let bits = u16::from_le_bytes([raw[offset], raw[offset + 1]]);
            let f = f16_to_f32(bits);
            out[i * 4 + ch] = (f.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
    }
    out
}

fn f16_to_f32(bits: u16) -> f32 {
    let sign = ((bits as u32) >> 15) << 31;
    let exp = ((bits >> 10) & 0x1f) as u32;
    let mant = (bits & 0x3ff) as u32;
    let f32_bits = match exp {
        0 => sign | (mant << 13),               // zero / subnormal
        31 => sign | 0x7f800000 | (mant << 13), // Inf / NaN
        e => sign | ((e + 112) << 23) | (mant << 13),
    };
    f32::from_bits(f32_bits)
}

/// Return `true` if every corresponding channel pair in `a` and `b` differs by at most
/// `tolerance` (inclusive).
pub fn images_within_tolerance(a: &[u8], b: &[u8], tolerance: u8) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b.iter())
            .all(|(x, y)| x.abs_diff(*y) <= tolerance)
}

/// Save RGBA pixel data as a PNG at `path`, creating parent directories as needed.
pub fn save_rgba_png(path: &std::path::Path, width: u32, height: u32, rgba: &[u8]) {
    let img = image::RgbaImage::from_raw(width, height, rgba.to_vec())
        .expect("Failed to create image from raw pixels");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create output dir");
    }
    img.save(path).expect("Failed to save PNG");
}

/// Golden image test helper.
///
/// Always saves `rgba` to `tests/output/{name}.png` for post-failure inspection.
///
/// - If `SPOUT_GENERATE_GOLDEN` is set: writes `tests/golden/{name}.png` and returns.
/// - Otherwise, if `tests/golden/{name}.png` exists: asserts pixel-wise within `tolerance=5`.
/// - Otherwise: prints a warning (no golden yet).
pub fn compare_or_generate_golden(name: &str, rgba: &[u8], width: u32, height: u32) {
    let output_path = format!("tests/output/{name}.png");
    save_rgba_png(std::path::Path::new(&output_path), width, height, rgba);
    eprintln!("Saved render output to {output_path}");

    let golden_path = format!("tests/golden/{name}.png");
    if std::env::var("SPOUT_GENERATE_GOLDEN").is_ok() {
        save_rgba_png(std::path::Path::new(&golden_path), width, height, rgba);
        eprintln!("Generated golden image at {golden_path}");
    } else if std::path::Path::new(&golden_path).exists() {
        let golden_img = image::open(&golden_path)
            .expect("Failed to open golden image")
            .to_rgba8();
        let golden_pixels = golden_img.into_raw();
        assert!(
            images_within_tolerance(rgba, &golden_pixels, 5),
            "Render output differs from golden by more than tolerance=5. \
            Check {} vs {}",
            output_path,
            golden_path
        );
        eprintln!("Golden image comparison passed.");
    } else {
        eprintln!(
            "No golden image at {golden_path}. \
            Run with SPOUT_GENERATE_GOLDEN=1 to generate."
        );
    }
}
