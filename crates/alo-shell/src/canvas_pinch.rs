//! A pinch on the plane, which is the canvas's other pointer zoom.
//!
//! `docs/design/the-shortcuts-and-the-edges.md` gives *zoom toward the pointer*
//! two pointer routes — *pinch, or Ctrl + wheel on empty canvas* — and the plan's
//! task 6 names the pinch beside the wheel. This is that route, and it needed a
//! protocol rather than a branch: `zwp_pointer_gestures_v1` is advertised in
//! `crate::surfaces`, and a touchpad's pinch reaches this compositor because of
//! it, not because anything here asked.
//!
//! # A pinch is absolute where a wheel is incremental, and that is the design
//!
//! A wheel notch is a step: one notch, one rung of `Zoom::STOPS`, and the wheel
//! road keeps a remainder because notches arrive in fractions.
//! `GesturePinchUpdateEvent::scale` is nothing like that — it is **the scale
//! against the moment the fingers went down**, so the honest answer is the zoom
//! that gesture started from, multiplied by it. Fingers half as far apart mean
//! half the zoom, the whole way through the gesture, and letting go and pinching
//! again starts from wherever it was left.
//!
//! That is why this does not step rungs. A pinch is a person moving a thing with
//! their hand, so it goes where their hand goes and nowhere else; rungs exist for
//! the wheel and the keyboard, where a press is a discrete request. **Neither is
//! accelerated**: at any moment the zoom is the starting zoom times the scale the
//! fingers make, with no curve of any kind between them.
//!
//! # Where it goes when it is not over the plane
//!
//! To the application, untouched. A pinch over a frame is that application's
//! gesture — a map, a photograph, a document — exactly as a scroll over a frame
//! is its scroll. The canvas takes the pinch only where the wheel pans: with
//! nothing focused, which is `crate::pointer`'s rule and not a second one.

use alo_canvas::Zoom;

impl crate::Server {
    /// Whether this person has two-finger pinch zoom turned on.
    ///
    /// Read from the recogniser's installed preferences rather than from a copy
    /// kept here: `alo_desktops::Gestures` is where a person's touchpad settings
    /// live, and a second copy would be a second answer to a question they have
    /// already answered once.
    pub(crate) fn pinch_zoom_is_wanted(&self) -> bool {
        self.gestures
            .preferences()
            .enabled(alo_desktops::gesture_events::Kind::Pinch)
    }

    /// Begin a pinch on the plane: remember what it is zooming from.
    ///
    /// The zoom at the moment the fingers went down is the only state a pinch
    /// needs, because every update is measured against that instant rather than
    /// against the update before it.
    pub(crate) fn begin_a_pinch_on_the_plane(&mut self) {
        let from = self.the_camera().zoom();
        if let Some(pointer) = self.surfaces.pointer.as_mut() {
            pointer.pinch_from = Some(from);
        }
    }

    /// Zoom to this scale of where the pinch began. Whether anything moved.
    ///
    /// `false` where no pinch is in progress, where the scale is not a number a
    /// zoom can be made of, and where the canvas is already as far in or out as
    /// it goes — an end rather than a fault, as it is for the wheel.
    pub(crate) fn pinch_the_plane_to(&mut self, scale: f64) -> bool {
        let Some(pointer) = self.surfaces.pointer.as_ref() else {
            return false;
        };
        let Some(from) = pointer.pinch_from else {
            return false;
        };
        let held = (pointer.location.x.trunc(), pointer.location.y.trunc());
        // A scale of zero or less is not a pinch anybody made, and a non-finite
        // one cannot be turned into thousandths at all.
        if !scale.is_finite() || scale <= 0.0 {
            return false;
        }
        let wanted = f64::from(from.thousandths()) * scale;
        if !wanted.is_finite() {
            return false;
        }
        // Clamped rather than refused, and this is the one place the canvas
        // clamps: a person's fingers do not stop at the end of the zoom range, so
        // pinching past it should rest at the end rather than abandon the gesture
        // and leave the next update to start a fight with the one before it.
        let wanted = wanted.clamp(f64::from(Zoom::FURTHEST_OUT), f64::from(Zoom::FURTHEST_IN));
        let Ok(zoom) = Zoom::of(wanted as u32) else {
            return false;
        };
        if zoom == self.the_camera().zoom() {
            return false;
        }
        let (Ok(across), Ok(down)) = (i32::try_from(held.0 as i64), i32::try_from(held.1 as i64))
        else {
            return false;
        };
        self.zoom_the_canvas(zoom, (across, down)).is_some()
    }

    /// The pinch is over, whether it finished or was cancelled.
    ///
    /// Nothing is undone on a cancel. A cancelled pinch is a gesture libinput
    /// stopped believing in, not one the person took back, and putting the canvas
    /// back where it started would throw away a zoom they have been watching.
    pub(crate) fn end_the_pinch_on_the_plane(&mut self) {
        if let Some(pointer) = self.surfaces.pointer.as_mut() {
            pointer.pinch_from = None;
        }
    }

    /// Whether a pinch is being spent on the plane right now.
    ///
    /// A gesture that began over the plane finishes there even if the frames move
    /// underneath it, because a person who started pinching the canvas is
    /// pinching the canvas until they let go.
    pub(crate) fn a_pinch_is_on_the_plane(&self) -> bool {
        self.surfaces
            .pointer
            .as_ref()
            .is_some_and(|pointer| pointer.pinch_from.is_some())
    }
}
