//! Nested WSLg/Wayland backend. It advertises no input until routing exists.

use crate::{FrameTarget, RenderError, drawing};
use smithay::{
    backend::{
        renderer::{Color32F, Frame, Renderer, gles::GlesRenderer, utils::draw_render_elements},
        winit::{self, WinitEvent, WinitEventLoop, WinitGraphicsBackend},
    },
    reexports::{
        wayland_server::protocol::wl_surface::WlSurface,
        winit::{
            platform::pump_events::PumpStatus,
            raw_window_handle::{HasDisplayHandle, RawDisplayHandle},
            window::Window,
        },
    },
    utils::{Physical, Rectangle, Size, Transform},
};

/// One native nested window and GLES context, constructed on the main thread.
///
/// Supply a translated title from the session UI (developer fixtures may use a
/// diagnostic title). This backend requires WAYLAND_DISPLAY and refuses X11.
/// Call `pump` before each server dispatch/render. Do not recreate winit's event
/// loop in the same process after failure; report the error and end the session.
pub struct Nested {
    /// Owned EGL window and renderer.
    backend: WinitGraphicsBackend<GlesRenderer>,
    /// Parent events, including resize and close.
    events: WinitEventLoop,
    /// Closing is terminal even if winit continues pumping.
    closed: bool,
}

impl Nested {
    /// Initialize graphics, refusing missing Wayland configuration and failures.
    pub fn new(title: &str, size: (u32, u32)) -> Result<Self, RenderError> {
        if size.0 == 0 || size.1 == 0 || size.0 > i32::MAX as u32 || size.1 > i32::MAX as u32 {
            return Err(RenderError::EmptySize);
        }
        let display = std::env::var_os("WAYLAND_DISPLAY")
            .filter(|value| !value.is_empty())
            .ok_or_else(|| RenderError::Initialization("WAYLAND_DISPLAY is missing".into()))?;
        let path = std::path::Path::new(&display);
        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            let runtime = std::env::var_os("XDG_RUNTIME_DIR")
                .ok_or_else(|| RenderError::Initialization("XDG_RUNTIME_DIR is missing".into()))?;
            let runtime = std::path::Path::new(&runtime);
            if !runtime.is_absolute() {
                return Err(RenderError::Initialization(
                    "XDG_RUNTIME_DIR must be absolute".into(),
                ));
            }
            runtime.join(path)
        };
        std::os::unix::net::UnixStream::connect(path)
            .map_err(|error| RenderError::Initialization(error.to_string()))?;
        let attributes = Window::default_attributes()
            .with_title(title)
            .with_inner_size(smithay::reexports::winit::dpi::PhysicalSize::new(
                size.0, size.1,
            ));
        let (backend, events) = winit::init_from_attributes::<GlesRenderer>(attributes)
            .map_err(|error| RenderError::Initialization(format!("{error:?}")))?;
        let handle = backend
            .window()
            .display_handle()
            .map_err(|error| RenderError::Initialization(error.to_string()))?;
        if !matches!(handle.as_raw(), RawDisplayHandle::Wayland(_)) {
            return Err(RenderError::Initialization(
                "parent display is not Wayland".into(),
            ));
        }
        Ok(Self {
            backend,
            events,
            closed: false,
        })
    }

    /// Process pending parent events. Close is terminal; input is not advertised.
    pub fn pump(&mut self) -> Result<(), RenderError> {
        if let PumpStatus::Exit(_) = self.events.dispatch_new_events(|event| {
            if matches!(event, WinitEvent::CloseRequested) {
                self.closed = true;
            }
        }) {
            self.closed = true;
        }
        if self.closed {
            Err(RenderError::Closed)
        } else {
            Ok(())
        }
    }
}

impl FrameTarget for Nested {
    fn size(&self) -> Size<i32, Physical> {
        self.backend.window_size()
    }

    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        if self.closed {
            return Err(RenderError::Closed);
        }
        let size = self.size();
        if size.w <= 0 || size.h <= 0 {
            return Err(RenderError::EmptySize);
        }
        let damage = Rectangle::from_size(size);
        let drawing = {
            let (renderer, mut framebuffer) = self.backend.bind().map_err(submission)?;
            let drawing = drawing::import(renderer, roots, damage)?;
            let mut frame = renderer
                .render(&mut framebuffer, size, Transform::Flipped180)
                .map_err(submission)?;
            // Neutral clear, not the shell's pending token-based visual design.
            frame
                .clear(Color32F::new(0.0, 0.0, 0.0, 1.0), &[damage])
                .map_err(submission)?;
            draw_render_elements(&mut frame, 1.0, &drawing.elements, &[damage])
                .map_err(submission)?;
            let _sync = frame.finish().map_err(submission)?;
            drawing
        };
        self.backend.submit(Some(&[damage])).map_err(submission)?;
        Ok(drawing.surfaces)
    }
}

/// Preserve the failing graphics stage's upstream diagnostic.
fn submission(error: impl std::fmt::Display) -> RenderError {
    RenderError::Submission(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_dimensions_refuse_before_creating_an_event_loop() {
        for size in [(0, 200), (320, 0), (u32::MAX, 1), (1, u32::MAX)] {
            assert!(matches!(
                Nested::new("test", size),
                Err(RenderError::EmptySize)
            ));
        }
    }
}
