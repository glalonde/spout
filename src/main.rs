#[path = "../examples/framework.rs"]
mod framework;
use futures::task::{LocalSpawn, LocalSpawnExt};
use log::{error, info};

gflags::define! {
    --config: &str = "game_config.toml"
}

#[derive(Debug, Copy, Clone)]
struct InputState {
    forward: bool,
    left: bool,
    right: bool,
    pause: bool,
}
impl InputState {
    pub fn default() -> Self {
        InputState {
            forward: false,
            left: false,
            right: false,
            pause: false,
        }
    }
}

#[derive(Debug)]
struct GameState {
    input_state: InputState,
    prev_input_state: InputState,
    ship_state: spout::ship::ShipState,
    score: i32,
    paused: bool,
}

struct Example {
    game_params: spout::game_params::GameParams,
    fps: spout::fps_estimator::FpsEstimator,
    state: GameState,
    level_manager: spout::level_manager::LevelManager,
    staging_belt: wgpu::util::StagingBelt,
    compute_locals: spout::particle_system::ComputeLocals,
    pre_glow_texture: wgpu::TextureView,
    post_glow_texture: wgpu::TextureView,
    game_view_texture: wgpu::TextureView,
    terrain_renderer: spout::terrain_renderer::TerrainRenderer,
    particle_renderer: spout::particle_system::ParticleRenderer,
    glow_renderer: spout::glow_pass::GlowRenderer,
    ship_renderer: spout::ship::ShipRenderer,
    viewport: spout::viewport::Viewport,
    debug_overlay: spout::debug_overlay::DebugOverlay,
    text_renderer: spout::text_renderer::TextRenderer,
    game_viewport: spout::game_viewport::GameViewport,
    game_time: std::time::Duration,
}

impl Example {
    // Update pre-render cpu logic
    fn update_state(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) -> f32 {
        let input_state = &self.state.input_state;
        let delta_t = self.fps.tick();
        let dt = delta_t.as_secs_f32();
        if input_state.pause && !self.state.prev_input_state.pause {
            // new pause signal.
            self.state.paused = !self.state.paused;
        }
        if self.state.paused {
            return 0.0;
        } else {
            self.game_time += delta_t;
        }

        let ship_state = &mut self.state.ship_state;

        // Update "ship"
        let rotation: spout::ship::RotationDirection = match (input_state.left, input_state.right) {
            (true, false) => spout::ship::RotationDirection::CCW,
            (false, true) => spout::ship::RotationDirection::CW,
            _ => spout::ship::RotationDirection::None,
        };
        ship_state.update(dt, input_state.forward, rotation);

        // TODO update scrolling state here.
        let ship_height = spout::int_grid::get_outer_grid(ship_state.position[1]) as i32
            - spout::int_grid::half_outer_grid_size() as i32;
        self.state.score = std::cmp::max(ship_height, self.state.score as i32);
        let viewport_bottom_height =
            self.state.score - (self.game_params.viewport_height / 2) as i32;
        self.level_manager
            .sync_height(device, std::cmp::max(0, viewport_bottom_height), encoder);
        self.terrain_renderer.update_render_state(
            device,
            &self.game_params,
            &self.level_manager,
            encoder,
        );
        self.compute_locals.update_state(
            device,
            &self.game_params,
            &self.level_manager,
            dt,
            encoder,
        );

        // Emit particles
        if input_state.forward {
            self.compute_locals.emitter.emit_over_time(
                device,
                encoder,
                dt,
                &ship_state.emit_params,
            );
        }
        dt
    }

    fn make_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: width,
                    height: height,
                    depth: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                label: None,
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn read_config_from_file(path: &str) -> anyhow::Result<spout::game_params::GameParams> {
        let params = std::fs::read_to_string(path)?.parse()?;
        Ok(params)
    }

    fn get_game_config() -> spout::game_params::GameParams {
        match Example::read_config_from_file(CONFIG.flag) {
            Ok(params) => params,
            Err(e) => {
                error!("Failed to parse config file({}): {:?}", CONFIG.flag, e);
                spout::game_params::GameParams::default()
            }
        }
    }
}

impl framework::Example for Example {
    fn init(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        log::info!("Running!");
        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let game_params = Example::get_game_config();

        let width = game_params.viewport_width;
        let height = game_params.viewport_height;
        let system_params = spout::particle_system::SystemParams {
            width,
            height,
            max_particle_life: 5.0,
        };
        let level_manager =
            spout::level_manager::LevelManager::init(device, &game_params, 0, &mut init_encoder);

        let compute_locals = spout::particle_system::ComputeLocals::init(
            device,
            &system_params,
            &game_params,
            &level_manager,
            &mut init_encoder,
        );
        let pre_glow_texture_view = Example::make_texture(
            device,
            game_params.viewport_width,
            game_params.viewport_height,
        );
        let post_glow_texture_view = Example::make_texture(
            device,
            game_params.viewport_width,
            game_params.viewport_height,
        );
        let game_view_texture_view = Example::make_texture(
            device,
            game_params.viewport_width,
            game_params.viewport_height,
        );

        let terrain_renderer = spout::terrain_renderer::TerrainRenderer::init(
            device,
            &compute_locals,
            &game_params,
            &level_manager,
        );

        let particle_renderer = spout::particle_system::ParticleRenderer::init(
            device,
            &compute_locals,
            &mut init_encoder,
        );
        let glow_renderer = spout::glow_pass::GlowRenderer::init(device, &pre_glow_texture_view);
        let viewport = spout::viewport::Viewport::init(
            sc_desc,
            device,
            &game_view_texture_view,
            width,
            height,
            &mut init_encoder,
        );

        let ship_position = [
            spout::int_grid::set_values_relative(system_params.width / 4, 0),
            spout::int_grid::set_values_relative(system_params.height / 4, 0),
        ];

        let text_renderer = spout::text_renderer::TextRenderer::init(
            device,
            game_params.viewport_width,
            game_params.viewport_height,
        );

        let game_viewport =
            spout::game_viewport::GameViewport::init(device, &post_glow_texture_view);
        let fps_controller = spout::fps_estimator::FpsEstimator::new(game_params.fps);
        let this = Example {
            game_params,
            fps: fps_controller,
            state: GameState {
                input_state: InputState::default(),
                prev_input_state: InputState::default(),
                ship_state: spout::ship::ShipState::init_from_flags(ship_position),
                score: 0,
                paused: false,
            },
            level_manager,
            staging_belt: wgpu::util::StagingBelt::new(1024),
            compute_locals: compute_locals,
            pre_glow_texture: pre_glow_texture_view,
            post_glow_texture: post_glow_texture_view,
            game_view_texture: game_view_texture_view,

            terrain_renderer,
            particle_renderer,
            glow_renderer,
            ship_renderer: spout::ship::ShipRenderer::init(
                device,
                system_params.width,
                system_params.height,
            ),
            viewport,
            debug_overlay: spout::debug_overlay::DebugOverlay::init(device, sc_desc),
            text_renderer,
            game_viewport,
            game_time: std::time::Duration::new(0, 0),
        };
        if this.game_params.music_starts_on {
            // let _ = spout::music_player::start_music_player_thread();
        }
        queue.submit(Some(init_encoder.finish()));
        this
    }
    fn update(&mut self, event: winit::event::WindowEvent) {
        macro_rules! bind_keys {
            ($input:expr, $($pat:pat => $result:expr),*) => (
                            match $input {
                                    $(
                            winit::event::KeyboardInput {
                                virtual_keycode: Some($pat),
                                state,
                                ..
                            } => match state {
                                winit::event::ElementState::Pressed =>  $result = true,
                                winit::event::ElementState::Released => $result = false,
                            }
                        ),*
                    _ => (),
                }
            );
        }
        match event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => bind_keys!(input,
                winit::event::VirtualKeyCode::W => self.state.input_state.forward,
                winit::event::VirtualKeyCode::A => self.state.input_state.left,
                winit::event::VirtualKeyCode::P => self.state.input_state.pause,
                winit::event::VirtualKeyCode::D => self.state.input_state.right),
            _ => (),
        }
    }
    fn resize(
        &mut self,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let viewport_aspect_ratio =
            self.game_params.viewport_width as f64 / self.game_params.viewport_height as f64;
        let new_window_aspect_ratio = sc_desc.width as f64 / sc_desc.height as f64;
        info!("Resizing: ({}, {})", sc_desc.width, sc_desc.height);
        info!("Game aspect ratio: {}", viewport_aspect_ratio);
        info!("Window aspect ratio: {}", new_window_aspect_ratio);
        self.viewport.resize(sc_desc, device, &mut encoder);
        self.debug_overlay.resize(sc_desc);
        queue.submit(Some(encoder.finish()));
    }

    fn render(
        &mut self,
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        spawner: &impl LocalSpawn,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let dt = self.update_state(device, &mut encoder);
        self.state.prev_input_state = self.state.input_state;

        if !self.state.paused {
            {
                // Clear the density texture.
                self.compute_locals.clear_density(&mut encoder);
            }
            {
                // Update the particles state and density texture.
                self.compute_locals
                    .compute(&self.level_manager, &mut encoder);
            }
        }
        {
            {
                // Clear the pre-glow pass
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &self.pre_glow_texture,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });
            }
            {
                // Render the terrain
                self.terrain_renderer.render(
                    &self.level_manager,
                    &self.pre_glow_texture,
                    &mut encoder,
                );
            }
            {
                // Render the density texture.
                self.particle_renderer
                    .render(&mut encoder, &self.pre_glow_texture);
            }
            {
                // Render the particle glow pass.
                self.glow_renderer
                    .render(&mut encoder, &self.post_glow_texture);
            }
            {
                // Render the ship.
                self.ship_renderer.render(
                    &self.post_glow_texture,
                    device,
                    &self.state.ship_state,
                    &self.level_manager,
                    &mut encoder,
                );
            }
        }

        // Flip the frame vertically. Before this everything is blitted in "world
        // coordinates".
        {
            self.game_viewport
                .render(&mut encoder, &self.game_view_texture);
        }
        if self.state.paused {
            // Display pause screen
            let width = self.game_params.viewport_width;
            let height = self.game_params.viewport_height;
            let paused_text = wgpu_glyph::Text::new("Paused")
                .with_color([1.0, 0.2, 0.2, 1.0])
                .with_scale(wgpu_glyph::ab_glyph::PxScale { x: 20.0, y: 20.0 });
            self.text_renderer.render_direct(
                device,
                &mut self.staging_belt,
                &self.game_view_texture,
                &mut encoder,
                &wgpu_glyph::Section {
                    text: vec![paused_text],
                    screen_position: (width as f32 / 2.0, height as f32 / 2.0),
                    bounds: (width as f32, height as f32),
                    layout: wgpu_glyph::Layout::default()
                        .h_align(wgpu_glyph::HorizontalAlign::Center)
                        .v_align(wgpu_glyph::VerticalAlign::Center),
                    ..wgpu_glyph::Section::default()
                },
            );
        }
        {
            // Render the score
            let width = self.game_params.viewport_width as f32;
            let height = self.game_params.viewport_height as f32;

            let score_t = format!("{}", self.state.score);
            let score_text = wgpu_glyph::Text::new(&score_t)
                .with_color([0.2, 1.0, 0.2, 1.0])
                .with_scale(wgpu_glyph::ab_glyph::PxScale { x: 28.0, y: 28.0 });
            self.text_renderer.render_direct(
                device,
                &mut self.staging_belt,
                &self.game_view_texture,
                &mut encoder,
                &wgpu_glyph::Section {
                    text: vec![score_text],
                    screen_position: (width, 0.0),
                    bounds: (width as f32 / 2.0, height as f32 / 2.0),
                    layout: wgpu_glyph::Layout::default_single_line()
                        .h_align(wgpu_glyph::HorizontalAlign::Right)
                        .v_align(wgpu_glyph::VerticalAlign::Top),
                    ..wgpu_glyph::Section::default()
                },
            );
        }
        {
            self.viewport.render(&frame, &mut encoder);
        }
        {
            self.debug_overlay
                .render(&device, &frame.view, &mut encoder, 1.0 / dt as f64);
        }

        self.staging_belt.finish();
        queue.submit(Some(encoder.finish()));

        let belt_future = self.staging_belt.recall();
        spawner.spawn_local(belt_future).unwrap();
    }

    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::default()
    }
}

fn main() {
    framework::run::<Example>("Particle System");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_config() {
        let serialized = std::fs::read_to_string(CONFIG.flag).unwrap();
        let deserialized_config: spout::game_params::GameParams =
            toml::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized_config);
    }
}
