//! What this machine can do with a file: three answers, and no fourth.
//!
//! [`Outcome::OpensAsItIs`], [`Outcome::Converts`] and [`Outcome::CannotOpen`].
//! **There is no variant meaning *probably*.** A maybe in a type is a spinner
//! somebody has to render, and whatever the shell drew for it would be a
//! promise the machine had not decided to keep. Every file this crate is handed
//! lands in one of the three, and each carries the sentence a person reads.
//!
//! # What converting costs, said before it happens
//!
//! [`Costs`] is what is known *before* a conversion: that what opens is a copy
//! and the original is left exactly as it was, that what the copy could not
//! carry will be said once it exists, and whether the document carries macros
//! that will be left behind. What a particular conversion actually lost — a
//! font, a field — cannot be known until it has run, and saying it is task 2's.

use alo_strings::{Filling, Said, Strings};

use crate::appears::{Appears, Container, Macros};
use crate::kind::Kind;
use crate::words::{self, Word};

/// What this machine can do with a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// It opens the file as it is.
    OpensAsItIs {
        /// What the file is.
        kind: Kind,
        /// Whether it carries macros, which never run.
        macros: Macros,
    },
    /// It opens a copy converted into a kind it opens.
    Converts {
        /// What the file is.
        from: Kind,
        /// What the copy is converted into.
        into: Kind,
        /// What converting costs, as far as it is known before converting.
        costs: Costs,
    },
    /// It cannot open the file.
    CannotOpen(Cannot),
}

/// What converting costs, as far as it is known before a conversion runs.
///
/// Two costs are true of every conversion and are said every time: **the copy
/// is what opens, and the original is untouched**, and **what the copy could not
/// carry is said once it has been converted**. The one that depends on the
/// file is here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Costs {
    /// Whether the document carries macros, which are not carried into the
    /// copy.
    macros: Macros,
}

impl Costs {
    /// What converting a file with these macros costs.
    #[must_use]
    pub const fn of(macros: Macros) -> Self {
        Self { macros }
    }

    /// Whether macros are left behind.
    #[must_use]
    pub const fn macros(self) -> Macros {
        self.macros
    }
}

/// Why this machine cannot open a file — each one a different place to send a
/// person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cannot {
    /// There is nothing in it.
    Empty,
    /// Nothing about its contents is recognised.
    Unrecognised,
    /// It is recognisably something and does not hold together.
    Damaged(Container),
    /// It is a program, and opening never runs one.
    AProgram,
    /// It is an Office document locked with a password.
    PasswordProtected,
    /// It is recognised, and nothing on this machine opens or converts it.
    NothingHereOpens(Kind),
}

impl Outcome {
    /// What the file's bytes said, which every outcome was reached from.
    #[must_use]
    pub const fn appears(self) -> Appears {
        match self {
            Self::OpensAsItIs { kind, .. } | Self::Converts { from: kind, .. } => Appears::A(kind),
            Self::CannotOpen(Cannot::Empty) => Appears::Nothing,
            Self::CannotOpen(Cannot::Unrecognised) => Appears::Unrecognised,
            Self::CannotOpen(Cannot::Damaged(container)) => Appears::Damaged(container),
            Self::CannotOpen(Cannot::AProgram) => Appears::AProgram,
            Self::CannotOpen(Cannot::PasswordProtected) => Appears::APasswordProtectedDocument,
            Self::CannotOpen(Cannot::NothingHereOpens(kind)) => Appears::A(kind),
        }
    }

    /// The sentences a person reads about this outcome, in order: what the
    /// machine can do, then what the macros in it will not do when there are
    /// any.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Vec<Said> {
        match self {
            Self::OpensAsItIs { kind, macros } => {
                let mut said = vec![strings.say(
                    &words::OPENS_AS_IT_IS.key(),
                    &Filling::nothing().and_said(words::WHAT, &kind.said(strings)),
                )];
                if macros == Macros::Inside {
                    said.push(plainly(strings, words::MACROS_WILL_NOT_RUN));
                }
                said
            }
            Self::Converts { from, into, costs } => {
                let mut said = vec![
                    strings.say(
                        &words::CONVERTS_A_COPY.key(),
                        &Filling::nothing()
                            .and_said(words::WHAT, &from.said(strings))
                            .and_said(words::INTO, &into.said(strings)),
                    ),
                ];
                if costs.macros() == Macros::Inside {
                    said.push(plainly(strings, words::MACROS_WILL_NOT_BE_CARRIED));
                }
                said
            }
            Self::CannotOpen(cannot) => vec![cannot.said(strings)],
        }
    }
}

impl Cannot {
    /// The sentence a person reads about why.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        match self {
            Self::Empty => plainly(strings, words::CANNOT_EMPTY),
            Self::Unrecognised => plainly(strings, words::CANNOT_UNRECOGNISED),
            Self::AProgram => plainly(strings, words::CANNOT_A_PROGRAM),
            Self::PasswordProtected => plainly(strings, words::CANNOT_PASSWORD_PROTECTED),
            Self::Damaged(container) => strings.say(
                &words::CANNOT_DAMAGED.key(),
                &Filling::nothing().and_said(
                    words::WHAT,
                    &strings.say(&container.word().key(), &Filling::nothing()),
                ),
            ),
            Self::NothingHereOpens(kind) => strings.say(
                &words::CANNOT_NOTHING_HERE_OPENS_IT.key(),
                &Filling::nothing().and_said(words::WHAT, &kind.said(strings)),
            ),
        }
    }
}

/// A sentence with no gaps.
fn plainly(strings: &Strings, word: Word) -> Said {
    strings.say(&word.key(), &Filling::nothing())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// The texts of some sentences.
    fn texts(said: &[Said]) -> Vec<&str> {
        said.iter().map(Said::text).collect()
    }

    /// **Each outcome says what the machine can do**, with the name of what
    /// the file is in it and nothing else.
    #[test]
    fn each_outcome_says_what_the_machine_can_do() {
        let strings = in_english();
        assert_eq!(
            texts(
                &Outcome::OpensAsItIs {
                    kind: Kind::Pdf,
                    macros: Macros::NoneSeen
                }
                .said(&strings)
            ),
            ["This is a PDF document, and this machine opens it as it is"]
        );
        assert_eq!(
            texts(
                &Outcome::Converts {
                    from: Kind::WordDocument,
                    into: Kind::Pdf,
                    costs: Costs::of(Macros::Inside)
                }
                .said(&strings)
            ),
            [
                "This is a Word document, and this machine opens it by converting a copy into a \
                 PDF document. The original stays exactly as it was, and what the copy could not \
                 carry over is said once it has been converted",
                "It contains macros: they will not run, and they will not be carried into the copy"
            ]
        );
        assert_eq!(
            texts(
                &Outcome::OpensAsItIs {
                    kind: Kind::ExcelWorkbook,
                    macros: Macros::Inside
                }
                .said(&strings)
            ),
            [
                "This is an Excel spreadsheet, and this machine opens it as it is",
                "It contains macros, and they will not run"
            ]
        );
    }

    /// **Every reason a file cannot be opened is its own sentence**, so no two
    /// reasons can send a person to the same place by reading the same.
    #[test]
    fn every_reason_is_its_own_sentence() {
        let strings = in_english();
        let reasons = [
            Cannot::Empty,
            Cannot::Unrecognised,
            Cannot::Damaged(Container::Pdf),
            Cannot::Damaged(Container::Compressed),
            Cannot::Damaged(Container::OlderOffice),
            Cannot::AProgram,
            Cannot::PasswordProtected,
            Cannot::NothingHereOpens(Kind::GifImage),
        ];
        let mut said: Vec<String> = reasons
            .iter()
            .map(|reason| reason.said(&strings).text().to_owned())
            .collect();
        assert_eq!(
            said.first().map(String::as_str),
            Some("This file is empty: there is nothing in it to open")
        );
        assert!(said.contains(
            &"This file is damaged: it is a compressed file, but it has been cut short or does \
              not hold together inside, so it cannot be opened as it is"
                .to_owned()
        ));
        assert!(
            said.contains(
                &"This is a GIF image, and nothing on this machine opens it or converts it into \
              something that does"
                    .to_owned()
            )
        );
        said.sort();
        said.dedup();
        assert_eq!(said.len(), reasons.len());
    }

    /// **Every outcome is reached from what the bytes said, and says so.**
    #[test]
    fn every_outcome_knows_what_it_was_reached_from() {
        assert_eq!(
            Outcome::Converts {
                from: Kind::OlderWordDocument,
                into: Kind::Pdf,
                costs: Costs::of(Macros::NoneSeen)
            }
            .appears(),
            Appears::A(Kind::OlderWordDocument)
        );
        assert_eq!(
            Outcome::CannotOpen(Cannot::PasswordProtected).appears(),
            Appears::APasswordProtectedDocument
        );
        assert_eq!(
            Outcome::CannotOpen(Cannot::NothingHereOpens(Kind::WebpImage)).appears(),
            Appears::A(Kind::WebpImage)
        );
    }

    /// **A translated outcome is translated all the way through**, including
    /// the name of the kind put inside it — and one left in English says so.
    #[test]
    fn a_translated_outcome_is_translated_all_the_way_through() {
        let whole = translated(&[
            (
                words::OPENS_AS_IT_IS,
                "Das ist {what}, und dieser Rechner öffnet es so, wie es ist",
            ),
            (words::KIND_PDF, "ein PDF-Dokument"),
        ]);
        let said = Outcome::OpensAsItIs {
            kind: Kind::Pdf,
            macros: Macros::NoneSeen,
        }
        .said(&whole);
        let first = said.first().map(|said| (said.text(), said.is_translated()));
        assert_eq!(
            first,
            Some((
                "Das ist ein PDF-Dokument, und dieser Rechner öffnet es so, wie es ist",
                true
            ))
        );

        let half = translated(&[(
            words::OPENS_AS_IT_IS,
            "Das ist {what}, und dieser Rechner öffnet es so, wie es ist",
        )]);
        let said = Outcome::OpensAsItIs {
            kind: Kind::Pdf,
            macros: Macros::NoneSeen,
        }
        .said(&half);
        assert!(said.first().is_some_and(|said| !said.is_translated()));
    }
}
