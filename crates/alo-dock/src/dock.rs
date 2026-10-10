//! The dock on this machine: what the release ships, what the person changed,
//! and the two of them resolved at the moment of asking.
//!
//! Nothing is worked out at load time. A person who turns their text up gets the
//! answer on the next frame rather than at the next sign-in — and a settings
//! panel asking *what would this look like at 200% on the projector* asks exactly
//! the question the compositor asks when the projector is plugged in.
//!
//! **Where the dock is, is not a question this file answers.** It is along the
//! bottom, by [ADR
//! 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md),
//! on every screen. This file read *One dock, one edge* and then *one dock per
//! display* before that record; both are gone, along with the per-display
//! exception, and the record is why.
//!
//! **What it does answer, it answers for the first time.** `Hiding` has existed
//! since 2026-09-27 and was stored by [`Changes`] without anything resolving it
//! against what the release ships — so *whether the dock gives way* had a written
//! answer that nothing could read back. [`Dock::hiding`] is that resolution, and
//! it is here because a `Shipped` holding one setting and a `Dock` that could not
//! read it would be the same half-wired setting with fewer places to notice.
//!
//! [`Dock::layout_on`] takes a screen and answers about its size.

use alo_appearance::TextScale;

use crate::changes::{Changes, Setting};
use crate::hiding::{Hiding, Showing, TheRoom};
use crate::layout::Layout;
use crate::screen::Screen;
use crate::shipped::Shipped;

/// The dock on this machine.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dock {
    /// What this release ships.
    shipped: Shipped,
    /// What the person changed, which is the only part written down.
    changes: Changes,
}

impl Dock {
    /// A dock as it is before anybody changes anything.
    #[must_use]
    pub fn shipped() -> Self {
        Self::default()
    }

    /// The same, over a default that is not the shipped one.
    #[must_use]
    pub fn over(shipped: Shipped) -> Self {
        Self {
            shipped,
            changes: Changes::untouched(),
        }
    }

    /// The changes read out of a settings file, applied over these defaults.
    ///
    /// **A file is not a settings panel**: what it says is what its owner
    /// decided.
    #[must_use]
    pub fn with(mut self, changes: Changes) -> Self {
        self.changes = changes;
        self
    }

    /// What has been changed, which is what gets written down.
    #[must_use]
    pub const fn changes(&self) -> &Changes {
        &self.changes
    }

    /// What this release ships.
    #[must_use]
    pub const fn shipped_dock(&self) -> Shipped {
        self.shipped
    }

    /// Whether the dock gives way when a window needs the room: the person's
    /// choice if they made one, and what the release ships otherwise.
    #[must_use]
    pub fn hiding(&self) -> Hiding {
        self.changes
            .hiding()
            .unwrap_or_else(|| self.shipped.hiding())
    }

    /// Say whether the dock gives way when a window needs the room.
    pub const fn set_hiding(&mut self, hiding: Hiding) {
        self.changes.set_hiding(hiding);
    }

    /// Whether the dock is on the screen at this moment, given what the windows
    /// want of the room it is in.
    #[must_use]
    pub fn showing(&self, room: TheRoom) -> Showing {
        self.hiding().showing(room)
    }

    /// Put one setting back to what this release ships.
    ///
    /// Says whether there was anything to put back.
    pub const fn put_back(&mut self, setting: Setting) -> bool {
        self.changes.forget(setting)
    }

    /// Put everything back to what this release ships.
    pub fn put_everything_back(&mut self) {
        self.changes.forget_everything();
    }

    /// The dock laid out along the bottom of this screen, with the text at this
    /// size.
    #[must_use]
    pub fn layout_on(&self, screen: Screen, text: TextScale) -> Layout {
        Layout::along(self.edge(), screen, text)
    }

    /// Which edge of the screen this dock is on.
    ///
    /// The shipped edge, because nothing can change it yet — `crate::Changes`
    /// carries no edge, by the owner's order of work of 2026-10-04. When a person
    /// can choose one, this is the method that starts consulting the change and
    /// **every caller already asks the right question**, which is the reason it
    /// exists now rather than then.
    #[must_use]
    pub const fn edge(&self) -> crate::Edge {
        self.shipped.edge()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::labels::Labels;

    /// A machine nobody has touched has the dock the release ships, and the one
    /// thing a person can change shows up in the settings file.
    #[test]
    fn a_fresh_dock_is_what_the_release_ships_and_a_person_can_change_it() {
        let mut dock = Dock::shipped();
        assert_eq!(dock.hiding(), Hiding::Never);
        assert!(dock.changes().is_untouched());

        dock.set_hiding(Hiding::WhenAWindowNeedsTheRoom);
        assert_eq!(dock.hiding(), Hiding::WhenAWindowNeedsTheRoom);
        assert!(!dock.changes().is_untouched());

        assert!(dock.put_back(Setting::Hiding));
        assert_eq!(dock.hiding(), Hiding::Never, "and it goes back");
        assert!(dock.changes().is_untouched());
    }

    /// **A release can move the default and reach every machine that never
    /// touched it, and no machine that did.** That is the whole reason only the
    /// difference is stored, and this is the test that says it works.
    #[test]
    fn a_new_default_reaches_the_untouched_machine_and_not_the_touched_one() {
        let untouched = Dock::over(Shipped::of(Hiding::WhenAWindowNeedsTheRoom));
        assert_eq!(untouched.hiding(), Hiding::WhenAWindowNeedsTheRoom);

        let mut chosen = Changes::untouched();
        chosen.set_hiding(Hiding::Never);
        let theirs = Dock::over(Shipped::of(Hiding::WhenAWindowNeedsTheRoom)).with(chosen);
        assert_eq!(
            theirs.hiding(),
            Hiding::Never,
            "their choice survives the release"
        );
    }

    /// **The choice reaches the answer the compositor actually asks.** A dock
    /// that stored *give way* and still reported itself shown would be a setting
    /// with nothing behind it, which is what this was until the edge came out.
    #[test]
    fn a_dock_asked_to_give_way_does_so_only_when_a_window_needs_the_room() {
        let mut dock = Dock::shipped();
        assert_eq!(dock.showing(TheRoom::AWindowNeedsIt), Showing::Shown);

        dock.set_hiding(Hiding::WhenAWindowNeedsTheRoom);
        assert_eq!(dock.showing(TheRoom::AWindowNeedsIt), Showing::Hidden);
        assert_eq!(dock.showing(TheRoom::Free), Showing::Shown);
    }

    /// **Nothing is worked out at load time.** The same dock answers about two
    /// screens and two text sizes without being rebuilt, so a panel previewing a
    /// change asks the question the compositor asks.
    #[test]
    fn one_dock_answers_about_whichever_screen_it_is_drawn_on() {
        let dock = Dock::shipped();
        let laptop = Screen::the_smallest();
        let desk = Screen::of(3840, 2160).unwrap();
        let large = TextScale::percent(300).unwrap();

        assert!(!dock.layout_on(laptop, large).labels().are_shown());
        assert!(
            dock.layout_on(desk, large).labels().are_shown(),
            "the same dock, the same text, a bigger screen"
        );
        assert_eq!(
            dock.layout_on(laptop, TextScale::ordinary()).labels(),
            Labels::Under
        );
    }

    /// **Every screen gets the same dock.** There is no per-display exception to
    /// be had, which is the point of ADR 0076 rather than a gap in this file: two
    /// screens laid out at the same size answer identically.
    #[test]
    fn two_screens_of_a_size_are_one_layout() {
        let dock = Dock::shipped();
        let text = TextScale::ordinary();
        let one = Screen::of(1920, 1080).unwrap();
        let other = Screen::of(1920, 1080).unwrap();
        assert_eq!(dock.layout_on(one, text), dock.layout_on(other, text));
    }

    /// Putting everything back is one call, and it leaves the machine as though
    /// nobody had touched it.
    #[test]
    fn everything_can_be_put_back_at_once() {
        let mut dock = Dock::shipped();
        dock.set_hiding(Hiding::WhenAWindowNeedsTheRoom);
        dock.put_everything_back();
        assert!(dock.changes().is_untouched());
        assert_eq!(dock, Dock::shipped());
    }
}
