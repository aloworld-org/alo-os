//! **Which of alo's own sheets is in front, and where the keyboard is on it.**
//!
//! The password sheet (ADR 0150) and the *Open with* sheet (frame `25:52`) are
//! drawn by the desktop itself and hold every key while they are up. A screen
//! reader is told one of them came up, and where its keys now go, from what
//! this records: the sheet, by a number that changes with each question, and
//! the node the keyboard is on.
//!
//! **A sheet is named by its question, not by its place.** Paths on the bus are
//! positions and move whenever anything above them changes, so a second
//! question in the same place would read as the first one still up. The
//! number is what tells them apart (`crate::access_events`).

/// Which sheet, and which question on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TheSheet {
    /// The password sheet, on the question with this number.
    Password(u64),
    /// The *Open with* sheet, the one with this number.
    OpenWith(u64),
}

/// The sheet in front, and where its keys go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ASheetInFront {
    /// Which sheet.
    pub(crate) which: TheSheet,
    /// Its own node.
    pub(crate) at: usize,
    /// The node the keyboard is on, under it.
    pub(crate) keyboard: usize,
}

impl ASheetInFront {
    /// The same sheet after the tree was renumbered by `moved`, or [`None`]
    /// where its node or the keyboard's went.
    pub(crate) fn moved(self, moved: &impl Fn(usize) -> Option<usize>) -> Option<Self> {
        Some(Self {
            at: moved(self.at)?,
            keyboard: moved(self.keyboard)?,
            ..self
        })
    }

    /// Where the keyboard is, counted from the sheet's own node — the same
    /// whatever else in the tree moved.
    pub(crate) const fn the_keyboard_from_the_sheet(&self) -> usize {
        self.keyboard.saturating_sub(self.at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A sheet renumbered with the tree keeps its keyboard**, and one whose
    /// node went is not in front at all.
    #[test]
    fn a_sheet_moves_with_the_tree_or_goes() {
        let sheet = ASheetInFront {
            which: TheSheet::Password(3),
            at: 10,
            keyboard: 12,
        };
        let down_by_two = |old: usize| old.checked_sub(2);
        assert_eq!(
            sheet.moved(&down_by_two),
            Some(ASheetInFront {
                which: TheSheet::Password(3),
                at: 8,
                keyboard: 10,
            })
        );
        assert_eq!(
            sheet
                .moved(&down_by_two)
                .map(|it| it.the_keyboard_from_the_sheet()),
            Some(2)
        );
        let its_node_went = |old: usize| (old != 10).then_some(old);
        assert_eq!(sheet.moved(&its_node_went), None);
    }
}
