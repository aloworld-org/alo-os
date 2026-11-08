//! **Left and Right along a row of choices** — the owner's answer of
//! 2026-11-08: *"Yes, also Left/Right"*.
//!
//! A row whose choices sit side by side — a segmented choice (*Pointer size*,
//! *Light and dark*, *Where the Dock sits*) or the accent's colours — is
//! walked by **Left and Right** as well as by Down and Up. Down and Enter work
//! as they did, so nothing is taken away.
//!
//! - **Along the row, and no further.** Left and Right stop at the row's first
//!   and last choice; Down is the key that leaves a row. A person pressing
//!   Right along *Default, Large, Larger* has not asked for *Kite*.
//! - **Moving is not choosing.** As with Down, the keyboard goes to the choice
//!   and Enter or Space chooses it, so walking along a row writes nothing.
//! - **The text size keeps its own Left and Right**, which make it smaller and
//!   larger; that is in `crate::settings_window`, before this is asked.
//!
//! **Left is towards the row's first choice.** Every machine reads left to
//! right today (`alo-desktop` sets it); a right-to-left reading draws the row
//! the other way round and would want Right towards the first. Nothing tells
//! the window the reading yet, and the text size's Left and Right have the same
//! gap.

use std::time::SystemTime;

use alo_strings::Strings;

use crate::settings_page::{ABlock, TheKind};
use crate::settings_window::Open;

/// Which way along a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Along {
    /// Left: towards the row's first choice.
    TowardsTheFirst,
    /// Right: towards the row's last choice.
    TowardsTheLast,
}

impl Open {
    /// Where the keyboard goes from the row at `focus` on the page shown, when
    /// it is one of a row of choices: the next choice `way` along that row,
    /// counted in the page's rows. [`None`] where the focus is not on such a
    /// row, and where it is already at the row's end.
    pub(crate) fn along_the_row(
        &self,
        focus: usize,
        way: Along,
        strings: &Strings,
        now: SystemTime,
    ) -> Option<usize> {
        let rows = self.rows_on(self.category, now);
        let focused = rows.get(focus)?;
        let page = crate::settings_page::the_page(self, self.category, strings, now);
        let choices: Vec<_> = page
            .blocks
            .iter()
            .filter_map(|block| match block {
                ABlock::Section(section) => Some(section),
                ABlock::AnAction(_) | ABlock::Said(_) => None,
            })
            .flat_map(|section| section.rows.iter())
            .filter(|row| matches!(row.kind, TheKind::ASegmented | TheKind::ASwatches))
            .map(|row| row.parts.iter().map(|part| &part.row).collect::<Vec<_>>())
            .find(|parts| parts.contains(&focused))?;
        let at = choices.iter().position(|row| *row == focused)?;
        let next = match way {
            Along::TowardsTheFirst => at.checked_sub(1)?,
            Along::TowardsTheLast => at + 1,
        };
        let next = choices.get(next)?;
        rows.iter().position(|row| row == *next)
    }
}

#[cfg(test)]
#[path = "settings_along_a_row_tests.rs"]
mod tests;
