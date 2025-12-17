use std::num::NonZeroU32;
use std::rc::Rc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
use winit::dpi::PhysicalSize;

#[path = "winit_app.rs"]
mod winit_app;

#[path="ppm_consumer.rs"]
mod ppm_consumer;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let context = softbuffer::Context::new(event_loop.owned_display_handle()).unwrap();
    let background = ppm_consumer::read_ppm_file("src/image.ppm");
    let size = PhysicalSize::new(background.width, background.height);

    let mut app = winit_app::WinitAppBuilder::with_init(
        |elwt| {
            let window = elwt.create_window(Window::default_attributes().with_inner_size(size));
            Rc::new(window.unwrap())
        },
        |_elwt, window| softbuffer::Surface::new(&context, window.clone()).unwrap(),
    )
    .with_event_handler(|window, surface, window_id, event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        if window_id != window.id() {
            return;
        }

        match event {
            WindowEvent::RedrawRequested => {
                let Some(surface) = surface else {
                    eprintln!("RedrawRequested fired before Resumed or after Suspended");
                    return;
                };
                let size = window.inner_size();
                surface
                    .resize(
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    )
                    .unwrap();

                let width = size.width;
                let height = size.height;
                println!("window size: width={width}, height={height}");
                let mut buffer = surface.buffer_mut().unwrap();
                for (x, y, pixel) in background.image.enumerate_pixels() {
                    let [red, blue, green] = pixel.0;
                    let index = (y as usize * width as usize) + x as usize;
                    buffer[index as usize] = (green as u32) | (blue as u32) << 8 | (red as u32) << 16;
                }

                buffer.present().unwrap();
            }
            WindowEvent::CloseRequested => {
                elwt.exit();
            }
            _ => {}
        }
    });

    event_loop.run_app(&mut app).unwrap();
}
