#[cfg(feature = "dx12")]
use gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;

use log::{debug, error, info, trace, warn};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct HalState {}
#[derive(Debug, Clone, Copy, Default)]
pub struct LocalState {
    pub frame_width: f64,
    pub frame_height: f64,
    pub mouse_x: f64,
    pub mouse_y: f64,
}

impl HalState {
    pub fn draw_clear_frame(&mut self, color: [f32; 4]) -> Result<(), &'static str> {
        unimplemented!()
    }
    pub fn new(window: &Window) -> Result<Self, &'static str> {
        unimplemented!()
    }
}

pub fn do_the_render(
    hal_state: &mut HalState,
    local_state: &LocalState,
) -> Result<(), &'static str> {
    // hal.draw_clear_frame(locals.color())
    let r = (local_state.mouse_x / local_state.frame_width) as f32;
    let g = (local_state.mouse_y / local_state.frame_height) as f32;
    let b = (r + g) * 0.3;
    let a = 1.0;
    hal_state.draw_clear_frame([r, g, b, a])
}

fn main() {
    scrub_log::init().unwrap();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Hello Window!")
        .with_inner_size(LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .build(&event_loop)
        .expect("Could not create window :(");

    let mut hal_state = HalState::new(&window).unwrap();
    let mut local_state = LocalState::default();

    event_loop.run(move |event, _, control_flow| match event {
        Event::EventsCleared => {
            // Application update code.
            // Queue a RedrawRequested event.
            window.request_redraw();
        }
        Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } => {
            // Redraw the application.
            if let Err(e) = do_the_render(&mut hal_state, &local_state) {
                error!("Rendering Error: {:?}", e);
                *control_flow = ControlFlow::Exit
            }
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            info!("The close button was pressed; stopping");
            *control_flow = ControlFlow::Exit
        }
        _ => *control_flow = ControlFlow::Poll,
    });
}
