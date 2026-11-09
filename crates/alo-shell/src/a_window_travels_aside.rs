//! **A window put aside travels to its card in the panel**, rather than
//! vanishing — and does not travel at all where the person asked for less
//! motion.
//!
//! `docs/design/the-window-controls.md`: *Minimise — puts this window aside,
//! travelling toward the panel of put-aside windows on the right*. And
//! `docs/features.md`, of reduced motion: *everything alo animates jumps to
//! where it is going instead … the next animation anybody builds is held to the
//! same switch.* This is that next animation, so it is held to it.
//!
//! **This makes an existing promise true rather than adding one**: the travel
//! is written into the Minimise control's own sentence, and nothing here gives
//! a person anything that sentence does not.
//!
//! # What travels is an outline, and the window is already gone
//!
//! The window is hidden the moment the panel takes it, exactly as before
//! (`crate::putting_a_window_aside`), so nothing a person can press or type
//! into changes while it travels — `docs/design/the-windows-put-aside.md`:
//! *animations must not change hit targets midway through a pointer move*.
//! What moves is an **outline of where the window was**, drawn in the shell's
//! own layer and answering no input, shrinking toward the window's card. Not
//! the window's own pixels: the drawing path has no per-window transform, and
//! giving it one is a change to the painter rather than to minimising.
//!
//! # Where it goes is asked every frame
//!
//! The card is where the panel lays it out **revealed**
//! (`crate::where_a_travelling_window_lands`), because the panel is usually
//! concealed while a window is minimised, and a concealed panel draws no cards.
//! Asked each frame rather than once: a second window put aside mid-travel
//! takes the top card and moves the first one down, and the first outline then
//! ends on the card its window really has. **A window with no card does not
//! travel** — it is still put aside, as it always was.
//!
//! # It is never longer than the glide
//!
//! [`THE_TRAVEL_LASTS`] is the view glide's length, for the glide's reasons.

use std::time::{Duration, Instant};

use alo_dock::WindowId;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Physical, Rectangle};

use crate::ease_in_and_out::eased;

/// How long a window takes to travel to its card, however far it goes.
///
/// **300 ms, the view glide's length** (`crate::the_view_glides::THE_GLIDE_LASTS`),
/// because no document names one for this and
/// `docs/design/where-a-new-window-opens.md` has: *the glide lasts 300 ms
/// however far it goes, eased in and out.* Its own constant rather than that
/// one, because the two are separate decisions that happen to agree today, and
/// the owner may change one without the other. Recorded in
/// `docs/design/the-window-controls.md` as the owner's to change.
pub(crate) const THE_TRAVEL_LASTS: Duration = Duration::from_millis(300);

/// One window's travel: from where it was on the screen, toward its card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheTravel {
    /// The window, as the panel knows it — which is how its card is found.
    window: WindowId,
    /// Where the window was drawn when it was put aside, in display pixels.
    from: Rectangle<i32, Physical>,
    /// When it was put aside.
    began: Instant,
    /// How long it lasts.
    lasts: Duration,
}

impl TheTravel {
    /// A travel for `window` from `from`, beginning at `began` — or `None`
    /// where there is nothing to travel: a window drawn at no size has no
    /// outline anybody could follow.
    #[must_use]
    pub(crate) fn from(
        window: WindowId,
        from: Rectangle<i32, Physical>,
        began: Instant,
    ) -> Option<Self> {
        if from.size.w <= 0 || from.size.h <= 0 {
            return None;
        }
        Some(Self {
            window,
            from,
            began,
            lasts: THE_TRAVEL_LASTS,
        })
    }

    /// Whether nobody saw it: its time was up a whole travel ago, so no frame
    /// was drawn while it moved — the screen was locked, say. Such a travel is
    /// let go rather than drawn late at its card.
    #[must_use]
    fn went_unseen(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.began) >= self.lasts.saturating_mul(2)
    }

    /// Where the outline is at `now`, travelling toward `to`, and whether the
    /// travel has ended.
    ///
    /// Eased in and out, its corner and its size together, so it shrinks as it
    /// goes rather than arriving and then shrinking. **At its end it is exactly
    /// the card**, never a pixel short from rounding, and that frame is its
    /// last.
    #[must_use]
    pub(crate) fn at(&self, now: Instant, to: Rectangle<i32, Physical>) -> (Rectangle<i32, Physical>, bool) {
        let gone = now.saturating_duration_since(self.began);
        if self.lasts.is_zero() || gone >= self.lasts {
            return (to, true);
        }
        let eased = eased(gone.as_secs_f64() / self.lasts.as_secs_f64());
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a length between two lengths on one display is between them, well inside i32"
        )]
        let between = |from: i32, to: i32| -> i32 {
            let moved = (f64::from(to) - f64::from(from)) * eased;
            (f64::from(from) + moved.round()) as i32
        };
        let from = self.from;
        let at = Rectangle::new(
            (between(from.loc.x, to.loc.x), between(from.loc.y, to.loc.y)).into(),
            (
                between(from.size.w, to.size.w),
                between(from.size.h, to.size.h),
            )
                .into(),
        );
        (at, false)
    }
}

/// Every travel in flight, one per window.
///
/// **Generic over what names a window** so the bookkeeping is tested without a
/// client: the server keys it by the window's surface, a test by a number.
///
/// **Not pruned against the windows that are showing**, as
/// `Server::controls_fading` is: every window here has just been hidden, so
/// that pruning would end every travel before its first frame. A travel ends
/// when its time is up, when its window comes back, when its card is gone, or
/// when the person asks for less motion.
#[derive(Debug, Clone)]
pub(crate) struct TheTravels<K> {
    /// The window and its travel, oldest first.
    travelling: Vec<(K, TheTravel)>,
}

impl<K: PartialEq> TheTravels<K> {
    /// Nothing travelling, which is how a session starts.
    #[must_use]
    pub(crate) const fn none() -> Self {
        Self {
            travelling: Vec::new(),
        }
    }

    /// Begin `travel` for `key`. A window that was already travelling begins
    /// again from where it is now put aside, rather than travelling twice.
    pub(crate) fn begin(&mut self, key: K, travel: TheTravel) {
        self.travelling.retain(|(it, _)| *it != key);
        self.travelling.push((key, travel));
    }

    /// End `key`'s travel at once — it came back mid-travel. Answers whether
    /// it was travelling.
    pub(crate) fn cancel(&mut self, key: &K) -> bool {
        let before = self.travelling.len();
        self.travelling.retain(|(it, _)| it != key);
        self.travelling.len() != before
    }

    /// End every travel at once — the person asked for less motion.
    pub(crate) fn end_all(&mut self) {
        self.travelling.clear();
    }

    /// How many are travelling.
    #[must_use]
    pub(crate) fn how_many(&self) -> usize {
        self.travelling.len()
    }

    /// **Move every travel on to `now`**, once a frame, and answer where each
    /// outline is, toward the card `card_of` answers for its window.
    ///
    /// A travel that has reached its card is answered at the card and then let
    /// go, so its last frame is drawn exactly there. **A travel whose window
    /// has no card is let go undrawn**, and so is one nobody saw.
    pub(crate) fn move_on(
        &mut self,
        now: Instant,
        card_of: impl Fn(WindowId) -> Option<Rectangle<i32, Physical>>,
    ) -> Vec<Rectangle<i32, Physical>> {
        let mut at = Vec::new();
        self.travelling.retain(|(_, travel)| {
            if travel.went_unseen(now) {
                return false;
            }
            let Some(card) = card_of(travel.window) else {
                return false;
            };
            let (outline, ended) = travel.at(now, card);
            at.push(outline);
            !ended
        });
        at
    }
}

impl crate::Server {
    /// **A window the person just put aside begins to travel to its card** —
    /// unless they asked for less motion, or it was drawn at no size.
    ///
    /// Asked from `put_aside_what_was_asked_for` only, after the panel took the
    /// window, so a window a client minimised itself never travels: it has no
    /// card to travel to.
    pub(crate) fn a_window_begins_to_travel_aside(
        &mut self,
        frame: &WlSurface,
        window: WindowId,
        now: Instant,
    ) {
        if self.motion_is_reduced {
            return;
        }
        // Hiding never moves a window, so where it is drawn is where it was.
        let drawn = self.where_a_window_is_drawn(frame);
        let from = Rectangle::new(
            (drawn.loc.x, drawn.loc.y).into(),
            (drawn.size.w, drawn.size.h).into(),
        );
        if let Some(travel) = TheTravel::from(window, from, now) {
            self.travelling_aside.begin(frame.clone(), travel);
        }
    }

    /// **How many windows are travelling to the panel**, for a frame that has
    /// nothing to lay out when none is, and a test that watches one end.
    #[must_use]
    pub fn how_many_windows_are_travelling_aside(&self) -> usize {
        self.travelling_aside.how_many()
    }

    /// **Move every travel on to `now` and answer where each outline is**, in
    /// display pixels, toward the cards `cards` lays out.
    ///
    /// Once a frame, from the frame that draws them: a travel moves on where
    /// it is drawn, so its last frame lands on its card rather than being let
    /// go a frame early by a step taken before the draw. A travel with no card
    /// among `cards` is let go undrawn.
    pub fn the_travels_aside_move_on(
        &mut self,
        now: Instant,
        cards: &crate::where_a_travelling_window_lands::TheCards,
    ) -> Vec<Rectangle<i32, Physical>> {
        self.travelling_aside.move_on(now, |window| cards.of(window))
    }

    /// End this window's travel: it was brought back mid-travel.
    pub(crate) fn this_window_stops_travelling_aside(&mut self, frame: &WlSurface) {
        let _was = self.travelling_aside.cancel(frame);
    }

    /// End every travel: the person asked for less motion.
    pub(crate) fn every_window_stops_travelling_aside(&mut self) {
        self.travelling_aside.end_all();
    }
}

#[cfg(test)]
#[path = "a_window_travels_aside_tests.rs"]
mod tests;
