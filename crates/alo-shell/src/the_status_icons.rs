//! **The task status's artwork**: the clock before *Waiting for your
//! permission* (`549:155`) and the circled tick before *Complete* (`549:281`).
//! alo's mark before *alo is working* (`549:46`) is the approval card's own
//! (`crate::the_approval_cards_icons::ACardsIcon::TheMark`), one file for one
//! drawing.
//!
//! The files are in `crates/alo-shell/icons/the-task-status/`, byte for byte as
//! the design file exported them, with where each came from in
//! `where-they-came-from.toml`. `alo_vector_shapes` reads and draws them at the
//! display's scale, as the approval card's are.

use std::sync::OnceLock;

use alo_vector_shapes::AShape;

/// The clock, as the design exported it.
pub(crate) const THE_CLOCK: &str = include_str!("../icons/the-task-status/clock.svg");

/// The circled tick, as the design exported it.
pub(crate) const THE_CIRCLED_TICK: &str = include_str!("../icons/the-task-status/circle-check.svg");

/// One of the status's own pieces of artwork.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AStatusIcon {
    /// The clock.
    TheClock,
    /// The circled tick.
    TheCircledTick,
}

impl AStatusIcon {
    /// Every one.
    pub(crate) const ALL: [Self; 2] = [Self::TheClock, Self::TheCircledTick];

    /// The file, as the design exported it.
    pub(crate) const fn svg(self) -> &'static str {
        match self {
            Self::TheClock => THE_CLOCK,
            Self::TheCircledTick => THE_CIRCLED_TICK,
        }
    }

    /// The file's name, as `where-they-came-from.toml` records it.
    pub(crate) const fn file_name(self) -> &'static str {
        match self {
            Self::TheClock => "clock.svg",
            Self::TheCircledTick => "circle-check.svg",
        }
    }

    /// The artwork, read once for the life of the shell.
    ///
    /// [`None`] for a file `alo_vector_shapes` refused, which is said once on
    /// the error stream and which this module's tests refuse on every build.
    pub(crate) fn shape(self) -> Option<&'static AShape> {
        static READ: OnceLock<[Option<AShape>; 2]> = OnceLock::new();
        let [clock, tick] = READ.get_or_init(|| {
            AStatusIcon::ALL.map(|icon| match AShape::read(icon.svg()) {
                Ok(shape) => Some(shape),
                Err(why) => {
                    eprintln!(
                        "alo-shell: the task status's {} could not be read and is not drawn: {why}",
                        icon.file_name()
                    );
                    None
                }
            })
        });
        match self {
            Self::TheClock => clock.as_ref(),
            Self::TheCircledTick => tick.as_ref(),
        }
    }
}

#[cfg(test)]
#[path = "the_status_icons_tests.rs"]
mod tests;
