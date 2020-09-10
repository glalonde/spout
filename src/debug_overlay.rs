use log::error;
use wgpu_glyph::GlyphBrushBuilder;

// Renders a text overlay as one of the final stages in the rendering pipeline.

pub struct DebugOverlay {
    pub glyph_brush: wgpu_glyph::GlyphBrush<()>,
    staging_belt: wgpu::util::StagingBelt,
    width: u32,
    height: u32,
}

impl DebugOverlay {
    pub fn init(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        let font = wgpu_glyph::ab_glyph::FontArc::try_from_slice(super::fonts::INCONSOLATA)
            .expect("Load font");
        DebugOverlay {
            glyph_brush: GlyphBrushBuilder::using_font(font)
                .texture_filter_method(wgpu::FilterMode::Nearest)
                .build(device, wgpu::TextureFormat::Bgra8UnormSrgb),
            staging_belt: wgpu::util::StagingBelt::new(1024),
            width: sc_desc.width,
            height: sc_desc.height,
        }
    }
    pub fn resize(&mut self, sc_desc: &wgpu::SwapChainDescriptor) {
        self.width = sc_desc.width;
        self.height = sc_desc.height;
    }
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        fps: f64,
    ) {
        self.staging_belt.recall();
        let width = 640;
        let height = 320;
        let text_color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        let debug_start = std::time::Instant::now();
        let render_text = format!("FPS: {:0.2}s", fps);
        let section = wgpu_glyph::Section {
            text: vec![wgpu_glyph::Text::new(&render_text).with_color(text_color)],
            screen_position: (00.0, 00.0),
            bounds: (width as f32, height as f32),
            ..wgpu_glyph::Section::default()
        };
        log::info!("d1: {:?}", debug_start.elapsed());
        self.glyph_brush.queue(section);
        log::info!("d2: {:?}", debug_start.elapsed());
        let result = self.glyph_brush.draw_queued(
            &device,
            &mut self.staging_belt,
            encoder,
            texture_view,
            width,
            height,
        );
        self.staging_belt.finish();
        log::info!("d3: {:?}", debug_start.elapsed());
        if !result.is_ok() {
            error!("Failed to draw glyph: {}", result.unwrap_err());
        }
    }
}
