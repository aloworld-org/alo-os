//! **The row that asks alo, said on its one line** — `561:339` draws *Ask alo
//! "…"* and *· a new task* on one line at 12, and where the answer would come
//! from on a second at 36. A request longer than the row used to wrap onto
//! that second line and draw over it (the walk of 2026-11-15).
//!
//! So a long request is shortened **inside its quotation marks**: the words
//! around it are the person's language's (`dock.bar.ask`), which may put the
//! request first, last or between marks of its own, and the request is the
//! only part that may be cut. *· a new task* keeps its room, so what pressing
//! the row does is never the part that is lost.

use cosmic_text::{Attrs, FontSystem, Metrics};
use unicode_segmentation::UnicodeSegmentation as _;

use crate::window_edge_title::{AN_ELLIPSIS, fitted_as};

/// `asking`, which says `request` somewhere inside it, on one line no wider
/// than `room` pixels.
///
/// Whole where it fits. Otherwise `request` is cut at a letter's edge and
/// ends in an ellipsis, with the words around it kept — the closing mark
/// included. Where even that cannot fit, or `request` is not inside
/// `asking`, the whole line ends in an ellipsis instead.
pub(crate) fn the_request_in_its_room(
    fonts: &mut FontSystem,
    (asking, request): (&str, &str),
    room: i32,
    metrics: Metrics,
    attrs: &Attrs<'_>,
) -> String {
    let wide = |fonts: &mut FontSystem, said: &str| {
        crate::painted_text::how_wide_as(fonts, said, metrics, attrs)
    };
    if room <= 0 {
        return String::new();
    }
    if wide(fonts, asking) <= room {
        return asking.to_owned();
    }
    let request = request.trim();
    let Some(at) = (!request.is_empty())
        .then(|| asking.find(request))
        .flatten()
    else {
        return fitted_as(fonts, asking, room, metrics, attrs).shown;
    };
    let (before, after) = (
        asking.get(..at).unwrap_or_default(),
        asking.get(at + request.len()..).unwrap_or_default(),
    );
    let cut_at = |kept: &str| format!("{before}{}{AN_ELLIPSIS}{after}", kept.trim_end());
    let boundaries: Vec<usize> = request.grapheme_indices(true).map(|(at, _)| at).collect();
    // The longest start of the request that still fits, found by halving: a
    // long request stepped one letter at a time is shaped hundreds of times.
    let (mut fits, mut beyond) = (0, boundaries.len());
    if wide(fonts, &cut_at("")) > room {
        return fitted_as(fonts, asking, room, metrics, attrs).shown;
    }
    while beyond - fits > 1 {
        let probe = fits + (beyond - fits) / 2;
        let kept = boundaries
            .get(probe)
            .and_then(|end| request.get(..*end))
            .unwrap_or_default();
        if wide(fonts, &cut_at(kept)) <= room {
            fits = probe;
        } else {
            beyond = probe;
        }
    }
    cut_at(
        boundaries
            .get(fits)
            .and_then(|end| request.get(..*end))
            .unwrap_or_default(),
    )
}

/// How much of `room` the request's words may take when *· a new task* is
/// `after` wide, with `gap` between them: all but what follows them, and
/// never less than half — on a row too narrow for both, what follows gives
/// way too.
pub(crate) const fn the_requests_room(room: i32, after: i32, gap: i32) -> i32 {
    let left = room - after - gap;
    if left < room / 2 { room / 2 } else { left }
}

#[cfg(test)]
#[path = "the_ask_rows_line_tests.rs"]
mod tests;
