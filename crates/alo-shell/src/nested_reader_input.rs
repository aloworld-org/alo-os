//! Ordered nested reader interception, before native controls and client delivery.
use crate::{
    InputError, NestedControlInput, NestedPointerEvent, Server, WindowControlPointerEvent,
    WindowControlRouteError,
};

impl NestedControlInput {
    /// Use this same owner for reader frame feedback and ordered input.
    pub fn reader_pointer(&mut self) -> &mut crate::WindowControlReaderPointer {
        self.reader.frame_pointer()
    }

    /// Cancel input after backend failure without losing owned releases.
    pub fn cancel(&mut self, server: &mut Server) {
        self.position = None;
        self.reader.synchronize(server, None, false);
        server.clear_input();
    }

    /// Route nested PageUp/PageDown/Escape as reader navigation, then ordinary
    /// typing exactly once. These are reader-local evdev keys, not configurable
    /// desktop shortcuts. Inactive events drain ownership without client delivery.
    pub fn reader_key(
        &mut self,
        server: &mut Server,
        reader: Option<&mut crate::WindowControlReader>,
        active: bool,
        event: (u32, smithay::backend::input::KeyState, u32),
    ) -> Result<crate::ReaderKeyRoute, InputError> {
        use crate::{ReaderKeyCommand as Command, ReaderKeyRoute as Route};
        let (code, state, time) = event;
        let command = match code {
            104 => Some(Command::Previous),
            109 => Some(Command::Next),
            1 => Some(Command::Dismiss),
            _ => None,
        };
        let route = self
            .reader
            .key(server, reader, active, code, state, command);
        if active
            && route == Route::Forward
            && let Err(error) = server.keyboard_key(code, state, time)
        {
            self.cancel(server);
            return Err(error);
        }
        Ok(route)
    }

    /// Route one ordered parent pointer/activation event through the published
    /// reader before controls and clients. None observes activation/removal.
    /// Keep using this method with no reader so cancelled releases still drain.
    /// Errors cancel input and must never be retried or forwarded.
    pub fn route_reader(
        &mut self,
        server: &mut Server,
        mut reader: Option<&mut crate::WindowControlReader>,
        active: bool,
        event: Option<NestedPointerEvent>,
    ) -> Result<crate::ReaderKeyRoute, WindowControlRouteError> {
        use crate::ReaderKeyRoute as Route;
        self.reader
            .synchronize(server, reader.as_deref_mut(), active);
        if !active {
            self.position = None;
            server.pointer_leave()?;
        }
        let invalid = match &event {
            Some(NestedPointerEvent::Button { code, .. }) => !(0x110..=0x117).contains(code),
            Some(NestedPointerEvent::Axis(frame)) => {
                !crate::pointer::bounded(frame.axis.0) || !crate::pointer::bounded(frame.axis.1)
            }
            _ => false,
        };
        if invalid {
            self.cancel(server);
            return Err(InputError::InvalidPointer.into());
        }
        if let Some(NestedPointerEvent::Motion { x, y, .. }) = &event {
            if !crate::pointer::bounded(*x) || !crate::pointer::bounded(*y) {
                self.cancel(server);
                return Err(InputError::InvalidPointer.into());
            }
            if active {
                if server.surfaces.pointer.is_none() {
                    self.cancel(server);
                    return Err(InputError::PointerUnavailable.into());
                }
                self.position = Some((*x, *y));
            }
        }
        let semantic = match &event {
            Some(NestedPointerEvent::Button { code, state, .. }) => {
                Some(WindowControlPointerEvent::Button(*code, *state))
            }
            Some(NestedPointerEvent::Motion { .. } | NestedPointerEvent::Axis(_)) => {
                Some(WindowControlPointerEvent::Motion)
            }
            None => None,
        };
        let route = semantic.map_or(Route::Forward, |event| {
            self.reader
                .pointer(server, reader, active, self.position, event)
        });
        if route == Route::Forward
            && let Err(error) = self.route(server, active, event)
        {
            self.cancel(server);
            return Err(error);
        }
        Ok(route)
    }
}
