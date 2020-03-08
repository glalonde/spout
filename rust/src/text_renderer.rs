use log::error;
use wgpu_glyph::GlyphBrushBuilder;
// Renders a text for the game menus/state/score, anything in the main game
// viewport.

pub struct TextRenderer {
    pub glyph_brush: wgpu_glyph::GlyphBrush<'static, ()>,
    width: u32,
    height: u32,
}

impl TextRenderer {
    // Width, height of the game viewport
    pub fn init(device: &wgpu::Device, width: u32, height: u32) -> Self {
        TextRenderer {
            glyph_brush: GlyphBrushBuilder::using_font_bytes(super::fonts::INCONSOLATA)
                .texture_filter_method(wgpu::FilterMode::Nearest)
                .build(device, wgpu::TextureFormat::Bgra8UnormSrgb),
            width,
            height,
        }
    }
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        text: &str,
    ) {
        let section = wgpu_glyph::Section {
            text,
            screen_position: (00.0, 00.0),
            color: [1.0, 1.0, 1.0, 1.0],
            scale: wgpu_glyph::Scale { x: 20.0, y: 20.0 },
            bounds: (self.width as f32, self.height as f32),
            ..wgpu_glyph::Section::default()
        };
        self.glyph_brush.queue(section);
        let result =
            self.glyph_brush
                .draw_queued(&device, encoder, texture_view, self.width, self.height);
        if !result.is_ok() {
            error!("Failed to draw glyph: {}", result.unwrap_err());
        }
    }
}
