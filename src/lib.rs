#![doc = include_str!("../README.md")]

mod app;
mod backend;
mod conf;
mod event;
mod fs;
mod input;

pub use app::{AppHandler, get_graphics_backend, start};
pub use backend::GraphicsBackend;
pub use conf::Config;
pub use event::AppEvent;
pub use fs::{FSError, FSResult, read_bytes, read_string};
pub use input::{KeyCode, MouseButton};
