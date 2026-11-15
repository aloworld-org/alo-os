//! **Where on the screen to put the pointer**, found in the picture rather than
//! assumed: the middle of the screen, the largest patch of one colour, or the
//! largest patch that changed since a picture the walk kept.
//!
//! Nothing here knows where a window opens. alo places windows itself
//! (`docs/design/where-a-new-window-opens.md`), and a walk that aimed at the
//! place a window opened last time would be aiming at a memory. So a window is
//! found as **what changed** when it opened, and a part of a web page as **its
//! colour**, which the walk's own page chooses.
//!
//! # A patch, not a pixel
//!
//! The screen is read in cells of [`CELL`] pixels ([`COLOUR_CELL`] for a
//! colour), and a cell counts when three
//! quarters of it match. The largest group of touching cells is the patch, and
//! the place aimed at is the cell of it nearest its middle — inside the patch
//! even where the patch is not a rectangle. So the clock changing in the panel,
//! a caret blinking or the pointer itself moving is never the largest patch
//! beside a window that opened.

use std::collections::VecDeque;

use crate::frame::Frame;

/// How many pixels across and down one cell is, for what changed.
pub const CELL: usize = 16;

/// How many pixels across and down one cell is, for a colour: smaller,
/// because a button is a colour too, and a 32-pixel button with its label
/// across the middle has almost no 16-pixel cell three quarters its colour —
/// measured on Firefox's *Continue* (run 43662980364), which the walk then
/// could not find.
pub const COLOUR_CELL: usize = 8;

/// The fewest cells a patch must have to be aimed at.
pub const FEWEST_CELLS: usize = 2;

/// How far a pixel may be from a colour and still be it, on each channel.
const SAME_COLOUR: u8 = 10;

/// How far a pixel must move on some channel to have changed.
///
/// **Small, because a white window on the cream desktop differs by about ten**:
/// at 24, the first walk run that reached Text Editor (43650359669) found the
/// Dock's shift larger than the window and aimed the pointer at the Dock. The
/// screen capture is exact, so a few levels is no noise.
const CHANGED: u8 = 3;

/// A place a walk can point at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// The middle of the screen.
    Middle,
    /// The largest patch of this colour.
    Colour([u8; 3]),
    /// The largest patch that changed since the picture the walk kept last.
    WhatChanged,
}

impl Target {
    /// How a report says it.
    #[must_use]
    pub fn said(self) -> String {
        match self {
            Self::Middle => "the middle of the screen".to_owned(),
            Self::Colour([red, green, blue]) => {
                format!("the largest patch of #{red:02X}{green:02X}{blue:02X}")
            }
            Self::WhatChanged => "the largest patch that changed".to_owned(),
        }
    }
}

/// The pixel to aim at for `target` on `now`, given the picture the walk kept,
/// or `None` where there is no such place.
#[must_use]
pub fn where_is(target: Target, now: &Frame, kept: Option<&Frame>) -> Option<(usize, usize)> {
    match target {
        Target::Middle => {
            let (width, height) = now.size();
            Some((width / 2, height / 2))
        }
        Target::Colour(colour) => largest_patch(now, COLOUR_CELL, |x, y| {
            now.rgb(x, y).is_some_and(|it| {
                it.iter()
                    .zip(colour)
                    .all(|(a, b)| a.abs_diff(b) <= SAME_COLOUR)
            })
        }),
        Target::WhatChanged => {
            let kept = kept?;
            if kept.size() != now.size() {
                return None;
            }
            largest_patch(now, CELL, |x, y| match (now.rgb(x, y), kept.rgb(x, y)) {
                (Some(one), Some(two)) => one.iter().zip(two).any(|(a, b)| a.abs_diff(b) > CHANGED),
                _ => false,
            })
        }
    }
}

/// The middle of the largest group of touching cells in which three quarters
/// of the pixels are `marked`.
fn largest_patch(
    frame: &Frame,
    cell: usize,
    marked: impl Fn(usize, usize) -> bool,
) -> Option<(usize, usize)> {
    let (width, height) = frame.size();
    let (across, down) = (width / cell, height / cell);
    let counts = |cx: usize, cy: usize| {
        let mut count = 0;
        for y in cy * cell..(cy + 1) * cell {
            for x in cx * cell..(cx + 1) * cell {
                if marked(x, y) {
                    count += 1;
                }
            }
        }
        count * 4 >= cell * cell * 3
    };
    let cells: Vec<bool> = (0..down)
        .flat_map(|cy| (0..across).map(move |cx| (cx, cy)))
        .map(|(cx, cy)| counts(cx, cy))
        .collect();
    let mut seen = vec![false; cells.len()];
    let mut best: Vec<usize> = Vec::new();
    for start in 0..cells.len() {
        if !cells.get(start).copied().unwrap_or(false) || seen.get(start).copied().unwrap_or(true) {
            continue;
        }
        let patch = gathered(start, across, &cells, &mut seen);
        if patch.len() > best.len() {
            best = patch;
        }
    }
    if best.len() < FEWEST_CELLS {
        return None;
    }
    let count = best.len();
    let mean_x = best.iter().map(|it| it % across).sum::<usize>() * 2 / count;
    let mean_y = best.iter().map(|it| it / across).sum::<usize>() * 2 / count;
    let nearest = best.iter().copied().min_by_key(|it| {
        let (cx, cy) = (it % across * 2, it / across * 2);
        cx.abs_diff(mean_x).pow(2) + cy.abs_diff(mean_y).pow(2)
    })?;
    Some((
        nearest % across * cell + cell / 2,
        nearest / across * cell + cell / 2,
    ))
}

/// Every marked cell touching `start`, marking each as seen.
fn gathered(start: usize, across: usize, cells: &[bool], seen: &mut [bool]) -> Vec<usize> {
    let mut patch = Vec::new();
    let mut waiting = VecDeque::from([start]);
    if let Some(it) = seen.get_mut(start) {
        *it = true;
    }
    while let Some(cell) = waiting.pop_front() {
        patch.push(cell);
        let (cx, cy) = (cell % across, cell / across);
        let mut neighbours = Vec::with_capacity(4);
        if cx > 0 {
            neighbours.push(cell - 1);
        }
        if cx + 1 < across {
            neighbours.push(cell + 1);
        }
        if cy > 0 {
            neighbours.push(cell - across);
        }
        neighbours.push(cell + across);
        for next in neighbours {
            let marked = cells.get(next).copied().unwrap_or(false);
            if let Some(was_seen) = seen.get_mut(next)
                && marked
                && !*was_seen
            {
                *was_seen = true;
                waiting.push_back(next);
            }
        }
    }
    patch
}

#[cfg(test)]
#[path = "a_place_on_the_screen_tests.rs"]
mod tests;
