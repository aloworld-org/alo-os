//! What a person typed into *More apps*, and which entries it leaves.
//!
//! The field is drawn and has been since the overview was built; this is what
//! it does. `docs/autonomy/the-dock-more-apps-and-the-minimised-panel.md` task
//! 5: *the search field, filtering with stable order and a clear empty result.*
//!
//! # It filters what is already there and never searches the machine
//!
//! `alo_dock::words::SEARCH_APPS` says so to a translator in as many words, and
//! it is a rule about this type too: nothing here reads a disk, asks a tool or
//! knows what is installed. The list it narrows is the one *More apps* already
//! shows — pinned, and open — so a person typing a name they have not got finds
//! nothing rather than an offer to install something.
//!
//! # The name is given rather than taken from the entry
//!
//! An application's name is its own, as its source reports it. **The Settings
//! entry has no name of that kind** — it is this compositor's own surface and
//! its label is translated — so there is no name this crate could read off an
//! `ADockEntry` for it. Asking the caller for the name it draws is what keeps
//! *what is matched* and *what is shown* the same string, which is the whole
//! of a search field being trustworthy: a row that matched on something the
//! person cannot see is a row they cannot explain.

use unicode_segmentation::UnicodeSegmentation;

/// The most a person can type into the field.
///
/// **A bound rather than none**, because the field takes keys from a keyboard
/// and a held key repeats: without it a person leaning on a key grows a
/// `String` until the compositor is the thing that is wrong with their machine.
/// Three hundred is far past any application name and far short of a problem.
pub const THE_MOST_THAT_CAN_BE_TYPED: usize = 300;

/// What a person has typed into *More apps*' search field.
///
/// **Graphemes and not bytes or chars**, which is the same rule
/// `AppId::first_letter` keeps and for the same reason: an accented letter and
/// a combined character are one thing a person sees, and erasing half of one
/// leaves a character they did not type and cannot name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WhatTheyTyped {
    /// Exactly what they typed, in order.
    typed: String,
}

impl WhatTheyTyped {
    /// Nothing typed, which is what the field holds when the surface opens.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// Add this character to the end.
    ///
    /// **A control character is not typed**, and a string already at
    /// [`THE_MOST_THAT_CAN_BE_TYPED`] takes no more. Says whether anything
    /// changed, so a caller can decide whether the frame is worth redrawing.
    pub fn typed(&mut self, letter: char) -> bool {
        if letter.is_control() || self.how_many_letters() >= THE_MOST_THAT_CAN_BE_TYPED {
            return false;
        }
        self.typed.push(letter);
        true
    }

    /// Take the last letter back.
    ///
    /// **One grapheme, not one `char`.** Erasing a `char` from `é` written as
    /// `e` plus a combining accent would leave the `e` — a letter the person
    /// did not type, which they would then have to erase again.
    ///
    /// Says whether anything was taken.
    pub fn erased(&mut self) -> bool {
        let Some(last) = self.typed.graphemes(true).next_back() else {
            return false;
        };
        let keep = self.typed.len().saturating_sub(last.len());
        self.typed.truncate(keep);
        true
    }

    /// Take everything back.
    ///
    /// Says whether there was anything to take.
    pub fn cleared(&mut self) -> bool {
        let had = !self.typed.is_empty();
        self.typed.clear();
        had
    }

    /// What they typed, to draw in the field.
    #[must_use]
    pub fn as_text(&self) -> &str {
        &self.typed
    }

    /// Whether they have typed anything.
    ///
    /// **Not the same as matching nothing.** An empty field shows every entry;
    /// a field with text in it that matches none shows a sentence saying so.
    /// The two look alike in a list of zero rows and are opposite states.
    #[must_use]
    pub fn is_nothing(&self) -> bool {
        self.typed.trim().is_empty()
    }

    /// How many letters a person would say they had typed.
    #[must_use]
    pub fn how_many_letters(&self) -> usize {
        self.typed.graphemes(true).count()
    }

    /// Whether this name is one of what they are looking for.
    ///
    /// **Anywhere in the name, not only at its start.** An application a person
    /// knows as *Firefox* is `org.mozilla.firefox` on this machine, and a
    /// search that only matched the beginning would find nothing for the name
    /// they actually know it by.
    ///
    /// **Case is folded both ways**, so `FIREFOX` and `firefox` are one search.
    /// `to_lowercase` is Unicode's, not ASCII's — a person searching in Greek
    /// or Turkish gets their own alphabet's answer rather than one that works
    /// for English and quietly fails for them.
    ///
    /// An empty search matches everything, which is what makes the unfiltered
    /// list and the filtered one the same code path.
    #[must_use]
    pub fn matches(&self, name: &str) -> bool {
        let looking_for = self.typed.trim().to_lowercase();
        if looking_for.is_empty() {
            return true;
        }
        name.to_lowercase().contains(&looking_for)
    }
}

/// Which of these names a person's typing leaves, **in the order they came
/// in**.
///
/// The order is the whole reason this is a function rather than a `filter` at
/// each call site. *More apps* draws pinned applications first in the person's
/// own order and open ones after in the order they opened; a filter that sorted
/// by how well a name matched would rearrange the grid under somebody's hand as
/// they typed, and the cell they were aiming at would move.
///
/// Answers the indices rather than the names, so a caller can take the rows
/// they belong to without this crate learning what a row is.
#[must_use]
pub fn which_are_left<'a>(
    names: impl IntoIterator<Item = &'a str>,
    typed: &WhatTheyTyped,
) -> Vec<usize> {
    names
        .into_iter()
        .enumerate()
        .filter_map(|(which, name)| typed.matches(name).then_some(which))
        .collect()
}

#[cfg(test)]
#[path = "searching_tests.rs"]
mod tests;
