//! Which edge of the screen the dock is on.
//!
//! **Bottom is the default and the person chooses**, by the owner's ruling of
//! 2026-09-30 which reversed the bottom-only decision within the hour of it:
//!
//! > My earlier bottom-only ruling was too restrictive. **Bottom should be the
//! > default; the person can choose bottom, left, right, or top.** Design each
//! > orientation properly, including labels, overflow, the alo Bar, activation
//! > regions, and conflicts with other controls.
//!
//! [ADR
//! 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
//! carries that reversal, and its own second paragraph warns that its **title is
//! now wrong and deliberately unchanged** — so a reader who learns from the
//! filename that the dock is fixed to the bottom has been misled by a record
//! that says so.
//!
//! # What this type is not, yet
//!
//! **An `Edge` a `Layout` can be laid out along is not the same set as an `Edge`
//! this enum has.** All four are here because the person's choice is all four;
//! two of them can be laid out today. `crate::layout::NotLaidOut` is the
//! difference, and it is a type rather than a comment so that a caller handling
//! only the edges that work cannot forget the ones that do not.
//!
//! **This is also not a setting yet.** The owner's authorisation of 2026-10-04 is
//! explicit that the structural work comes first and that nonfunctional edge
//! choices are not to be exposed as finished settings — so nothing in
//! `crate::changes` offers this, and `crate::words` declares no name for an edge
//! until a person can pick one and have it work.

/// Which edge of the screen the dock sits on.
///
/// **Four, and the order is reading order rather than preference.** Bottom first
/// because it is the default; the rest as a person would say them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Edge {
    /// Along the bottom, which is where a fresh machine puts it.
    #[default]
    Bottom,
    /// Down the left side.
    Left,
    /// Down the right side.
    Right,
    /// Along the top.
    Top,
}

impl Edge {
    /// Every edge a person may choose, in reading order.
    ///
    /// Named as a constant rather than derived, so a fifth edge — there is no
    /// fifth edge — would be a deliberate act and the tests that walk this array
    /// would have to be read again.
    pub const EVERY: [Self; 4] = [Self::Bottom, Self::Left, Self::Right, Self::Top];

    /// Whether the dock runs across the screen rather than down it.
    ///
    /// **This is the whole of the geometric difference between the four.** A dock
    /// along the bottom and one along the top take their thickness out of the
    /// screen's **height** and run the whole of its **width**; one down either
    /// side does the opposite. Where on that edge it sits — which of bottom and
    /// top, which of left and right — is an origin and not an extent, and
    /// `alo-dock` does not place things on screens: `alo_shell`'s own layer does,
    /// which is why that question is not answered here.
    #[must_use]
    pub const fn runs_across(self) -> bool {
        matches!(self, Self::Bottom | Self::Top)
    }

    /// Whether a name can be drawn **under** an icon on this edge.
    ///
    /// True where the dock runs across the screen: the name's line sits below the
    /// picture and the dock's thickness grows by a line of text, which is
    /// `crate::Room::a_dock_with_names_under`'s arithmetic and is the same on the
    /// top edge as on the bottom.
    ///
    /// False down a side, and **not because there is no room** — a side dock has
    /// height to spare. It is that a name under an icon is constrained by the
    /// dock's *thickness*, which down a side is its width, so the question
    /// becomes how wide a name needs to be. That is a measurement this crate does
    /// not have; see `crate::layout::NotLaidOut`.
    #[must_use]
    pub const fn a_name_fits_under_an_icon(self) -> bool {
        self.runs_across()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Bottom is the default**, which is the half of the owner's ruling that is
    /// not about choice.
    #[test]
    fn a_fresh_machine_has_its_dock_along_the_bottom() {
        assert_eq!(Edge::default(), Edge::Bottom);
    }

    /// **Two run across and two run down, and that is the only geometric split.**
    ///
    /// Asserted as a partition rather than per-variant, so an edge added without
    /// an answer here fails rather than defaulting into one of the halves.
    #[test]
    fn exactly_two_edges_run_across_the_screen() {
        let across: Vec<Edge> = Edge::EVERY
            .into_iter()
            .filter(|edge| edge.runs_across())
            .collect();
        assert_eq!(across, vec![Edge::Bottom, Edge::Top]);
        let down: Vec<Edge> = Edge::EVERY
            .into_iter()
            .filter(|edge| !edge.runs_across())
            .collect();
        assert_eq!(down, vec![Edge::Left, Edge::Right]);
    }

    /// **A name goes under an icon on exactly the edges that run across.**
    ///
    /// The two questions are the same answer today and are **not** the same
    /// question: one is about which side of the screen the thickness comes out
    /// of, the other about whether a line of text fits below a picture. They are
    /// held together here so that a change to one which should have changed both
    /// is a failing test rather than a dock with names nobody measured.
    #[test]
    fn names_go_under_an_icon_on_the_edges_that_run_across() {
        for edge in Edge::EVERY {
            assert_eq!(
                edge.a_name_fits_under_an_icon(),
                edge.runs_across(),
                "{edge:?} disagrees about names and orientation"
            );
        }
    }
}
