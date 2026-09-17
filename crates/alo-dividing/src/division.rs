//! One display's division: the shares, and every change a person makes to them.
//!
//! A [`Division`] starts empty — a display nobody has divided — and becomes a
//! tree of shares when a window is dragged to an edge or split from the keyboard
//! ([`crate::dropping`], [`crate::keyboard`]). What is here is what every road
//! into it shares: reading the shares, halving one, moving the boundary between
//! two, and closing a window.
//!
//! **Decided, then applied.** Each change is worked out on a copy of the tree
//! and the copy replaces the division only when the whole change holds, so a
//! refusal is always a division left exactly as it was — the test of that is
//! beside every refusal in this file.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::area::Area;
use crate::node::{Branch, Node};
use crate::refusing::Refused;
use crate::share::Share;
use crate::side::Side;
use crate::window::{Window, WindowId};

/// Where the next division's identity comes from.
///
/// A proposal names the division it was made against, so a proposal made on the
/// laptop's screen cannot be committed on the external one.
static NEXT_DIVISION: AtomicU64 = AtomicU64::new(1);

/// How one display is divided.
///
/// Deliberately not `Clone`: a proposal is committed against the division it
/// came from, and two copies of one division would both accept it.
#[derive(Debug, PartialEq, Eq)]
pub struct Division {
    /// The display's area, in logical units.
    display: Area,
    /// The shares, or `None` while the display is not divided.
    tree: Option<Node>,
    /// Which division this is.
    identity: u64,
    /// How many changes have been made to it, so a stale proposal is refused.
    changes: u64,
}

impl Division {
    /// A display nobody has divided.
    #[must_use]
    pub fn of(display: Area) -> Self {
        Self {
            display,
            tree: None,
            identity: NEXT_DIVISION.fetch_add(1, Ordering::Relaxed),
            changes: 0,
        }
    }

    /// The display's area.
    #[must_use]
    pub const fn display(&self) -> Area {
        self.display
    }

    /// Whether nothing is divided.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.tree.is_none()
    }

    /// Every share, laid out, left to right and top to bottom through the tree.
    #[must_use]
    pub fn shares(&self) -> Vec<Share> {
        let mut shares = Vec::new();
        if let Some(tree) = &self.tree {
            tree.lay_out(self.display, &mut shares);
        }
        shares
    }

    /// Where this window's share is, if it has one.
    #[must_use]
    pub fn share_of(&self, window: WindowId) -> Option<Area> {
        self.shares()
            .into_iter()
            .find(|share| share.window() == window)
            .map(Share::area)
    }

    /// Whether this window has a share.
    #[must_use]
    pub fn holds(&self, window: WindowId) -> bool {
        self.tree.as_ref().is_some_and(|tree| tree.holds(window))
    }

    /// Divide `target`'s share in half and give `incoming` the half on `side`.
    ///
    /// This is how an existing half is split again: a quarter is a half halved.
    /// If `incoming` already has a share somewhere else, it leaves that one
    /// first — its old share goes to its neighbour — so one window is never in
    /// two places.
    ///
    /// # Errors
    /// - [`Refused::NothingToShareWith`] when `incoming` is `target`;
    /// - [`Refused::NotDivided`] when `target` has no share;
    /// - [`Refused::TooNarrow`] or [`Refused::TooShort`], naming the window,
    ///   when either half would be smaller than its window's minimum.
    pub fn divide(
        &mut self,
        target: WindowId,
        incoming: Window,
        side: Side,
    ) -> Result<(), Refused> {
        if target == incoming.id() {
            return Err(Refused::NothingToShareWith);
        }
        let tree = self
            .without(incoming.id())
            .ok_or(Refused::NotDivided(target))?;
        self.replace(Self::halved(tree, self.display, target, incoming, side)?);
        Ok(())
    }

    /// Move the boundary on `edge` of `window`'s share to `to`, a coordinate on
    /// the display along that edge's axis.
    ///
    /// **Both shares move.** The boundary belongs to the cut on either side of
    /// it, so the share on the far side grows by what this one loses; and every
    /// share inside either of them that is cut the same way keeps its proportion.
    /// Where a window is in more than one cut along that edge, the boundary
    /// moved is the nearest one — the edge of the window's own share.
    ///
    /// # Errors
    /// - [`Refused::NotDivided`] when `window` has no share;
    /// - [`Refused::NoNeighbour`] when that edge is the display's;
    /// - [`Refused::TooNarrow`] or [`Refused::TooShort`] when either side would
    ///   squeeze a window below its minimum — the boundary does not move at all,
    ///   rather than stopping part of the way.
    pub fn move_boundary(&mut self, window: WindowId, edge: Side, to: u32) -> Result<(), Refused> {
        let tree = self.tree.as_ref().ok_or(Refused::NotDivided(window))?;
        let path = tree.path_to(window).ok_or(Refused::NotDivided(window))?;
        let (depth, area) = Self::boundary_on(tree, self.display, &path, edge)
            .ok_or(Refused::NoNeighbour { window, edge })?;

        let above = path
            .get(..depth)
            .ok_or(Refused::NoNeighbour { window, edge })?;
        let mut tree = tree.clone();
        let Some(Node::Cut(cut)) = tree.at_mut(above) else {
            return Err(Refused::NoNeighbour { window, edge });
        };
        let axis = cut.axis;
        let length = area.size().along(axis);
        let first = to.saturating_sub(area.start_along(axis)).min(length);
        cut.first.fits(axis, first)?;
        cut.second.fits(axis, length - first)?;
        let old = cut.first_length;
        cut.first.fit(axis, old, first);
        cut.second.fit(axis, length - old, length - first);
        cut.first_length = first;
        if old != first {
            self.replace(tree);
        }
        Ok(())
    }

    /// Take a closed window out, and give its share to its neighbour.
    ///
    /// Closing the last window leaves the display undivided.
    ///
    /// # Errors
    /// [`Refused::NotDivided`] when `window` has no share.
    pub fn close(&mut self, window: WindowId) -> Result<(), Refused> {
        if !self.holds(window) {
            return Err(Refused::NotDivided(window));
        }
        self.tree = self.without(window);
        self.changes += 1;
        Ok(())
    }

    /// The tree with this window's share taken out, if it had one, as a copy.
    pub(crate) fn without(&self, window: WindowId) -> Option<Node> {
        let tree = self.tree.clone()?;
        if tree.holds(window) {
            tree.without(window, self.display)
        } else {
            Some(tree)
        }
    }

    /// The tree, to decide a change against.
    pub(crate) const fn tree(&self) -> Option<&Node> {
        self.tree.as_ref()
    }

    /// Which division this is, and how many changes it has seen.
    pub(crate) const fn version(&self) -> (u64, u64) {
        (self.identity, self.changes)
    }

    /// Replace the tree with a decided change.
    pub(crate) fn replace(&mut self, tree: Node) {
        self.tree = Some(tree);
        self.changes += 1;
    }

    /// `tree` with `target`'s share halved for `incoming` on `side`.
    pub(crate) fn halved(
        mut tree: Node,
        display: Area,
        target: WindowId,
        incoming: Window,
        side: Side,
    ) -> Result<Node, Refused> {
        tree.halve(display, target, incoming, side)?;
        Ok(tree)
    }

    /// The nearest cut above a window's share whose boundary is on `edge`:
    /// how deep it is, and the area it cuts.
    fn boundary_on(
        tree: &Node,
        display: Area,
        path: &[Branch],
        edge: Side,
    ) -> Option<(usize, Area)> {
        let mut node = tree;
        let mut area = display;
        let mut found = None;
        for (depth, branch) in path.iter().enumerate() {
            let Node::Cut(cut) = node else {
                return found;
            };
            let in_first = *branch == Branch::First;
            if cut.axis == edge.axis() && in_first != edge.is_first() {
                found = Some((depth, area));
            }
            let (first, second) = area.cut(cut.axis, cut.first_length);
            (node, area) = if in_first {
                (&cut.first, first)
            } else {
                (&cut.second, second)
            };
        }
        found
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{any, at_least, display, divided_in_halves};

    /// The shares of a division, as windows and areas in logical units.
    fn laid_out(division: &Division) -> Vec<(u64, (u32, u32, u32, u32))> {
        division
            .shares()
            .into_iter()
            .map(|share| {
                let area = share.area();
                (
                    share.window().to_compositor(),
                    (area.x(), area.y(), area.width(), area.height()),
                )
            })
            .collect()
    }

    /// **A division is a tree of shares: halves, quarters, and a half split
    /// again**, each naming the window in it.
    #[test]
    fn a_division_holds_halves_quarters_and_a_split_half() {
        let mut division = divided_in_halves(1, 2);
        assert_eq!(
            laid_out(&division),
            vec![(1, (0, 0, 960, 1080)), (2, (960, 0, 960, 1080))]
        );

        // The right half split into two quarters.
        division
            .divide(WindowId::from_compositor(2), any(3), Side::Bottom)
            .unwrap();
        // And the bottom-right quarter split again, side by side.
        division
            .divide(WindowId::from_compositor(3), any(4), Side::Right)
            .unwrap();
        assert_eq!(
            laid_out(&division),
            vec![
                (1, (0, 0, 960, 1080)),
                (2, (960, 0, 960, 540)),
                (3, (960, 540, 480, 540)),
                (4, (1440, 540, 480, 540)),
            ]
        );
        assert_eq!(
            division.share_of(WindowId::from_compositor(4)),
            Some(crate::testing::area(1440, 540, 480, 540))
        );
    }

    /// **Resizing the boundary between two shares resizes both.** The left
    /// half grows by exactly what the right loses, and the quarters inside the
    /// right half both narrow with it.
    #[test]
    fn moving_a_boundary_resizes_the_shares_on_both_sides() {
        let mut division = divided_in_halves(1, 2);
        division
            .divide(WindowId::from_compositor(2), any(3), Side::Bottom)
            .unwrap();

        division
            .move_boundary(WindowId::from_compositor(1), Side::Right, 1200)
            .unwrap();
        assert_eq!(
            laid_out(&division),
            vec![
                (1, (0, 0, 1200, 1080)),
                (2, (1200, 0, 720, 540)),
                (3, (1200, 540, 720, 540)),
            ]
        );

        // Dragged from the other side, by the quarter's left edge: the same
        // boundary, because it is the nearest one on that edge.
        division
            .move_boundary(WindowId::from_compositor(3), Side::Left, 700)
            .unwrap();
        assert_eq!(
            laid_out(&division),
            vec![
                (1, (0, 0, 700, 1080)),
                (2, (700, 0, 1220, 540)),
                (3, (700, 540, 1220, 540)),
            ]
        );

        // The boundary between the two quarters moves only them.
        division
            .move_boundary(WindowId::from_compositor(2), Side::Bottom, 300)
            .unwrap();
        assert_eq!(
            laid_out(&division),
            vec![
                (1, (0, 0, 700, 1080)),
                (2, (700, 0, 1220, 300)),
                (3, (700, 300, 1220, 780)),
            ]
        );
    }

    /// An edge that is the display's has nothing on the far side, and a window
    /// with no share has no edge: both are refused and nothing moves.
    #[test]
    fn a_boundary_with_nothing_beyond_it_does_not_move() {
        let mut division = divided_in_halves(1, 2);
        let before = laid_out(&division);
        let one = WindowId::from_compositor(1);
        assert_eq!(
            division.move_boundary(one, Side::Left, 100),
            Err(Refused::NoNeighbour {
                window: one,
                edge: Side::Left
            })
        );
        assert_eq!(
            division.move_boundary(one, Side::Bottom, 100),
            Err(Refused::NoNeighbour {
                window: one,
                edge: Side::Bottom
            })
        );
        assert_eq!(
            division.move_boundary(WindowId::from_compositor(9), Side::Right, 100),
            Err(Refused::NotDivided(WindowId::from_compositor(9)))
        );
        assert_eq!(laid_out(&division), before);
    }

    /// **A window with a minimum larger than its share is not squeezed below
    /// it** — not by a boundary dragged past it, and not by a split. The
    /// division refuses, names the window and the way it would have been
    /// squeezed, and is left exactly as it was.
    #[test]
    fn a_window_is_not_squeezed_below_its_minimum() {
        let mut division = Division::of(display());
        division.replace(Node::Share(at_least(1, 800, 600)));
        division
            .divide(WindowId::from_compositor(1), any(2), Side::Right)
            .unwrap();
        let before = laid_out(&division);

        // Dragging the boundary left of 800 would make window 1 too narrow.
        assert_eq!(
            division.move_boundary(WindowId::from_compositor(2), Side::Left, 799),
            Err(Refused::TooNarrow(WindowId::from_compositor(1)))
        );
        assert_eq!(laid_out(&division), before);
        // To 800 exactly is allowed.
        division
            .move_boundary(WindowId::from_compositor(2), Side::Left, 800)
            .unwrap();
        division
            .move_boundary(WindowId::from_compositor(2), Side::Left, 960)
            .unwrap();

        // Splitting window 1's half again, one above the other, would give it
        // 540 of the 600 it needs.
        let before = laid_out(&division);
        assert_eq!(
            division.divide(WindowId::from_compositor(1), any(3), Side::Top),
            Err(Refused::TooShort(WindowId::from_compositor(1)))
        );
        // A window that needs more than a half of the half is refused as the
        // incoming one.
        assert_eq!(
            division.divide(
                WindowId::from_compositor(2),
                at_least(4, 500, 1),
                Side::Left
            ),
            Err(Refused::TooNarrow(WindowId::from_compositor(4)))
        );
        assert_eq!(
            division.divide(
                WindowId::from_compositor(2),
                at_least(4, 1, 1100),
                Side::Left
            ),
            Err(Refused::TooShort(WindowId::from_compositor(4)))
        );
        assert_eq!(laid_out(&division), before);
    }

    /// A boundary is not moved part of the way: a quarter inside the far share
    /// that could not follow stops the whole move.
    #[test]
    fn a_window_deep_inside_the_far_side_stops_the_whole_move() {
        let mut division = divided_in_halves(1, 2);
        division
            .divide(
                WindowId::from_compositor(2),
                at_least(3, 700, 1),
                Side::Bottom,
            )
            .unwrap();
        let before = laid_out(&division);
        assert_eq!(
            division.move_boundary(WindowId::from_compositor(1), Side::Right, 1300),
            Err(Refused::TooNarrow(WindowId::from_compositor(3)))
        );
        assert_eq!(laid_out(&division), before);
    }

    /// **A window closed inside a division gives its share to its neighbour**
    /// rather than leaving a hole, and closing the last one leaves the display
    /// undivided.
    #[test]
    fn a_closed_window_gives_its_share_to_its_neighbour() {
        let mut division = divided_in_halves(1, 2);
        division
            .divide(WindowId::from_compositor(2), any(3), Side::Bottom)
            .unwrap();

        division.close(WindowId::from_compositor(3)).unwrap();
        assert_eq!(
            laid_out(&division),
            vec![(1, (0, 0, 960, 1080)), (2, (960, 0, 960, 1080))]
        );

        division.close(WindowId::from_compositor(1)).unwrap();
        assert_eq!(laid_out(&division), vec![(2, (0, 0, 1920, 1080))]);

        assert_eq!(
            division.close(WindowId::from_compositor(1)),
            Err(Refused::NotDivided(WindowId::from_compositor(1)))
        );
        division.close(WindowId::from_compositor(2)).unwrap();
        assert!(division.is_empty());
    }

    /// A window moved into another share leaves its old one to its neighbour,
    /// so it is never in two places; and a window cannot share with itself.
    #[test]
    fn a_window_divided_again_leaves_where_it_was() {
        let mut division = divided_in_halves(1, 2);
        division
            .divide(WindowId::from_compositor(2), any(3), Side::Bottom)
            .unwrap();
        division
            .divide(WindowId::from_compositor(1), any(3), Side::Bottom)
            .unwrap();
        assert_eq!(
            laid_out(&division),
            vec![
                (1, (0, 0, 960, 540)),
                (3, (0, 540, 960, 540)),
                (2, (960, 0, 960, 1080)),
            ]
        );
        assert_eq!(
            division.divide(WindowId::from_compositor(1), any(1), Side::Left),
            Err(Refused::NothingToShareWith)
        );
        assert_eq!(
            Division::of(display()).divide(WindowId::from_compositor(1), any(2), Side::Left),
            Err(Refused::NotDivided(WindowId::from_compositor(1)))
        );
    }
}
