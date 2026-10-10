//! Parent pointer position for the nested event pump, and ordinary routing.
//!
//! # What changed when the control strip was retired
//!
//! This routed every pointer event through
//! `Server::route_presented_window_control_pointer`, which **intercepted**
//! presses landing on one of the three control tiles and otherwise fell through
//! to ordinary seat routing. With the strip gone there is nothing to intercept,
//! so it calls the ordinary routes directly — the same `pointer_motion`,
//! `pointer_button` and `pointer_axis` the interception ended in.
//!
//! **So the refusal narrowed, and the type says so.** `route` returned
//! `WindowControlRouteError`, whose three variants were ordinary input
//! validation, a native press refused, and a native release consumed. The last
//! two were the tiles'. What is left is [`InputError`], and a wrapper enum with
//! one variant would claim a choice that no longer exists.
//!
//! **What the strip's router did that this deliberately stops doing:** it
//! drained an owned primary *release* even while the parent was deactivated,
//! because a tile that had taken ownership of a press had to be told the button
//! came back up. Nothing owns a press now, so a release while deactivated is an
//! ordinary release and `pointer_leave` is the whole of what deactivation owes.
//!
//! **The name is owed a change and has not had one.** This is the nested
//! backend's pointer history and has nothing to do with window controls. The
//! rename is left out of this removal deliberately: `WindowControlLabels` in the
//! same crate is the shell's only font system, and renaming that is 220 mentions
//! across 85 files. Both belong in one change about names rather than inside one
//! about deletions.

use crate::{InputError, NestedPointerEvent, Server};

/// One nested backend's pointer history, independent of the client seat position.
///
/// Keep this with its backend and server for their entire lifetime.
#[derive(Default)]
pub struct NestedControlInput {
    /// Last finite parent motion since activation; never inferred from client focus.
    pub(crate) position: Option<(f64, f64)>,
}

impl NestedControlInput {
    /// Current parent position for fresh hover feedback, absent after input loss.
    #[must_use]
    pub fn position(&self) -> Option<(f64, f64)> {
        self.position
    }

    /// Route one parent pointer event to the client seat.
    ///
    /// Supply `None` on activation changes and on close. Deactivation forgets
    /// the position and requires fresh motion. Invalid motion forgets the
    /// position and leaves the pointer **before** refusing, so a retry cannot
    /// resume from a coordinate nothing accepted. Errors must not be retried or
    /// forwarded.
    pub fn route(
        &mut self,
        server: &mut Server,
        active: bool,
        event: Option<NestedPointerEvent>,
    ) -> Result<(), InputError> {
        if !active {
            self.position = None;
            server.pointer_leave()?;
            return Ok(());
        }
        match event {
            Some(NestedPointerEvent::Motion { x, y, time }) => {
                if !crate::pointer::bounded(x) || !crate::pointer::bounded(y) {
                    self.position = None;
                    server.pointer_leave()?;
                    return Err(InputError::InvalidPointer);
                }
                if server.surfaces.pointer.is_none() {
                    return Err(InputError::PointerUnavailable);
                }
                server.pointer_motion(x, y, time)?;
                self.position = Some((x, y));
            }
            Some(NestedPointerEvent::Button { code, state, time }) => {
                // **Only with a position**, as before: a button arriving before
                // any motion has no place to have happened, and routing it
                // would put a press wherever the seat was last left.
                if self.position.is_some() {
                    server.pointer_button(code, state, time)?;
                }
            }
            Some(NestedPointerEvent::Axis(frame)) if self.position.is_some() => {
                server.pointer_axis(frame)?;
            }
            // A scroll before any motion, like a button before any motion: no
            // place for it to have happened, so it does not happen.
            Some(NestedPointerEvent::Axis(_)) => {}
            None => {}
        }
        Ok(())
    }
}
