use winit::event::WindowEvent;

gflags::define! {
    --log_filter: &str = "warn,spout=info"
}
gflags::define! {
    -h, --help = false
}

// "Framework" for a windowed executable.
pub trait Example: 'static + Sized {
    fn init(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) -> (Self, Option<wgpu::CommandBuffer>);
    fn resize(
        &mut self,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) -> Option<wgpu::CommandBuffer>;
    fn handle_event(&mut self, event: WindowEvent);
    fn render(
        &mut self,
        frame: &wgpu::SwapChainOutput,
        device: &wgpu::Device,
    ) -> wgpu::CommandBuffer;
}

async fn run_async<E: Example>(title: &str) {
    use winit::{
        event,
        event_loop::{ControlFlow, EventLoop},
    };

    gflags::parse();
    if HELP.flag {
        gflags::print_help_and_exit(0);
    }
    scrub_log::init_with_filter_string(LOG_FILTER.flag).unwrap();

    let event_loop = EventLoop::new();
    log::info!("Initializing the window...");

    let (window, size, surface) = {
        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_decorations(false)
            .with_inner_size(winit::dpi::Size::from(winit::dpi::LogicalSize::new(
                640 * 2,
                360 * 2,
            )))
            .build(&event_loop)
            .unwrap();
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);
        (window, size, surface)
    };

    window.set_cursor_visible(false);

    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: Some(&surface),
        },
        wgpu::BackendBit::PRIMARY,
    )
    .await
    .unwrap();

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        })
        .await;

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Immediate,
    };
    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    log::info!("Initializing the example...");
    let (mut example, init_command_buf) = E::init(&sc_desc, &device);
    if let Some(command_buf) = init_command_buf {
        queue.submit(&[command_buf]);
    }
    let mut last_frame_start = std::time::Instant::now();

    log::info!("Entering render loop...");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };
        match event {
            event::Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                log::info!("Resizing to {:?}", size);
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
                let command_buf = example.resize(&sc_desc, &device);
                if let Some(command_buf) = command_buf {
                    queue.submit(&[command_buf]);
                }
            }
            event::Event::WindowEvent { event, .. } => match event {
                // TODO factor out a better way to handle user requested exits.
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Q),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Escape),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {
                    example.handle_event(event);
                }
            },
            event::Event::MainEventsCleared => window.request_redraw(),
            event::Event::RedrawRequested(_) => {
                let frame = swap_chain
                    .get_next_texture()
                    .expect("Timeout when acquiring next swap chain texture");
                let cpu_time_start = std::time::Instant::now();
                let command_buf = example.render(&frame, &device);
                let cpu_time = cpu_time_start.elapsed();
                let gpu_time_start = std::time::Instant::now();
                queue.submit(&[command_buf]);
                device.poll(wgpu::Maintain::Wait);
                let gpu_time = gpu_time_start.elapsed();
                let frame_time = last_frame_start.elapsed();
                last_frame_start = std::time::Instant::now();
                log::info!(
                    "Frame time: {:?}, GPU time: {:?}, CPU time: {:?}",
                    frame_time,
                    gpu_time,
                    cpu_time
                );
            }
            _ => (),
        }
    });
}

pub fn run<E: Example>(title: &str) {
    futures::executor::block_on(run_async::<E>(title));
}

// This allows treating the framework as a standalone example,
// thus avoiding listing the example names in `Cargo.toml`.
#[allow(dead_code)]
fn main() {}
