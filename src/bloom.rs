//! HDR bloom post-processing using dual-filter (mip-pyramid) bloom.
//!
//! Pipeline (Jimenez "Next-gen Post-Processing in Call of Duty: Advanced
//! Warfare", Siggraph 2014):
//!
//!   game_view ── prefilter ──▶ mip 0
//!                              ├── downsample ──▶ mip 1
//!                              │                  ├── downsample ──▶ mip 2
//!                              │                  │                  ⋮
//!                              │                  │            mip N-1
//!                              │                  │             ▼ upsample (add)
//!                              │                  ◀──────── mip N-2
//!                              │                              ⋮
//!                              ◀──────────────────────── upsample (add)
//!                              │
//!                            mip 0  (final bloom output, exposed as `bloom_view()`)
//!
//! The pyramid texture starts at half surface resolution and has N mip levels
//! (configurable via `VisualParams::bloom_mip_levels`). Going from full to half
//! resolution in the prefilter, plus the geometric falloff of mip area, makes
//! this dramatically cheaper than the previous separable-Gaussian ping-pong
//! approach (~30x less memory bandwidth at the same halo width).

use wgpu::util::DeviceExt;

/// Format used for the bloom pyramid.
pub const BLOOM_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// HDR format used for the game view texture (terrain + particles + ship render into this).
pub const GAME_VIEW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// Number of mip levels actually used. Caller-supplied `bloom_mip_levels` is
/// clamped to this. Floor of 1 — at minimum we need a single mip to render to.
fn clamp_mip_levels(width: u32, height: u32, requested: u32) -> u32 {
    let half = width.min(height).max(2) / 2;
    let max_levels = u32::BITS - half.leading_zeros();
    requested.clamp(1, max_levels.max(1))
}

struct MipLevel {
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

/// Dual-filter bloom pipeline.
pub struct Bloom {
    prefilter_pipeline: wgpu::RenderPipeline,
    downsample_pipeline: wgpu::RenderPipeline,
    upsample_pipeline: wgpu::RenderPipeline,

    /// `[i]` reads mip i, used by downsample passes (i = 0..N-2 for downsample,
    /// i = 1..N-1 for upsample sampling). One bind group per readable mip.
    sampling_bind_groups: Vec<wgpu::BindGroup>,
    /// Reads game_view, used by the prefilter pass.
    prefilter_bind_group: wgpu::BindGroup,

    mip_levels: Vec<MipLevel>,
    pub pyramid_texture: wgpu::Texture,
}

impl Bloom {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        game_view: &wgpu::TextureView,
        visual_params: &crate::game_params::VisualParams,
    ) -> Self {
        let n = clamp_mip_levels(width, height, visual_params.bloom_mip_levels);

        // Pyramid: half surface size, N mip levels.
        let pyramid_width = (width / 2).max(1);
        let pyramid_height = (height / 2).max(1);
        let pyramid_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("bloom_pyramid"),
            size: wgpu::Extent3d {
                width: pyramid_width,
                height: pyramid_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: n,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: BLOOM_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let mip_levels: Vec<MipLevel> = (0..n)
            .map(|i| {
                let view = pyramid_texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("bloom_mip_view"),
                    base_mip_level: i,
                    mip_level_count: Some(1),
                    ..Default::default()
                });
                let w = (pyramid_width >> i).max(1);
                let h = (pyramid_height >> i).max(1);
                MipLevel {
                    view,
                    width: w,
                    height: h,
                }
            })
            .collect();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("bloom_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        // Same bind-group layout for all three pipelines: source texture +
        // sampler + uniform with the source's texel size.
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(16),
                    },
                    count: None,
                },
            ],
        });

        let make_uniform = |label: &str, w: u32, h: u32| {
            let v: [f32; 4] = [1.0 / w as f32, 1.0 / h as f32, 0.0, 0.0];
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(&v),
                usage: wgpu::BufferUsages::UNIFORM,
            })
        };

        let make_bind_group = |label: &str,
                               source_view: &wgpu::TextureView,
                               uniform: &wgpu::Buffer|
         -> wgpu::BindGroup {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: uniform.as_entire_binding(),
                    },
                ],
            })
        };

        // Prefilter: source is the full-size game_view.
        let prefilter_uniform = make_uniform("bloom_prefilter_u", width, height);
        let prefilter_bind_group =
            make_bind_group("bloom_prefilter_bg", game_view, &prefilter_uniform);

        // Sampling bind groups: one per mip, used to read FROM that mip during
        // downsample (read mip i → write mip i+1) and upsample (read mip i+1 →
        // write mip i, additively). We don't need a sampling bind group for
        // mip 0 specifically used by sampling, but generating one for every
        // mip keeps indexing simple — we only ever sample mips 0..N-2 for
        // downsample and 1..N-1 for upsample.
        let sampling_bind_groups: Vec<wgpu::BindGroup> = mip_levels
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let uniform = make_uniform(&format!("bloom_mip{}_u", i), m.width, m.height);
                make_bind_group(&format!("bloom_mip{}_bg", i), &m.view, &uniform)
            })
            .collect();

        // Pipeline layouts (all share `bgl`).
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bloom_pipeline_layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });

        let prefilter_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom_prefilter_shader"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("bloom_prefilter.wgsl")),
        });
        let downsample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom_downsample_shader"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("bloom_downsample.wgsl")),
        });
        let upsample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom_upsample_shader"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("bloom_upsample.wgsl")),
        });

        let make_pipeline = |label: &str,
                             shader: &wgpu::ShaderModule,
                             constants: &[(&str, f64)],
                             blend: Option<wgpu::BlendState>|
         -> wgpu::RenderPipeline {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: BLOOM_FORMAT,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants,
                        ..Default::default()
                    },
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                cache: None,
                multiview_mask: None,
            })
        };

        let prefilter_pipeline = make_pipeline(
            "bloom_prefilter",
            &prefilter_shader,
            &[("bloom_threshold", visual_params.bloom_threshold as f64)],
            None,
        );
        let downsample_pipeline = make_pipeline("bloom_downsample", &downsample_shader, &[], None);
        // Upsample uses additive blending: dst_color = src_color + dst_color.
        // On TBDR GPUs the destination read for the blend stays in tile memory.
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::REPLACE,
        };
        let upsample_pipeline =
            make_pipeline("bloom_upsample", &upsample_shader, &[], Some(additive));

        Bloom {
            prefilter_pipeline,
            downsample_pipeline,
            upsample_pipeline,
            sampling_bind_groups,
            prefilter_bind_group,
            mip_levels,
            pyramid_texture,
        }
    }

    /// Run the bloom pipeline. Call after all game renders, before the final composite.
    pub fn render(&self, encoder: &mut wgpu::CommandEncoder) {
        // Prefilter: game_view → mip 0. Clear because the prefilter writes
        // every pixel.
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom_prefilter"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.mip_levels[0].view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.prefilter_pipeline);
            pass.set_bind_group(0, &self.prefilter_bind_group, &[]);
            pass.draw(0..4, 0..1);
        }

        // Downsample: mip i → mip i+1, for i in 0..N-1.
        for i in 0..(self.mip_levels.len() - 1) {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom_downsample"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.mip_levels[i + 1].view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.downsample_pipeline);
            pass.set_bind_group(0, &self.sampling_bind_groups[i], &[]);
            pass.draw(0..4, 0..1);
        }

        // Upsample: mip i+1 → mip i, additively, for i in (0..N-1).rev().
        // LoadOp::Load preserves the downsample result already in mip i so the
        // additive blend layers the smaller mip's contribution on top.
        for i in (0..(self.mip_levels.len() - 1)).rev() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom_upsample"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.mip_levels[i].view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.upsample_pipeline);
            pass.set_bind_group(0, &self.sampling_bind_groups[i + 1], &[]);
            pass.draw(0..4, 0..1);
        }
    }

    /// The final bloom texture (mip 0 of the pyramid, half surface resolution).
    /// The composite samples this with bilinear filtering.
    pub fn bloom_view(&self) -> &wgpu::TextureView {
        &self.mip_levels[0].view
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_test_utils as gpu;

    const TEST_W: u32 = 64;
    const TEST_H: u32 = 64;
    /// Bytes per pixel for Rgba16Float: 4 channels × 2 bytes (f16) = 8.
    const BPP: u32 = 8;

    #[test]
    fn clamp_mip_levels_includes_one_pixel_tail() {
        assert_eq!(clamp_mip_levels(64, 64, 99), 6);
        assert_eq!(clamp_mip_levels(32, 32, 99), 5);
        assert_eq!(clamp_mip_levels(3, 3, 99), 1);
        assert_eq!(clamp_mip_levels(64, 64, 3), 3);
        assert_eq!(clamp_mip_levels(64, 64, 0), 1);
    }

    fn make_game_view_texture(device: &wgpu::Device) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("test_game_view"),
            size: wgpu::Extent3d {
                width: TEST_W,
                height: TEST_H,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: GAME_VIEW_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }

    /// Upload pixel data to `texture`. Unlisted pixels are black (zero).
    /// Each entry is `(col, row, f16_le_bytes)` — all four channels set to the given value.
    fn upload_pixels(queue: &wgpu::Queue, texture: &wgpu::Texture, pixels: &[(u32, u32, [u8; 2])]) {
        let mut data = vec![0u8; (TEST_W * TEST_H * BPP) as usize];
        for &(col, row, f16_bytes) in pixels {
            let base = ((row * TEST_W + col) * BPP) as usize;
            for ch in 0..4usize {
                data[base + ch * 2] = f16_bytes[0];
                data[base + ch * 2 + 1] = f16_bytes[1];
            }
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(TEST_W * BPP),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: TEST_W,
                height: TEST_H,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Verifies that all bloom shaders compile and all pipelines/bind groups
    /// construct without error. Does not check pixel output.
    #[test]
    fn test_bloom_construction() {
        let Some((device, _queue)) = gpu::try_create_headless_device() else {
            eprintln!("No GPU available, skipping bloom construction test");
            return;
        };

        let game_texture = make_game_view_texture(&device);
        let game_view = game_texture.create_view(&Default::default());

        let _bloom = Bloom::new(
            &device,
            TEST_W,
            TEST_H,
            &game_view,
            &crate::game_params::VisualParams::default(),
        );
    }

    /// Smoke test: place two bright pixels at asymmetric positions and verify
    /// the bloom output has visibly brighter pixels at those positions than at
    /// the corners (ensuring no Y/X flip in the pyramid).
    #[test]
    fn test_bloom_runs_and_preserves_orientation() {
        let Some((device, queue)) = gpu::try_create_headless_device() else {
            eprintln!("No GPU available, skipping bloom orientation test");
            return;
        };

        // f16 encoding of 8.0: sign=0, exp=3+15=18=0x12, mantissa=0 → 0x4800 LE.
        let f16_8: [u8; 2] = [0x00, 0x48];

        let game_texture = make_game_view_texture(&device);
        upload_pixels(&queue, &game_texture, &[(12, 8, f16_8), (48, 40, f16_8)]);
        queue.submit([]);

        let game_view = game_texture.create_view(&Default::default());
        let visual_params = crate::game_params::VisualParams {
            bloom_threshold: 0.5,
            bloom_mip_levels: 4,
            ..crate::game_params::VisualParams::default()
        };
        let bloom = Bloom::new(&device, TEST_W, TEST_H, &game_view, &visual_params);

        // Read back mip 0 of the pyramid (half-resolution: TEST_W/2 × TEST_H/2).
        let mip0_w = TEST_W / 2;
        let mip0_h = TEST_H / 2;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("test_bloom_encoder"),
        });
        bloom.render(&mut encoder);

        let staging = gpu::create_readback_buffer(&device, mip0_w, mip0_h, BPP);
        gpu::encode_texture_readback(
            &mut encoder,
            &bloom.pyramid_texture,
            &staging,
            mip0_w,
            mip0_h,
            BPP,
        );
        queue.submit(std::iter::once(encoder.finish()));

        let raw = gpu::readback_pixels(&device, &staging);
        let rgba8 = gpu::rgba16f_to_rgba8(&raw, mip0_w, mip0_h);

        // The two input bright pixels are in the upper-left and lower-right
        // quadrants. After bloom, those quadrants should be brighter than the
        // opposite corners (which had no bright source nearby).
        let sample = |x: u32, y: u32| -> u32 {
            let idx = ((y * mip0_w + x) * 4) as usize;
            rgba8[idx] as u32 + rgba8[idx + 1] as u32 + rgba8[idx + 2] as u32
        };
        let upper_left = sample(mip0_w / 4, mip0_h / 4);
        let lower_right = sample(3 * mip0_w / 4, 3 * mip0_h / 4);
        let upper_right = sample(3 * mip0_w / 4, mip0_h / 4);
        let lower_left = sample(mip0_w / 4, 3 * mip0_h / 4);

        assert!(
            upper_left > upper_right && upper_left > lower_left,
            "expected upper-left to be brighter than upper-right ({upper_right}) and lower-left ({lower_left}), got {upper_left}",
        );
        assert!(
            lower_right > upper_right && lower_right > lower_left,
            "expected lower-right to be brighter than upper-right ({upper_right}) and lower-left ({lower_left}), got {lower_right}",
        );
    }
}
