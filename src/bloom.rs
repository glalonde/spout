//! HDR bloom post-processing pipeline: threshold extraction, separable
//! Gaussian blur (configurable passes), and additive composite.

use wgpu::util::DeviceExt;

/// Intermediate texture format used for bloom ping-pong buffers.
pub const BLOOM_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// HDR format used for the game view texture (terrain + particles + ship render into this).
/// Float allows bloom to add values back without clipping.
pub const GAME_VIEW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// Threshold + separable Gaussian bloom pass.
///
/// Pipeline:
///   game_view → [threshold] → bright_tex
///   bright_tex → [h_blur]   → blur_tex
///   blur_tex   → [v_blur]   → bright_tex  (final bloom output)
///
/// `bloom_view()` exposes the final bloom result for compositing in the blit pass.
pub struct Bloom {
    threshold_pipeline: wgpu::RenderPipeline,
    threshold_bind_group: wgpu::BindGroup,

    blur_pipeline: wgpu::RenderPipeline,
    /// bright_tex → blur_tex (horizontal pass)
    h_blur_bind_group: wgpu::BindGroup,
    /// blur_tex → bright_tex (vertical pass)
    v_blur_bind_group: wgpu::BindGroup,

    /// Receives threshold output; also the final bloom result after v_blur overwrites it.
    pub bright_texture: wgpu::Texture,
    bright_view: wgpu::TextureView,

    /// Intermediate buffer for horizontal blur output.
    pub blur_texture: wgpu::Texture,
    blur_view: wgpu::TextureView,

    /// Number of H+V blur iterations. Each pass widens the halo by ~√2.
    bloom_passes: u32,
}

impl Bloom {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        game_view: &wgpu::TextureView,
        visual_params: &crate::game_params::VisualParams,
    ) -> Self {
        let make_tex = |label| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: BLOOM_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };
        let bright_texture = make_tex("bloom_bright");
        let bright_view = bright_texture.create_view(&Default::default());
        let blur_texture = make_tex("bloom_blur");
        let blur_view = blur_texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("bloom_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        // --- Threshold pipeline ---
        let threshold_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom_threshold_bgl"),
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
            ],
        });
        let threshold_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom_threshold_bg"),
            layout: &threshold_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(game_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let threshold_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom_threshold_shader"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("bloom_threshold.wgsl")),
        });
        let threshold_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bloom_threshold_layout"),
            bind_group_layouts: &[Some(&threshold_bgl)],
            immediate_size: 0,
        });
        let threshold_constants = [("bloom_threshold", visual_params.bloom_threshold as f64)];

        let threshold_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bloom_threshold"),
            layout: Some(&threshold_layout),
            vertex: wgpu::VertexState {
                module: &threshold_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &threshold_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: BLOOM_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &threshold_constants,
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
        });

        // --- Blur pipeline ---
        let blur_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom_blur_bgl"),
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

        // Direction uniforms: one-texel step in UV space for the bilinear blur kernel.
        let h_dir: [f32; 4] = [1.0 / width as f32, 0.0, 0.0, 0.0];
        let v_dir: [f32; 4] = [0.0, 1.0 / height as f32, 0.0, 0.0];

        let h_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bloom_h_uniform"),
            contents: bytemuck::cast_slice(&h_dir),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let v_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bloom_v_uniform"),
            contents: bytemuck::cast_slice(&v_dir),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let h_blur_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom_h_blur_bg"),
            layout: &blur_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bright_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: h_uniform.as_entire_binding(),
                },
            ],
        });
        let v_blur_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom_v_blur_bg"),
            layout: &blur_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: v_uniform.as_entire_binding(),
                },
            ],
        });

        let blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom_blur_shader"),
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("bloom_blur.wgsl")),
        });
        let blur_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bloom_blur_layout"),
            bind_group_layouts: &[Some(&blur_bgl)],
            immediate_size: 0,
        });
        let blur_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bloom_blur"),
            layout: Some(&blur_layout),
            vertex: wgpu::VertexState {
                module: &blur_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &blur_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: BLOOM_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        Bloom {
            threshold_pipeline,
            threshold_bind_group,
            blur_pipeline,
            h_blur_bind_group,
            v_blur_bind_group,
            bright_texture,
            bright_view,
            blur_texture,
            blur_view,
            bloom_passes: visual_params.bloom_passes.max(1),
        }
    }

    /// Run the bloom pipeline. Call after all game renders, before the final blit.
    ///
    /// Executes: threshold, then `bloom_passes` × (horizontal blur + vertical blur).
    /// Each H+V iteration widens the halo by ~√2 via Gaussian convolution.
    pub fn render(&self, encoder: &mut wgpu::CommandEncoder) {
        let clear = wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
        };

        // Threshold: game_view → bright_tex
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom_threshold"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.bright_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: clear,
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.threshold_pipeline);
            pass.set_bind_group(0, &self.threshold_bind_group, &[]);
            pass.draw(0..4, 0..1);
        }

        // Blur iterations: each pass is H (bright_tex→blur_tex) + V (blur_tex→bright_tex).
        for _ in 0..self.bloom_passes {
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("bloom_h_blur"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.blur_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: clear,
                    })],
                    ..Default::default()
                });
                pass.set_pipeline(&self.blur_pipeline);
                pass.set_bind_group(0, &self.h_blur_bind_group, &[]);
                pass.draw(0..4, 0..1);
            }
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("bloom_v_blur"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.bright_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: clear,
                    })],
                    ..Default::default()
                });
                pass.set_pipeline(&self.blur_pipeline);
                pass.set_bind_group(0, &self.v_blur_bind_group, &[]);
                pass.draw(0..4, 0..1);
            }
        }
    }

    /// The final bloom texture, ready to be additively composited onto the game view.
    pub fn bloom_view(&self) -> &wgpu::TextureView {
        &self.bright_view
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
    /// Row 0 is the top of the texture.
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

    /// Verifies that all three bloom shaders compile and all pipelines/bind groups
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

    /// Golden image test: places bright pixels at two asymmetric positions, runs all
    /// three bloom passes, and compares the output against a stored golden PNG.
    ///
    /// Input pixels are f16 8.0 so the Gaussian bloom halo is clearly visible after two
    /// passes of attenuation (≈ 98/255 at the peak; see calculation below).
    ///
    /// Pixel positions (col, row) — texture row 0 = top:
    ///   (12, 8)  — upper-left quadrant
    ///   (48, 40) — lower-right quadrant
    ///
    /// Any Y-flip or X-flip in the bloom would move the halos to the wrong quadrant
    /// and fail the comparison.
    ///
    /// Math: threshold=0.5, input=8.0 → threshold output ≈ 7.5 → center after
    /// h_blur ≈ 1.70 → center after v_blur ≈ 0.39 → rgba8 ≈ 99.
    ///
    /// Regenerate: SPOUT_GENERATE_GOLDEN=1 cargo test test_bloom_golden -- --nocapture
    #[test]
    fn test_bloom_golden() {
        let Some((device, queue)) = gpu::try_create_headless_device() else {
            eprintln!("No GPU available, skipping bloom golden test");
            return;
        };

        // f16 encoding of 8.0: sign=0, exp=3+15=18=0x12, mantissa=0 → 0x4800 LE.
        let f16_8: [u8; 2] = [0x00, 0x48];

        let game_texture = make_game_view_texture(&device);
        upload_pixels(&queue, &game_texture, &[(12, 8, f16_8), (48, 40, f16_8)]);
        // Flush write_texture before recording render commands that read it.
        queue.submit([]);

        let game_view = game_texture.create_view(&Default::default());
        let visual_params = crate::game_params::VisualParams {
            bloom_threshold: 0.5,
            ..crate::game_params::VisualParams::default()
        };
        let bloom = Bloom::new(&device, TEST_W, TEST_H, &game_view, &visual_params);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("test_bloom_golden_encoder"),
        });
        bloom.render(&mut encoder);

        let staging = gpu::create_readback_buffer(&device, TEST_W, TEST_H, BPP);
        gpu::encode_texture_readback(
            &mut encoder,
            &bloom.bright_texture,
            &staging,
            TEST_W,
            TEST_H,
            BPP,
        );
        queue.submit(std::iter::once(encoder.finish()));

        let raw = gpu::readback_pixels(&device, &staging);
        let rgba8 = gpu::rgba16f_to_rgba8(&raw, TEST_W, TEST_H);

        // The RGB channels should have visible halos. (Alpha is always 1.0.)
        let max_r = (0..TEST_W * TEST_H)
            .map(|i| rgba8[(i * 4) as usize])
            .max()
            .unwrap_or(0);
        assert!(
            max_r > 10,
            "bloom RGB output is unexpectedly dark (max_r={max_r}); check upload or passes"
        );

        gpu::compare_or_generate_golden("bloom_golden", &rgba8, TEST_W, TEST_H);
    }
}
