//! Why an answer to setup went nowhere, in the words the person reads.
//!
//! Five, and every one of them ends by saying **setup is still waiting** — or,
//! for the one that is about setup already being over, where to go instead. A
//! person who answered a question and was told nothing would reasonably
//! conclude they had answered it.
//!
//! # The sixth is somebody else's sentence
//!
//! [`NotSetUp::NotWritten`] is `alo_choosing::NotWritten`, carried rather than
//! reworded: the file is the person's, the refusals about it are
//! `alo-choosing`'s, and they already end *nothing in your settings has been
//! changed*. A second sentence here would be two accounts of one moment, and
//! the one this crate wrote would be the one nobody kept up to date. It is the
//! only one of the six that can name a path, because it is the only one about a
//! file.
//!
//! # None of them says what to answer instead
//!
//! A refusal that recommended one of the remaining choices would be ADR 0009's
//! *no persuasion attached* broken at the one moment a person is most likely to
//! take the advice — they have just been told their answer did not work.
//! `crate::nudging` walks these sentences with the rest.

use alo_choosing::NotWritten;
use alo_strings::{Filling, Said, Strings};

use crate::offered::Offered;
use crate::words;

/// Why setup was not answered.
///
/// Deliberately no `Display`, which is `alo_choosing::NotSet`'s rule: a
/// `Display` is one `to_string()` from a screen whose author had no reason to
/// think about language, and every one of these is read by the person sitting
/// in front of setup. The only road to words is [`said`](NotSetUp::said).
#[derive(Debug)]
pub enum NotSetUp {
    /// Answered with nothing selected.
    ///
    /// Not a mistake anybody made: nothing is selected because alo OS selects
    /// nothing (ADR 0025), so pressing on without choosing is the ordinary way
    /// to arrive here and the machine simply keeps asking.
    NothingSelected,

    /// Answered with the material for a choice other than the selected one.
    ///
    /// A defect in whatever is drawing setup rather than anything the person
    /// did — a stale selection, or two things clicked at once. It is refused
    /// rather than resolved, because resolving it would be alo OS deciding
    /// which of the two the person meant.
    AnotherChoice {
        /// Which of the four was selected.
        selected: Offered,
        /// Which of the four was answered with.
        answered: Offered,
    },

    /// A machine on this network was answered, and this machine has paired with
    /// none.
    ///
    /// ADR 0003: a pairing is a deliberate act on both machines, and nothing in
    /// this repository makes one yet. A true sentence about the machine rather
    /// than a fault in it.
    NoPairedMachine,

    /// A provider was answered with no model to ask it for.
    ///
    /// Held apart from [`Self::NothingSelected`] because they send a person to
    /// two different places: one of them has not picked one of the four, and
    /// the other has picked a provider and left the box beside it empty.
    NothingToAskFor,

    /// Setup has already been answered on this machine.
    ///
    /// It is asked once (ADR 0025). Everything it decides can be changed
    /// afterwards in Settings, and the sentence says so — a person who arrived
    /// here wants the thing they were going to change, not an explanation.
    AlreadyAnswered,

    /// The person's own settings would not take the answer.
    ///
    /// `alo_choosing::NotWritten`'s own refusal, carried whole. Setup is **not**
    /// recorded as answered in any of its cases: the file is exactly as it was,
    /// and the question is still waiting.
    NotWritten(NotWritten),
}

impl NotSetUp {
    /// The string said for this reason.
    ///
    /// [`None`] for the one that is `alo-choosing`'s own, because its sentence
    /// takes gaps this crate does not fill — [`said`](Self::said) is the door
    /// that answers for all six.
    #[must_use]
    pub const fn word(&self) -> Option<words::Word> {
        match self {
            Self::NothingSelected => Some(words::NOTHING_SELECTED),
            Self::AnotherChoice { .. } => Some(words::ANOTHER_CHOICE),
            Self::NoPairedMachine => Some(words::NO_PAIRED_MACHINE),
            Self::NothingToAskFor => Some(words::NOTHING_TO_ASK_FOR),
            Self::AlreadyAnswered => Some(words::ALREADY_ANSWERED),
            Self::NotWritten(_) => None,
        }
    }

    /// What the person is told, in the language they read.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::setting_up_words`] answers with the key, marked, and setup is
    /// unanswered either way.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match (self, self.word()) {
            // The file's own refusal already has a sentence for this person,
            // filled from the file rather than from here.
            (Self::NotWritten(why), _) => why.said(strings),
            (_, Some(word)) => strings.say(&word.key(), &Filling::nothing()),
            // Unreachable while every variant above names a word, and written
            // as an answer rather than a panic: a surface that could not tell
            // somebody why their answer failed must still not take the machine
            // down in front of them.
            (_, None) => strings.say(&words::NOTHING_SELECTED.key(), &Filling::nothing()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use std::path::PathBuf;

    /// Every way setup is not answered, so the tests below walk all of them
    /// rather than the two somebody remembered.
    fn every_reason() -> Vec<NotSetUp> {
        vec![
            NotSetUp::NothingSelected,
            NotSetUp::AnotherChoice {
                selected: Offered::OnThisMachine,
                answered: Offered::NotAtAll,
            },
            NotSetUp::NoPairedMachine,
            NotSetUp::NothingToAskFor,
            NotSetUp::AlreadyAnswered,
            NotSetUp::NotWritten(NotWritten::NotKept {
                at: PathBuf::from("/home/ada/.config/alo/settings.toml"),
                why: "permission denied".to_owned(),
            }),
        ]
    }

    /// **Every reason reaches the person as a sentence**, and none of them
    /// reaches them as a key.
    #[test]
    fn every_reason_is_a_sentence_somebody_can_read() {
        let strings = in_english();
        for reason in every_reason() {
            let said = reason.said(&strings);
            assert!(!said.is_a_bug(), "{reason:?}: {said}");
        }
    }

    /// **Six reasons, six sentences.** A machine that said the same thing about
    /// a disk that would not take the file and a person who selected nothing
    /// would be sending somebody to the wrong place.
    #[test]
    fn no_two_reasons_share_a_sentence() {
        let strings = in_english();
        let reasons = every_reason();
        let mut said: Vec<String> = reasons
            .iter()
            .map(|reason| reason.said(&strings).into_text())
            .collect();
        said.sort();
        said.dedup();
        assert_eq!(said.len(), reasons.len());
        assert_eq!(said.len(), 6);
    }

    /// **The five this crate words itself say setup is still waiting**, which
    /// is what the person acts on: they have chosen nothing, and they are still
    /// being asked. The sixth is `alo-choosing`'s own and says the file did not
    /// move, which is the same fact about a different thing.
    #[test]
    fn the_reasons_this_crate_words_itself_say_the_question_is_still_there() {
        let strings = in_english();
        for reason in every_reason() {
            let said = reason.said(&strings);
            match reason {
                NotSetUp::AlreadyAnswered => {
                    assert!(said.text().contains("already been answered"), "{said}");
                }
                NotSetUp::NotWritten(_) => {
                    assert!(
                        said.text()
                            .contains("nothing in your settings has been changed"),
                        "{said}"
                    );
                }
                _ => assert!(said.text().contains("setup is still waiting"), "{said}"),
            }
        }
    }

    /// **Only the one about a file names a file.** The other five are about the
    /// moment rather than about the disk, and a path in them would be a person
    /// sent to open something that is not the matter.
    #[test]
    fn only_the_refusal_about_the_file_names_a_file() {
        let strings = in_english();
        for reason in every_reason() {
            let said = reason.said(&strings);
            assert_eq!(
                said.text().contains("/home/ada/.config/alo/settings.toml"),
                matches!(reason, NotSetUp::NotWritten(_)),
                "{said}"
            );
        }
    }

    /// And every one of them is read in the language the machine is showing.
    #[test]
    fn a_refusal_is_read_in_the_language_the_machine_is_showing() {
        let strings = translated(&[(
            words::NO_PAIRED_MACHINE,
            "dieser Computer wurde mit keinem anderen verbunden",
        )]);
        let said = NotSetUp::NoPairedMachine.said(&strings);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().contains("verbunden"), "{said}");
    }
}
