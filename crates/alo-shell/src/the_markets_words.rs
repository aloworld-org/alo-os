//! Everything in the window of what a person can install that a person reads,
//! said once a frame.
//!
//! **The drawing path may not word a sentence.** `crate::the_docks_words` says
//! why at length and the rule is the same here: every sentence a person meets
//! comes from the crate that owns the subject, and a raster that looked one up
//! would be a second place deciding what a person reads.
//!
//! So this says them and hands the results over, exactly as the Dock's does.
//!
//! # One bundle rather than eleven fields
//!
//! A struct cannot be half-filled: [`TheMarketsWords::said`] builds all of them
//! or none, from one `Strings`, in one place. Eleven borrowed `&str` fields on
//! a frame would be eleven chances to set ten of them, and the one forgotten is
//! a blank where a word should be — which draws cleanly and says nothing.
//!
//! # What is **not** here, and why
//!
//! An application's **name**, **version** and **licence** are not words. They
//! are what the catalogue holds, written the way its source writes them, and
//! nothing translates `org.gimp.GIMP` or `GPL-3.0+`. They reach the drawing
//! from `alo_software::Listed` directly. `crate::the_docks_words` carries the
//! person's own typed text for the same reason and says so in the same place.

use alo_software::{Category, words};
use alo_strings::{Filling, Strings};

/// One word, said. `Strings::say` takes a key and a filling; none of these
/// has a gap in it.
fn say(strings: &Strings, word: &alo_strings::Word) -> String {
    strings.say(&word.key(), &Filling::nothing()).into_text()
}

/// What the window of what a person can install says, already said.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheMarketsWords {
    /// The window's name, in its title bar.
    pub heading: String,
    /// The line under the list, which is the promise the list keeps.
    pub promise: String,
    /// What the control on an application not yet here says.
    pub install: String,
    /// What it says while the machine is fetching one.
    pub installing: String,
    /// What it says once an application is on the machine.
    pub open: String,
    /// Every kind of work a person can narrow by, in the order the window
    /// shows them — the widest first.
    ///
    /// **Nine, not eight.** `Category` has eight; the first of these is
    /// *everything*, which narrows nothing and is not a `Category` at all. A
    /// caller walking `Category::EVERY` beside this must skip the first or it
    /// draws every label one place to the left, which looks like a layout
    /// fault and is a counting one.
    pub kinds_of_work: Vec<String>,
}

impl TheMarketsWords {
    /// Say all of them, from this machine's vocabulary.
    #[must_use]
    pub fn said(strings: &Strings) -> Self {
        Self {
            heading: say(strings, &words::THE_APPLICATIONS),
            promise: say(strings, &words::EACH_ARRIVES_WITH_NOTHING),
            install: say(strings, &words::INSTALL_IT),
            installing: say(strings, &words::INSTALLING_IT),
            open: say(strings, &words::OPEN_IT),
            kinds_of_work: words::EVERY_CATEGORY
                .iter()
                .map(|word| say(strings, word))
                .collect(),
        }
    }

    /// What one kind of work is called, or the widest view when it is [`None`].
    ///
    /// Returns an empty string only for a vocabulary that could not say a word
    /// it declares, which `alo-strings` does not do — never for a category this
    /// window does not know, because [`Category`] is closed.
    #[must_use]
    pub fn the(&self, kind: Option<Category>) -> &str {
        // **`Category::EVERY`'s own order, asked of it**, rather than
        // `Category::place`, which is `alo-software`'s own and not this
        // crate's to reach. One place along, because *everything* is first
        // here and is not a `Category` at all.
        let at = match kind {
            None => 0,
            Some(category) => match Category::EVERY.iter().position(|one| *one == category) {
                Some(place) => place + 1,
                None => return "",
            },
        };
        self.kinds_of_work.get(at).map_or("", String::as_str)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn said() -> TheMarketsWords {
        TheMarketsWords::said(&Strings::of(
            alo_saying::everything_this_machine_can_say().unwrap(),
        ))
    }

    #[test]
    fn every_word_this_window_needs_is_said() {
        let words = said();
        assert!(!words.heading.is_empty());
        assert!(!words.promise.is_empty());
        assert!(!words.install.is_empty());
        assert!(!words.installing.is_empty());
        assert!(!words.open.is_empty());
        assert_eq!(words.kinds_of_work.len(), 9);
        for one in &words.kinds_of_work {
            assert!(!one.is_empty());
        }
    }

    /// **The widest view is first and the eight follow it in their own order**,
    /// so a caller that walked `Category::EVERY` beside this list would be one
    /// place out — which is what `the` exists to prevent.
    #[test]
    fn the_widest_view_is_first_and_each_kind_answers_for_itself() {
        let words = said();
        let everything = words.the(None);
        assert!(!everything.is_empty());
        let mut seen = vec![everything.to_owned()];
        for kind in Category::EVERY {
            let name = words.the(Some(kind));
            assert!(!name.is_empty(), "{}", kind.named());
            assert!(
                !seen.contains(&name.to_owned()),
                "{} shares a name with something before it",
                kind.named()
            );
            seen.push(name.to_owned());
        }
        assert_eq!(seen.len(), 9);
    }

    /// **No identifier, version or licence is in here.** Those are the
    /// catalogue's data, written as its source writes them, and a vocabulary
    /// that carried one would be translating something nobody may translate.
    #[test]
    fn nothing_here_is_an_applications_own_data() {
        let words = said();
        let everything = [
            &words.heading,
            &words.promise,
            &words.install,
            &words.installing,
            &words.open,
        ]
        .into_iter()
        .chain(words.kinds_of_work.iter());
        let mut read = 0;
        for text in everything {
            for forbidden in ["org.", "GPL", "MPL", "flathub", "Flathub"] {
                assert!(!text.contains(forbidden), "{text:?} contains {forbidden}");
            }
            read += 1;
        }
        assert_eq!(read, 14, "the loop read every word");
    }
}
