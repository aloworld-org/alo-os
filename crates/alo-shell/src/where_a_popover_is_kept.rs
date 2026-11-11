//! Where a list opened from the Dock is allowed to go: **inside the display and
//! clear of the column the put-aside panel owns.**
//!
//! `docs/features.md` promises it of *More apps* — *inside the display and clear
//! of the panel of minimised windows and the other fixed controls* — and the
//! owner's decision of 2026-11-11 put a second list on the Dock, the matches
//! that rise from its field as a person types. **Two lists, one rule**, so the
//! rule is one function: written twice, the day one of them learned about a
//! right-to-left panel the other would cover it.

use smithay::utils::{Physical, Rectangle};

/// Where a list of `size` that would like to open at `origin` goes on a
/// display of `display`, kept clear of `clear_of` by `gap`.
///
/// **The display wins over the column**: a list too wide for the room beside
/// the panel is pushed against the room's near side rather than off the
/// display, which is the order *More apps* has always kept.
pub(crate) fn kept_on_the_display(
    origin: (i32, i32),
    size: (i32, i32),
    display: (i32, i32),
    clear_of: Option<Rectangle<i32, Physical>>,
    gap: i32,
) -> Rectangle<i32, Physical> {
    let (across, tall) = size;
    let (width, height) = display;
    let (from, upto) = match clear_of {
        None => (0, width),
        Some(column) if column.loc.x <= 0 => (column.size.w + gap, width),
        Some(column) => (0, column.loc.x - gap),
    };
    let left = origin.0.clamp(from, (upto - across).max(from));
    let top = origin.1.clamp(0, (height - tall).max(0));
    Rectangle::new((left, top).into(), (across, tall).into())
}

#[cfg(test)]
#[path = "where_a_popover_is_kept_tests.rs"]
mod tests;
