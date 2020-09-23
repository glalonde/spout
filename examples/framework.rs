use futures::task::LocalSpawn;
use winit::event::WindowEvent;

gflags::define! {
    --log_filter: &str = "warn,spout=info"
}
gflags::define! {
    -h, --help = false
}

// "Framework" for a windowed executable.
pub trait Example: 'static + Sized {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::default()
    }
    fn init(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self;
    fn resize(
        &mut self,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );
    fn update(&mut self, event: WindowEvent);
    fn render(
        &mut self,
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        spawner: &impl LocalSpawn,
    );
}

struct AppState<E: Example> {
    // This becomes available when we get a vulkan instance
    setup: Option<Setup>,
    // This will come and go as the app is suspended an resumed
    swap_chain: Option<wgpu::SwapChain>,
    example: Option<E>,
}

struct Setup {
    instance: wgpu::Instance,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

fn setup_window<E: Example>(
    title: &str,
) -> (winit::window::Window, winit::event_loop::EventLoop<()>) {
    gflags::parse();
    if HELP.flag {
        gflags::print_help_and_exit(0);
    }

    scrub_log::init_with_filter_string(LOG_FILTER.flag).unwrap_or_else(|_| {
        println!("Failed to init logging with arg: {}", LOG_FILTER.flag);
    });

    let event_loop = winit::event_loop::EventLoop::new();
    let mut builder = winit::window::WindowBuilder::new();
    builder = builder
        .with_title(title)
        .with_decorations(false)
        .with_inner_size(winit::dpi::Size::from(winit::dpi::LogicalSize::new(
            1280, 720,
        )));

    let window = builder.build(&event_loop).unwrap();
    (window, event_loop)
}

async fn setup_surface<E: Example>(window: &mut winit::window::Window) -> Setup {
    log::info!("Initializing the surface...");

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let (size, surface) = unsafe {
        let size = window.inner_size();
        let surface = instance.create_surface(window);
        (size, surface)
    };

    log::info!("Requesting adapter...");
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();

    let optional_features = E::optional_features();
    let required_features = E::required_features();
    let adapter_features = adapter.features();
    assert!(
        adapter_features.contains(required_features),
        "Adapter does not support required features for this example: {:?}",
        required_features - adapter_features
    );

    let needed_limits = E::required_limits();

    let trace_dir = std::env::var("WGPU_TRACE");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: (optional_features & adapter_features) | required_features,
                limits: needed_limits,
                shader_validation: true,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .unwrap();

    log::info!("Done with setup!");
    Setup {
        instance,
        size,
        surface,
        adapter,
        device,
        queue,
    }
}

fn start<E: Example>(
    mut window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
) {
    let mut app: AppState<E> = AppState {
        setup: None,
        swap_chain: None,
        example: None,
    };

    let (mut _pool, spawner) = {
        let local_pool = futures::executor::LocalPool::new();
        let spawner = local_pool.spawner();
        (local_pool, spawner)
    };

    // On android it's too soon to create the app (because we can't get the handle to the surface)
    #[cfg(not(target_os = "android"))]
    {
        app.setup = Some(futures::executor::block_on(setup_surface::<E>(&mut window)));
    }

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        // TODO: Allow srgb unconditionally
        format: if cfg!(target_arch = "wasm32") {
            wgpu::TextureFormat::Bgra8Unorm
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        },
        width: 0,
        height: 0,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    event_loop.run(move |event, _, control_flow| {
        let _ = &mut app; // force ownership by the closure
        *control_flow = winit::event_loop::ControlFlow::Poll;
        match event {
            // Create app on resume for android target. Now the handle to the surface should be accessible.
            // #[cfg(target_os = "android")]
            winit::event::Event::Resumed => {
                log::info!("Application was resumed");
                app.setup
                    .replace(futures::executor::block_on(setup_surface::<E>(&mut window)));
                match &app.setup {
                    None => {
                        log::error!("Failed to inialize app on resume.");
                    }
                    Some(setup) => {
                        sc_desc.width = setup.size.width;
                        sc_desc.height = setup.size.height;
                        app.swap_chain
                            .replace(setup.device.create_swap_chain(&setup.surface, &sc_desc));
                        app.example
                            .replace(E::init(&sc_desc, &setup.device, &setup.queue));
                        log::info!("Initialized app");
                    }
                }
            }
            // Destroy app on suspend for android target.
            // #[cfg(target_os = "android")]
            winit::event::Event::Suspended => {
                log::info!("Application was suspended");
                // TODO: Wait for gpu, save state and tear down
                app.example.take();
                app.swap_chain.take();
            }

            winit::event::Event::MainEventsCleared => {
                // Main update logic
                window.request_redraw();
            }
            winit::event::Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => match (&mut app.example, &app.setup) {
                (Some(example), Some(setup)) => {
                    log::info!("Resizing to {:?}", size);
                    sc_desc.width = if size.width == 0 { 1 } else { size.width };
                    sc_desc.height = if size.height == 0 { 1 } else { size.height };
                    example.resize(&sc_desc, &setup.device, &setup.queue);
                    app.swap_chain
                        .replace(setup.device.create_swap_chain(&setup.surface, &sc_desc));
                }
                _ => {}
            },
            winit::event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                            state: winit::event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            virtual_keycode: Some(winit::event::VirtualKeyCode::Q),
                            state: winit::event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                _ => match &mut app.example {
                    Some(example) => {
                        example.update(event);
                    }
                    _ => {}
                },
            },
            winit::event::Event::RedrawRequested(_) => {
                if let Some(setup) = &mut app.setup.as_ref() {
                    let mut frame = if app.swap_chain.is_some() {
                        app.swap_chain.as_mut().unwrap().get_current_frame()
                    } else {
                        app.swap_chain =
                            Some(setup.device.create_swap_chain(&setup.surface, &sc_desc));
                        app.swap_chain.as_mut().unwrap().get_current_frame()
                    };
                    match (&mut app.example, &mut frame) {
                        (Some(example), Ok(frame)) => {
                            example.render(&frame.output, &setup.device, &setup.queue, &spawner);
                        }
                        _ => {}
                    }
                }
            }

            _ => {}
        };
    });
}

pub fn run<E: Example>(title: &str) {
    let (window, event_loop) = setup_window::<E>(title);
    start::<E>(window, event_loop);
}

// This allows treating the framework as a standalone example,
// thus avoiding listing the example names in `Cargo.toml`.
#[allow(dead_code)]
fn main() {}
