//! App and context

use std::sync::{Arc, Mutex, OnceLock};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window,
};

use crate::{
    AppEvent,
    backend::{EarlyGraphicsDevice, GraphicsBackend},
};

/// Global app context
static APP_CTX: OnceLock<Mutex<AppCtx>> = OnceLock::new();

/// Initialize an app context if it wasn't already initialised
fn init_app_ctx(new_ctx: AppCtx) {
    assert!(APP_CTX.get().is_none(), "App context already initialized");

    let _ = APP_CTX.set(Mutex::new(new_ctx));
}

/// Borrow the app and perform operations on it
pub(crate) fn with_app_ctx<F, R>(f: F) -> R
where
    F: Fn(&mut AppCtx) -> R,
{
    let ctx = APP_CTX.get().expect("App context not initialized");

    f(&mut ctx.lock().unwrap())
}

/// Get the globally available graphics backend
///
/// # Panics
/// If called outside of the app or callback passed to [start]
pub fn get_graphics_backend() -> Arc<GraphicsBackend> {
    with_app_ctx(|ctx| {
        ctx.get_backend()
            .expect("Graphics backend must be initialized")
    })
}

pub(crate) struct AppCtx {
    /// Our window 
    window: Mutex<Option<Arc<Window>>>,

    /// Our graphics context
    backend: Mutex<Option<Arc<GraphicsBackend>>>,

    /// Our temporary early-initialized device
    early_device: Mutex<Option<EarlyGraphicsDevice>>,
}

impl AppCtx {
    fn new() -> Self {
        let device = pollster::block_on(EarlyGraphicsDevice::new());

        Self {
            window: Mutex::new(None),
            backend: Mutex::new(None),
            early_device: Mutex::new(Some(device)),
        }
    }

    /// Set our window
    fn set_window(&self, new_window: Option<Window>) {
        let mut window = self.window.lock().unwrap();
        let mut backend = self.backend.lock().unwrap();

        *window = new_window.map(Arc::new);

        // Initialize our backend if it's not already
        if backend.is_none() && window.is_some() {
            let mut early_device = self.early_device.lock().unwrap();

            *backend = Some(Arc::new(GraphicsBackend::new(
                window.as_ref().unwrap().clone(),
                early_device.take().unwrap(),
            )));
        }
    }

    fn get_window(&self) -> Option<Arc<Window>> {
        let window = self.window.lock().unwrap();
        window.clone()
    }

    fn get_backend(&self) -> Option<Arc<GraphicsBackend>> {
        let backend = self.backend.lock().unwrap();
        backend.clone()
    }

    fn request_redraw(&self) {
        if let Some(window) = self.get_window() {
            window.request_redraw();
        }
    }
}

struct WinitApp {
    app_initializer: Option<Box<dyn FnOnce() -> Box<dyn AppHandler>>>,
    app: Option<Box<dyn AppHandler>>,
}

impl WinitApp {
    fn new(app_initializer: Box<dyn FnOnce() -> Box<dyn AppHandler>>) -> Self {
        Self {
            app_initializer: Some(app_initializer),
            app: None,
        }
    }

    /// Initialize the underlying app when the context is ready.
    ///
    /// Will panic if the app is already initialized
    fn init_app(&mut self) {
        let initializer = self.app_initializer.take().unwrap();
        self.app = Some(initializer());
    }

    fn is_initialized(&self) -> bool {
        self.app.is_some()
    }
}

impl ApplicationHandler for WinitApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Initialize our context window
        with_app_ctx(|ctx| {
            let window = Window::default_attributes();
            ctx.set_window(Some(event_loop.create_window(window).unwrap()));
        });

        // If our window isn't already initialized - do it
        if !self.is_initialized() {
            self.init_app();
        };
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        // Destroy our window
        with_app_ctx(|ctx| ctx.set_window(None));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if !self.is_initialized() {
            return;
        };

        let app = self.app.as_mut().unwrap();
        match event {
            WindowEvent::RedrawRequested => {
                app.draw();
                get_graphics_backend().flip();
            }
            WindowEvent::CloseRequested => {
                // TODO: In the future the app should be able to respond to this request by either denying it, or not doing anything

                app.app_event(AppEvent::QuitRequested);
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                get_graphics_backend().resize(new_size.width, new_size.height);
                with_app_ctx(|ctx| ctx.request_redraw())
            }
            _ => {
                // Just push an event if it's convertible
                if let Ok(app_event) = event.try_into() {
                    app.app_event(app_event);
                }
            }
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        if !self.is_initialized() {
            return;
        };

        let app = self.app.as_mut().unwrap();
        app.quitting();
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        with_app_ctx(|ctx| ctx.request_redraw());
    }
}

/// An application handler for single-window apps
pub trait AppHandler {
    /// A draw operation that works as both an update and draw event.
    fn draw(&mut self);

    /// An app event was received
    fn app_event(&mut self, event: AppEvent);

    /// An irreversible quit event
    fn quitting(&mut self) {}
}

/// Start an application
#[allow(clippy::missing_panics_doc)]
pub fn start<F>(f: F)
where
    F: FnOnce() -> Box<dyn AppHandler> + 'static,
{
    init_app_ctx(AppCtx::new());

    let mut app = WinitApp::new(Box::new(f));

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let _ = event_loop.run_app(&mut app);
}
