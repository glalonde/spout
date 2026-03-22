//! Shared GPU test utilities. Compiled only under `#[cfg(test)]`.

/// Try to create a wgpu device and queue without a surface (headless). Returns `None` if no
/// adapter is available (e.g. no GPU and no software renderer installed).
pub fn try_create_headless_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await?;
        let info = adapter.get_info();
        log::info!(
            "GPU test adapter: {:?} (vendor 0x{:x})",
            info.name,
            info.vendor
        );
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
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

/// Create an offscreen `Bgra8UnormSrgb` render target of the given size.
///
/// Width must be 64 so that `bytes_per_row` (64 × 4 = 256) satisfies
/// `COPY_BYTES_PER_ROW_ALIGNMENT` exactly, enabling zero-padding readback.
pub fn create_offscreen_target(device: &wgpu::Device, width: u32, height: u32) -> OffscreenTarget {
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
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    OffscreenTarget { texture, view }
}

/// Create a staging buffer for CPU readback of a texture with the given dimensions.
pub fn create_readback_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Buffer {
    let size = (width * 4 * height) as wgpu::BufferAddress;
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Readback staging buffer"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

/// Encode a copy from `texture` into `staging_buffer`. Call before `queue.submit()`.
pub fn encode_texture_readback(
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    staging_buffer: &wgpu::Buffer,
    width: u32,
    height: u32,
) {
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: staging_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
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
    device.poll(wgpu::Maintain::Wait);
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
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    });
}

/// Swap red and blue channels of raw BGRA pixel data, producing RGBA.
pub fn bgra_to_rgba(bgra: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut rgba = vec![0u8; bgra.len()];
    for i in 0..(width * height) as usize {
        rgba[i * 4] = bgra[i * 4 + 2]; // R ← B
        rgba[i * 4 + 1] = bgra[i * 4 + 1]; // G
        rgba[i * 4 + 2] = bgra[i * 4]; // B ← R
        rgba[i * 4 + 3] = bgra[i * 4 + 3]; // A
    }
    rgba
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
