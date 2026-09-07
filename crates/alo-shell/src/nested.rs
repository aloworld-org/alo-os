//! Nested WSLg/Wayland graphics and seat backend.

use crate::{FrameTarget, RenderError, drawing};
use smithay::{
    backend::{
        input::{AbsolutePositionEvent, Event, InputEvent, KeyboardKeyEvent, PointerButtonEvent},
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
/// Call `pump_seat` before each server dispatch/render when both inputs are enabled,
/// or `pump_keyboard` for a keyboard-only seat.
/// Render-only fixtures may use `pump`. Do not recreate winit's event
/// loop in the same process after failure; report the error and end the session.
pub struct Nested {
    /// Owned EGL window and renderer.
    backend: WinitGraphicsBackend<GlesRenderer>,
    /// Parent events, including resize and close.
    events: WinitEventLoop,
    /// Closing is terminal even if winit continues pumping.
    closed: bool,
    /// Parent activation; unfocused nested windows must never route keys.
    focused: bool,
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
            focused: false,
        })
    }

    /// Process graphics events without routing input (render-only fixtures).
    pub fn pump(&mut self) -> Result<(), RenderError> {
        self.pump_events(|_, _| {})
    }

    /// Route parent keyboard events to the frontmost mapped root.
    ///
    /// Creation order is the renderer's current front-to-back order. This minimal
    /// policy gives clients usable keyboard input until window management supplies
    /// explicit activation. Parent focus loss/close releases keys and clears focus.
    /// A keyboard-enabled server is required; pointer events remain unconnected.
    pub fn pump_keyboard(&mut self, server: &mut crate::Server) -> Result<(), RenderError> {
        self.pump_input(server, false)
    }

    /// Route keyboard and pointer events after `Server::enable_pointer`.
    ///
    /// Parent deactivation and close cancel held input. Smithay 0.7 does not
    /// forward cursor-leave events; leave-only cancellation remains backend work.
    /// Client cursor presentation happens on render. This is not a production session.
    pub fn pump_seat(&mut self, server: &mut crate::Server) -> Result<(), RenderError> {
        self.pump_input(server, true)
    }

    /// Share ordered parent activation handling across keyboard-only and full seats.
    fn pump_input(&mut self, server: &mut crate::Server, pointer: bool) -> Result<(), RenderError> {
        let mut failure = None;
        let result = self.pump_events(|event, focused| {
            if pointer {
                let translated = match &event {
                    Some(WinitEvent::Input(InputEvent::PointerMotionAbsolute { event })) => {
                        Ok(Some(crate::NestedPointerEvent::Motion {
                            x: event.x(),
                            y: event.y(),
                            time: event.time_msec(),
                        }))
                    }
                    Some(WinitEvent::Input(InputEvent::PointerButton { event })) => {
                        Ok(Some(crate::NestedPointerEvent::Button {
                            code: event.button_code(),
                            state: event.state(),
                            time: event.time_msec(),
                        }))
                    }
                    Some(WinitEvent::Input(InputEvent::PointerAxis { event })) => {
                        crate::nested_pointer::axis(event)
                            .map(|frame| Some(crate::NestedPointerEvent::Axis(frame)))
                    }
                    _ => Ok(None),
                };
                if let Err(error) =
                    translated.and_then(|event| server.nested_pointer(focused, event))
                {
                    failure = Some(error);
                }
            }
            let focus = if focused {
                server.mapped_surfaces().next().cloned()
            } else {
                None
            };
            if let Err(error) = server.keyboard_focus(focus.as_ref()) {
                failure = Some(error);
                return;
            }
            if let Some(WinitEvent::Input(InputEvent::Keyboard { event })) = event {
                let code = u32::from(event.key_code()).saturating_sub(8);
                if let Err(error) = server.keyboard_key(code, event.state(), event.time_msec()) {
                    failure = Some(error);
                }
            }
        });
        result?;
        if let Some(error) = failure {
            return Err(RenderError::Input(error));
        }
        Ok(())
    }

    /// Keep event order and apply parent activation before delivering a key.
    fn pump_events(
        &mut self,
        mut route: impl FnMut(Option<WinitEvent>, bool),
    ) -> Result<(), RenderError> {
        route(None, self.focused && !self.closed);
        if let PumpStatus::Exit(_) = self.events.dispatch_new_events(|event| {
            match event {
                WinitEvent::CloseRequested => {
                    self.closed = true;
                    self.focused = false;
                }
                WinitEvent::Focus(focused) => self.focused = focused,
                _ => {}
            }
            route(Some(event), self.focused && !self.closed);
        }) {
            self.closed = true;
        }
        if self.closed {
            self.focused = false;
            route(None, false);
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
        self.submit_scene(roots, &crate::Cursor::Default)
    }

    fn submit_scene(
        &mut self,
        roots: &[WlSurface],
        cursor: &crate::Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        self.submit_popups(roots, &[], cursor)
    }

    fn submit_popups(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
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
            let mut drawing = drawing::Drawing {
                elements: Vec::new(),
                surfaces: Vec::new(),
            };
            for (surface, location) in crate::scene::trees(roots, popups) {
                let mut tree =
                    drawing::import_at(renderer, &[surface], damage, location.to_physical(1.0))?;
                drawing.elements.append(&mut tree.elements);
                drawing.surfaces.append(&mut tree.surfaces);
            }
            if let crate::Cursor::Surface { surface, location } = cursor {
                let mut cursor_drawing = drawing::import_at(
                    renderer,
                    std::slice::from_ref(surface),
                    damage,
                    location.to_physical(1.0),
                )?;
                cursor_drawing.elements.append(&mut drawing.elements);
                cursor_drawing.surfaces.append(&mut drawing.surfaces);
                drawing = cursor_drawing;
            }
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
        self.backend
            .window()
            .set_cursor_visible(matches!(cursor, crate::Cursor::Default));
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
