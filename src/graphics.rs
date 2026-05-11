use spout::background;
use spout::bloom;
use spout::game_params;
use spout::render;
use spout::ship;
use spout::text;
use spout::title_overlay;
use spout::touch_zone_indicator;
use spout::ui;

pub(crate) struct Graphics {
    pub(crate) game_view_texture: wgpu::TextureView,
    pub(crate) title_ui_view: wgpu::TextureView,
    pub(crate) upscaled_view: wgpu::TextureView,
    pub(crate) bloom: bloom::Bloom,
    pub(crate) renderer: render::Render,
    pub(crate) title_overlay: title_overlay::TitleOverlay,
    pub(crate) ship_renderer: ship::ShipRenderer,
    pub(crate) background: background::BackgroundRenderer,
    /// Renders into the game view (240x135) — pixel-perfect with terrain/particles.
    pub(crate) game_text: text::TextRenderer,
    pub(crate) ui: ui::UiRenderer,
    pub(crate) staging_belt: wgpu::util::StagingBelt,
    pub(crate) touch_zone_indicator: touch_zone_indicator::TouchZoneIndicator,
    /// Debug overlay: FPS counter rendered at display resolution. Debug builds only.
    #[cfg(debug_assertions)]
    pub(crate) overlay_text: text::TextRenderer,
}

impl Graphics {
    pub(crate) fn new(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        game_params: &game_params::GameParams,
    ) -> Self {
        let game_view_texture = make_texture(
            device,
            game_params.viewport_width,
            game_params.viewport_height,
        );
        let title_ui_view = make_texture(
            device,
            game_params.viewport_width,
            game_params.viewport_height,
        );
        let upscaled_view = make_texture(device, config.width, config.height);

        let bloom = bloom::Bloom::new(
            device,
            config.width,
            config.height,
            &upscaled_view,
            &game_params.visual_params,
        );
        let renderer = render::Render::init(
            config,
            game_params,
            adapter,
            device,
            queue,
            &game_view_texture,
            &upscaled_view,
            bloom.bloom_view(),
        );
        let title_overlay = title_overlay::TitleOverlay::new(
            device,
            config.format,
            &title_ui_view,
            config.width,
            config.height,
            game_params.viewport_width,
            game_params.viewport_height,
        );
        let ship_renderer = ship::ShipRenderer::init(device);
        let background = background::BackgroundRenderer::init(device, queue);
        let game_text = text::TextRenderer::init(
            device,
            queue,
            bloom::GAME_VIEW_FORMAT,
            game_params.viewport_width,
            game_params.viewport_height,
            text::YDirection::Up,
            text::Font::O4b11,
        );
        let ui = ui::UiRenderer::new(
            device,
            bloom::GAME_VIEW_FORMAT,
            game_params.viewport_width,
            game_params.viewport_height,
            text::YDirection::Up,
        );
        #[cfg(debug_assertions)]
        let overlay_text = text::TextRenderer::init(
            device,
            queue,
            config.format,
            config.width,
            config.height,
            text::YDirection::Down,
            text::Font::O4b11,
        );
        let touch_zone_indicator = touch_zone_indicator::TouchZoneIndicator::new(
            device,
            queue,
            config.format,
            config.width,
            config.height,
        );

        // Chunk size covers one terrain tile upload; the belt grows as needed.
        let staging_belt = wgpu::util::StagingBelt::new(
            device.clone(),
            (game_params.level_width * game_params.level_height * 4) as u64,
        );

        Self {
            game_view_texture,
            title_ui_view,
            upscaled_view,
            bloom,
            renderer,
            title_overlay,
            ship_renderer,
            background,
            game_text,
            ui,
            staging_belt,
            touch_zone_indicator,
            #[cfg(debug_assertions)]
            overlay_text,
        }
    }

    pub(crate) fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        game_params: &game_params::GameParams,
    ) {
        let new_upscaled = make_texture(device, config.width, config.height);
        self.bloom = bloom::Bloom::new(
            device,
            config.width,
            config.height,
            &new_upscaled,
            &game_params.visual_params,
        );
        self.renderer
            .resize(config, device, &new_upscaled, self.bloom.bloom_view());
        self.upscaled_view = new_upscaled;
        self.title_overlay.resize_with_game(
            queue,
            config.width,
            config.height,
            game_params.viewport_width,
            game_params.viewport_height,
        );
        self.touch_zone_indicator
            .resize(queue, config.width, config.height);
        #[cfg(debug_assertions)]
        self.overlay_text.resize(queue, config.width, config.height);
    }
}

fn make_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: bloom::GAME_VIEW_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}
