mod utility;
mod renderer;

use simple_logger::SimpleLogger;
use winit::{
    dpi::PhysicalSize, event::{Event, WindowEvent}, event_loop::EventLoop, window::{Window, WindowBuilder}
};
use utility::Clock;
use renderer::Renderer;

fn main() {
    SimpleLogger::new().init().unwrap();

    log::info!("Initializing client...");

    let mut is_running = true;
    let mut delta_clock = Clock::new();
    let mut delta_time = 0u128;

    let mut dirty_swapchain = false;

    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new()
        .with_title("Industria")
        .with_inner_size(PhysicalSize::new(800, 600))
        .build(&event_loop)
        .expect("Failed to create client window.");

    let renderer = Renderer::new(&window);

    event_loop
        .run(move |event, elwt| {
            match event {
                Event::NewEvents(_) => {

                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized { .. } => dirty_swapchain = true,
                    _ => {}

                }
                _ => {}
            }
        })
        .unwrap();
}
