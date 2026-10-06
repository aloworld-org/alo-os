//! **A frame is drawn per display** —
//! `docs/autonomy/more-than-one-display-plan.md` task 4.
//!
//! One display is drawn by calling `render_frame` with its target. Several are
//! drawn by calling it with each, and **the whole of this task that is not
//! device plumbing is what happens when one of them refuses.**
//!
//! # A display that fails does not take the others with it
//!
//! The acceptance's third clause, and the one that decides whether a machine
//! with a broken output is usable: *a display that fails to submit does not
//! stop the other from painting, and says which one failed.*
//!
//! `render_frame` answers `Result`, so a caller written the obvious way — `for
//! target in targets { self.render_frame(target, time)?; }` — stops at the
//! first refusal and leaves every display after it black. With one display
//! that `?` is correct and is what the loop has always done. With two it is a
//! single bad monitor turning the working one off.
//!
//! So every display is drawn, every refusal is kept, and the answer names
//! **which** display refused rather than that something did. A caller that
//! only wants to know whether anything is on the screen asks
//! [`DrawnPerDisplay::anything_drawn`]; one that has to say what went wrong
//! reads [`DrawnPerDisplay::refusals`], which carries the display's own name.
//!
//! # The order is the displays' own
//!
//! Drawn in the order discovery gave them — a usable internal panel first,
//! then by ascending connector — so the display that was the only one is still
//! drawn first, and a refusal on a later display cannot delay the first
//! display's frame.

use crate::{FrameTarget, RenderError, Server};

/// Disable every one of these displays, and name the ones that refused.
///
/// **Each is attempted even where an earlier one refused**, which is the same
/// rule as [`Server::render_each_display`] and for the same reason: a backend
/// that could not be disabled is a reason to report, never a reason to leave
/// the next display lit. A session ending with one monitor still scanning out
/// is a machine whose screen never goes dark.
///
/// This retires **backends**, not `wl_output` globals. Task 3 split the two
/// apart — see [`Server::retire_output`], which withdraws every display's
/// global once, after the last backend has gone.
pub fn retire_each_display<T: FrameTarget + ?Sized>(
    displays: &mut [&mut T],
) -> Vec<(String, RenderError)> {
    let mut refusals = Vec::new();
    for (which, display) in displays.iter_mut().enumerate() {
        // Asked **before** the retirement: a retired backend is not obliged
        // to still answer for its own identity, and a refusal that could not
        // say which display it came from would be the thing this returns a
        // name to avoid.
        let named = the_name_of(*display, which);
        if let Err(why) = display.retire() {
            refusals.push((named, why));
        }
    }
    refusals
}

/// What to call a display, including one that will not say.
///
/// A target whose metadata refuses is still a display somebody has to be told
/// about, and *an unnamed display refused* is a worse answer than the
/// position it was handed over in.
fn the_name_of<T: FrameTarget + ?Sized>(display: &T, which: usize) -> String {
    display.metadata().map_or_else(
        |_| format!("the display handed over {which}th"),
        |it| it.name,
    )
}

/// What became of one frame across every display.
#[derive(Debug, Default)]
pub struct DrawnPerDisplay {
    /// How many surfaces each display that drew reported submitting.
    drawn: Vec<(String, usize)>,
    /// The displays that refused, each with its own reason.
    refusals: Vec<(String, RenderError)>,
}

impl DrawnPerDisplay {
    /// Whether any display put a frame on a screen.
    ///
    /// **The question a loop asks**, because a session whose every display
    /// refused is a session showing nothing, while one display of two
    /// refusing is a machine a person can still use.
    #[must_use]
    pub fn anything_drawn(&self) -> bool {
        !self.drawn.is_empty()
    }

    /// Every display that refused, and why.
    ///
    /// Named rather than counted: *a display failed* sends somebody to look at
    /// both, and the name is the whole difference between a diagnosis and a
    /// hunt.
    #[must_use]
    pub fn refusals(&self) -> &[(String, RenderError)] {
        &self.refusals
    }

    /// Every display that drew, and how many surfaces it submitted.
    #[must_use]
    pub fn drawn(&self) -> &[(String, usize)] {
        &self.drawn
    }

    /// Take the refusals, so a caller may raise one rather than describe it.
    ///
    /// **A lane with one display needs this and a lane with two does not.**
    /// Where there is one display, *nothing drew* and *this display refused*
    /// are the same fact, and the honest thing is to raise that display's own
    /// error with its own cause attached — which borrowing cannot do, because
    /// [`RenderError`] is not `Clone` and a rebuilt error loses its source.
    /// Where there are two, a caller reports instead of raising, and reads
    /// [`refusals`](Self::refusals) without taking anything.
    #[must_use]
    pub fn into_refusals(self) -> Vec<(String, RenderError)> {
        self.refusals
    }
}

impl Server {
    /// Draw this frame on every display, and say what became of each.
    ///
    /// **No `?` over the displays.** A refusal is recorded against the display
    /// that made it and the next display is still drawn — see this file's
    /// header for why the obvious loop is wrong the moment there are two.
    ///
    /// The name is the display's own, taken from the target's metadata before
    /// anything is submitted, so a display that refuses *at* submission is
    /// still named in the refusal. A target whose metadata cannot be read at
    /// all is named by the position it was handed over in, which is the only
    /// thing true about it.
    pub fn render_each_display<T: FrameTarget + ?Sized>(
        &mut self,
        displays: &mut [&mut T],
        time: u32,
    ) -> DrawnPerDisplay {
        let mut became = DrawnPerDisplay::default();
        for (which, display) in displays.iter_mut().enumerate() {
            let named = the_name_of(*display, which);
            match self.render_frame(*display, time) {
                Ok(submitted) => became.drawn.push((named, submitted)),
                Err(why) => became.refusals.push((named, why)),
            }
        }
        became
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_frame_per_display_tests.rs"]
mod tests;
