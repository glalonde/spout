use log::error;
use wgpu_glyph::GlyphBrushBuilder;

gflags::define! {
    --fps_overlay: bool = true
}

static INCONSOLATA: &[u8] = include_bytes!("../assets/Inconsolata-Regular.ttf");

// Renders a text overlay as one of the final stages in the rendering pipeline.

pub struct DebugOverlay {
    pub glyph_brush: wgpu_glyph::GlyphBrush<'static, ()>,
    width: u32,
    height: u32,
}

impl DebugOverlay {
    pub fn init(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        DebugOverlay {
            glyph_brush: GlyphBrushBuilder::using_font_bytes(INCONSOLATA)
                .texture_filter_method(wgpu::FilterMode::Nearest)
                .build(device, wgpu::TextureFormat::Bgra8UnormSrgb),
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
        let section = wgpu_glyph::Section {
            text: &format!("FPS: {:0.2}s", fps),
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
