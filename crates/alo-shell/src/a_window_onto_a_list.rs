//! **A window of so many rows onto a first-start list**, around the cursor.
//!
//! Every list on the first-start sheets — the languages, the regions, the
//! keyboards, the time zones and the overview's three rows — shows a window of
//! it at a time, and the window follows the cursor: the arrow keys and the
//! wheel move the cursor and the window comes with it, so every row of a long
//! list is reachable from a short screen.
//!
//! **How many rows is the drawing's to say**, because only the drawing knows
//! how tall the screen is and how the heading wrapped
//! (`crate::a_sheet_that_fits`). So the window is asked of the list with that
//! number, and is derived from the cursor rather than kept beside it: there is
//! one answer to *where is the list scrolled to*, and it cannot disagree with
//! the cursor. The cursor sits in the middle of the window where there is room
//! on both sides, and the window stops at each end rather than scrolling past
//! it into blank rows.

/// The rows of a list a sheet draws now, and where its cursor and its choice
/// fall among them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AWindow<'a, T> {
    /// Where in the whole list the first row drawn is.
    pub(crate) first: usize,
    /// The rows drawn.
    pub(crate) rows: &'a [T],
    /// The cursor's row among them; always there while the list has rows.
    pub(crate) cursor: Option<usize>,
    /// The chosen row among them, and [`None`] when nothing is chosen **or
    /// what was chosen is out of sight** — both mean *draw no selected border
    /// on any row*, so they are one answer.
    pub(crate) chosen: Option<usize>,
}

/// A window of at most `rows` rows onto `all`, with the cursor `on` in it and
/// `chosen` marked where it falls inside.
///
/// A window of no rows is a window of one: a list drawn with nothing in it
/// would be a list nobody could reach.
pub(crate) fn a_window<T>(
    all: &[T],
    on: usize,
    chosen: Option<usize>,
    rows: usize,
) -> AWindow<'_, T> {
    let rows = rows.max(1);
    let first = on
        .saturating_sub(rows / 2)
        .min(all.len().saturating_sub(rows));
    let to = first.saturating_add(rows).min(all.len());
    let inside = |at: usize| at.checked_sub(first).filter(|within| first + within < to);
    AWindow {
        first,
        rows: all.get(first..to).unwrap_or_default(),
        cursor: inside(on),
        chosen: chosen.and_then(inside),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The window follows the cursor and stops at each end.**
    #[test]
    fn the_window_follows_the_cursor_and_stops_at_each_end() {
        let all: Vec<usize> = (0..10).collect();
        let at = |on| a_window(&all, on, None, 3);
        assert_eq!(at(0).rows, [0, 1, 2]);
        assert_eq!(at(0).cursor, Some(0));
        assert_eq!(at(5).rows, [4, 5, 6]);
        assert_eq!(at(5).cursor, Some(1));
        assert_eq!(at(9).rows, [7, 8, 9]);
        assert_eq!(at(9).cursor, Some(2));
    }

    /// **Every row is reachable with a window of one**, which is the
    /// shortest screen a sheet may be drawn on.
    #[test]
    fn a_window_of_one_reaches_every_row() {
        let all: Vec<usize> = (0..5).collect();
        for on in 0..all.len() {
            let window = a_window(&all, on, None, 1);
            assert_eq!(window.rows, [on]);
            assert_eq!(window.first, on);
        }
        assert_eq!(a_window(&all, 3, None, 0).rows, [3], "no rows is one row");
    }

    /// **The choice is marked only while it is in sight.**
    #[test]
    fn the_choice_is_marked_only_while_it_is_in_sight() {
        let all: Vec<usize> = (0..10).collect();
        assert_eq!(a_window(&all, 0, Some(2), 3).chosen, Some(2));
        assert_eq!(a_window(&all, 0, Some(3), 3).chosen, None);
        assert_eq!(a_window(&all, 9, Some(8), 3).chosen, Some(1));
    }

    /// **A list shorter than the window is all of it**, and an empty one is
    /// nothing at all, with no cursor drawn on a row that is not there.
    #[test]
    fn a_short_list_is_all_of_it_and_an_empty_one_is_nothing() {
        let two = [0, 1];
        assert_eq!(a_window(&two, 1, Some(0), 3).rows, [0, 1]);
        let none: [usize; 0] = [];
        let window = a_window(&none, 0, None, 3);
        assert!(window.rows.is_empty());
        assert_eq!(window.cursor, None);
    }
}
