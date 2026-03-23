use std::future::Future;
use std::sync::Arc;
use web_time::Instant;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;
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

/// GPU state produced by async initialization. Separated from `FrameworkApp`
/// so it can be written into a shared cell from a spawned future on WASM.
struct GpuState<E: Example> {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    example: E,
}

#[allow(dead_code)]
struct FrameworkApp<E: Example> {
    title: String,
    /// Set as soon as the window is created, before GPU init completes.
    window: Option<Arc<winit::window::Window>>,
    /// Set once GPU init completes.
    gpu: Option<GpuState<E>>,
    spawner: Spawner,
    last_frame: Instant,
    frame_count: u32,
    accum_time: f32,
    /// On WASM, `init_gpu` runs via `spawn_local`. The completed result lands
    /// here and is picked up in `about_to_wait`.
    #[cfg(target_arch = "wasm32")]
    pending_gpu: std::rc::Rc<std::cell::RefCell<Option<GpuState<E>>>>,
}

impl<E: Example> FrameworkApp<E> {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            window: None,
            gpu: None,
            spawner: Spawner::new(),
            last_frame: Instant::now(),
            frame_count: 0,
            accum_time: 0.0,
            #[cfg(target_arch = "wasm32")]
            pending_gpu: std::rc::Rc::new(std::cell::RefCell::new(None)),
        }
    }
}

/// Performs all async wgpu initialization and returns the completed `GpuState`.
///
/// On native this is driven by `pollster::block_on`; on WASM it is driven by
/// `wasm_bindgen_futures::spawn_local` so the JS event loop keeps running while
/// `request_adapter` / `request_device` resolve their underlying Promises.
async fn init_gpu<E: Example>(
    window: Arc<winit::window::Window>,
    display_handle: winit::event_loop::OwnedDisplayHandle,
) -> GpuState<E> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle_from_env(
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
    log::info!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

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

    GpuState {
        adapter,
        device,
        queue,
        surface,
        config,
        example,
    }
}

impl<E: Example> ApplicationHandler for FrameworkApp<E> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Only initialize once (guards against repeated calls on Android/iOS).
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window({
                    let attrs = winit::window::Window::default_attributes().with_title(&self.title);
                    #[cfg(target_arch = "wasm32")]
                    let attrs = attrs.with_append(true);
                    attrs
                })
                .expect("Failed to create window"),
        );
        self.window = Some(window.clone());

        let display_handle = event_loop.owned_display_handle();

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.gpu = Some(pollster::block_on(init_gpu::<E>(window, display_handle)));
        }

        // On WASM, block_on would deadlock: request_adapter / request_device
        // back onto JS Promises that only resolve when the JS event loop runs.
        // spawn_local schedules the future cooperatively on that event loop.
        // Results are written into `pending_gpu` and picked up in about_to_wait.
        #[cfg(target_arch = "wasm32")]
        {
            let pending = std::rc::Rc::clone(&self.pending_gpu);
            wasm_bindgen_futures::spawn_local(async move {
                let gpu = init_gpu::<E>(window, display_handle).await;
                *pending.borrow_mut() = Some(gpu);
            });
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Drop events until GPU init is complete.
        if self.gpu.is_none() {
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
                let gpu = self.gpu.as_mut().unwrap(); // safe: checked above
                let max_dim = gpu.adapter.limits().max_texture_dimension_2d;
                if size.width == 0 || size.height == 0 {
                    // minimized, skip
                } else if size.width > max_dim || size.height > max_dim {
                    log::warn!("Resize {:?} exceeds adapter limit {}", size, max_dim);
                } else {
                    log::info!("Resizing to {:?}", size);
                    gpu.config.width = size.width;
                    gpu.config.height = size.height;
                    gpu.surface.configure(&gpu.device, &gpu.config);
                    gpu.example.resize(&gpu.config, &gpu.device, &gpu.queue);
                }
            }
            WindowEvent::RedrawRequested => {
                self.accum_time += self.last_frame.elapsed().as_secs_f32();
                self.last_frame = Instant::now();
                self.frame_count += 1;
                if self.frame_count == 100 {
                    log::info!(
                        "Avg frame time {}ms",
                        self.accum_time * 1000.0 / self.frame_count as f32
                    );
                    self.accum_time = 0.0;
                    self.frame_count = 0;
                }

                self.spawner.run_until_stalled();

                let gpu = self.gpu.as_mut().unwrap(); // safe: checked above
                let window = self.window.as_ref().unwrap(); // safe: set before gpu

                let frame = match gpu.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(frame) => frame,
                    wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
                    wgpu::CurrentSurfaceTexture::Outdated => {
                        gpu.surface.configure(&gpu.device, &gpu.config);
                        match gpu.surface.get_current_texture() {
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

                gpu.example
                    .render(&view, &gpu.device, &gpu.queue, &self.spawner, window);
                frame.present();
            }
            event => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.example.update(event);
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // On WASM: check if async GPU init has completed and transfer the result.
        #[cfg(target_arch = "wasm32")]
        if self.gpu.is_none() {
            if let Some(gpu) = self.pending_gpu.borrow_mut().take() {
                self.gpu = Some(gpu);
            }
        }

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
    #[cfg(not(target_arch = "wasm32"))]
    scrub_log::init_with_filter_string("info").unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("could not initialize console_log");
    }

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = FrameworkApp::<E>::new(title);
    event_loop.run_app(&mut app).expect("Event loop error");
}

// This allows treating the framework as a standalone example,
// thus avoiding listing the example names in `Cargo.toml`.
#[allow(dead_code)]
fn main() {}
