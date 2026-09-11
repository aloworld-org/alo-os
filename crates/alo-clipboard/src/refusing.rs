//! The five ways a paste does not happen, each with a sentence.
//!
//! A person presses paste and nothing appears. On every desktop anybody has
//! used that is where it ends: nothing appears, nothing is said, and the person
//! is left to work out whether they copied the wrong thing, closed the wrong
//! window, or own a broken computer.
//!
//! So there is no silent failure anywhere in this crate. Every way a transfer
//! can fail to happen is one of these, and every one of them has a string in
//! [`crate::words`] — answered through [`NotPasted::said`], with no `Display`
//! that would put English on a screen by accident.
//!
//! # Why five and not one
//!
//! Because the repair is different in each, and a sentence that covered two of
//! them would send a person to fix the wrong thing. *Nothing has been copied* is
//! copy something; *something else has been copied since* is copy it again;
//! *the application has given it up* is the window you closed; *not in that
//! form* is nothing the person can do at all and is the one that must not read
//! like their mistake; *it did not hand it over* is try again.
//!
//! # What none of them says
//!
//! What was copied. See [`crate::words`]: a sentence quoting the clipboard puts
//! whatever a person last moved out of a password manager into a notification.

use alo_strings::{Filling, Said, Strings};

use crate::giving::CouldNotGive;
use crate::kind::Kind;
use crate::words::{self, AS_WHAT, Word};

/// Why nothing was pasted.
///
/// `non_exhaustive` because a wiring that can tell two of these apart more
/// finely is a variant added additively — `docs/contracts/` is what that rule
/// is for, and a clipboard is a public surface an adapter is written against.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NotPasted {
    /// Nothing has been copied on this machine since it started, so there is
    /// nothing to paste from.
    ///
    /// Deliberately distinct from [`NotPasted::NoLongerOffered`]: *nothing was
    /// ever copied* and *what you copied is gone* are two different facts, and
    /// the acceptance for this task names the difference — pasting when nothing
    /// has been copied must be *nothing to copy from* rather than the thing
    /// before.
    NothingCopied,

    /// The selection was replaced: somebody copied something else, and what
    /// this handle names is not what is on the clipboard now.
    CopiedOver,

    /// The owner withdrew the selection, or stopped. Nothing has replaced it.
    NoLongerOffered,

    /// The owner never offered that form.
    ///
    /// Carries the form asked for, because the sentence names it — and because
    /// a paster shows a person what they *can* paste as next, which it reads
    /// off the offer rather than off this.
    NotThatForm {
        /// What was asked for.
        form: Kind,
    },

    /// The owner was asked and did not hand it over.
    CouldNotGive(CouldNotGive),
}

impl NotPasted {
    /// The string this crate declares for it: the key a translator's file is
    /// sorted by, and the English beside it.
    #[must_use]
    pub fn word(&self) -> Word {
        match self {
            Self::NothingCopied => words::NOTHING_COPIED,
            Self::CopiedOver => words::COPIED_OVER,
            Self::NoLongerOffered => words::NO_LONGER_OFFERED,
            Self::NotThatForm { .. } => words::NOT_THAT_FORM,
            Self::CouldNotGive(_) => words::COULD_NOT_GIVE,
        }
    }

    /// What to tell the person, in the language they read.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not:
    /// there is always something to put in front of the person, and where it
    /// came from is on the [`Said`]. A `Strings` that was never given
    /// [`crate::clipboard_words`] answers with the key, marked `Said::is_a_bug`
    /// — the honest answer to *the shell forgot to declare what this crate can
    /// say*.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::NotThatForm { form } => Filling::of(AS_WHAT, form.shown(strings)),
            _ => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Every way a paste does not happen, so no test below quietly skips one.
    fn every_refusal() -> Vec<NotPasted> {
        vec![
            NotPasted::NothingCopied,
            NotPasted::CopiedOver,
            NotPasted::NoLongerOffered,
            NotPasted::NotThatForm { form: Kind::text() },
            NotPasted::CouldNotGive(CouldNotGive::TheApplicationDidNot),
        ]
    }

    /// **Every refusal says something a person could read**, and no two of them
    /// read the same — a person told one sentence for *nothing was copied* and
    /// for *what you copied is gone* would go and fix the wrong one.
    #[test]
    fn every_refusal_reads_and_no_two_read_the_same() {
        let strings = in_english();
        let mut seen: Vec<String> = Vec::new();
        for refusal in every_refusal() {
            let said = refusal.said(&strings);
            assert!(!said.text().is_empty(), "{refusal:?} says nothing");
            assert!(!said.is_a_bug(), "{refusal:?} is not declared");
            assert!(
                said.unfilled().is_empty(),
                "{refusal:?} has a gap left in it"
            );
            assert!(
                !seen.contains(&said.text().to_owned()),
                "two refusals both say {said}"
            );
            seen.push(said.text().to_owned());
        }
    }

    /// **The form asked for is named in the sentence**, in the words a person
    /// uses for it rather than as a media type — and a form with no name of
    /// ours is named as itself, which is the thing somebody can search for.
    #[test]
    fn the_form_asked_for_is_named_in_the_persons_own_words() {
        let strings = in_english();
        for (form, reads) in [
            (Kind::text(), "as text:"),
            (Kind::image_png(), "as an image:"),
            (Kind::files(), "as files:"),
            (
                Kind::named("application/pdf").unwrap(),
                "as application/pdf:",
            ),
        ] {
            let said = NotPasted::NotThatForm { form }.said(&strings);
            assert!(said.text().contains(reads), "{said}");
        }
    }

    /// **The refusal arrives in the language the person reads** when somebody
    /// has translated it, and the form inside it is translated too — a sentence
    /// in German with an English word in the middle is the failure a gap that
    /// carries a translated value exists to prevent.
    #[test]
    fn a_refusal_and_the_form_in_it_are_read_in_the_persons_language() {
        let strings = translated(&[
            (
                words::NOT_THAT_FORM,
                "Das kann nicht als {as_what} eingefügt werden: die Anwendung, aus der es kopiert \
                 wurde, hat es in dieser Form nicht angeboten",
            ),
            (words::THE_FORM_AN_IMAGE, "ein Bild"),
        ]);

        let said = NotPasted::NotThatForm {
            form: Kind::image_png(),
        }
        .said(&strings);
        assert!(said.is_translated());
        assert!(said.text().contains("als ein Bild"), "{said}");

        // The one nobody translated is still English, and says it is.
        let untranslated = NotPasted::NothingCopied.said(&strings);
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug());
    }

    /// **Each refusal is the string this crate declared for it**, so a sentence
    /// cannot drift from the key a translator's file is sorted by.
    #[test]
    fn every_refusal_says_the_word_this_crate_declared() {
        let strings = in_english();
        for refusal in every_refusal() {
            if matches!(refusal, NotPasted::NotThatForm { .. }) {
                continue; // It has a gap, so what it says is longer than the word.
            }
            assert_eq!(refusal.said(&strings).text(), refusal.word().says());
        }
    }
}
