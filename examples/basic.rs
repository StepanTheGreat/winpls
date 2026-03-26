//! A super basic example usage of the crate

use std::sync::Arc;

use winpls::{AppHandler, Config, GraphicsBackend, get_graphics_backend, start};

struct App {
    backend: Arc<GraphicsBackend>,
}

impl App {
    fn new() -> Self {
        Self {
            backend: get_graphics_backend(),
        }
    }
}

impl AppHandler for App {
    fn draw(&mut self) {
        self.backend.clear_screen(0.0, 0.0, 1.0, 1.0);
    }

    fn app_event(&mut self, _: winpls::AppEvent) {}

    fn quitting(&mut self) {}
}

fn main() {
    let conf = Config {
        resizable: true,
        title: "Hello".to_owned(),
        ..Default::default()
    };

    start(|| Box::new(App::new()), conf);
}
