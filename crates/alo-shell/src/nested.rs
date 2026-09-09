//! Nested WSLg/Wayland graphics and seat backend.

use crate::{FrameTarget, RenderError};
use smithay::{
    backend::{
        input::{AbsolutePositionEvent, Event, InputEvent, KeyboardKeyEvent, PointerButtonEvent},
        renderer::gles::GlesRenderer,
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
    /// Actual parent position, including motion consumed by native controls.
    control_input: crate::NestedControlInput,
}

impl Nested {
    /// Actual parent position for transaction feedback, never cached client focus.
    pub(crate) fn control_position(&self) -> Option<(f64, f64)> {
        self.control_input.position()
    }
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
            control_input: crate::NestedControlInput::default(),
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
    /// Published native controls intercept primary gestures; absent presentation
    /// uses ordinary client routing. This method does not compose controls/labels.
    pub fn pump_seat(&mut self, server: &mut crate::Server) -> Result<(), RenderError> {
        self.pump_input(server, true)
    }

    /// Share ordered parent activation handling across keyboard-only and full seats.
    fn pump_input(&mut self, server: &mut crate::Server, pointer: bool) -> Result<(), RenderError> {
        let mut failure = None;
        let mut control_input = std::mem::take(&mut self.control_input);
        let result = self.pump_events(|event, focused| {
            if failure.is_some() {
                return;
            }
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
                if let Err(error) = translated.map_err(RenderError::Input).and_then(|event| {
                    control_input
                        .route(server, focused, event)
                        .map_err(RenderError::WindowControl)
                }) {
                    failure = Some(error);
                    return;
                }
            }
            let focus = if focused {
                server.mapped_surfaces().next().cloned()
            } else {
                None
            };
            if let Err(error) = server.keyboard_focus(focus.as_ref()) {
                failure = Some(RenderError::Input(error));
                return;
            }
            if let Some(WinitEvent::Input(InputEvent::Keyboard { event })) = event {
                let code = u32::from(event.key_code()).saturating_sub(8);
                if let Err(error) = server.keyboard_key(code, event.state(), event.time_msec()) {
                    failure = Some(RenderError::Input(error));
                }
            }
        });
        self.control_input = control_input;
        if let Some(error) = failure {
            self.control_input = crate::NestedControlInput::default();
            server.clear_input();
            return Err(error);
        }
        result?;
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
    fn submit_reader(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
        reader: &crate::WindowControlReaderScene<'_>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        self.submit_native_scene(
            roots,
            popups,
            cursor,
            Some(crate::scene_native::NativeScene::Reader(reader)),
        )
    }
    fn submit_controls(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
        controls: Option<crate::WindowControlScene<'_>>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        self.submit_control_scene(roots, popups, cursor, controls)
    }
    fn metadata(&self) -> Result<crate::OutputMetadata, RenderError> {
        Ok(crate::OutputMetadata {
            name: "alo-nested".into(),
            make: "alo".into(),
            model: "nested".into(),
            ..crate::OutputMetadata::virtual_output()
        })
    }
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
        self.submit_control_scene(roots, popups, cursor, None)
    }
}

impl Nested {
    /// Submit clients, native strip/label, then cursor using the shared painter.
    ///
    /// This low-level call does not publish controls or send callbacks. The host
    /// must own overlay input policy, refresh the live mapping without dispatch
    /// during submission, publish only on success and retire on failure/removal.
    /// Output mismatch, clipped labels and labels covering controls refuse before
    /// painting. None removes native pixels; nothing is retained for next frame.
    pub fn submit_control_scene(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
        controls: Option<crate::WindowControlScene<'_>>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        self.submit_native_scene(
            roots,
            popups,
            cursor,
            controls.map(crate::scene_native::NativeScene::Controls),
        )
    }

    /// One shared submission boundary for complete labels and paged readers.
    fn submit_native_scene(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
        controls: Option<crate::scene_native::NativeScene<'_>>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        if self.closed {
            return Err(RenderError::Closed);
        }
        let size = self.size();
        if size.w <= 0 || size.h <= 0 {
            return Err(RenderError::EmptySize);
        }
        let damage = Rectangle::from_size(size);
        let trace = std::env::var_os("ALO_NESTED_TRACE_SUBMISSION").is_some();
        let start = std::time::Instant::now();
        if trace {
            eprintln!("Nested submission: before bind, {} roots", roots.len());
        }
        let drawing = {
            let (renderer, mut framebuffer) = self.backend.bind().map_err(submission)?;
            if trace {
                eprintln!("Nested submission {:?}: after bind", start.elapsed());
            }
            crate::scene_drawing::paint(
                renderer,
                &mut framebuffer,
                roots,
                popups,
                cursor,
                Transform::Flipped180,
                controls,
            )?
        };
        if trace {
            eprintln!("Nested submission {:?}: after paint", start.elapsed());
        }
        self.backend.submit(Some(&[damage])).map_err(submission)?;
        if trace {
            eprintln!("Nested submission {:?}: after swap", start.elapsed());
        }
        // Positioned arrows are now in the submitted scene, just like client
        // cursors. Change host visibility only after that submission succeeds.
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
