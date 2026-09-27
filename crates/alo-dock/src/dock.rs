//! The dock on this machine: what the release ships, what the person changed,
//! and the two of them resolved at the moment of asking.
//!
//! Nothing is worked out at load time. A person who moves their dock, or turns
//! their text up, gets the answer on the next frame rather than at the next
//! sign-in — and a settings panel asking *what would this look like at 200% on
//! the projector* asks exactly the question the compositor asks when the
//! projector is plugged in.
//!
//! **One dock per display, and it was additive as this file said it would be.**
//! *Per display, so the dock can sit along the bottom of the laptop and down the
//! side of the external screen* is the `[v0.5]` promise in `docs/features.md`, and
//! [`Dock::edge_on`] is where it is answered: **a display singled out is an
//! exception to the edge, exactly as `alo-appearance` made a display an exception
//! to a background.** This file read *One dock, one edge* until 2026-09-27 and
//! quoted that promise as the thing it did not do.
//!
//! [`Dock::layout_on`] still takes a screen and answers about its size, and is
//! unchanged: the shell overrides the edge per screen already, from the answer
//! `alo_displays::Wearing::of` gives it for that screen, which is what let this
//! promise be paid in two crates and none of the drawing.
//!
//! What is still not here is the other half of the same line in
//! `docs/features.md`: *the dock's size, and whether it hides when a window needs
//! the room*. [`crate::layout`] sizes it; the hiding has no code, and says so
//! where it would go.

use alo_appearance::{DisplayId, TextScale};
use alo_strings::Direction;

use crate::changes::{Changes, Setting};
use crate::edge::Edge;
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
    /// decided. What it cannot say is an edge that does not exist, because that
    /// is checked where the file is read rather than here.
    #[must_use]
    pub fn with(mut self, changes: Changes) -> Self {
        self.changes = changes;
        self
    }

    /// What has been changed, which is what gets written down.
    #[must_use]
    pub fn changes(&self) -> &Changes {
        &self.changes
    }

    /// What this release ships.
    #[must_use]
    pub const fn shipped_dock(&self) -> Shipped {
        self.shipped
    }

    /// Which edge the dock is on: the person's choice if they made one, and what
    /// the release ships otherwise.
    #[must_use]
    pub fn edge(&self) -> Edge {
        self.changes.edge().unwrap_or_else(|| self.shipped.edge())
    }

    /// Put the dock on this edge, on every display not singled out.
    pub fn set_edge(&mut self, edge: Edge) {
        self.changes.set_edge(edge);
    }

    /// Which edge of this display the dock is on.
    ///
    /// The exception the person made for this display, or the edge they chose for
    /// everywhere, or the edge the release ships — in that order, which is
    /// `alo_appearance::Appearance::background_on`'s order for the same reason.
    ///
    /// This is the whole of *per display, so the dock can sit along the bottom of
    /// the laptop and down the side of the external screen*: two screens with
    /// different exceptions answer differently here, and
    /// `alo_displays::Wearing::of` asks it once per screen.
    #[must_use]
    pub fn edge_on(&self, display: &DisplayId) -> Edge {
        self.changes.edge_on(display).unwrap_or_else(|| self.edge())
    }

    /// Single this display out, putting the dock on this edge of it alone.
    pub fn set_edge_on(&mut self, display: DisplayId, edge: Edge) {
        self.changes.set_edge_on(display, edge);
    }

    /// Stop singling this display out, putting it back to the edge chosen for
    /// everywhere.
    ///
    /// Says whether there was anything to put back.
    pub fn put_display_back(&mut self, display: &DisplayId) -> bool {
        self.changes.forget_display(display)
    }

    /// Put one setting back to what this release ships.
    ///
    /// Says whether there was anything to put back.
    pub fn put_back(&mut self, setting: Setting) -> bool {
        self.changes.forget(setting)
    }

    /// Put everything back to what this release ships.
    pub fn put_everything_back(&mut self) {
        self.changes.forget_everything();
    }

    /// The dock laid out on this screen, with the text at this size, for
    /// somebody reading this way.
    #[must_use]
    pub fn layout_on(&self, screen: Screen, text: TextScale, reading: Direction) -> Layout {
        Layout::of(self.edge(), screen, text, reading)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::along::Along;
    use crate::labels::Labels;

    /// A machine nobody has touched has its dock where the release put it, and
    /// moving it is one call that shows up in the settings file.
    #[test]
    fn a_fresh_dock_is_where_the_release_put_it_and_a_person_can_move_it() {
        let mut dock = Dock::shipped();
        assert_eq!(dock.edge(), Edge::Bottom);
        assert!(dock.changes().is_untouched());

        dock.set_edge(Edge::Left);
        assert_eq!(dock.edge(), Edge::Left);
        assert!(!dock.changes().is_untouched());

        assert!(dock.put_back(Setting::Edge));
        assert_eq!(dock.edge(), Edge::Bottom, "and it goes back");
        assert!(dock.changes().is_untouched());
    }

    /// **A release can move the default and reach every machine that never
    /// touched it, and no machine that did.** That is the whole reason only the
    /// difference is stored, and this is the test that says it works.
    #[test]
    fn a_new_default_reaches_the_untouched_machine_and_not_the_touched_one() {
        let untouched = Dock::over(Shipped::of(Edge::Top));
        assert_eq!(untouched.edge(), Edge::Top);

        let mut moved = Changes::untouched();
        moved.set_edge(Edge::Right);
        let theirs = Dock::over(Shipped::of(Edge::Top)).with(moved);
        assert_eq!(
            theirs.edge(),
            Edge::Right,
            "their choice survives the release"
        );
    }

    /// Moving the dock changes the layout, and everything about the layout
    /// follows from the edge — which is what says the person has one decision to
    /// make rather than four.
    #[test]
    fn moving_the_dock_changes_how_it_is_laid_out() {
        let screen = Screen::the_smallest();
        let text = TextScale::ordinary();
        let mut dock = Dock::shipped();

        let across = dock.layout_on(screen, text, Direction::LeftToRight);
        assert_eq!(across.along(), Along::Across);
        assert_eq!(across.labels(), Labels::Under);
        assert_eq!(across.length(), screen.width());

        dock.set_edge(Edge::Right);
        let down = dock.layout_on(screen, text, Direction::LeftToRight);
        assert_eq!(down.along(), Along::Down);
        assert_eq!(down.labels(), Labels::Beside);
        assert_eq!(down.length(), screen.height());
        assert_ne!(down.thickness(), across.thickness());
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

        assert!(
            !dock
                .layout_on(laptop, large, Direction::LeftToRight)
                .labels()
                .are_shown()
        );
        assert!(
            dock.layout_on(desk, large, Direction::LeftToRight)
                .labels()
                .are_shown(),
            "the same dock, the same text, a bigger screen"
        );
    }

    /// Putting everything back is one call, and it leaves the machine as though
    /// nobody had touched it.
    #[test]
    fn everything_can_be_put_back_at_once() {
        let mut dock = Dock::shipped();
        dock.set_edge(Edge::Left);
        dock.put_everything_back();
        assert!(dock.changes().is_untouched());
        assert_eq!(dock, Dock::shipped());
    }

    /// **The laptop along the bottom and the external screen down the side**,
    /// which is the promise in the words `docs/features.md` uses.
    ///
    /// Two screens, one singled out, two different answers out of one dock. A
    /// screen nobody singled out gets the edge chosen for everywhere, and that
    /// order — exception, then everywhere, then shipped — is
    /// `alo_appearance::Appearance::background_on`'s.
    #[test]
    fn the_laptop_keeps_the_bottom_while_the_external_screen_takes_a_side() {
        let laptop = DisplayId::named("eDP-1 Built-in").unwrap();
        let desk = DisplayId::named("DP-3 Dell U2720Q").unwrap();

        let mut dock = Dock::shipped();
        assert_eq!(dock.edge_on(&laptop), Edge::Bottom, "the shipped edge");
        assert_eq!(dock.edge_on(&desk), Edge::Bottom);

        dock.set_edge_on(desk.clone(), Edge::Left);
        assert_eq!(dock.edge_on(&desk), Edge::Left, "the screen singled out");
        assert_eq!(
            dock.edge_on(&laptop),
            Edge::Bottom,
            "and the other screen is not touched by it"
        );

        // The edge for everywhere moves the screens nobody singled out, and
        // leaves the one that was.
        dock.set_edge(Edge::Top);
        assert_eq!(dock.edge_on(&laptop), Edge::Top);
        assert_eq!(dock.edge_on(&desk), Edge::Left);

        assert!(dock.put_display_back(&desk), "there was one to put back");
        assert_eq!(dock.edge_on(&desk), Edge::Top, "back to everywhere's");
        assert!(!dock.put_display_back(&desk), "and only once");
    }

    /// **Putting the edge back does not stop singling a screen out**, and vice
    /// versa: they are two things a person means and two calls.
    ///
    /// `Setting::Edge` says so in its own words, because a panel offering *put it
    /// back* has to offer the right one.
    #[test]
    fn the_edge_and_a_screen_singled_out_go_back_separately() {
        let desk = DisplayId::named("DP-3 Dell U2720Q").unwrap();
        let mut dock = Dock::shipped();
        dock.set_edge(Edge::Right);
        dock.set_edge_on(desk.clone(), Edge::Left);

        assert!(dock.put_back(Setting::Edge));
        assert_eq!(dock.edge(), Edge::Bottom, "everywhere went back");
        assert_eq!(
            dock.edge_on(&desk),
            Edge::Left,
            "and the screen singled out did not"
        );
        assert!(
            !dock.changes().is_untouched(),
            "a machine with one screen singled out has been changed"
        );

        dock.put_everything_back();
        assert!(dock.changes().is_untouched());
        assert_eq!(dock.edge_on(&desk), Edge::Bottom);
    }
}
