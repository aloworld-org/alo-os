//! Opt-in full-name opening and retained reader state for the nested seat.
use std::sync::Arc;

use alo_strings::Strings;
use smithay::backend::input::KeyState;

use crate::{
    LabelGeometry, NestedControlInput, ReaderKeyRoute, RenderError, Server, WindowControlLabels,
    WindowControlReader, WindowControlReaderStyle,
};

/// Borrow one host's reader and immutable preparation configuration while pumping.
/// Keep the reader slot across frames, render a returned reader before navigation,
/// and remove dismissed pixels. Reopening is explicit; dismissal never reopens it.
/// Before configuration changes, retire server controls and remove the old reader;
/// republish and refocus with the new configuration before starting another gesture.
pub struct NestedReaderSession<'a> {
    /// Retained reading session; None enables a fresh F1 opening gesture.
    pub reader: &'a mut Option<WindowControlReader>,
    /// Externalized complete name and navigation vocabulary.
    pub strings: &'a Strings,
    /// Reusable text shaper.
    pub labels: &'a mut WindowControlLabels,
    /// Reader appearance and preferred capacity.
    pub style: WindowControlReaderStyle,
    /// Complete navigation capacity, checked for every page before acquisition.
    pub chrome: LabelGeometry,
}

/// Owned F1 release survives cancellation and switching to another pump mode.
#[derive(Default)]
pub(crate) struct NameOpening {
    /// Whether this backend still owes the client-free F1 release.
    held: bool,
    /// Continuous native focus and fully prepared reader, absent after cancellation.
    pending: Option<(Arc<()>, WindowControlReader)>,
}

impl NameOpening {
    /// Disarm opening without forgetting who owns its release.
    pub(crate) fn cancel(&mut self) {
        self.pending = None;
    }

    /// Every ordered event observes publication and native selection continuity.
    pub(crate) fn synchronize(&mut self, server: &mut Server, active: bool) {
        if let Some((selection, reader)) = &mut self.pending {
            let current = server.control_name_focus();
            if !active
                || !current.is_some_and(|(_, current)| Arc::ptr_eq(selection, &current))
                || server.read_window_control_page(reader, 0).is_none()
            {
                self.cancel();
            }
        }
    }

    /// Drain only owned F1 events, never acquire a client-owned release.
    pub(crate) fn drain(&mut self, code: u32, state: KeyState) -> bool {
        if code != 59 || !self.held {
            return false;
        }
        if state == KeyState::Released {
            self.held = false;
            self.cancel();
        }
        true
    }
}

impl NestedControlInput {
    /// Route an ordered key with opt-in F1 native-name opening.
    ///
    /// Only explicit live native focus acquires F1; hover alone and client-owned
    /// keys retain ordinary application routing. Prepare atomically at press,
    /// open once on release, and consume repeats/cancelled releases. No window
    /// command executes. Competing presses cancel opening but still route once.
    /// Call `route_reader` for all intervening pointer/activation events, even None.
    /// Errors cancel ownership and must not be retried or forwarded.
    pub fn reader_session_key(
        &mut self,
        server: &mut Server,
        session: &mut NestedReaderSession<'_>,
        active: bool,
        event: (u32, KeyState, u32),
    ) -> Result<ReaderKeyRoute, RenderError> {
        let (code, state, _) = event;
        self.opening.synchronize(server, active);
        if session.reader.is_some() {
            self.opening.cancel();
        }
        if code == 59 && self.opening.held {
            if state == KeyState::Released {
                self.opening.held = false;
                if let Some((_, reader)) = self.opening.pending.take() {
                    *session.reader = Some(reader);
                    return Ok(ReaderKeyRoute::Changed);
                }
            }
            return Ok(ReaderKeyRoute::Consumed);
        }
        if state == KeyState::Pressed {
            self.opening.cancel();
        }
        if code == 59
            && state == KeyState::Pressed
            && active
            && session.reader.is_none()
            && server
                .surfaces
                .keyboard
                .as_ref()
                .is_some_and(|k| !k.has_pressed_keys())
            && let Some((_, selection)) = server.control_name_focus()
        {
            self.opening.held = true;
            self.reader.cancel();
            let prepared = server.open_presented_window_control_reader(
                session.labels,
                session.strings,
                None,
                session.style,
                session.chrome,
            );
            match prepared {
                Ok(reader) => self.opening.pending = reader.map(|reader| (selection, reader)),
                Err(error) => {
                    self.cancel(server);
                    return Err(error);
                }
            }
            return Ok(ReaderKeyRoute::Consumed);
        }
        self.reader_key(server, session.reader.as_mut(), active, event)
            .map_err(RenderError::Input)
    }
}
