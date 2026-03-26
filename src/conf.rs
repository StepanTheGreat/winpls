/// App configuration
#[derive(Clone, Debug)]
pub struct Config {
    /// The width of the window
    pub width: u32,

    /// The height of the window
    pub height: u32,

    /// The graphics backends to use
    pub backend: wgpu::Backends,

    /// Title to use
    pub title: String,

    /// Whether to open the app in fullscreen
    pub fullscreen: bool,

    /// Whether the user can resize the window
    pub resizable: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            backend: wgpu::Backends::all(),
            title: "Window".to_owned(),
            resizable: true,
            fullscreen: false,
        }
    }
}
