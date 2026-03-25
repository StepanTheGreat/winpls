use winit::event::WindowEvent;

use crate::input::keycode_from_physical_key;

/// An app lifecycle event
#[derive(Debug)]
pub enum AppEvent {
    /// The app has requested to quit. This however can be cancelled.
    QuitRequested,

    /// A mouse button was pressed
    MouseButtonDown(crate::MouseButton),

    /// A mouse button was released
    MouseButtonUp(crate::MouseButton),

    /// A mouse has entered the window
    MouseEntered,

    /// A mouse has left the window
    MouseLeft,

    /// A mouse has moved to the provided coordinates
    MouseMotion {
        /// The x coordinate of the mouse
        x: f32,
        /// The y coordinate of the mouse
        y: f32,
    },

    /// A key was pressed
    KeyDown {
        /// The keycode being pressed
        key: crate::KeyCode,

        /// Whether this is a repeated event (on some OS this happens when holding a key down for a few seconds)
        repeated: bool,
    },

    /// A key was released
    KeyUp(crate::KeyCode),

    /// The window changed its dimensions
    WindowResized {
        /// The new width of the window
        width: u32,

        /// The new height of the window
        height: u32,
    },
}

impl TryFrom<WindowEvent> for AppEvent {
    type Error = ();
    fn try_from(value: WindowEvent) -> Result<Self, Self::Error> {
        match value {
            WindowEvent::CloseRequested => Ok(Self::QuitRequested),
            WindowEvent::MouseInput { state, button, .. } => Ok(if state.is_pressed() {
                Self::MouseButtonDown(button.into())
            } else {
                Self::MouseButtonUp(button.into())
            }),
            WindowEvent::CursorEntered { .. } => Ok(Self::MouseEntered),
            WindowEvent::CursorLeft { .. } => Ok(Self::MouseLeft),
            WindowEvent::CursorMoved { position, .. } => Ok(Self::MouseMotion {
                x: position.x as f32,
                y: position.y as f32,
            }),
            WindowEvent::Resized(size) => Ok(Self::WindowResized {
                width: size.width,
                height: size.height,
            }),
            WindowEvent::KeyboardInput { event, .. } => {
                // Unknow keys don't get any events as of now
                match keycode_from_physical_key(event.physical_key) {
                    Some(keycode) => Ok(if event.state.is_pressed() {
                        Self::KeyDown {
                            key: keycode,
                            repeated: event.repeat,
                        }
                    } else {
                        Self::KeyUp(keycode)
                    }),

                    None => Err(()),
                }
            }

            _ => Err(()),
        }
    }
}
