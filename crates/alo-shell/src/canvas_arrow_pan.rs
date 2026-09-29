//! **Arrows pan the canvas**, which is the keyboard road ADR 0065 requires and
//! task 5 was marked done without.
//!
//! ADR 0065 is explicit: *pan — by pinch and drag, by wheel with a modifier, and
//! **by key**, so a mouse and a keyboard both have all of it.* The plan's task 5
//! names the same three roads — *wheel and trackpad over empty canvas, a keyboard
//! route, and a drag gesture*. Two were built. This is the third, and until it
//! existed a person with no pointer could zoom the canvas and reach frames by
//! keyboard but could not move across the plane at all.
//!
//! # Why it was missed, which is worth more than the fix
//!
//! `crate::canvas_command` answers the canvas's keyboard actions — zoom in, zoom
//! out, show all — and it answers them from `alo_shortcuts::Action`. So *does pan
//! have a keyboard route* was asked of the `Action` enum, found zoom and show-all
//! there, and stopped.
//!
//! **The road could not have been in that enum.** `docs/design/the-shortcuts-and-
//! the-edges.md` says *pan in any direction: arrows while the canvas is focused*,
//! and an `alo_shortcuts::Chord` requires Super, Ctrl or Alt — Shift does not
//! count. A bare arrow is not a chord and cannot be made one. The search was run
//! in a space the answer is structurally excluded from, and it came back empty for
//! that reason rather than because nothing was there.
//!
//! That is exactly `crate::canvas_space_drag`'s situation and its header says so
//! about Space for the same reason: this is not a shortcut somebody rebinds, it is
//! how a canvas moves.
//!
//! # A step is a fraction of the viewport, not a number of pixels
//!
//! `docs/design/the-shortcuts-and-the-edges.md` gives the keys and not the
//! distance, so the distance is decided here and the reasoning belongs here.
//! [`A_STEP`] is a **quarter of the viewport**: four presses cross a screen and
//! every one of them leaves three quarters of what was already there, which is
//! what makes a keyboard pan followable rather than teleporting. A fixed pixel
//! count would be a different gesture on a 1366-wide laptop and a 3840-wide
//! display, and the person on the smaller screen is the one who most needs the
//! canvas to move a useful amount.
//!
//! Measured on the viewport rather than on the plane because
//! [`crate::Server::pan_the_canvas`] is in screen pixels: what a person sees move
//! is a quarter of their screen at any zoom, which is the same promise the
//! space-drag makes when it moves 1:1 with the hand.
//!
//! # Shift goes faster, and *faster* is a whole screen
//!
//! The design file's second row is *pan faster: Shift + arrow*. A whole viewport
//! per press, so Shift is exactly four ordinary presses and a person can predict
//! it. Nothing is skipped: a full screen still leaves what was at the edge on the
//! opposite edge.

use smithay::backend::input::KeyState;

/// `KEY_UP`, `KEY_LEFT`, `KEY_RIGHT` and `KEY_DOWN`, as a keyboard sends them.
///
/// Before XKB's offset of 8, like `crate::canvas_space_drag`'s `SPACE`, because
/// this is the number that arrives from libinput.
const UP: u32 = 103;
/// `KEY_LEFT`.
const LEFT: u32 = 105;
/// `KEY_RIGHT`.
const RIGHT: u32 = 106;
/// `KEY_DOWN`.
const DOWN: u32 = 108;

/// How much of the viewport one press moves, as a divisor.
///
/// Four: a quarter of the screen, so four presses cross it and each one keeps
/// three quarters of what was already shown. See the header for why this is a
/// fraction rather than a count of pixels.
const A_STEP: i32 = 4;

impl crate::Server {
    /// Whether Shift is held, which is the design file's *pan faster*.
    ///
    /// Read from the seat's own modifier state, like
    /// `crate::canvas_wheel_zoom::ctrl_is_held` — one answer to *is Shift down*,
    /// rather than a second record kept beside the first.
    fn shift_is_held(&self) -> bool {
        self.surfaces
            .keyboard
            .as_ref()
            .is_some_and(|keyboard| keyboard.handle.modifier_state().shift)
    }

    /// Pan the canvas because an arrow was pressed with nothing focused.
    ///
    /// Whether the canvas took the key. `false` for every key that is not an
    /// arrow, for a release, where something has keyboard focus — *while the
    /// canvas is focused* is the design file's condition and an application with
    /// focus owns its own arrow keys — where there is no output to measure a step
    /// against, and where the pan would leave the plane.
    ///
    /// **A refused pan still answers `true`.** Pressing left at the left-hand edge
    /// of the plane is an ordinary thing to do and the screen already shows there
    /// is nowhere further; handing the key on to nobody would be the same key
    /// meaning two things depending on where the canvas happens to sit. This is
    /// `crate::canvas_command`'s reasoning about zooming at the end of the ladder,
    /// one road over.
    pub(crate) fn pan_the_plane_by_an_arrow(&mut self, code: u32, state: KeyState) -> bool {
        if state != KeyState::Pressed {
            return false;
        }
        if !matches!(code, UP | LEFT | RIGHT | DOWN) {
            return false;
        }
        if self
            .surfaces
            .keyboard
            .as_ref()
            .is_some_and(|keyboard| keyboard.handle.current_focus().is_some())
        {
            return false;
        }
        let Some(room) = self.surfaces.popups.output_size else {
            return false;
        };
        let (across, down) = if self.shift_is_held() {
            (room.w, room.h)
        } else {
            (room.w / A_STEP, room.h / A_STEP)
        };
        // Pressing an arrow moves the view that way, so the plane goes the other
        // way — the same relationship the space-drag has with the hand, and the
        // reason `pan_the_canvas` is given the camera's movement rather than the
        // eye's.
        let (x, y) = match code {
            LEFT => (-across, 0),
            RIGHT => (across, 0),
            UP => (0, -down),
            _ => (0, down),
        };
        let _ = self.pan_the_canvas(x, y);
        true
    }
}
