use std::future::Future;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowId,
};

#[rustfmt::skip]
#[allow(unused)]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[allow(dead_code)]
pub trait Example: 'static + Sized {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::empty(),
            shader_model: wgpu::ShaderModel::Sm5,
            ..wgpu::DownlevelCapabilities::default()
        }
    }
    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::downlevel_webgl2_defaults()
    }
    fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: &winit::window::Window,
    ) -> Self;
    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );
    fn update(&mut self, event: WindowEvent);
    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        spawner: &Spawner,
        window: &winit::window::Window,
    );
}

#[allow(dead_code)]
struct FrameworkApp<E: Example> {
    title: String,
    window: Option<Arc<winit::window::Window>>,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    example: Option<E>,
    spawner: Spawner,
    last_frame: std::time::Instant,
    frame_count: u32,
    accum_time: f32,
}

impl<E: Example> FrameworkApp<E> {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            window: None,
            adapter: None,
            device: None,
            queue: None,
            surface: None,
            config: None,
            example: None,
            spawner: Spawner::new(),
            last_frame: std::time::Instant::now(),
            frame_count: 0,
            accum_time: 0.0,
        }
    }
}

impl<E: Example> ApplicationHandler for FrameworkApp<E> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Only initialize once.
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(winit::window::Window::default_attributes().with_title(&self.title))
                .expect("Failed to create window"),
        );

        let display_handle = event_loop.owned_display_handle();

        pollster::block_on(async {
            let instance =
                wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle_from_env(
                    Box::new(display_handle),
                ));

            let surface = instance
                .create_surface(window.clone())
                .expect("Failed to create surface");

            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .expect("No suitable GPU adapter found on the system!");

            let adapter_info = adapter.get_info();
            println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

            let optional_features = E::optional_features();
            let required_features = E::required_features();
            let adapter_features = adapter.features();
            assert!(
                adapter_features.contains(required_features),
                "Adapter does not support required features for this example: {:?}",
                required_features - adapter_features
            );

            let required_downlevel = E::required_downlevel_capabilities();
            let downlevel = adapter.get_downlevel_capabilities();
            assert!(
                downlevel.shader_model >= required_downlevel.shader_model,
                "Adapter does not support the minimum shader model required: {:?}",
                required_downlevel.shader_model
            );
            assert!(
                downlevel.flags.contains(required_downlevel.flags),
                "Adapter does not support the required downlevel capabilities: {:?}",
                required_downlevel.flags - downlevel.flags
            );

            let needed_limits = E::required_limits().using_resolution(adapter.limits());

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: None,
                    required_features: (optional_features & adapter_features) | required_features,
                    required_limits: needed_limits,
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                    trace: wgpu::Trace::Off,
                })
                .await
                .expect("Unable to find a suitable GPU adapter!");

            let size = window.inner_size();
            let config = surface
                .get_default_config(&adapter, size.width.max(1), size.height.max(1))
                .expect("Surface not supported by the adapter");
            surface.configure(&device, &config);

            log::info!("Initializing the example...");
            let example = E::init(&config, &adapter, &device, &queue, &window);

            self.adapter = Some(adapter);
            self.device = Some(device);
            self.queue = Some(queue);
            self.surface = Some(surface);
            self.config = Some(config);
            self.example = Some(example);
            self.window = Some(window);
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Drop events until wgpu is initialized.
        if self.example.is_none() {
            return;
        }

        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            }
            | WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                let max_dim = self
                    .adapter
                    .as_ref()
                    .unwrap()
                    .limits()
                    .max_texture_dimension_2d;
                if size.width == 0 || size.height == 0 {
                    // minimized, skip
                } else if size.width > max_dim || size.height > max_dim {
                    log::warn!("Resize {:?} exceeds adapter limit {}", size, max_dim);
                } else {
                    log::info!("Resizing to {:?}", size);
                    let config = self.config.as_mut().unwrap();
                    config.width = size.width;
                    config.height = size.height;
                    let surface = self.surface.as_ref().unwrap();
                    let device = self.device.as_ref().unwrap();
                    let queue = self.queue.as_ref().unwrap();
                    surface.configure(device, config);
                    self.example.as_mut().unwrap().resize(config, device, queue);
                }
            }
            WindowEvent::RedrawRequested => {
                self.accum_time += self.last_frame.elapsed().as_secs_f32();
                self.last_frame = std::time::Instant::now();
                self.frame_count += 1;
                if self.frame_count == 100 {
                    println!(
                        "Avg frame time {}ms",
                        self.accum_time * 1000.0 / self.frame_count as f32
                    );
                    self.accum_time = 0.0;
                    self.frame_count = 0;
                }

                self.spawner.run_until_stalled();

                let surface = self.surface.as_ref().unwrap();
                let device = self.device.as_ref().unwrap();
                let config = self.config.as_ref().unwrap();

                let frame = match surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(frame) => frame,
                    wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
                    wgpu::CurrentSurfaceTexture::Outdated => {
                        surface.configure(device, config);
                        match surface.get_current_texture() {
                            wgpu::CurrentSurfaceTexture::Success(f)
                            | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
                            other => {
                                log::warn!("get_current_texture retry failed: {:?}", other);
                                return;
                            }
                        }
                    }
                    other => {
                        log::warn!("get_current_texture: {:?}", other);
                        return;
                    }
                };

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let queue = self.queue.as_ref().unwrap();
                let window = self.window.as_ref().unwrap();
                self.example
                    .as_mut()
                    .unwrap()
                    .render(&view, device, queue, &self.spawner, window);
                frame.present();
            }
            event => {
                if let Some(example) = self.example.as_mut() {
                    example.update(event);
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

pub struct Spawner {
    executor: async_executor::LocalExecutor<'static>,
}

#[allow(unused)]
impl Spawner {
    fn new() -> Self {
        Self {
            executor: async_executor::LocalExecutor::new(),
        }
    }

    #[allow(dead_code)]
    pub fn spawn_local(&self, future: impl Future<Output = ()> + 'static) {
        self.executor.spawn(future).detach();
    }

    fn run_until_stalled(&self) {
        while self.executor.try_tick() {}
    }
}

#[allow(dead_code)]
pub fn run<E: Example>(title: &str) {
    scrub_log::init_with_filter_string("info").unwrap();
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = FrameworkApp::<E>::new(title);
    event_loop.run_app(&mut app).expect("Event loop error");
}

// This allows treating the framework as a standalone example,
// thus avoiding listing the example names in `Cargo.toml`.
#[allow(dead_code)]
fn main() {}
