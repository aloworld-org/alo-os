//! **What a screen reader is told the alo Bar's matches are**, while they rise
//! from the Dock's field.
//!
//! The rows are named by the words they are drawn with — ADR 0089's rule, kept
//! by every surface a reader meets: an application by its own name, and the row
//! that asks alo by the sentence it shows, *Ask alo "…"*. The one the keyboard
//! is on is **selected**, and that is the first until an arrow moves it, which
//! is also the row Enter acts on (`Server::what_the_bar_would_open`).

use alo_strings::{Filling, Strings};

impl crate::Server {
    /// Each row of the matches shown now, in reading order, and whether it is
    /// the one the keyboard is on. Empty while no list is shown.
    pub(crate) fn the_bars_matches_as_read(&self, strings: &Strings) -> Vec<(String, bool)> {
        if !self.the_composer.has_the_keyboard() || self.the_bars_matches_places.is_none() {
            return Vec::new();
        }
        let typed = self.the_composer.typed();
        let mut rows: Vec<String> = crate::the_bar_finds::what_the_bar_finds(
            typed,
            &self.what_this_machine_has,
            &self.what_each_is_called,
            &self.what_each_is,
            strings,
        )
        .into_iter()
        .map(|found| found.name)
        .collect();
        if self.the_launcher_offers_to_ask() {
            rows.push(
                strings
                    .say(
                        &alo_dock::words::ASK_ALO_ABOUT.key(),
                        &Filling::of("what", typed.trim().to_owned()),
                    )
                    .into_text(),
            );
        }
        let chosen = self.which_row_in_the_bar.unwrap_or(0);
        rows.into_iter()
            .enumerate()
            .map(|(which, name)| (name, which == chosen))
            .collect()
    }
}

#[cfg(test)]
#[path = "the_bars_matches_as_read_tests.rs"]
mod tests;
