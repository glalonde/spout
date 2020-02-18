use log::error;

// Renders a text overlap as one of the final stages in the rendering pipeline.

pub struct DebugOverlay {
    pub glyph_brush: wgpu_glyph::GlyphBrush<'static, ()>,
}

impl DebugOverlay {
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        width: u32,
        height: u32,
        fps: f64,
    ) {
        let section = wgpu_glyph::Section {
            text: &format!("FPS: {:0.2}s", fps),
            screen_position: (00.0, 00.0),
            color: [1.0, 1.0, 1.0, 1.0],
            scale: wgpu_glyph::Scale { x: 20.0, y: 20.0 },
            bounds: (width as f32, height as f32),
            ..wgpu_glyph::Section::default()
        };
        self.glyph_brush.queue(section);
        let result =
            self.glyph_brush
                .draw_queued(&device, encoder, texture_view, 3 * width, 3 * height);
        if !result.is_ok() {
            error!("Failed to draw glyph: {}", result.unwrap_err());
        }
    }
}
