//! A window dragged to an edge or a corner: what dropping it there would do,
//! shown before anything moves.
//!
//! **A proposal changes nothing.** [`Division::propose_drop`] works out the whole
//! division as it would be after the drop and hands it back as a [`Proposal`]
//! the shell draws as an outline, labelled with its [`Place`]. The division
//! itself is untouched until [`Division::commit`], so a person who carries the
//! window away from the edge again, or presses Escape, has abandoned the drop by
//! doing nothing — the proposal is simply dropped.
//!
//! A proposal is committed at most once, because committing takes it by value,
//! and only against the division it was made from and as that division was when
//! it was made: a window that closed while a person was dragging makes the
//! outline a lie, and [`Refused::Changed`] is what they are told instead.
//!
//! # What an edge and a corner propose
//!
//! **An edge proposes a half of the display.** The dragged window takes that
//! half, and everything already divided moves into the other half, keeping its
//! proportions.
//!
//! **A corner proposes a quarter by halving the share already in that corner**,
//! on whichever axis that share is longer. On a display already in halves —
//! the ordinary case, after one window was dropped on an edge — the half in
//! that corner is taller than it is wide, so it is cut one above the other and
//! the dragged window gets that corner's quarter. On a display holding one
//! undivided window a corner can only honestly offer a half, since a quarter
//! with one other window would leave three quarters for it that are not a
//! rectangle; the proposal then says *Left half*, because its [`Place`] is read
//! off the area it would really give.
//!
//! # The first division needs a second window
//!
//! A display nobody has divided has no shares, and a half with nothing in the
//! other half would be the gap this crate rules out. So the first drop names
//! the window in front of the dragged one — the one it will share the display
//! with — and is refused with [`Refused::NothingToShareWith`] when there is
//! none.

use crate::area::{Area, Point};
use crate::division::Division;
use crate::node::Node;
use crate::place::Place;
use crate::refusing::Refused;
use crate::side::Side;
use crate::window::{Window, WindowId};

/// How close to an edge, in logical units, a pointer proposes a half.
pub const NEAR_AN_EDGE: u32 = 16;

/// How close to both edges of a corner, in logical units, a pointer at an edge
/// proposes a quarter instead.
pub const NEAR_A_CORNER: u32 = 96;

/// What a pointer somewhere on the display offers while a window is held.
#[derive(Debug, PartialEq, Eq)]
pub enum Offer {
    /// Not near an edge: dropping here divides nothing.
    Nothing,
    /// Dropping here would do this.
    Proposed(Proposal),
    /// Dropping here cannot divide the screen, and this is why.
    Refused(Refused),
}

/// What a drop would do, before it is done.
#[derive(Debug, PartialEq, Eq)]
pub struct Proposal {
    /// The window being dropped.
    window: WindowId,
    /// Where it would go.
    area: Area,
    /// What that place is called.
    place: Place,
    /// The whole division as it would be.
    tree: Node,
    /// The division it was made against, and as it was then.
    made_against: (u64, u64),
}

impl Proposal {
    /// The window being dropped.
    #[must_use]
    pub const fn window(&self) -> WindowId {
        self.window
    }

    /// Where the window would go — the outline the shell draws.
    #[must_use]
    pub const fn area(&self) -> Area {
        self.area
    }

    /// What that place is called — the label beside the outline.
    #[must_use]
    pub const fn place(&self) -> Place {
        self.place
    }
}

/// Where on a display a pointer is, as far as dropping is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Zone {
    /// At one edge.
    Edge(Side),
    /// At a corner: the left or right side, and the top or bottom.
    Corner(Side, Side),
}

impl Division {
    /// What dropping `dragged` at `pointer` would do, without doing it.
    ///
    /// `in_front` is the window the dragged one would share the display with
    /// when nothing is divided yet; it is ignored otherwise.
    #[must_use]
    pub fn propose_drop(&self, pointer: Point, dragged: Window, in_front: Option<Window>) -> Offer {
        let Some(zone) = zone_of(self.display(), pointer) else {
            return Offer::Nothing;
        };
        match self.proposed(zone, dragged, in_front) {
            Ok(proposal) => Offer::Proposed(proposal),
            Err(refused) => Offer::Refused(refused),
        }
    }

    /// Do what a proposal said.
    ///
    /// # Errors
    /// [`Refused::Changed`] when the division is not the one the proposal was
    /// made against, or has changed since — nothing moves.
    pub fn commit(&mut self, proposal: Proposal) -> Result<(), Refused> {
        if proposal.made_against != self.version() {
            return Err(Refused::Changed);
        }
        self.replace(proposal.tree);
        Ok(())
    }

    /// The proposal for a drop in `zone`.
    fn proposed(
        &self,
        zone: Zone,
        dragged: Window,
        in_front: Option<Window>,
    ) -> Result<Proposal, Refused> {
        let display = self.display();
        let rest = match self.without(dragged.id()) {
            Some(rest) => rest,
            None => {
                let other = in_front
                    .filter(|other| other.id() != dragged.id())
                    .ok_or(Refused::NothingToShareWith)?;
                let whole = Node::Share(other);
                for axis in [crate::Axis::SideBySide, crate::Axis::OneAboveTheOther] {
                    whole.fits(axis, display.size().along(axis))?;
                }
                whole
            }
        };
        let tree = match zone {
            Zone::Edge(side) => half_of_the_display(rest, display, dragged, side)?,
            Zone::Corner(across, down) => {
                let corner = Point::at(
                    if across.is_first() {
                        display.x()
                    } else {
                        display.right() - 1
                    },
                    if down.is_first() {
                        display.y()
                    } else {
                        display.bottom() - 1
                    },
                );
                let share = rest
                    .share_at(display, corner)
                    .ok_or(Refused::NotDivided(dragged.id()))?;
                let side = if share.area().width() >= share.area().height() {
                    across
                } else {
                    down
                };
                Self::halved(rest, display, share.window(), dragged, side)?
            }
        };
        let mut shares = Vec::new();
        tree.lay_out(display, &mut shares);
        let area = shares
            .into_iter()
            .find(|share| share.window() == dragged.id())
            .map(crate::Share::area)
            .ok_or(Refused::NotDivided(dragged.id()))?;
        Ok(Proposal {
            window: dragged.id(),
            area,
            place: Place::of(area, display),
            tree,
            made_against: self.version(),
        })
    }
}

/// The dragged window on the half of the display at `side`, and everything else
/// fitted into the other half.
fn half_of_the_display(
    mut rest: Node,
    display: Area,
    dragged: Window,
    side: Side,
) -> Result<Node, Refused> {
    let axis = side.axis();
    let length = display.size().along(axis);
    let first_length = length / 2;
    let dragged_length = if side.is_first() {
        first_length
    } else {
        length - first_length
    };
    let rest_length = length - dragged_length;
    let alone = Node::Share(dragged);
    alone.fits(axis, dragged_length)?;
    alone.fits(axis.across(), display.size().along(axis.across()))?;
    rest.fits(axis, rest_length)?;
    rest.fit(axis, length, rest_length);
    Ok(if side.is_first() {
        Node::cut(axis, first_length, alone, rest)
    } else {
        Node::cut(axis, first_length, rest, alone)
    })
}

/// Which edge or corner a pointer is at, if any.
fn zone_of(display: Area, pointer: Point) -> Option<Zone> {
    if !display.holds(pointer) {
        return None;
    }
    let from_left = pointer.x - display.x();
    let from_right = display.right() - 1 - pointer.x;
    let from_top = pointer.y - display.y();
    let from_bottom = display.bottom() - 1 - pointer.y;

    let (across, to_across) = if from_left <= from_right {
        (Side::Left, from_left)
    } else {
        (Side::Right, from_right)
    };
    let (down, to_down) = if from_top <= from_bottom {
        (Side::Top, from_top)
    } else {
        (Side::Bottom, from_bottom)
    };

    let at_an_edge = to_across < NEAR_AN_EDGE || to_down < NEAR_AN_EDGE;
    if !at_an_edge {
        return None;
    }
    if to_across < NEAR_A_CORNER && to_down < NEAR_A_CORNER {
        return Some(Zone::Corner(across, down));
    }
    Some(if to_across <= to_down {
        Zone::Edge(across)
    } else {
        Zone::Edge(down)
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{any, area, at_least, display, divided_in_halves};

    /// The proposal an offer holds, or the offer itself as the failure.
    fn proposal(offer: Offer) -> Proposal {
        match offer {
            Offer::Proposed(proposal) => proposal,
            other => {
                assert!(matches!(other, Offer::Proposed(_)), "{other:?}");
                unreachable!()
            }
        }
    }

    /// **A window dragged to an edge proposes a half**, and nothing moves until
    /// the proposal is committed.
    #[test]
    fn an_edge_proposes_a_half_and_nothing_moves_until_it_is_committed() {
        let mut division = Division::of(display());
        let offer = division.propose_drop(Point::at(3, 500), any(1), Some(any(2)));
        let proposal = proposal(offer);
        assert_eq!(proposal.place(), Place::LeftHalf);
        assert_eq!(proposal.area(), area(0, 0, 960, 1080));
        assert_eq!(proposal.window(), WindowId::from_compositor(1));
        assert!(division.is_empty(), "a proposal moved something");

        division.commit(proposal).unwrap();
        assert_eq!(
            division.share_of(WindowId::from_compositor(2)),
            Some(area(960, 0, 960, 1080))
        );

        let bottom = proposal_of(&division, Point::at(900, 1079), 3);
        assert_eq!(bottom.place(), Place::BottomHalf);
        assert_eq!(bottom.area(), area(0, 540, 1920, 540));
    }

    /// The proposal for a drop at this point, with window `id`.
    fn proposal_of(division: &Division, pointer: Point, id: u64) -> Proposal {
        proposal(division.propose_drop(pointer, any(id), None))
    }

    /// **A corner proposes a quarter** on a display already in halves.
    #[test]
    fn a_corner_proposes_a_quarter() {
        let division = divided_in_halves(1, 2);
        for (pointer, place, expected) in [
            (Point::at(0, 0), Place::TopLeftQuarter, area(0, 0, 960, 540)),
            (
                Point::at(1919, 5),
                Place::TopRightQuarter,
                area(960, 0, 960, 540),
            ),
            (
                Point::at(10, 1079),
                Place::BottomLeftQuarter,
                area(0, 540, 960, 540),
            ),
            (
                Point::at(1910, 1070),
                Place::BottomRightQuarter,
                area(960, 540, 960, 540),
            ),
        ] {
            let proposed = proposal_of(&division, pointer, 3);
            assert_eq!(proposed.place(), place, "{pointer:?}");
            assert_eq!(proposed.area(), expected, "{pointer:?}");
        }
    }

    /// **A drop can be abandoned.** A proposal that is never committed leaves
    /// the division exactly as it was, and a pointer carried away from the edge
    /// offers nothing.
    #[test]
    fn a_proposal_never_committed_changes_nothing() {
        let division = divided_in_halves(1, 2);
        let before = division.shares();
        let abandoned = proposal_of(&division, Point::at(0, 0), 3);
        drop(abandoned);
        assert_eq!(division.shares(), before);
        assert_eq!(
            division.propose_drop(Point::at(900, 500), any(3), None),
            Offer::Nothing
        );
        assert_eq!(
            division.propose_drop(Point::at(5000, 500), any(3), None),
            Offer::Nothing
        );
    }

    /// A proposal made before the division changed is refused rather than
    /// committed over the change, and so is one made on another display.
    #[test]
    fn a_proposal_for_a_screen_that_has_changed_is_refused() {
        let mut division = divided_in_halves(1, 2);
        let stale = proposal_of(&division, Point::at(0, 0), 3);
        division.close(WindowId::from_compositor(2)).unwrap();
        let before = division.shares();
        assert_eq!(division.commit(stale), Err(Refused::Changed));
        assert_eq!(division.shares(), before);

        let elsewhere = divided_in_halves(1, 2);
        let theirs = proposal_of(&elsewhere, Point::at(0, 0), 3);
        let mut ours = divided_in_halves(1, 2);
        assert_eq!(ours.commit(theirs), Err(Refused::Changed));
    }

    /// The first drop with nothing else open is refused, and a window too wide
    /// for the half it would be dropped into is refused and named.
    #[test]
    fn a_drop_that_cannot_divide_says_why() {
        let empty = Division::of(display());
        assert_eq!(
            empty.propose_drop(Point::at(0, 500), any(1), None),
            Offer::Refused(Refused::NothingToShareWith)
        );
        assert_eq!(
            empty.propose_drop(Point::at(0, 500), any(1), Some(any(1))),
            Offer::Refused(Refused::NothingToShareWith)
        );
        assert_eq!(
            empty.propose_drop(Point::at(0, 500), at_least(1, 1000, 1), Some(any(2))),
            Offer::Refused(Refused::TooNarrow(WindowId::from_compositor(1)))
        );
        let halves = divided_in_halves(1, 2);
        assert_eq!(
            halves.propose_drop(Point::at(0, 0), at_least(3, 1, 600), None),
            Offer::Refused(Refused::TooShort(WindowId::from_compositor(3)))
        );
    }

    /// A window already divided, dragged to another edge, leaves its share to
    /// its neighbour and takes the new one.
    #[test]
    fn a_divided_window_dragged_elsewhere_moves_rather_than_doubling() {
        let mut division = divided_in_halves(1, 2);
        let moved = proposal_of(&division, Point::at(1919, 500), 1);
        assert_eq!(moved.place(), Place::RightHalf);
        division.commit(moved).unwrap();
        assert_eq!(division.shares().len(), 2);
        assert_eq!(
            division.share_of(WindowId::from_compositor(2)),
            Some(area(0, 0, 960, 1080))
        );
    }

    /// On a single undivided window a corner offers the half it can honestly
    /// give, and says so.
    #[test]
    fn a_corner_of_one_window_offers_the_half_it_can_give() {
        let empty = Division::of(display());
        let proposed = proposal(empty.propose_drop(Point::at(0, 0), any(1), Some(any(2))));
        assert_eq!(proposed.place(), Place::LeftHalf);
    }
}
