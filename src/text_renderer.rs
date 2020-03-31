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
            glyph_brush: GlyphBrushBuilder::using_font_bytes(super::fonts::VISITOR)
                .unwrap()
                .texture_filter_method(wgpu::FilterMode::Linear)
                .build(device, wgpu::TextureFormat::Bgra8UnormSrgb),
            width,
            height,
        }
    }

    pub fn render_direct(
        &mut self,
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        section: &wgpu_glyph::Section,
    ) {
        self.glyph_brush.queue(section);
        let result =
            self.glyph_brush
                .draw_queued(&device, encoder, texture_view, self.width, self.height);
        if !result.is_ok() {
            error!("Failed to draw glyph: {}", result.unwrap_err());
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        text: &str,
    ) {
        self.render_direct(
            device,
            texture_view,
            encoder,
            &wgpu_glyph::Section {
                text,
                screen_position: (self.width as f32 / 2.0, self.height as f32 / 2.0),
                color: [1.0, 1.0, 1.0, 1.0],
                scale: wgpu_glyph::Scale { x: 20.0, y: 20.0 },
                bounds: (self.width as f32, self.height as f32),
                layout: wgpu_glyph::Layout::default()
                    .h_align(wgpu_glyph::HorizontalAlign::Center)
                    .v_align(wgpu_glyph::VerticalAlign::Center),
                ..wgpu_glyph::Section::default()
            },
        );
    }

    pub fn make<'a>(&mut self) -> SectionBuilder<'a> {
        SectionBuilder::new(self.width, self.height)
    }
}

pub struct SectionBuilder<'a> {
    width: u32,
    height: u32,
    text_specs: wgpu_glyph::Section<'a>,
}

impl<'a> SectionBuilder<'a> {
    pub fn new(width: u32, height: u32) -> SectionBuilder<'a> {
        SectionBuilder {
            width,
            height,
            text_specs: wgpu_glyph::Section::default(),
        }
    }

    pub fn text(&'a mut self, text: &'a str) -> &mut SectionBuilder<'a> {
        self.text_specs.text = text;
        self
    }

    pub fn color(&'a mut self, color: [f32; 4]) -> &mut SectionBuilder<'a> {
        self.text_specs.color = color;
        self
    }

    pub fn finish(&self) -> wgpu_glyph::Section {
        self.text_specs
    }
}
