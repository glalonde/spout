use winit::{
    dpi::LogicalSize,
    error::OsError,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// Makes a window and an eventloop and returns the event loop, already running.
fn make_default_window<T: Into<String>>(
    title: T,
    size: LogicalSize,
) -> Result<EventLoop<()>, OsError> {
    let event_loop = EventLoop::new();
    let output = WindowBuilder::new()
        .with_title(title)
        .with_inner_size(size)
        .build(&event_loop);
    output.map(|window| {
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
                //
                // It's preferrable to render in this event rather than in EventsCleared, since
                // rendering in here allows the program to gracefully handle redraws requested
                // by the OS.
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            }
            // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
            // dispatched any events. This is ideal for games and similar applications.
            _ => *control_flow = ControlFlow::Poll,
            // ControlFlow::Wait pauses the event loop if no events are available to process.
            // This is ideal for non-game applications that only update in response to user
            // input, and uses significantly less power/CPU time than ControlFlow::Poll.
            // _ => *control_flow = ControlFlow::Wait,
        });
    })
}

fn main() {
    make_default_window(
        "Hello Window!",
        LogicalSize {
            width: 800.0,
            height: 600.0,
        },
    )
    .expect("Could not create window :(");
}
