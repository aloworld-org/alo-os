//! Which part of the screen the keyboard is in, and how F6 moves between them.
//!
//! `docs/design/the-shortcuts-and-the-edges.md` settles the route:
//!
//! | Move among canvas, dock, panel, notifications and status | `F6` / `Shift`+`F6` |
//! | Move among controls in the focused area | `Tab` / `Shift`+`Tab` |
//! | Activate a focused control | `Enter` |
//! | Close a menu or a card | `Esc` |
//!
//! Measured 2026-10-12: **none of it existed.** `F6` appeared nowhere in this
//! workspace but a table of key names, and nothing anywhere modelled which area
//! had the keyboard. The Dock's own surfaces had taken the arrows, Enter and
//! Escape since 2026-10-11 — and a person could not reach them without lifting
//! a hand to a pointer, because the opening move was missing.
//!
//! # Why an area with nothing in it is skipped
//!
//! F6 that parked the keyboard on an empty panel would be a key a person
//! presses twice for no reason, and a person who cannot see the screen would
//! have no way to tell *this area is empty* from *this key did nothing*. So the
//! cycle asks each area whether it has anything to act on and passes over the
//! ones that do not.
//!
//! **That is also why the answer is computed rather than stored.** A panel
//! fills and empties while a person works; a remembered list of areas would be
//! a list of what was true when they last pressed F6.

/// A part of the screen the keyboard can be in, **other than the work**.
///
/// **Four, not five.** The design names five — *canvas, dock, panel,
/// notifications and status* — and the canvas is the absence below rather than
/// a variant here. It had both for one afternoon, and a test caught it at once:
/// F6 from the work landed on `TheCanvas` instead of the Dock, because the walk
/// had two canvases in it and visited each in turn.
///
/// The order is the design's, with the canvas taken out of the middle of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnArea {
    /// The Dock and the alo Bar.
    TheDock,
    /// The panel of put-aside windows at its edge.
    ThePanel,
    /// Notifications.
    Notifications,
    /// The clock, battery, network, volume and the privacy indicator.
    TheStatusArea,
}

impl AnArea {
    /// Every area, in the order F6 walks them.
    pub const EVERY: [Self; 4] = [
        Self::TheDock,
        Self::ThePanel,
        Self::Notifications,
        Self::TheStatusArea,
    ];
}

/// Which area has the keyboard.
///
/// **The canvas is not a value here.** `None` *is* the canvas: it is where the
/// keyboard is when the shell has not taken it, and the keys then reach the
/// person's application exactly as they always did. Making it a variant would
/// mean every key road in this crate had to ask whether the canvas was focused
/// before passing a key on, and the answer would have to be *yes* by default
/// for the machine to keep working — which is a default that will be got wrong.
pub type TheFocusedArea = Option<AnArea>;

/// Which way F6 was pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichWay {
    /// F6: on to the next area.
    Onward,
    /// Shift+F6: back to the one before.
    Back,
}

/// Where the keyboard goes when F6 is pressed.
///
/// `has_anything` is asked of each area in turn and is the whole of why this
/// takes a function rather than a list: *whether the panel has anything to act
/// on* is a fact about this moment, and a caller that passed a list would be
/// passing what was true when it built it.
///
/// **Answers `None` for the canvas**, which is both *nothing was focused* and
/// *the cycle has come back round to the work*. A person pressing F6 through
/// every area returns to what they were doing rather than to the first area
/// again, which is what every landmark cycle a person has used does.
#[must_use]
pub fn where_the_keyboard_goes(
    from: TheFocusedArea,
    pressed: WhichWay,
    has_anything: &dyn Fn(AnArea) -> bool,
) -> TheFocusedArea {
    // The canvas sits at one end of the walk rather than in the list, so the
    // positions are 0 for the canvas and 1..=5 for the areas.
    let at = from.map_or(0, |area| {
        AnArea::EVERY
            .iter()
            .position(|which| *which == area)
            .map_or(0, |at| at + 1)
    });
    let how_many = AnArea::EVERY.len() + 1;

    // Walk until an area with something in it, or until the canvas, which is
    // always a place the keyboard may be. At most one lap: a machine where
    // nothing has anything leaves the keyboard on the canvas rather than
    // spinning.
    let mut step = at;
    for _ in 0..how_many {
        step = match pressed {
            WhichWay::Onward => (step + 1) % how_many,
            WhichWay::Back => (step + how_many - 1) % how_many,
        };
        if step == 0 {
            return None;
        }
        // `?` rather than a `let ... else`, which clippy prefers and which is
        // the same answer: a step past the end of the list cannot happen —
        // `step` is taken modulo the list's length plus the work — and
        // answering *the work* is the right thing if it ever did, because this
        // workspace denies indexing a slice and a panic here would be a
        // keyboard a person could not move.
        let area = AnArea::EVERY.get(step - 1)?;
        if has_anything(*area) {
            return Some(*area);
        }
    }
    None
}

#[cfg(test)]
#[path = "which_area_has_the_keyboard_tests.rs"]
mod tests;
