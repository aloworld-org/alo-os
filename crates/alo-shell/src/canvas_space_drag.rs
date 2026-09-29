//! **Space and drag**, the other way to pan — ADR 0065 names it beside the wheel.
//!
//! Task 5 of `docs/autonomy/the-smallest-canvas-worth-showing.md` landed the wheel
//! road and left this one, calling it *a gesture rather than arithmetic*. It is
//! arithmetic after all; what it needed was something else entirely, which is why
//! it waited.
//!
//! # It needed the compositor to know what is held with nothing focused
//!
//! Holding Space over empty canvas is the exact case `Server::keyboard_key` used
//! to be blind to: it returned before `KeyboardHandle::input`, which is what
//! advances XKB, whenever `current_focus()` was `None`. So the shell could not
//! tell that Space was down at the only moment this gesture happens. That was
//! fixed for Ctrl+wheel, and this road is the same fix spent twice.
//!
//! # Why Space, and why it is not a chord
//!
//! `alo_shortcuts::Chord` requires Super, Ctrl or Alt — *Shift does not count* —
//! so a bare Space is not a chord and cannot be one. That is correct rather than
//! inconvenient: this is not a shortcut a person rebinds, it is the pointer
//! gesture every canvas application has, and putting it in the settings panel
//! would be offering to rebind the way somebody holds a piece of paper.
//!
//! # A frame under the pointer does not take it
//!
//! Unlike the wheel, which goes to whatever is focused, a space-drag is the
//! canvas's wherever it starts. Somebody holding Space has said what they mean,
//! and a drag that panned over empty canvas but scrolled a document the moment it
//! crossed one would be a gesture that changes meaning mid-stroke. The application
//! is told nothing: it does not see the motion, so it cannot act on it.

use smithay::utils::{Logical, Point};

/// `KEY_SPACE`, as a real keyboard sends it, before XKB's offset.
const SPACE: u32 = 57;

impl crate::Server {
    /// Whether Space is held right now, as this compositor sees it.
    ///
    /// Read from the seat's own pressed keys rather than from anything kept
    /// beside them: one answer to *is Space down*, which is the same discipline
    /// `ctrl_is_held` keeps for the wheel.
    #[must_use]
    pub(crate) fn space_is_held(&self) -> bool {
        self.surfaces
            .keyboard
            .as_ref()
            .is_some_and(|keyboard| keyboard.handle.pressed_keys().contains(&(SPACE + 8).into()))
    }

    /// Pan by this pointer movement, if a space-drag is what it is.
    ///
    /// Whether the canvas took it. `false` when Space is not held, when there is
    /// no pointer, when the movement rounds to nothing yet — the remainder is kept
    /// — and when the pan would leave the plane.
    ///
    /// **1:1 with the hand, at every zoom**, which is `crate::canvas_pan`'s own
    /// contract and the same arithmetic: what a person drags moves under their
    /// finger by that much whether they are far out or close in.
    pub(crate) fn pan_the_plane_by_a_space_drag(&mut self, to: Point<f64, Logical>) -> bool {
        if !self.space_is_held() {
            return false;
        }
        let Some(pointer) = self.surfaces.pointer.as_mut() else {
            return false;
        };
        // The hand moves one way and the plane moves the other: dragging the
        // canvas to the right brings what is on the left into view, which is what
        // dragging a sheet of paper does.
        let moved = (pointer.location.x - to.x, pointer.location.y - to.y);
        let wanted = (
            pointer.unspent_space.0 + moved.0,
            pointer.unspent_space.1 + moved.1,
        );
        let (across, down) = (wanted.0.trunc(), wanted.1.trunc());
        pointer.unspent_space = (wanted.0 - across, wanted.1 - down);
        let (Ok(across), Ok(down)) = (i32::try_from(across as i64), i32::try_from(down as i64))
        else {
            return false;
        };
        if across == 0 && down == 0 {
            return false;
        }
        if self.pan_the_canvas(across, down).is_none() {
            // A refused pan keeps nothing, for `canvas_pan`'s reason: the person
            // is at the plane's edge and holding their movement would spend it
            // the moment they turned round.
            if let Some(pointer) = self.surfaces.pointer.as_mut() {
                pointer.unspent_space = (0.0, 0.0);
            }
            return false;
        }
        true
    }
}
