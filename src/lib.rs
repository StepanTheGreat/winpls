#![doc = include_str!("../README.md")]

mod fs;
mod app;
mod backend;
mod input;
mod event;

pub use fs::{FSError, FSResult, read_bytes, read_string};
pub use app::{AppHandler, get_graphics_backend, start};
pub use event::AppEvent;
pub use backend::GraphicsBackend;
pub use input::{KeyCode, MouseButton};
