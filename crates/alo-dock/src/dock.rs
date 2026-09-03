//! The dock on this machine: what the release ships, what the person changed,
//! and the two of them resolved at the moment of asking.
//!
//! Nothing is worked out at load time. A person who moves their dock, or turns
//! their text up, gets the answer on the next frame rather than at the next
//! sign-in — and a settings panel asking *what would this look like at 200% on
//! the projector* asks exactly the question the compositor asks when the
//! projector is plugged in.
//!
//! **One dock, one edge.** *Per display, so the dock can sit along the bottom of
//! the laptop and down the side of the external screen* is v0.5 in
//! `docs/features.md`, so [`Dock::layout_on`] takes a screen and answers about
//! it rather than the dock holding one screen's answer. That is also why the
//! v0.5 setting is additive: a display singled out becomes an exception to the
//! edge, exactly as `alo-appearance` made a display an exception to a
//! background.

use alo_appearance::TextScale;
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

    /// Put the dock on this edge.
    pub fn set_edge(&mut self, edge: Edge) {
        self.changes.set_edge(edge);
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
}
