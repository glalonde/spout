use log::error;
use wgpu_glyph::GlyphBrushBuilder;
// Renders a text for the game menus/state/score, anything in the main game
// viewport.

pub struct TextRenderer {
    pub glyph_brush: wgpu_glyph::GlyphBrush<()>,
    width: u32,
    height: u32,
}

impl TextRenderer {
    // Width, height of the game viewport
    pub fn init(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let font = wgpu_glyph::ab_glyph::FontArc::try_from_slice(super::fonts::INCONSOLATA)
            .expect("Load font");
        TextRenderer {
            glyph_brush: GlyphBrushBuilder::using_font(font)
                .texture_filter_method(wgpu::FilterMode::Linear)
                .build(device, wgpu::TextureFormat::Bgra8UnormSrgb),
            width,
            height,
        }
    }

    pub fn render_direct(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        section: &wgpu_glyph::Section,
    ) {
        self.glyph_brush.queue(section);
        let result = self.glyph_brush.draw_queued(
            &device,
            staging_belt,
            encoder,
            texture_view,
            self.width,
            self.height,
        );
        if !result.is_ok() {
            error!("Failed to draw glyph: {}", result.unwrap_err());
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        text: &str,
    ) {
        self.render_direct(
            device,
            staging_belt,
            texture_view,
            encoder,
            &wgpu_glyph::Section {
                text: vec![wgpu_glyph::Text::new(text)],
                screen_position: (self.width as f32 / 2.0, self.height as f32 / 2.0),
                bounds: (self.width as f32, self.height as f32),
                layout: wgpu_glyph::Layout::default()
                    .h_align(wgpu_glyph::HorizontalAlign::Center)
                    .v_align(wgpu_glyph::VerticalAlign::Center),
                ..wgpu_glyph::Section::default()
            },
        );
    }
}
