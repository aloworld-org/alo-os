//! Publication-bound coordination of a backend's ordered reader input.
use std::sync::Arc;

use smithay::backend::input::{ButtonState, KeyState};

use crate::{
    ReaderKeyCommand, ReaderKeyRoute, Server, WindowControlPointerEvent, WindowControlReader,
    WindowControlReaderKeys, WindowControlReaderPointer,
};

/// Reader input ownership for one backend and server, kept across removal/loss.
///
/// Call with the live reader and current seat/session activation on every event.
/// Only continuously published geometry can execute; refreshes of identical
/// frames preserve presses. Changed publication cancels both devices even if no
/// event was delivered during the gap. Forward ordinary input exactly once only
/// on `Forward`; request a new frame on `Changed` or removal on `Dismissed`.
/// This component neither maps keys nor installs a backend event filter.
#[derive(Default)]
pub struct WindowControlReaderInput {
    /// Continuous submitted page/geometry authority observed at the last event.
    publication: Option<Arc<()>>,
    /// Key releases must remain owned across cancellation.
    keys: WindowControlReaderKeys,
    /// Pointer releases and feedback share the frame's exact owner.
    pointer: WindowControlReaderPointer,
}

impl WindowControlReaderInput {
    /// Supply this same pointer owner to `WindowControlReaderFrame`.
    pub fn frame_pointer(&mut self) -> &mut WindowControlReaderPointer {
        &mut self.pointer
    }

    /// Cancel both devices on input loss or mapping changes, retaining releases.
    /// Hosts must call this even when loss delivers no key or pointer event.
    pub fn cancel(&mut self) {
        self.publication = None;
        self.keys.cancel();
        self.pointer.cancel();
    }

    /// Observe activation/publication without an input event. Returns whether
    /// new reader ownership is available; false still requires routing releases.
    /// Deactivation retires reader publication; reactivation needs a fresh frame.
    pub fn synchronize(
        &mut self,
        server: &mut Server,
        reader: Option<&mut WindowControlReader>,
        active: bool,
    ) -> bool {
        if !active {
            server.reader_presentation = None;
        }
        let publication = if active
            && reader.is_some_and(|reader| server.window_control_reader_presented(reader))
        {
            server
                .reader_presentation
                .as_ref()
                .map(|p| p.authority.clone())
        } else {
            None
        };
        if !self
            .publication
            .as_ref()
            .zip(publication.as_ref())
            .is_some_and(|(old, new)| Arc::ptr_eq(old, new))
        {
            self.cancel();
        }
        self.publication = publication;
        self.publication.is_some()
    }

    /// Route a trusted semantic key mapping against the successfully drawn page.
    /// None preserves ordinary typing. Any key press cancels pointer execution;
    /// owned repeats/releases drain even while inactive or after reader removal.
    pub fn key(
        &mut self,
        server: &mut Server,
        mut reader: Option<&mut WindowControlReader>,
        active: bool,
        code: u32,
        state: KeyState,
        command: Option<ReaderKeyCommand>,
    ) -> ReaderKeyRoute {
        let published = self.synchronize(server, reader.as_deref_mut(), active);
        if state == KeyState::Pressed {
            self.pointer.cancel();
        }
        let route = self
            .keys
            .route(server, reader.filter(|_| published), code, state, command);
        self.finish(route)
    }

    /// Route motion or a button using the actual backend position, never client
    /// seat position (native grabs freeze that position). Motion also determines
    /// whether a following axis frame is consumed. None/nonfinite positions hit
    /// nothing and disarm presses; owned releases still drain. Validate malformed
    /// backend coordinates in its ordinary input path if this returns Forward.
    /// Pointer presses cancel held key commands; motion alone does not.
    pub fn pointer(
        &mut self,
        server: &mut Server,
        mut reader: Option<&mut WindowControlReader>,
        active: bool,
        position: Option<(f64, f64)>,
        event: WindowControlPointerEvent,
    ) -> ReaderKeyRoute {
        let published = self.synchronize(server, reader.as_deref_mut(), active);
        let hit = if published {
            reader
                .as_deref_mut()
                .zip(position)
                .and_then(|(reader, position)| {
                    server.presented_window_control_reader_hit(reader, position)
                })
        } else {
            None
        };
        let reader = reader.filter(|_| published);
        let route = match event {
            WindowControlPointerEvent::Motion => {
                if self.pointer.motion(server, reader, hit) {
                    ReaderKeyRoute::Consumed
                } else {
                    ReaderKeyRoute::Forward
                }
            }
            WindowControlPointerEvent::Button(code, state) => {
                if state == ButtonState::Pressed {
                    self.keys.cancel();
                }
                self.pointer.button(server, reader, hit, code, state)
            }
        };
        self.finish(route)
    }

    /// Navigation/removal retires both devices immediately, before the next frame.
    fn finish(&mut self, route: ReaderKeyRoute) -> ReaderKeyRoute {
        if matches!(route, ReaderKeyRoute::Changed | ReaderKeyRoute::Dismissed) {
            self.cancel();
        }
        route
    }
}
