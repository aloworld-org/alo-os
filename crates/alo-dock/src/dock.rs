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

    /// The dock laid out along this person's chosen edge of this screen.
    ///
    /// **It took a text size until 2026-10-10** and no longer does: the only
    /// thing a dock's thickness ever read it for was the row of names under the
    /// icons, and the owner removed that row. A name is shown on hover and on
    /// keyboard focus, outside the bar, and never changes its height.
    #[must_use]
    pub fn layout_on(&self, screen: Screen) -> Layout {
        Layout::along(self.edge(), screen)
    }

    /// Which edge of the screen this dock is on: the person's choice if they
    /// made one, and what the release ships otherwise.
    ///
    /// `Self::hiding`'s shape, which this note promised: *when a person can
    /// choose one, this is the method that starts consulting the change, and
    /// every caller already asks the right question.* Nothing that calls it
    /// changed — the draw, the layout and the reveal regions were all asking
    /// `dock.edge()` already, which is why a person's choice reaches a screen
    /// through this one line.
    #[must_use]
    pub fn edge(&self) -> crate::Edge {
        self.changes.edge().unwrap_or_else(|| self.shipped.edge())
    }

    /// Say which edge of the screen the dock is on.
    pub const fn set_edge(&mut self, edge: crate::Edge) {
        self.changes.set_edge(edge);
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::Room;
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

    /// **The edge is release-coupled exactly as hiding is**, which is the whole
    /// point of putting it on `Changes` rather than leaving it on `Shipped`.
    ///
    /// Held separately from the test below rather than folded into it: the two
    /// settings share a shape and not an implementation, and a single test over
    /// both would pass while one of them read the other's field.
    #[test]
    fn a_new_default_edge_reaches_the_untouched_machine_and_not_the_touched_one() {
        let shipped = Shipped::of(Hiding::Never, crate::Edge::Left);
        let untouched = Dock::over(shipped);
        assert_eq!(
            untouched.edge(),
            crate::Edge::Left,
            "a machine that never chose follows the release"
        );

        let mut chosen = Changes::untouched();
        chosen.set_edge(crate::Edge::Top);
        let theirs = Dock::over(shipped).with(chosen);
        assert_eq!(
            theirs.edge(),
            crate::Edge::Top,
            "their choice survives a release that ships a different edge"
        );

        // **And the bottom is a choice, not an absence.** A person who picks the
        // bottom on purpose keeps it when a release moves the default away,
        // which is the case an `Option` exists for and a bare `Edge` could not
        // express.
        let mut deliberate = Changes::untouched();
        deliberate.set_edge(crate::Edge::Bottom);
        assert_eq!(
            Dock::over(shipped).with(deliberate).edge(),
            crate::Edge::Bottom,
            "choosing the bottom read as never having chosen"
        );
    }

    /// **Putting the edge back follows the release again.**
    #[test]
    fn putting_the_edge_back_returns_to_what_the_release_ships() {
        let shipped = Shipped::of(Hiding::Never, crate::Edge::Bottom);
        let mut dock = Dock::over(shipped);
        dock.set_edge(crate::Edge::Right);
        assert_eq!(dock.edge(), crate::Edge::Right);

        assert!(
            dock.put_back(Setting::WhereItGoes),
            "there was a choice to put back"
        );
        assert_eq!(dock.edge(), crate::Edge::Bottom);
        assert!(
            !dock.put_back(Setting::WhereItGoes),
            "and it is not there to put back twice"
        );
    }

    /// **Putting one setting back leaves the other alone.**
    ///
    /// Two settings on one `Changes` is the first time this could go wrong, and
    /// `forget` is a `match` that could name the wrong field in either arm
    /// while every single-setting test passed.
    #[test]
    fn putting_one_setting_back_does_not_disturb_the_other() {
        let mut dock = Dock::over(Shipped::of(Hiding::Never, crate::Edge::Bottom));
        dock.set_edge(crate::Edge::Top);
        dock.set_hiding(Hiding::WhenAWindowNeedsTheRoom);

        assert!(dock.put_back(Setting::WhereItGoes));
        assert_eq!(
            dock.hiding(),
            Hiding::WhenAWindowNeedsTheRoom,
            "putting the edge back forgot what they chose about hiding"
        );
        assert_eq!(dock.edge(), crate::Edge::Bottom);

        dock.set_edge(crate::Edge::Left);
        assert!(dock.put_back(Setting::Hiding));
        assert_eq!(
            dock.edge(),
            crate::Edge::Left,
            "putting hiding back forgot which edge they chose"
        );
        assert_eq!(dock.hiding(), Hiding::Never);
    }

    /// **A release can move the default and reach every machine that never
    /// touched it, and no machine that did.** That is the whole reason only the
    /// difference is stored, and this is the test that says it works.
    #[test]
    fn a_new_default_reaches_the_untouched_machine_and_not_the_touched_one() {
        let untouched = Dock::over(Shipped::of(
            Hiding::WhenAWindowNeedsTheRoom,
            crate::Edge::Bottom,
        ));
        assert_eq!(untouched.hiding(), Hiding::WhenAWindowNeedsTheRoom);

        let mut chosen = Changes::untouched();
        chosen.set_hiding(Hiding::Never);
        let theirs = Dock::over(Shipped::of(
            Hiding::WhenAWindowNeedsTheRoom,
            crate::Edge::Bottom,
        ))
        .with(chosen);
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
    /// screens without being rebuilt, so a panel previewing a change asks the
    /// question the compositor asks.
    ///
    /// **This asserted that names gave way on a small screen at 300% text, and
    /// that is behaviour the owner removed on 2026-10-10.** There is no row of
    /// names in the bar to give way: a name is shown on hover and on keyboard
    /// focus, outside the bar. So the thing to hold is the opposite one — that
    /// the dock is the same on both screens — and it is held below rather than
    /// deleted.
    #[test]
    fn one_dock_answers_about_whichever_screen_it_is_drawn_on() {
        let dock = Dock::shipped();
        let laptop = Screen::the_smallest();
        let desk = Screen::of(3840, 2160).unwrap();

        assert_eq!(
            dock.layout_on(laptop).thickness(),
            dock.layout_on(desk).thickness(),
            "the same dock is the same thickness on a small screen and a large one"
        );
        assert_eq!(dock.layout_on(laptop).labels(), Labels::Beside);
        assert!(
            dock.layout_on(laptop).length() < dock.layout_on(desk).length(),
            "the premise: these two screens really are different sizes, so the equality above \
             is about the dock and not about two identical inputs"
        );
    }

    /// **A dock's thickness does not move with the text size**, which is the
    /// ruling of 2026-10-10 held as a test rather than as a comment.
    ///
    /// It used to: `a_dock_with_names_under` added a line of text and made the
    /// bar **85** where the design measures 76. `layout_on` no longer takes a
    /// text size at all, so this test can only be written one way — and that is
    /// the point, because a parameter that is not there cannot be read by a
    /// later change.
    #[test]
    fn the_text_size_cannot_reach_the_docks_thickness() {
        let dock = Dock::shipped();
        let screen = Screen::of(1920, 1080).unwrap();
        assert_eq!(
            dock.layout_on(screen).thickness(),
            Room::a_dock_of_icons(),
            "the thickness is the icon and its two faces, and nothing else"
        );
        assert_eq!(
            Room::a_dock_of_icons().as_pixels(),
            76,
            "the measured height of `Dock + alo Bar`: 14 above a 48 target and 14 below"
        );
    }

    /// **Every screen gets the same dock.** Two screens laid out at the same
    /// size answer identically.
    #[test]
    fn two_screens_of_a_size_are_one_layout() {
        let dock = Dock::shipped();
        let one = Screen::of(1920, 1080).unwrap();
        let other = Screen::of(1920, 1080).unwrap();
        assert_eq!(dock.layout_on(one), dock.layout_on(other));
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
