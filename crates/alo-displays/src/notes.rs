//! What a person is told about their screens, and nothing more.
//!
//! A note is not a refusal: everything here happened, and the arrangement is in
//! force. It is the sentence that turns *why is my second screen suddenly on
//! the right* into an answer — and, for the two that are about night light,
//! *why did my screens not go warm this evening*, which is a question only
//! somebody inside a polar circle ever asks and which nothing else on their
//! machine would answer.
//!
//! **There is no note for the ordinary morning.** Plugging the same screens
//! into the same machine and getting the same arrangement says nothing at all —
//! [`Note::AsYouLeftThem`] exists because a surface that lists what happened
//! needs the line, not because anything is shown. A system that announces every
//! screen every time is a system whose announcements nobody reads, including the
//! one that mattered.

use alo_strings::{Filling, Said, Strings, Word};

use crate::identity::Identity;
use crate::scale::Rounded;
use crate::words;

/// One thing worth telling a person about how their screens were set up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Note {
    /// These screens were laid out the way they were last left.
    AsYouLeftThem,
    /// This screen has never been used with this machine, so it was put beside
    /// the others at a size worked out from how large it is.
    NewHere(Identity),
    /// This screen says nothing about itself, so it is remembered by the socket
    /// it is plugged into — which is weaker, and the person is told so.
    RememberedByItsSocket(Identity),
    /// Two screens say exactly the same thing about themselves, so they are
    /// told apart by which socket each is in.
    ToldApartByTheirSockets,
    /// The arrangement these screens were left in no longer fits them, so they
    /// were laid out side by side again.
    DidNotFit,
    /// Night light follows the sun, and the sun does not set here today, so the
    /// screens are not being warmed.
    TheSunDoesNotSet,
    /// Night light follows the sun, and the sun does not rise here today, so
    /// the screens are being warmed all day.
    TheSunDoesNotRise,
    /// This screen cannot draw at the size it was given, so the nearest size it
    /// can draw was used.
    SizeRounded {
        /// Which screen.
        display: Identity,
        /// What was asked for, and what was drawn.
        rounded: Rounded,
    },
}

impl Note {
    /// The string this crate declares for this note.
    #[must_use]
    pub const fn word(&self) -> Word {
        match *self {
            Self::AsYouLeftThem => words::AS_YOU_LEFT_THEM,
            Self::NewHere(_) => words::NEW_HERE,
            Self::RememberedByItsSocket(_) => words::REMEMBERED_BY_ITS_SOCKET,
            Self::ToldApartByTheirSockets => words::TOLD_APART_BY_THEIR_SOCKETS,
            Self::DidNotFit => words::DID_NOT_FIT,
            Self::TheSunDoesNotSet => words::THE_SUN_DOES_NOT_SET,
            Self::TheSunDoesNotRise => words::THE_SUN_DOES_NOT_RISE,
            Self::SizeRounded { .. } => words::SIZE_ROUNDED,
        }
    }

    /// Which screen this is about, where it is about one.
    #[must_use]
    pub const fn about(&self) -> Option<&Identity> {
        match self {
            Self::NewHere(display)
            | Self::RememberedByItsSocket(display)
            | Self::SizeRounded { display, .. } => Some(display),
            Self::AsYouLeftThem
            | Self::ToldApartByTheirSockets
            | Self::DidNotFit
            | Self::TheSunDoesNotSet
            | Self::TheSunDoesNotRise => None,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::NewHere(display) | Self::RememberedByItsSocket(display) => {
                display.named_in("display", Filling::nothing(), strings)
            }
            Self::SizeRounded { display, rounded } => display
                .named_in("display", Filling::nothing(), strings)
                .and("asked", format!("{}%", rounded.asked().as_per_cent()))
                .and("used", format!("{}%", rounded.used().as_per_cent())),
            Self::AsYouLeftThem
            | Self::ToldApartByTheirSockets
            | Self::DidNotFit
            | Self::TheSunDoesNotSet
            | Self::TheSunDoesNotRise => Filling::nothing(),
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
    use crate::scale::{Scale, Support};
    use crate::testing::{in_english, the_laptop, the_office_screen};

    /// Every note this crate makes says a sentence that is declared, with every
    /// gap filled and nothing left as a key.
    #[test]
    fn every_note_says_a_declared_sentence_with_every_gap_filled() {
        let strings = in_english();
        let rounded =
            Rounded::of(Scale::per_cent(150).unwrap(), Support::WholeMultiplesOnly).unwrap();
        for note in [
            Note::AsYouLeftThem,
            Note::NewHere(the_office_screen()),
            Note::RememberedByItsSocket(the_laptop()),
            Note::ToldApartByTheirSockets,
            Note::DidNotFit,
            Note::TheSunDoesNotSet,
            Note::TheSunDoesNotRise,
            Note::SizeRounded {
                display: the_office_screen(),
                rounded,
            },
        ] {
            let said = note.said(&strings);
            assert!(words::EVERY_WORD.contains(&note.word()), "{note:?}");
            assert!(said.unfilled().is_empty(), "{note:?}: {said}");
            assert!(!said.is_a_bug(), "{note:?}: {said}");
        }
    }

    /// **A note about one screen names it**, in what a person reads rather than
    /// in anything the machine calls it.
    #[test]
    fn a_note_about_one_screen_names_it_as_a_person_reads_it() {
        let strings = in_english();
        let note = Note::NewHere(the_office_screen());
        assert_eq!(note.about(), Some(&the_office_screen()));
        assert!(
            note.said(&strings).text().contains("Dell U2720Q"),
            "{note:?}"
        );

        let laptop = Note::RememberedByItsSocket(the_laptop());
        let said = laptop.said(&strings);
        assert!(said.text().contains("Built-in screen"), "{said}");
        assert!(!said.text().contains("eDP-1"), "{said}");
        assert_eq!(Note::DidNotFit.about(), None);
    }

    /// **A size that was rounded names both sizes**, because *it is not what
    /// you chose* is only useful beside what it is instead.
    #[test]
    fn a_rounded_size_names_what_was_asked_and_what_was_drawn() {
        let strings = in_english();
        let rounded =
            Rounded::of(Scale::per_cent(150).unwrap(), Support::WholeMultiplesOnly).unwrap();
        let said = Note::SizeRounded {
            display: the_office_screen(),
            rounded,
        }
        .said(&strings);
        assert!(said.text().contains("150%"), "{said}");
        assert!(said.text().contains("200%"), "{said}");
    }
}
