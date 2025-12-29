use std::rc::Rc;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::core::App;
use crate::graphics::Renderer;

struct AppState<A: App> {
    app: A,
    window: Rc<Window>,
    renderer: Renderer,
}

struct WinitHandler<A: App> {
    pending_app: Option<A>,
    context: softbuffer::Context<winit::event_loop::OwnedDisplayHandle>,
    size: PhysicalSize<u32>,
    resizable: bool,
    title: String,
    state: Option<AppState<A>>,
}

impl<A: App> WinitHandler<A> {
    fn new(app: A, context: softbuffer::Context<winit::event_loop::OwnedDisplayHandle>, size: PhysicalSize<u32>, resizable: bool, title: String) -> Self {
        Self {
            pending_app: Some(app),
            context,
            size,
            resizable,
            title,
            state: None,
        }
    }
}

impl<A: App> ApplicationHandler for WinitHandler<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(app) = self.pending_app.take() else {
            return;
        };

        let attrs = WindowAttributes::default()
            .with_inner_size(self.size)
            .with_resizable(self.resizable)
            .with_title(&self.title);

        let window = Rc::new(
            event_loop
                .create_window(attrs)
                .expect("Failed to create window"),
        );

        let renderer = Renderer::new(&self.context, window.clone());

        self.state = Some(AppState {
            app,
            window,
            renderer,
        });
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = self.state.take() {
            self.pending_app = Some(state.app);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        event_loop.set_control_flow(ControlFlow::Wait);

        let Some(state) = &mut self.state else {
            return;
        };

        if window_id != state.window.id() {
            return;
        }

        match &event {
            WindowEvent::RedrawRequested => {
                let size = state.window.inner_size();
                state.renderer.resize(size.width, size.height);
                state.renderer.render(state.app.view());
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }

        if state.app.on_event(&event) {
            state.window.request_redraw();
        }
    }
}

/// Configuration for running an application.
pub struct RunConfig {
    pub resizable: bool,
    pub title: String,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            resizable: false,
            title: String::from("Curvy"),
        }
    }
}

impl RunConfig {
    /// Set the window title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
}

/// Run an application with the given configuration.
/// The window size is determined by the app's view size.
pub fn run<A: App + 'static>(app: A, config: RunConfig) {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let context = softbuffer::Context::new(event_loop.owned_display_handle())
        .expect("Failed to create softbuffer context");

    // Get the size from the app's view
    let (width, height) = app.view().size();
    let size = PhysicalSize::new(width, height);
    let mut handler = WinitHandler::new(app, context, size, config.resizable, config.title);

    event_loop.run_app(&mut handler).expect("Event loop failed");
}
