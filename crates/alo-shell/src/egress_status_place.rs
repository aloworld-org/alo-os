//! Where the egress indicator sits: at the far end of the dock, wherever the
//! dock is.
//!
//! `docs/features.md` puts the indicator in the status area *so "nothing has
//! left this machine" sits where a person already glances*. Which edge the dock
//! is on and which end of it is the far one are `alo-dock`'s decisions —
//! `alo_dock::Layout::edge` and `alo_dock::Layout::status` — and this file only
//! turns them into pixels on one output.
//!
//! The lines are stacked from that corner away from the dock, so a dock along
//! the bottom grows them upwards, a dock along the top downwards, and a dock
//! down either side grows them up from the bottom of the screen, beside it.
//! The first line stays nearest the dock and a new one is added beyond it, so
//! nothing already on the screen moves when something else starts leaving.

use alo_dock::{End, Layout};

/// Which way a row is aligned across the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Across {
    /// Rows start at this x and run right.
    FromLeft(i32),
    /// Rows end at this x.
    FromRight(i32),
}

/// Which way rows are stacked down the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stacked {
    /// The first row's top is at this y, and the next is below it.
    Downwards(i32),
    /// The first row's bottom is at this y, and the next is above it.
    Upwards(i32),
}

/// The corner the indicator grows from, and how much room it has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Place {
    /// Where rows are aligned across.
    pub(crate) across: Across,
    /// Where rows are stacked from.
    pub(crate) stacked: Stacked,
    /// The widest a row may be.
    pub(crate) room_across: i32,
    /// The tallest the whole stack may be.
    pub(crate) room_down: i32,
}

impl Place {
    /// The same place, with `taken` pixels already used at the corner.
    ///
    /// **What is in use has the corner and what is leaving stacks beyond it.**
    /// ADR 0010 makes the in-use indicator's position one of the three things a
    /// person is given besides a colour, and the reason is that position costs
    /// nothing to learn — which only holds while that stack's origin does not
    /// move. So it keeps the fixed corner and this moves the other one along.
    ///
    /// The egress indicator's own rule is untouched: its first line stays
    /// nearest its origin and a new one is added beyond, so nothing already on
    /// the screen moves when something else starts leaving.
    pub(crate) fn beyond(self, taken: i32) -> Self {
        if taken <= 0 {
            return self;
        }
        Self {
            stacked: match self.stacked {
                Stacked::Downwards(y) => Stacked::Downwards(y + taken),
                Stacked::Upwards(y) => Stacked::Upwards(y - taken),
            },
            room_down: (self.room_down - taken).max(0),
            ..self
        }
    }

    /// The status area's corner on an output of `size`, for a dock laid out
    /// as `layout`, keeping `margin` pixels from the dock and the output's
    /// edges.
    pub(crate) fn of(layout: Layout, size: (i32, i32), margin: i32) -> Self {
        let (width, height) = size;
        let thickness = i32::try_from(layout.thickness().as_pixels())
            .unwrap_or(i32::MAX)
            .clamp(0, width.min(height));
        let at = layout.status().at();
        match layout.edge() {
            alo_dock::Edge::Bottom | alo_dock::Edge::Top => {
                let across = match at {
                    End::Left => Across::FromLeft(margin),
                    End::Right | End::Top | End::Bottom => Across::FromRight(width - margin),
                };
                let stacked = if layout.edge() == alo_dock::Edge::Bottom {
                    Stacked::Upwards(height - thickness - margin)
                } else {
                    Stacked::Downwards(thickness + margin)
                };
                Self {
                    across,
                    stacked,
                    room_across: width - 2 * margin,
                    room_down: height - thickness - 2 * margin,
                }
            }
            alo_dock::Edge::Left | alo_dock::Edge::Right => {
                let across = if layout.edge() == alo_dock::Edge::Left {
                    Across::FromLeft(thickness + margin)
                } else {
                    Across::FromRight(width - thickness - margin)
                };
                let stacked = match at {
                    End::Top => Stacked::Downwards(margin),
                    End::Bottom | End::Left | End::Right => Stacked::Upwards(height - margin),
                };
                Self {
                    across,
                    stacked,
                    room_across: width - thickness - 2 * margin,
                    room_down: height - 2 * margin,
                }
            }
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
    use alo_appearance::TextScale;
    use alo_dock::{Dock, Edge, Screen};
    use alo_strings::Direction;

    /// A dock on `edge` of a 1920×1080 screen, read `reading`.
    fn laid_out(edge: Edge, reading: Direction) -> Layout {
        let mut dock = Dock::shipped();
        dock.set_edge(edge);
        dock.layout_on(
            Screen::of(1920, 1080).unwrap(),
            TextScale::ordinary(),
            reading,
        )
    }

    /// **The far end of a dock along the bottom is the corner a person reads
    /// last**, and the lines grow up from it, clear of the dock.
    #[test]
    fn a_dock_along_the_bottom_puts_it_at_the_end_read_last() {
        let layout = laid_out(Edge::Bottom, Direction::LeftToRight);
        let thick = i32::try_from(layout.thickness().as_pixels()).unwrap();
        let place = Place::of(layout, (1920, 1080), 8);
        assert_eq!(place.across, Across::FromRight(1912));
        assert_eq!(place.stacked, Stacked::Upwards(1080 - thick - 8));

        let mirrored = Place::of(
            laid_out(Edge::Bottom, Direction::RightToLeft),
            (1920, 1080),
            8,
        );
        assert_eq!(mirrored.across, Across::FromLeft(8));
    }

    /// A dock along the top grows its lines downwards, below it.
    #[test]
    fn a_dock_along_the_top_grows_it_downwards() {
        let layout = laid_out(Edge::Top, Direction::LeftToRight);
        let thick = i32::try_from(layout.thickness().as_pixels()).unwrap();
        let place = Place::of(layout, (1920, 1080), 8);
        assert_eq!(place.stacked, Stacked::Downwards(thick + 8));
        assert_eq!(place.across, Across::FromRight(1912));
    }

    /// **A dock down either side keeps its status area at the bottom**, with
    /// the lines beside the dock rather than over it, in both reading
    /// directions — `alo-dock`'s column does not turn over.
    #[test]
    fn a_dock_down_the_side_puts_it_at_the_bottom_beside_the_dock() {
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            let left = laid_out(Edge::Left, reading);
            let thick = i32::try_from(left.thickness().as_pixels()).unwrap();
            let place = Place::of(left, (1920, 1080), 8);
            assert_eq!(place.across, Across::FromLeft(thick + 8));
            assert_eq!(place.stacked, Stacked::Upwards(1072));

            let right = laid_out(Edge::Right, reading);
            let thick = i32::try_from(right.thickness().as_pixels()).unwrap();
            let place = Place::of(right, (1920, 1080), 8);
            assert_eq!(place.across, Across::FromRight(1920 - thick - 8));
            assert_eq!(place.stacked, Stacked::Upwards(1072));
        }
    }

    /// The room left over never includes the dock.
    #[test]
    fn the_room_leaves_the_dock_out() {
        for edge in Edge::ALL {
            let layout = laid_out(edge, Direction::LeftToRight);
            let thick = i32::try_from(layout.thickness().as_pixels()).unwrap();
            let place = Place::of(layout, (1920, 1080), 8);
            assert!(place.room_across + place.room_down <= 1920 + 1080 - thick - 32);
        }
    }
}
