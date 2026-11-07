//! Ctrl and the wheel, over the plane: the pointer-centred zoom.
//!
//! `docs/design/the-shortcuts-and-the-edges.md` gives *zoom toward the pointer*
//! two pointer routes — *pinch, or Ctrl + wheel on empty canvas* — and this is the
//! second. Ctrl is the documented modifier the plan's task 6 asks for.
//!
//! # Why this is a separate road from `crate::canvas_pan`
//!
//! Both start at a scroll over the plane and they spend it on different things, so
//! each keeps its own remainder. Sharing one would make a person who scrolled,
//! pressed Ctrl and scrolled again spend the first gesture's leftovers on the
//! second's zoom — a third of a movement arriving as a zoom step nobody asked for.
//!
//! # One notch is one rung, and that is the whole of the rate
//!
//! `crate::libinput_scroll` divides libinput's v120 by eight, so a full wheel
//! notch of 120 arrives here as `15.0` and a high-resolution wheel's fraction of a
//! notch arrives as that fraction of it. One notch moves one rung of
//! `alo_canvas::Zoom::STOPS`, always. **Nothing here is accelerated**: two notches
//! in quick succession are two rungs, exactly as two notches slowly are, which is
//! what the plan says of panning and is true here for the same reason — a curve
//! nobody can predict is a curve everybody fights.
//!
//! # With nothing focused, the shell still sees Ctrl
//!
//! This used to be a rough edge: `Server::keyboard_key` returned before
//! `KeyboardHandle::input`, which advances xkb's state, whenever nothing was
//! focused, so on a canvas with no window focused a Ctrl+wheel panned. That was
//! decided and fixed in `crate::keyboard` — the compositor reads its own
//! modifiers whatever has the focus, and only forwarding to a client depends on
//! it — and `zoom_and_show_all::ctrl_is_seen_with_nothing_focused_and_the_wheel_still_zooms`
//! holds it. This paragraph named a test that pinned the old behaviour until
//! 2026-11-07, when it was found pointing at a test that no longer exists.

use smithay::input::pointer::AxisFrame;

/// One wheel notch, in the units an [`AxisFrame`] carries here.
///
/// Named rather than written as `15.0` where it is used, because it is a
/// consequence of `crate::libinput_scroll` dividing v120 by eight and not a number
/// somebody liked: if that division changes, this is what has to change with it.
const A_NOTCH: f64 = 15.0;

impl crate::Server {
    /// Whether Ctrl is held right now, as the keyboard sees it.
    ///
    /// [`false`] with no keyboard: a machine with no keyboard cannot be holding a
    /// modifier, so a scroll there is a pan, which is the harmless answer of the
    /// two.
    pub(crate) fn ctrl_is_held(&self) -> bool {
        self.surfaces
            .keyboard
            .as_ref()
            .is_some_and(|keyboard| keyboard.handle.modifier_state().ctrl)
    }

    /// Zoom the plane by this scroll, about the pointer. Whether anything moved.
    ///
    /// `false` where the scroll has not yet added up to a notch — the remainder is
    /// kept — and where the canvas is already as far in or out as it goes, which
    /// is not a fault, just an end.
    pub(crate) fn zoom_the_plane_by_scroll(&mut self, frame: &AxisFrame) -> bool {
        let Some(pointer) = self.surfaces.pointer.as_mut() else {
            return false;
        };
        // The pointer's own place on the glass, which is what a pointer-centred
        // zoom holds still. Read before the remainder is touched so an early
        // refusal below cannot leave the two disagreeing.
        let held = (pointer.location.x.trunc(), pointer.location.y.trunc());
        // Only the vertical axis zooms. A wheel tilted sideways with Ctrl held is
        // not a zoom in any desktop a person arrives from, and guessing that it is
        // would make a tilt-wheel mouse unusable on this canvas.
        let wanted = pointer.unspent_zoom + frame.axis.1;
        let rungs = (wanted / A_NOTCH).trunc();
        pointer.unspent_zoom = wanted - rungs * A_NOTCH;
        let (Ok(across), Ok(down)) = (i32::try_from(held.0 as i64), i32::try_from(held.1 as i64))
        else {
            return false;
        };
        let held = (across, down);
        let mut left = rungs as i64;
        if left == 0 {
            return false;
        }
        // Scrolling **up** is a negative axis value and zooms in, which is what
        // every desktop with a Ctrl+wheel zoom already does.
        let was = self.the_camera().zoom();
        let mut zoom = was;
        while left != 0 {
            let Some(next) = (if left < 0 {
                zoom.one_step_in()
            } else {
                zoom.one_step_out()
            }) else {
                break;
            };
            zoom = next;
            left += if left < 0 { 1 } else { -1 };
        }
        if zoom == was {
            // **The wheel reaches the World too, and by the same step.** If the
            // zoom did not move because it was already at the furthest out and
            // the person kept scrolling out, that is the step `one_step_out`
            // answers `None` for — and task 2's constraint is that *the gesture
            // is the canvas's existing zoom on all three of its roads*, so a
            // World reachable by key and not by wheel would be a second
            // navigation model wearing one name.
            if left > 0 {
                return self.step_out_into_the_world();
            }
            return false;
        }
        // A refused zoom keeps no remainder, for `canvas_pan`'s reason: the person
        // is at an edge, and holding their scroll would spend it the moment they
        // turned round.
        if self.zoom_the_canvas(zoom, held).is_none() {
            if let Some(pointer) = self.surfaces.pointer.as_mut() {
                pointer.unspent_zoom = 0.0;
            }
            return false;
        }
        true
    }
}
