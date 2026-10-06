//! **The session holds the arrangement of its screens** —
//! `docs/autonomy/more-than-one-display-plan.md` task 3a.
//!
//! After task 3 this compositor knows **how big** each display is and **where
//! none of them is**. `ScreenPlace` has carried `at: Position` — *its corner
//! on the desk the arrangement makes* — since before any of this, and nothing
//! in production ever built a [`crate::Screens`]: the only caller of
//! `Screens::of` was `screens_testing.rs`.
//!
//! Three tasks wait on that and would each have found it separately. Task 5
//! lays the desktop out per display, task 6 constrains a popup to the screen
//! it is on, task 7 gives each viewport a camera — and *which display*, *the
//! left display's edge* and *a viewport rectangle* are all questions about
//! position.
//!
//! # What this file does and deliberately does not
//!
//! It **describes** the displays the compositor has presented, in
//! `alo-displays`' own words, and keeps whatever arrangement comes back. It
//! **decides nothing** about where a screen sits: that is `alo-displays`'
//! judgement from what a person arranged, and reaching it needs their kept
//! `Changes`, an `Appearance` and a `Tonight` — none of which this crate may
//! read. `crate::TheDesktop::the_screens_of` is the road, for the reason the
//! canvas layout and the shortcuts already travel it.
//!
//! # A description is not an arrangement, and a refusal here is not a fault
//!
//! `Reported::of` refuses a display with no pixels or with a physical size of
//! zero in one direction. Those are **real refusals about one display** and
//! they do not stop the others being described — a machine with one good
//! screen and one that reports nonsense is a machine a person can use, which
//! is the same answer task 2 gives for a display that cannot be routed.

use alo_displays::{Reported, Socket};

use crate::Server;

impl Server {
    /// Every presented display, described in `alo-displays`' words.
    ///
    /// Built from the per-display metadata task 3 began keeping: the
    /// connector's name is its socket, the make and model are its panel where
    /// it gave any, the current mode is its pixels, and the physical size is
    /// its glass.
    ///
    /// **A display this cannot describe is left out rather than guessed at.**
    /// `Reported::of` refuses no pixels and a zero physical dimension, and an
    /// invented figure for either would reach `Scale::worked_out_for` and
    /// become a size somebody's text is drawn at.
    ///
    /// # No panel is offered at all, and the cost of that is real
    ///
    /// `Panel::of` wants a make, a model and a serial, and **nothing in
    /// `OutputMetadata` carries a serial.** A panel built from make and model
    /// alone is not an identity: `Identity::Panel` *prefers* the panel when
    /// one is given, so **two identical monitors become one screen** — which
    /// is not a prediction, it is what the first version of this did. Two
    /// displays described, one identity, measured by a test that expected to
    /// see both named.
    ///
    /// So the socket is offered and the panel is not, and the screen is
    /// identified by the connector it is plugged into.
    ///
    /// **What that costs, said plainly:** a monitor moved from one port to
    /// another is a *different* screen to the arrangement, so whatever a
    /// person arranged for it is not found again. That is a real loss and it
    /// is the lesser one. A forgotten arrangement is a person dragging a
    /// window back; two live displays collapsed into one is an arrangement
    /// that cannot describe the desk in front of them at all.
    ///
    /// **And the fix is upstream rather than here.** A serial reaches this
    /// crate only if the compositor reads EDID, which it does not;
    /// `OutputMetadata` is make, model, name, physical size and refresh.
    /// Inventing one would be worse than having none — it would make the two
    /// monitors *stably* one screen instead of visibly one, which is the
    /// failure that survives a restart.
    #[must_use]
    pub fn the_displays_as_reported(&self) -> Vec<Reported> {
        self.presentations
            .values()
            .filter_map(|presented| {
                let metadata = presented.metadata.as_ref()?;
                let mode = presented.output.as_ref()?.current_mode()?;
                let socket = Socket::named(&metadata.name).ok()?;
                // No panel: see this function's own note. A make and a model
                // without a serial identify a *kind* of monitor, not a
                // monitor, and `Identity` prefers a panel when it is given
                // one.
                let panel = None;
                let pixels = (
                    u32::try_from(mode.size.w).ok()?,
                    u32::try_from(mode.size.h).ok()?,
                );
                let millimetres = match metadata.physical_size {
                    // `OutputMetadata` says (0, 0) means unknown, and
                    // `Reported::of` refuses a zero as a fault. The two
                    // vocabularies disagree about what a zero is, so it is
                    // translated here rather than passed on.
                    (0, _) | (_, 0) => None,
                    (across, down) => {
                        Some((u32::try_from(across).ok()?, u32::try_from(down).ok()?))
                    }
                };
                Reported::of(socket, panel, pixels, millimetres).ok()
            })
            .collect()
    }

    /// The screens this session is drawing on, as the desktop arranged them.
    ///
    /// [`None`] until a desktop answers with one, which is every session that
    /// was not given a display model — and every surface then lays out
    /// against the one viewport exactly as it did before.
    #[must_use]
    pub fn the_screens(&self) -> Option<&crate::Screens> {
        self.screens.as_ref()
    }

    /// Keep the arrangement the desktop made of this session's displays.
    pub fn these_screens_are(&mut self, screens: Option<crate::Screens>) {
        self.screens = screens;
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_session_holds_its_screens_tests.rs"]
mod tests;
