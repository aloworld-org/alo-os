//! **Finding alo's teal edge round a window in a picture of the screen**, by the
//! colour the shell draws it in.
//!
//! While alo works in a window, that window wears a thin deep teal edge round
//! its whole card (`alo_shell::alos_band_in_a_window`; ADR 0143; the design
//! note *the window controls*, *Readings taken building stage 3*). This is how
//! a walk sees it on the screen rather than taking the machine's word for it.
//!
//! # The reference is the shell's own colour
//!
//! [`TEAL`] is `accent/default` read from `alo-appearance` — the token the shell
//! paints the edge with — so a change to the colour moves the reference with
//! it. Each pixel is compared within [`TOLERANCE`] on every channel. The band's
//! ground is `accent/soft`, far from this, and is not the edge.
//!
//! # What counts as an edge
//!
//! Four thin straight teal sides that close round one rectangle: a left and a
//! right side at least [`SHORTEST_SIDE`] long, beside each other down the
//! screen, and a top and a foot each within [`CORNER_ROOM`] of where those sides
//! end — the room a rounded corner takes, at a display scale of up to 3 —
//! lying between them. **Thin** means at most [`THICKEST`] pixels: a teal block
//! has teal sides too, and is not a window wearing alo.
//!
//! A side may be broken — the egress indicator keeps the top-left corner over
//! everything (law 1), and was half over VSCodium's top in the screenshot this
//! began from — so each side needs one unbroken stretch of [`SHORTEST_SIDE`],
//! not the whole length.
//!
//! # What would have to be true for this to find an edge that is not there
//!
//! Something else drawing four thin deep-teal lines round a rectangle at least
//! [`SHORTEST_SIDE`] each way: a picture of a window wearing alo, shown in
//! another window. Nothing on a fresh machine does.
//!
//! # And for it to miss one that is there
//!
//! A renderer that blended the edge into what is under it, or a display scale
//! above 3, whose corners take more than [`CORNER_ROOM`]. Then the check fails
//! and says the longest teal it did find, which is the opposite of a silent
//! pass.

use crate::frame::Frame;

/// Deep teal, `accent/default`: the colour the shell draws alo's edge in.
pub const TEAL: [u8; 3] = {
    let accent = alo_appearance::Role::AccentDefault.colour();
    [accent.red(), accent.green(), accent.blue()]
};

/// How far a pixel on the screen may be from [`TEAL`], on each channel.
pub const TOLERANCE: u8 = 16;

/// The shortest unbroken stretch a side must have, in pixels: longer than any
/// control's outline, shorter than the side of any window alo can work in.
pub const SHORTEST_SIDE: usize = 120;

/// How far a corner may take a side's end from the line across: the card's
/// corner of 16, at a scale of up to 3.
pub const CORNER_ROOM: usize = 48;

/// The thickest a side may be, in pixels: the border round a window alo
/// operates is 3 at rest and 6 while it sends, at a scale of 1, so 18 at 3.
pub const THICKEST: usize = 18;

/// Where an edge is on the screen: its outermost columns and rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ring {
    /// The left side's first column.
    pub left: usize,
    /// The top's first row.
    pub top: usize,
    /// The right side's last column.
    pub right: usize,
    /// The foot's last row.
    pub bottom: usize,
}

/// No edge, and the longest teal lines that were there instead, for a report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nearest {
    /// The longest unbroken teal line across.
    pub across: usize,
    /// The longest unbroken teal line down.
    pub down: usize,
}

/// One thin straight stretch of teal: where it is along its line, and which
/// line.
#[derive(Debug, Clone, Copy)]
struct Stretch {
    /// The row (across) or column (down) it is on — the stroke's first.
    on: usize,
    /// Where it starts along the line.
    from: usize,
    /// Where it ends along the line, not included.
    to: usize,
    /// How thick its stroke is.
    thick: usize,
}

/// Whether the pixel at `(x, y)` is alo's teal.
fn teal(frame: &Frame, x: usize, y: usize) -> bool {
    frame.rgb(x, y).is_some_and(|rgb| {
        rgb.iter()
            .zip(TEAL)
            .all(|(have, want)| have.abs_diff(want) <= TOLERANCE)
    })
}

/// Every thin stretch of at least [`SHORTEST_SIDE`] along one direction, and
/// the longest stretch of any thickness. `across` reads rows; otherwise
/// columns.
fn stretches(frame: &Frame, across: bool) -> (Vec<Stretch>, usize) {
    let (width, height) = frame.size();
    let (lines, length) = if across {
        (height, width)
    } else {
        (width, height)
    };
    let at = |line: usize, along: usize| {
        if across {
            teal(frame, along, line)
        } else {
            teal(frame, line, along)
        }
    };
    let mut found = Vec::new();
    let mut longest = 0;
    for line in 0..lines {
        let mut along = 0;
        while along < length {
            if !at(line, along) {
                along += 1;
                continue;
            }
            let from = along;
            while along < length && at(line, along) {
                along += 1;
            }
            let run = along - from;
            longest = longest.max(run);
            if run < SHORTEST_SIDE {
                continue;
            }
            // **Counted once, at its stroke's first line**, and only if thin.
            let middle = from + run / 2;
            if line > 0 && at(line - 1, middle) {
                continue;
            }
            let thick = (line..lines).take_while(|next| at(*next, middle)).count();
            if thick <= THICKEST {
                found.push(Stretch {
                    on: line,
                    from,
                    to: along,
                    thick,
                });
            }
        }
    }
    (found, longest)
}

/// **The first edge in the picture**, or the longest teal it held instead.
///
/// # Errors
/// [`Nearest`] when no four thin teal sides close round one rectangle.
pub fn a_ring_in(frame: &Frame) -> Result<Ring, Nearest> {
    let (rows, across) = stretches(frame, true);
    let (columns, down) = stretches(frame, false);
    for left in &columns {
        for right in &columns {
            if right.on < left.on + SHORTEST_SIDE {
                continue;
            }
            let together = left
                .to
                .min(right.to)
                .saturating_sub(left.from.max(right.from));
            if together < SHORTEST_SIDE {
                continue;
            }
            let (top, foot) = (left.from.min(right.from), left.to.max(right.to));
            let between =
                |row: &&Stretch| row.from >= left.on && row.to <= right.on + right.thick + THICKEST;
            let over = rows
                .iter()
                .filter(between)
                .find(|row| row.on <= top && top - row.on <= CORNER_ROOM);
            let under = rows.iter().filter(between).find(|row| {
                row.on >= foot.saturating_sub(THICKEST) && row.on - foot.min(row.on) <= CORNER_ROOM
            });
            if let (Some(over), Some(under)) = (over, under) {
                return Ok(Ring {
                    left: left.on,
                    top: over.on,
                    right: right.on + right.thick - 1,
                    bottom: under.on + under.thick - 1,
                });
            }
        }
    }
    Err(Nearest { across, down })
}

/// How far outside the edge's line the canvas behind a window is looked at.
const OUTSIDE: usize = 4;

/// How far inside it the window's own pixels are looked at: past the edge's
/// widest stroke, so what is read is the window, not the edge.
const INSIDE: usize = 10;

/// How different, summed over the three channels, a pixel just inside must be
/// from the one just outside for a window's own edge to lie between them.
const A_WINDOWS_EDGE: u32 = 96;

/// **Whether a window is still where alo's edge was** — read off the screen
/// as it is now, by the window's own edge: all the way round where the edge
/// ran, clear of its corners, the pixel just inside differs from the one just
/// outside on at least half of the points looked at.
///
/// **Not by its pixels being as they were.** A window's content changes while
/// alo works in it — the walk of 2026-11-16 (run 43872632801) found alo's edge
/// round VSCodium before VSCodium had drawn anything, and asking the pixels
/// inside to stay the same then said VSCodium had gone when it was there,
/// editor drawn. So this asks where the window is, not what it shows.
///
/// What would have to be true for this to pass with the window gone: another
/// window, or a picture of one, with its edge at the same place. What it
/// refuses: a screen gone black, the canvas alone, and a window covering more
/// than half of where the edge ran.
#[must_use]
pub fn a_window_is_where_the_edge_was(ring: Ring, now: &Frame) -> bool {
    let differs = |outside: Option<(usize, usize)>, inside: Option<(usize, usize)>| {
        let (Some((ox, oy)), Some((ix, iy))) = (outside, inside) else {
            return false;
        };
        let (Some(out), Some(within)) = (now.rgb(ox, oy), now.rgb(ix, iy)) else {
            return false;
        };
        out.iter()
            .zip(within)
            .map(|(a, b)| u32::from(a.abs_diff(b)))
            .sum::<u32>()
            >= A_WINDOWS_EDGE
    };
    let mut looked = 0_usize;
    let mut found = 0_usize;
    let down = (ring.top + CORNER_ROOM)..ring.bottom.saturating_sub(CORNER_ROOM);
    for y in down.step_by(4) {
        for (outside, inside) in [
            (
                ring.left.checked_sub(OUTSIDE).map(|x| (x, y)),
                Some((ring.left + INSIDE, y)),
            ),
            (
                Some((ring.right + OUTSIDE, y)),
                ring.right.checked_sub(INSIDE).map(|x| (x, y)),
            ),
        ] {
            looked += 1;
            found += usize::from(differs(outside, inside));
        }
    }
    let across = (ring.left + CORNER_ROOM)..ring.right.saturating_sub(CORNER_ROOM);
    for x in across.step_by(4) {
        for (outside, inside) in [
            (
                ring.top.checked_sub(OUTSIDE).map(|y| (x, y)),
                Some((x, ring.top + INSIDE)),
            ),
            (
                Some((x, ring.bottom + OUTSIDE)),
                ring.bottom.checked_sub(INSIDE).map(|y| (x, y)),
            ),
        ] {
            looked += 1;
            found += usize::from(differs(outside, inside));
        }
    }
    looked > 0 && found * 2 >= looked
}

#[cfg(test)]
#[path = "the_edge_seen_tests.rs"]
mod tests;
