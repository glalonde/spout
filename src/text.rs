//! Bitmap font text renderer using `fontdue` for glyph rasterization and a
//! GPU texture atlas. Designed for the Pixel Six 8px bitmap font but works
//! with any TTF/OTF.

use wgpu::util::DeviceExt;

/// Available embedded bitmap fonts.
pub enum Font {
    /// 04b_30 — 17px tall, blocky uppercase style.
    O4b30,
    /// 04b_11 — 16px tall, narrower style.
    O4b11,
}

const FONT_04B_30: &[u8] = include_bytes!("../assets/fonts/04b_30.ttf");
const FONT_04B_11: &[u8] = include_bytes!("../assets/fonts/04b_11.ttf");

impl Font {
    fn data(&self) -> &'static [u8] {
        match self {
            Font::O4b30 => FONT_04B_30,
            Font::O4b11 => FONT_04B_11,
        }
    }

    fn size(&self) -> f32 {
        match self {
            Font::O4b30 => 17.0,
            Font::O4b11 => 16.0,
        }
    }
}

/// Range of ASCII characters baked into the atlas.
const FIRST_CHAR: u8 = 32; // space
const LAST_CHAR: u8 = 126; // tilde
const GLYPH_COUNT: usize = (LAST_CHAR - FIRST_CHAR + 1) as usize;

/// Per-glyph metrics stored CPU-side for layout.
#[derive(Clone, Copy, Debug)]
struct GlyphInfo {
    /// UV coordinates in the atlas (normalized 0–1).
    uv_x: f32,
    uv_y: f32,
    uv_w: f32,
    uv_h: f32,
    /// Pixel dimensions of the rasterized glyph.
    width: f32,
    height: f32,
    /// Offset from the pen position to the top-left of the glyph bitmap.
    x_offset: f32,
    y_offset: f32,
    /// Horizontal advance to the next glyph.
    advance: f32,
}

/// Instance data for a single glyph quad, sent to the GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GlyphInstance {
    /// Screen position (top-left corner), in pixels.
    pos: [f32; 2],
    /// Size in pixels.
    size: [f32; 2],
    /// UV rect: [u_min, v_min, u_max, v_max].
    uv: [f32; 4],
    /// RGBA color.
    color: [f32; 4],
}

/// Y-axis direction for the target coordinate system.
#[derive(Debug, Clone, Copy)]
pub enum YDirection {
    /// Screen space: origin at top-left, Y increases downward.
    Down,
    /// Game view: origin at bottom-left, Y increases upward.
    Up,
}

pub struct TextRenderer {
    glyphs: Vec<GlyphInfo>,
    atlas_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    screen_uniform_buf: wgpu::Buffer,
    screen_bind_group: wgpu::BindGroup,
    pub surface_width: f32,
    pub surface_height: f32,
    y_dir: f32,
    /// Maximum ascent across all glyphs (ymin + height), in native font pixels.
    /// Used to position the baseline relative to the line top.
    ascent: f32,
}

impl TextRenderer {
    pub fn init(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        surface_width: u32,
        surface_height: u32,
        y_direction: YDirection,
        font_choice: Font,
    ) -> Self {
        let font = fontdue::Font::from_bytes(font_choice.data(), fontdue::FontSettings::default())
            .expect("failed to parse font");

        // Rasterize at the font's native size for crisp, pixel-perfect glyphs.
        let font_size = font_choice.size();

        // Rasterize all glyphs and collect metrics.
        let mut rasterized: Vec<(fontdue::Metrics, Vec<u8>)> = Vec::with_capacity(GLYPH_COUNT);
        for c in FIRST_CHAR..=LAST_CHAR {
            let (metrics, mut bitmap) = font.rasterize(c as char, font_size);
            // Threshold to 1-bit: anything above 50% → fully opaque.
            // This eliminates anti-aliasing artifacts on a pixel font.
            for px in bitmap.iter_mut() {
                *px = if *px > 127 { 255 } else { 0 };
            }
            rasterized.push((metrics, bitmap));
        }

        // Pack glyphs into a texture atlas (single row for simplicity; fine for <100 glyphs).
        let padding = 1u32;
        let atlas_height = rasterized
            .iter()
            .map(|(m, _)| m.height as u32)
            .max()
            .unwrap_or(1)
            + padding * 2;
        let atlas_width: u32 = rasterized
            .iter()
            .map(|(m, _)| m.width as u32 + padding)
            .sum::<u32>()
            + padding;

        let mut atlas_data = vec![0u8; (atlas_width * atlas_height * 4) as usize]; // RGBA
        let mut glyphs = Vec::with_capacity(GLYPH_COUNT);
        let mut cursor_x = padding;

        for (metrics, bitmap) in &rasterized {
            let gw = metrics.width as u32;
            let gh = metrics.height as u32;

            // Copy alpha bitmap into RGBA atlas (white text, alpha from rasterizer).
            for row in 0..gh {
                for col in 0..gw {
                    let src = (row * gw + col) as usize;
                    let dst_x = cursor_x + col;
                    let dst_y = padding + row;
                    let dst = ((dst_y * atlas_width + dst_x) * 4) as usize;
                    let alpha = bitmap[src];
                    atlas_data[dst] = 255;
                    atlas_data[dst + 1] = 255;
                    atlas_data[dst + 2] = 255;
                    atlas_data[dst + 3] = alpha;
                }
            }

            glyphs.push(GlyphInfo {
                uv_x: cursor_x as f32 / atlas_width as f32,
                uv_y: padding as f32 / atlas_height as f32,
                uv_w: gw as f32 / atlas_width as f32,
                uv_h: gh as f32 / atlas_height as f32,
                width: gw as f32,
                height: gh as f32,
                x_offset: metrics.xmin as f32,
                y_offset: metrics.ymin as f32,
                advance: metrics.advance_width,
            });

            cursor_x += gw + padding;
        }

        // Compute max ascent: the highest point any glyph reaches above the
        // baseline. In fontdue, ascent = ymin + height for each glyph.
        let ascent = rasterized
            .iter()
            .map(|(m, _)| m.ymin as f32 + m.height as f32)
            .fold(0.0f32, f32::max);

        // Upload atlas to GPU.
        let atlas_texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Text Atlas"),
                size: wgpu::Extent3d {
                    width: atlas_width,
                    height: atlas_height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &atlas_data,
        );
        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest, // crisp pixel font
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Bind group for atlas texture + sampler.
        let atlas_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Text Atlas BGL"),
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
        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Atlas BG"),
            layout: &atlas_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Screen-size uniform for pixel → NDC conversion.
        let y_dir = match y_direction {
            YDirection::Down => 1.0f32,
            YDirection::Up => -1.0,
        };
        let screen_data = [
            surface_width as f32,
            surface_height as f32,
            y_dir,
            0.0, /* padding */
        ];
        let screen_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Text Screen Uniform"),
            contents: bytemuck::cast_slice(&screen_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let screen_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Text Screen BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let screen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Screen BG"),
            layout: &screen_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_uniform_buf.as_entire_binding(),
            }],
        });

        // Shader and pipeline.
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/text.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[Some(&atlas_bgl), Some(&screen_bgl)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<GlyphInstance>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        // pos
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        // size
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        // uv
                        wgpu::VertexAttribute {
                            offset: 16,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        // color
                        wgpu::VertexAttribute {
                            offset: 32,
                            shader_location: 3,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        TextRenderer {
            glyphs,
            atlas_bind_group,
            pipeline,
            screen_uniform_buf,
            screen_bind_group,
            surface_width: surface_width as f32,
            surface_height: surface_height as f32,
            y_dir,
            ascent,
        }
    }

    /// Measure the width of a text string in pixels at the given scale.
    pub fn text_width(&self, text: &str, scale: f32) -> f32 {
        let mut w = 0.0;
        for ch in text.bytes() {
            if !(FIRST_CHAR..=LAST_CHAR).contains(&ch) {
                continue;
            }
            let idx = (ch - FIRST_CHAR) as usize;
            w += self.glyphs[idx].advance * scale;
        }
        w
    }

    /// Call when the surface is resized.
    pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        self.surface_width = width as f32;
        self.surface_height = height as f32;
        let data = [self.surface_width, self.surface_height, self.y_dir, 0.0];
        queue.write_buffer(&self.screen_uniform_buf, 0, bytemuck::cast_slice(&data));
    }

    /// Draw text strings onto the given render target.
    ///
    /// `texts` is a list of `(text, x, y, scale, color)` where x/y are pixel
    /// coordinates from the top-left and scale is an integer multiplier (1 = native
    /// size, 2 = 2x, etc.).
    pub fn draw(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        texts: &[(&str, f32, f32, f32, [f32; 4])],
    ) {
        let mut instances: Vec<GlyphInstance> = Vec::new();

        for &(text, start_x, start_y, scale, color) in texts {
            let mut pen_x = start_x;
            for ch in text.bytes() {
                if !(FIRST_CHAR..=LAST_CHAR).contains(&ch) {
                    continue;
                }
                let idx = (ch - FIRST_CHAR) as usize;
                let g = &self.glyphs[idx];

                if g.width > 0.0 && g.height > 0.0 {
                    // In screen coords (y-down), baseline is at start_y + ascent.
                    // Glyph top = baseline - (ymin + height), bottom = baseline - ymin.
                    let baseline_y = start_y + self.ascent * scale;
                    let glyph_top = baseline_y - (g.y_offset + g.height) * scale;

                    instances.push(GlyphInstance {
                        pos: [pen_x + g.x_offset * scale, glyph_top],
                        size: [g.width * scale, g.height * scale],
                        uv: [g.uv_x, g.uv_y, g.uv_x + g.uv_w, g.uv_y + g.uv_h],
                        color,
                    });
                }

                pen_x += g.advance * scale;
            }
        }

        if instances.is_empty() {
            return;
        }

        let instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Text Instances"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Text Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // draw on top of existing content
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.atlas_bind_group, &[]);
        rpass.set_bind_group(1, &self.screen_bind_group, &[]);
        rpass.set_vertex_buffer(0, instance_buf.slice(..));
        rpass.draw(0..4, 0..instances.len() as u32); // 4 vertices per quad (triangle strip)
    }
}

/// Rasterize a text string into a 2D terrain grid.
///
/// Returns a `(width, height, Vec<i32>)` where each cell is either
/// `terrain_health` (solid glyph pixel) or `0` (empty). The grid is
/// Y-up: row 0 is the bottom.
pub fn rasterize_text_to_terrain(
    text: &str,
    font_choice: &Font,
    scale: f32,
    terrain_health: i32,
) -> (u32, u32, Vec<i32>) {
    let font =
        fontdue::Font::from_bytes(font_choice.data(), fontdue::FontSettings::default())
            .expect("failed to parse font");
    let font_size = font_choice.size();

    // Rasterize glyphs and compute layout.
    let mut glyphs: Vec<(fontdue::Metrics, Vec<u8>)> = Vec::new();
    for ch in text.bytes() {
        let (metrics, mut bitmap) = font.rasterize(ch as char, font_size);
        for px in bitmap.iter_mut() {
            *px = if *px > 127 { 255 } else { 0 };
        }
        glyphs.push((metrics, bitmap));
    }

    // Compute ascent (max ymin + height across all glyphs).
    let ascent = glyphs
        .iter()
        .map(|(m, _)| m.ymin as f32 + m.height as f32)
        .fold(0.0f32, f32::max);

    // Total width in native pixels, then scale.
    let native_width: f32 = glyphs.iter().map(|(m, _)| m.advance_width).sum();
    let grid_w = (native_width * scale).ceil() as u32;
    let grid_h = (ascent * scale).ceil() as u32 + 2; // small padding

    let mut grid = vec![0i32; (grid_w * grid_h) as usize];

    let mut pen_x: f32 = 0.0;
    for (metrics, bitmap) in &glyphs {
        let gw = metrics.width;
        let gh = metrics.height;
        if gw == 0 || gh == 0 {
            pen_x += metrics.advance_width * scale;
            continue;
        }

        // Glyph top in Y-down font space: baseline - (ymin + height).
        // Convert to Y-up grid space.
        let baseline_y = ascent * scale;
        let glyph_top_ydown = baseline_y - (metrics.ymin as f32 + gh as f32) * scale;

        for row in 0..gh {
            for col in 0..gw {
                if bitmap[row * gw + col] == 0 {
                    continue;
                }
                // Scaled pixel position (Y-down).
                for sy in 0..scale as u32 {
                    for sx in 0..scale as u32 {
                        let px = (pen_x + metrics.xmin as f32 * scale + col as f32 * scale + sx as f32) as i32;
                        let py_down = (glyph_top_ydown + row as f32 * scale + sy as f32) as i32;
                        // Flip to Y-up.
                        let py_up = grid_h as i32 - 1 - py_down;
                        if px >= 0 && px < grid_w as i32 && py_up >= 0 && py_up < grid_h as i32 {
                            grid[(py_up as u32 * grid_w + px as u32) as usize] = terrain_health;
                        }
                    }
                }
            }
        }
        pen_x += metrics.advance_width * scale;
    }

    (grid_w, grid_h, grid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_metrics_sanity() {
        let font = fontdue::Font::from_bytes(Font::O4b30.data(), fontdue::FontSettings::default())
            .expect("font parse");
        for ch in ['F', 'P', 'S', '0', ':', ' '] {
            let (m, _) = font.rasterize(ch, 10.0);
            println!(
                "{:?}: w={} h={} xmin={} ymin={} advance={:.1}",
                ch, m.width, m.height, m.xmin, m.ymin, m.advance_width
            );
        }
    }
}
