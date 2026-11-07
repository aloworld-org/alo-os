//! **The view glides**, rather than jumping, when alo moves it for a person — and
//! jumps when the person has asked for less motion.
//!
//! `docs/design/where-a-new-window-opens.md`, amended 2026-11-07: when a new
//! window has no room in view, *the view moves smoothly to the nearest free
//! space*. And two rules about how: **the glide follows the person's
//! reduced-motion setting** — with it on, the view jumps rather than glides —
//! and **its length is a duration, never a speed the person has to match**.
//! `docs/design/the-canvas-as-a-workspace.md` §4 says the same of every camera
//! transition: *reduced-motion preferences apply to camera transitions*.
//!
//! # A duration, so a long way is not a long wait
//!
//! [`THE_GLIDE_LASTS`] is the same however far the view goes. A glide paced as a
//! speed would make a far space slow to reach and a near one a twitch; a fixed
//! time is what a person can get used to, and it is never longer than that time.
//!
//! # The person's hand always wins
//!
//! Anything else that moves the camera — a pan, a zoom, *Show all*, going to a
//! window — goes through `Server::set_the_camera`, and that **ends the glide**
//! where it is. A glide that carried on after a person had taken hold of the
//! canvas would be a canvas fighting them. A glide also ends if the Place being
//! looked at changes, because the point it was heading for is on another one.
//!
//! # Only the person's own opening glides
//!
//! This file moves the view when asked; who may ask is
//! `crate::a_new_window_is_placed`'s, which asks only for a window the person
//! opened — rule 8: *background applications and alo must not … move the camera
//! without an authorised interaction that calls for it.*

use std::time::{Duration, Instant};

use alo_canvas::{At, Place};

/// How long the view takes to glide, however far it goes.
///
/// **300 ms, chosen by this lane on 2026-11-07, because the design is silent.**
/// It is the length the common desktop guidelines give a transition that moves
/// the whole view — long enough to see where things went, short enough that
/// nobody waits for it. Recorded in `docs/design/where-a-new-window-opens.md`
/// as the owner's to change.
pub(crate) const THE_GLIDE_LASTS: Duration = Duration::from_millis(300);

/// A glide in flight: from where the view was to where it is going.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheGlide {
    /// The display whose view is moving, by name.
    display: String,
    /// The Place it is moving across.
    place: Place,
    /// Where the view's corner was when it began.
    from: At,
    /// Where the view's corner is going.
    to: At,
    /// When it began.
    began: Instant,
    /// How long it lasts.
    lasts: Duration,
}

impl TheGlide {
    /// Where the view's corner is at `now`, and whether the glide has ended.
    ///
    /// Eased in and out, so it starts and stops gently rather than at full
    /// speed. At the end it is **exactly** where it was going, never a unit
    /// short from rounding.
    fn at(&self, now: Instant) -> (At, bool) {
        let gone = now.saturating_duration_since(self.began);
        if self.lasts.is_zero() || gone >= self.lasts {
            return (self.to, true);
        }
        let t = gone.as_secs_f64() / self.lasts.as_secs_f64();
        let eased = eased(t);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a point between two points on the plane is on the plane, well inside i32"
        )]
        let between = |from: i32, to: i32| -> i32 {
            let moved = (f64::from(to) - f64::from(from)) * eased;
            (f64::from(from) + moved.round()) as i32
        };
        let at = At::checked(
            between(self.from.x, self.to.x),
            between(self.from.y, self.to.y),
        )
        .unwrap_or(self.to);
        (at, false)
    }

    /// Where it is going.
    pub(crate) const fn going_to(&self) -> At {
        self.to
    }
}

/// Ease in and out, cubic: slow at both ends, fastest in the middle.
fn eased(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let back = -2.0 * t + 2.0;
        1.0 - back * back * back / 2.0
    }
}

impl crate::Server {
    /// **Whether the person asked for less motion**, as their accessibility
    /// settings say — told at stand-up, like the rest of what they chose.
    ///
    /// Told rather than read, for `Server::the_window_titles_are`'s reason: a
    /// compositor that opened a person's settings file would be a compositor
    /// measuring. Off until told, which is how this computer comes.
    pub fn the_motion_is_reduced(&mut self, reduced: bool) {
        self.motion_is_reduced = reduced;
        if reduced && let Some(glide) = self.the_glide.take() {
            // Asked for less motion mid-glide: finish it now.
            self.the_glide_writes(&glide, glide.to);
        }
    }

    /// Whether the person asked for less motion.
    #[must_use]
    pub const fn motion_is_reduced(&self) -> bool {
        self.motion_is_reduced
    }

    /// **Take the view to `to`** on the display being worked on, gliding — or
    /// jumping, where the person asked for less motion. Answers whether the
    /// view is moving, which is false where it is already there.
    pub(crate) fn the_view_glides_to(&mut self, to: At, now: Instant) -> bool {
        let camera = self.the_camera();
        if camera.at() == to {
            return false;
        }
        if self.motion_is_reduced {
            return self.look_at_the_canvas(to).is_some();
        }
        // Anything still in flight is replaced from where the view is now, so
        // two windows opened in quick succession glide on from the first
        // glide's position rather than jumping back to where it began.
        self.the_glide = Some(TheGlide {
            display: self.the_display_being_worked_on(),
            place: self.the_place_now(),
            from: camera.at(),
            to,
            began: now,
            lasts: THE_GLIDE_LASTS,
        });
        true
    }

    /// **Move a glide on to `now`**, once a frame before anything is drawn.
    /// Answers whether a glide is still in flight afterwards.
    pub fn the_glide_moves_on(&mut self, now: Instant) -> bool {
        let Some(glide) = self.the_glide.take() else {
            return false;
        };
        if glide.place != self.the_place_now() {
            // The person went to another Place; this point is on the old one.
            return false;
        }
        let (at, ended) = glide.at(now);
        self.the_glide_writes(&glide, at);
        if !ended {
            self.the_glide = Some(glide);
        }
        !ended
    }

    /// Whether the view is gliding.
    #[must_use]
    pub const fn the_view_is_gliding(&self) -> bool {
        self.the_glide.is_some()
    }

    /// Where the view will be once any glide has ended: where a person will be
    /// looking, which is what a window opening mid-glide is placed against.
    #[must_use]
    pub(crate) fn the_camera_after_the_glide(&self) -> alo_canvas::Camera {
        let camera = self.the_camera();
        match &self.the_glide {
            Some(glide) if glide.display == self.the_display_being_worked_on() => {
                camera.looking_at(glide.going_to()).unwrap_or(camera)
            }
            _ => camera,
        }
    }

    /// The one write a glide makes, which is the one camera write that does not
    /// end a glide — every other goes through `set_the_camera`, which does.
    fn the_glide_writes(&mut self, glide: &TheGlide, at: At) {
        let camera = self.surfaces.camera_of(&glide.display);
        if let Some(moved) = camera.looking_at(at) {
            self.surfaces.cameras.insert(glide.display.clone(), moved);
        }
    }
}

#[cfg(test)]
#[path = "the_view_glides_tests.rs"]
mod tests;
