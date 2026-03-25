use winit::event::MouseButton as WMouseButton;
use winit::keyboard::{KeyCode as WKeyCode, PhysicalKey};

/// A mouse button
#[derive(Debug)]
pub enum MouseButton {
    /// Left mouse button
    Left,

    /// Middle mouse button
    Middle,

    /// Right mouse button
    Right,

    /// Any other unrecognized mouse button
    Unknown,
}

impl From<WMouseButton> for MouseButton {
    fn from(value: WMouseButton) -> Self {
        match value {
            WMouseButton::Left => Self::Left,
            WMouseButton::Middle => Self::Middle,
            WMouseButton::Right => Self::Right,

            _ => Self::Unknown,
        }
    }
}

/// A keyboard key
pub type KeyCode = WKeyCode;

/// Convert a physical key into a key code (if possible)
pub(crate) fn keycode_from_physical_key(key: PhysicalKey) -> Option<KeyCode> {
    match key {
        PhysicalKey::Code(code) => Some(code),
        PhysicalKey::Unidentified(_) => None,
    }
}
