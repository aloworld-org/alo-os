//! The tree a division is: a share holding one window, or a cut into two.
//!
//! **Why a tree, and why the cut is stored as a length.** A division held as a
//! list of rectangles has to be checked after every change for overlaps and
//! gaps, and fixed when it has one. A tree cannot have either: every cut hands
//! its area to exactly two pieces that meet at one whole unit, so the shares are
//! a tiling of the display by construction, and the test in
//! `tests/a_division_never_overlaps_or_leaves_a_gap.rs` is there to hold that
//! claim rather than to catch the tree failing it.
//!
//! The same structure is what makes a pair *hold*. Two halves are two children
//! of one cut, so the boundary between them is one number; moving it is
//! changing that number, and both halves follow because both are computed from
//! it. There is no second rectangle to forget.
//!
//! # Fitting a piece to a new length
//!
//! When a cut's boundary moves, every piece on either side changes length along
//! that axis, and pieces inside them that are cut the same way have to decide
//! where their own boundaries go. [`Node::fit`] keeps each one in proportion,
//! then moves it just enough to keep every window at or above its minimum. It
//! is only ever called after [`Node::minimum_along`] has said the new length is
//! enough, so a place that satisfies every minimum always exists.

use crate::area::Area;
use crate::refusing::Refused;
use crate::share::Share;
use crate::side::{Axis, Side};
use crate::window::{Window, WindowId};

/// Which piece of a cut a path goes into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Branch {
    /// The left or top piece.
    First,
    /// The right or bottom piece.
    Second,
}

/// One node of a division.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Node {
    /// A share, and the window in it.
    Share(Window),
    /// A cut into two.
    Cut(Box<Cut>),
}

/// An area cut in two.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Cut {
    /// Which way it is cut.
    pub(crate) axis: Axis,
    /// How long the first piece is along that axis, in logical units; the second
    /// piece is the rest.
    pub(crate) first_length: u32,
    /// The left or top piece.
    pub(crate) first: Node,
    /// The right or bottom piece.
    pub(crate) second: Node,
}

impl Node {
    /// A cut of this length along `axis`, with `first` the length of its first
    /// piece.
    pub(crate) fn cut(axis: Axis, first_length: u32, first: Self, second: Self) -> Self {
        Self::Cut(Box::new(Cut {
            axis,
            first_length,
            first,
            second,
        }))
    }

    /// Whether this window has a share anywhere in here.
    pub(crate) fn holds(&self, window: WindowId) -> bool {
        match self {
            Self::Share(held) => held.id() == window,
            Self::Cut(cut) => cut.first.holds(window) || cut.second.holds(window),
        }
    }

    /// The least length along `axis` this node can be given without squeezing a
    /// window below its minimum.
    pub(crate) fn minimum_along(&self, axis: Axis) -> u32 {
        match self {
            Self::Share(window) => window.minimum_along(axis),
            Self::Cut(cut) if cut.axis == axis => cut
                .first
                .minimum_along(axis)
                .saturating_add(cut.second.minimum_along(axis)),
            Self::Cut(cut) => cut
                .first
                .minimum_along(axis)
                .max(cut.second.minimum_along(axis)),
        }
    }

    /// The window with the largest minimum along `axis`, and that minimum.
    fn largest_minimum_along(&self, axis: Axis) -> (u32, WindowId) {
        match self {
            Self::Share(window) => (window.minimum_along(axis), window.id()),
            Self::Cut(cut) => cut
                .first
                .largest_minimum_along(axis)
                .max(cut.second.largest_minimum_along(axis)),
        }
    }

    /// Refused when this node cannot be given `length` along `axis`, naming the
    /// window that stops it.
    pub(crate) fn fits(&self, axis: Axis, length: u32) -> Result<(), Refused> {
        if self.minimum_along(axis) <= length {
            return Ok(());
        }
        let stopped_by = match self {
            Self::Share(window) => window.id(),
            Self::Cut(cut) if cut.axis == axis => self.largest_minimum_along(axis).1,
            Self::Cut(cut) => {
                return cut
                    .first
                    .fits(axis, length)
                    .and_then(|()| cut.second.fits(axis, length));
            }
        };
        Err(Refused::squeezed(stopped_by, axis))
    }

    /// Change this node's length along `axis` from `old` to `new`.
    ///
    /// Every boundary inside stays in proportion, moved only as far as it takes
    /// to keep each window at or above its minimum. The caller has already held
    /// `new` to [`Node::fits`].
    pub(crate) fn fit(&mut self, axis: Axis, old: u32, new: u32) {
        let Self::Cut(cut) = self else {
            return;
        };
        if cut.axis != axis {
            cut.first.fit(axis, old, new);
            cut.second.fit(axis, old, new);
            return;
        }
        let old_first = cut.first_length;
        let old_second = old.saturating_sub(old_first);
        let in_proportion = if old == 0 {
            new / 2
        } else {
            let scaled =
                (u64::from(old_first) * u64::from(new) + u64::from(old) / 2) / u64::from(old);
            u32::try_from(scaled).unwrap_or(new)
        };
        let lowest = cut.first.minimum_along(axis);
        let highest = new.saturating_sub(cut.second.minimum_along(axis));
        let first = in_proportion.max(lowest).min(highest).min(new);
        cut.first.fit(axis, old_first, first);
        cut.second.fit(axis, old_second, new - first);
        cut.first_length = first;
    }

    /// Every share in here, laid out over `area`, in reading order.
    pub(crate) fn lay_out(&self, area: Area, shares: &mut Vec<Share>) {
        match self {
            Self::Share(window) => shares.push(Share::of(window.id(), area)),
            Self::Cut(cut) => {
                let (first, second) = area.cut(cut.axis, cut.first_length);
                cut.first.lay_out(first, shares);
                cut.second.lay_out(second, shares);
            }
        }
    }

    /// The share holding this point, with this node laid out over `area`.
    pub(crate) fn share_at(&self, area: Area, point: crate::Point) -> Option<Share> {
        match self {
            Self::Share(window) => area.holds(point).then(|| Share::of(window.id(), area)),
            Self::Cut(cut) => {
                let (first, second) = area.cut(cut.axis, cut.first_length);
                cut.first
                    .share_at(first, point)
                    .or_else(|| cut.second.share_at(second, point))
            }
        }
    }

    /// The way from here down to this window's share, one branch per cut.
    pub(crate) fn path_to(&self, window: WindowId) -> Option<Vec<Branch>> {
        match self {
            Self::Share(held) => (held.id() == window).then(Vec::new),
            Self::Cut(cut) => {
                let (branch, rest) = if cut.first.holds(window) {
                    (Branch::First, cut.first.path_to(window)?)
                } else {
                    (Branch::Second, cut.second.path_to(window)?)
                };
                let mut path = Vec::with_capacity(rest.len() + 1);
                path.push(branch);
                path.extend(rest);
                Some(path)
            }
        }
    }

    /// The node at the end of a path, to change.
    pub(crate) fn at_mut(&mut self, path: &[Branch]) -> Option<&mut Self> {
        let mut node = self;
        for branch in path {
            node = match node {
                Self::Share(_) => return None,
                Self::Cut(cut) => match branch {
                    Branch::First => &mut cut.first,
                    Branch::Second => &mut cut.second,
                },
            };
        }
        Some(node)
    }

    /// This node with a window's share taken out, laid over an area of `area`.
    ///
    /// **The share goes to its neighbour.** The piece on the other side of the
    /// cut the share was in takes the whole of that cut's area, so nothing is
    /// left empty. `None` when this node was that share and nothing is left.
    pub(crate) fn without(self, window: WindowId, area: Area) -> Option<Self> {
        match self {
            Self::Share(held) if held.id() == window => None,
            Self::Share(_) => Some(self),
            Self::Cut(cut) => {
                let Cut {
                    axis,
                    first_length,
                    first,
                    second,
                } = *cut;
                let length = area.size().along(axis);
                let (first_area, second_area) = area.cut(axis, first_length);
                if first.holds(window) {
                    match first.without(window, first_area) {
                        Some(first) => Some(Self::cut(axis, first_length, first, second)),
                        None => {
                            let mut neighbour = second;
                            neighbour.fit(axis, length - first_length, length);
                            Some(neighbour)
                        }
                    }
                } else {
                    match second.without(window, second_area) {
                        Some(second) => Some(Self::cut(axis, first_length, first, second)),
                        None => {
                            let mut neighbour = first;
                            neighbour.fit(axis, first_length, length);
                            Some(neighbour)
                        }
                    }
                }
            }
        }
    }

    /// Halve `target`'s share, laid over `area`, and give `incoming` the half on
    /// `side`.
    ///
    /// Exactly half, the first piece rounded down. **Refused rather than
    /// squeezed**: when either window's minimum does not fit its half, nothing
    /// changes and the refusal names the window.
    pub(crate) fn halve(
        &mut self,
        area: Area,
        target: WindowId,
        incoming: Window,
        side: Side,
    ) -> Result<(), Refused> {
        let path = self.path_to(target).ok_or(Refused::NotDivided(target))?;
        let Some(share) = self.share_at_path(area, &path) else {
            return Err(Refused::NotDivided(target));
        };
        let axis = side.axis();
        let length = share.area().size().along(axis);
        let first_length = length / 2;
        let (incoming_length, target_length) = if side.is_first() {
            (first_length, length - first_length)
        } else {
            (length - first_length, first_length)
        };
        let across = share.area().size().along(axis.across());
        if incoming.minimum_along(axis) > incoming_length {
            return Err(Refused::squeezed(incoming.id(), axis));
        }
        if incoming.minimum_along(axis.across()) > across {
            return Err(Refused::squeezed(incoming.id(), axis.across()));
        }
        let Some(node) = self.at_mut(&path) else {
            return Err(Refused::NotDivided(target));
        };
        let Self::Share(existing) = *node else {
            return Err(Refused::NotDivided(target));
        };
        if existing.minimum_along(axis) > target_length {
            return Err(Refused::squeezed(existing.id(), axis));
        }
        let (first, second) = if side.is_first() {
            (Self::Share(incoming), Self::Share(existing))
        } else {
            (Self::Share(existing), Self::Share(incoming))
        };
        *node = Self::cut(axis, first_length, first, second);
        Ok(())
    }

    /// The share at the end of a path, laid over `area`.
    fn share_at_path(&self, area: Area, path: &[Branch]) -> Option<Share> {
        let mut node = self;
        let mut area = area;
        for branch in path {
            let Self::Cut(cut) = node else {
                return None;
            };
            let (first, second) = area.cut(cut.axis, cut.first_length);
            (node, area) = match branch {
                Branch::First => (&cut.first, first),
                Branch::Second => (&cut.second, second),
            };
        }
        match node {
            Self::Share(window) => Some(Share::of(window.id(), area)),
            Self::Cut(_) => None,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::area::{Point, Size};

    /// A window no minimum of which is stated.
    fn any(id: u64) -> Window {
        Window::any_size(WindowId::from_compositor(id))
    }

    /// Where a cut's boundary is, if the node is a cut.
    fn first_length(node: &Node) -> Option<u32> {
        match node {
            Node::Cut(cut) => Some(cut.first_length),
            Node::Share(_) => None,
        }
    }

    /// A 1920 by 1080 display.
    fn display() -> Area {
        Area::of(Point::at(0, 0), Size::of(1920, 1080)).unwrap()
    }

    /// A same-axis cut needs both its pieces' minimums; a cross cut needs the
    /// larger of them.
    #[test]
    fn a_minimum_adds_along_a_cut_and_not_across_it() {
        let wide = Window::at_least(WindowId::from_compositor(1), Size::of(800, 300));
        let narrow = Window::at_least(WindowId::from_compositor(2), Size::of(400, 500));
        let side_by_side = Node::cut(
            Axis::SideBySide,
            960,
            Node::Share(wide),
            Node::Share(narrow),
        );
        assert_eq!(side_by_side.minimum_along(Axis::SideBySide), 1200);
        assert_eq!(side_by_side.minimum_along(Axis::OneAboveTheOther), 500);
        assert_eq!(
            side_by_side.fits(Axis::SideBySide, 1000),
            Err(Refused::TooNarrow(WindowId::from_compositor(1)))
        );
        assert_eq!(
            side_by_side.fits(Axis::OneAboveTheOther, 400),
            Err(Refused::TooShort(WindowId::from_compositor(2)))
        );
    }

    /// Fitting keeps a boundary in proportion, and moves it off proportion only
    /// as far as a minimum demands.
    #[test]
    fn fitting_keeps_proportion_until_a_minimum_says_otherwise() {
        let mut halves = Node::cut(
            Axis::SideBySide,
            480,
            Node::Share(any(1)),
            Node::Share(any(2)),
        );
        halves.fit(Axis::SideBySide, 960, 480);
        assert_eq!(first_length(&halves), Some(240));

        let wants_300 = Window::at_least(WindowId::from_compositor(3), Size::of(300, 1));
        let mut kept = Node::cut(
            Axis::SideBySide,
            480,
            Node::Share(any(1)),
            Node::Share(wants_300),
        );
        kept.fit(Axis::SideBySide, 960, 480);
        assert_eq!(first_length(&kept), Some(180));
    }

    /// Taking a share out hands its area to the piece beside it.
    #[test]
    fn a_share_taken_out_goes_to_its_neighbour() {
        let mut tree = Node::Share(any(1));
        tree.halve(display(), WindowId::from_compositor(1), any(2), Side::Right)
            .unwrap();
        tree.halve(
            display(),
            WindowId::from_compositor(2),
            any(3),
            Side::Bottom,
        )
        .unwrap();
        let left = tree
            .without(WindowId::from_compositor(3), display())
            .unwrap();
        let mut shares = Vec::new();
        left.lay_out(display(), &mut shares);
        assert_eq!(shares.len(), 2);
        assert_eq!(shares.get(1).unwrap().area().height(), 1080);
        assert!(
            Node::Share(any(1))
                .without(WindowId::from_compositor(1), display())
                .is_none()
        );
    }
}
