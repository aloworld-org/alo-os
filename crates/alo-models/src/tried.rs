//! What a person is told when a provider has been tested.
//!
//! Both shapes of one answer, in one file, because they are read by one person
//! in one dialogue: [`Tried`] when the provider answered and the key was
//! accepted, [`NotTried`] when it was not — including when this machine's own
//! policy would not let the attempt happen at all.
//!
//! **The model names came from somewhere else.** A provider writes them, and
//! they land in a settings panel next to things the system said itself. So they
//! are held to the same rule as a filename in `alo-files`: a name that cannot
//! be shown is counted and left out rather than shown, and a list longer than
//! anybody will read is cut and says it was cut. A bounded answer that does not
//! say so reads exactly like a complete one.
//!
//! **No sentence here counts anything out loud**, and that is deliberate rather
//! than terse: *one model* and *two models* is one sentence in English and
//! three in Polish, and the plural rules are item 9a in `docs/autonomy/QUEUE.md`
//! and are not written from memory. The numbers are here as numbers, for
//! whatever shows them — and item 9a is built now, so whoever shows them counts
//! them with `alo_strings::Strings::count` in the reader's own language.
//!
//! **The answer is lines rather than a sentence with clauses appended.** It was
//! one string with `" — "` between its parts, and that separator is punctuation
//! a program picked: `alo-shortcuts` settled in item 9c that a sentence never
//! joins a list, because the separator is not the same everywhere and the
//! joining word would be placed by a machine that does not know the sentence.
//! So [`Tried::said`] is the line that answers the question, and
//! [`Tried::caveats`] is nought, one or two lines to be drawn beneath it.

use alo_strings::{Filling, Said, Strings};

use crate::refusing::NotAllowed;
use crate::words;

/// The most names kept from one answer. A provider offering more than this is
/// offering a list nobody reads in a settings dialogue, and the cut is
/// reported rather than silent.
const MOST_NAMES: usize = 200;

/// The longest a model name may be, in characters, and still be shown. Longer
/// than any real one, short enough that a line in a settings panel stays a
/// line. Counted in characters rather than bytes so that a name in Greek is
/// held to the same length as one in English.
const LONGEST_NAME: usize = 100;

/// A provider answered, and the key it was given was accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tried {
    /// The names it offers, as it spells them, in the order it gave them.
    models: Vec<String>,
    /// How many names were left out because they could not be shown.
    unshowable: usize,
    /// Whether every name it offered is in the list.
    all_of_them: bool,
}

impl Tried {
    /// What came back, checked before any of it can reach a screen.
    ///
    /// `pub(crate)`: the only thing that makes one of these is a provider
    /// actually answering ([`crate::trying`]). A `Tried` a caller could build
    /// would be a provider that was never reached.
    pub(crate) fn of(names: impl IntoIterator<Item = String>) -> Self {
        let mut models = Vec::new();
        let mut unshowable = 0usize;
        let mut all_of_them = true;
        for name in names {
            let name = name.trim();
            if name.is_empty()
                || name.chars().any(char::is_control)
                || name.chars().count() > LONGEST_NAME
            {
                // Counted, not shown. A name carrying a line break could show
                // one thing in a list and say another — the objection
                // `alo_capability::Value` makes at the door, arriving from the
                // other side.
                unshowable = unshowable.saturating_add(1);
                continue;
            }
            if models.len() == MOST_NAMES {
                all_of_them = false;
                continue;
            }
            models.push(name.to_owned());
        }
        Self {
            models,
            unshowable,
            all_of_them,
        }
    }

    /// The names the provider offers, as it spells them.
    #[must_use]
    pub fn models(&self) -> &[String] {
        &self.models
    }

    /// The names, to be kept on the provider that is about to be saved.
    #[must_use]
    pub fn into_models(self) -> Vec<String> {
        self.models
    }

    /// How many names were left out because they could not be shown. Zero for
    /// every provider anybody will actually meet.
    #[must_use]
    pub fn unshowable(&self) -> usize {
        self.unshowable
    }

    /// Whether this is everything the provider offered, or the start of it.
    #[must_use]
    pub fn is_all(&self) -> bool {
        self.all_of_them
    }

    /// The string this crate declares for the line that answers the question.
    #[must_use]
    pub fn word(&self) -> words::Word {
        if self.models.is_empty() {
            words::THAT_WORKED_WITH_NOTHING
        } else {
            words::THAT_WORKED
        }
    }

    /// The line a person reads when the test comes back, in their own language.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::model_words`] answers with the key, marked.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }

    /// What else the person needs to know, one line each, in the order they are
    /// worth reading.
    ///
    /// Empty for every provider anybody will actually meet. **Lines rather than
    /// clauses**, for the reason at the top of this file: a machine that pushed
    /// them onto the end of [`said`](Self::said) with a dash between would be
    /// choosing punctuation the sentence's own language chooses.
    #[must_use]
    pub fn caveats(&self, strings: &Strings) -> Vec<Said> {
        let mut lines = Vec::new();
        if !self.all_of_them {
            lines.push(strings.say(&words::THE_LIST_WAS_CUT.key(), &Filling::nothing()));
        }
        if self.unshowable > 0 {
            lines.push(strings.say(&words::SOME_NAMES_LEFT_OUT.key(), &Filling::nothing()));
        }
        lines
    }
}

/// Why a provider was not tested, or was tested and did not work.
///
/// Every message says what to do, and none of them repeats the provider's own
/// words: an error surface that quotes whatever a remote service said is a way
/// for somebody else's text to arrive on a person's screen wearing ours.
///
/// **No `Display`, and therefore not a `std::error::Error`** (item 9f). The
/// only road to words is [`NotTried::said`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotTried {
    /// This machine's policy does not permit reaching that provider at all, so
    /// nothing was sent.
    ///
    /// Carries the policy's own refusal
    /// ([`SourcePolicy::refusal`](crate::SourcePolicy::refusal)) rather than a
    /// second explanation that could disagree with the first — as a **value**
    /// since item 9f, so the words are the policy's in whichever language the
    /// person reads, rather than a rendering made before anybody knew who was
    /// reading.
    Forbidden(NotAllowed),
    /// Nothing answered.
    Unreachable,
    /// The address sent us somewhere else. Refused rather than followed: the
    /// address the policy answered about is the address that gets reached.
    Redirected,
    /// The provider will not answer without a key, and none was given.
    NeedsAKey,
    /// A key was given and the provider did not accept it. **The whole reason
    /// this feature exists**: found while somebody is looking at the settings
    /// panel they typed it into, rather than in the middle of a question.
    KeyNotAccepted,
    /// Something answered, but not like a provider this system can talk to.
    NotUnderstood,
    /// The provider answered, and said it was having trouble. Carries the
    /// status it answered with, which is an identifier rather than a count.
    NotWell(u16),
}

impl NotTried {
    /// The string this crate declares for this refusal.
    ///
    /// For [`Forbidden`](Self::Forbidden) it is the policy's own, because the
    /// rule that refused is the thing to say.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::Forbidden(refusal) => refusal.word(),
            Self::Unreachable => words::PROVIDER_UNREACHABLE,
            Self::Redirected => words::PROVIDER_REDIRECTED,
            Self::NeedsAKey => words::PROVIDER_NEEDS_A_KEY,
            Self::KeyNotAccepted => words::KEY_NOT_ACCEPTED,
            Self::NotUnderstood => words::NOT_A_PROVIDER,
            Self::NotWell(_) => words::PROVIDER_NOT_WELL,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::model_words`] answers with the key, marked. **What was refused
    /// never depends on the string table** — the test had already not happened
    /// before this was called.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::Forbidden(refusal) => refusal.said(strings),
            Self::NotWell(status) => strings.say(
                &self.word().key(),
                &Filling::of("status", status.to_string()),
            ),
            Self::Unreachable
            | Self::Redirected
            | Self::NeedsAKey
            | Self::KeyNotAccepted
            | Self::NotUnderstood => strings.say(&self.word().key(), &Filling::nothing()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    fn names(of: &[&str]) -> Tried {
        Tried::of(of.iter().map(|n| (*n).to_owned()))
    }

    #[test]
    fn the_names_a_provider_offers_come_back_as_it_spells_them() {
        let tried = names(&["mistral-small-latest", "mistral-large-latest"]);
        assert_eq!(
            tried.models(),
            ["mistral-small-latest", "mistral-large-latest"]
        );
        assert!(tried.is_all());
        assert_eq!(tried.unshowable(), 0);
        let strings = in_english();
        assert!(
            tried.said(&strings).text().starts_with("that worked"),
            "{tried:?}"
        );
        assert!(tried.caveats(&strings).is_empty());
    }

    /// A provider writes these, and they land next to things the system said
    /// itself. A name carrying a line break could show one thing and say
    /// another, so it is counted rather than shown.
    #[test]
    fn a_name_that_cannot_be_shown_is_counted_and_not_shown() {
        let tried = names(&[
            "mistral-small-latest",
            "small\nkey accepted: no",
            "  ",
            "tab\there",
        ]);
        assert_eq!(tried.models(), ["mistral-small-latest"]);
        assert_eq!(tried.unshowable(), 3);
        let strings = in_english();
        let caveats = tried.caveats(&strings);
        assert_eq!(caveats.len(), 1);
        assert!(
            caveats
                .first()
                .is_some_and(|line| line.text().contains("could not be shown")),
            "{caveats:?}"
        );
    }

    /// A name longer than any real one is a name that would take a settings
    /// panel apart.
    #[test]
    fn a_name_too_long_to_be_a_name_is_left_out() {
        let long = "m".repeat(LONGEST_NAME + 1);
        let tried = Tried::of([long, "fits".to_owned()]);
        assert_eq!(tried.models(), ["fits"]);
        assert_eq!(tried.unshowable(), 1);
    }

    /// Every bound says it was reached. A cut list that did not say so reads
    /// exactly like a complete one, and somebody would conclude from it that a
    /// model is not offered.
    #[test]
    fn a_list_longer_than_anybody_reads_is_cut_and_says_it_was_cut() {
        let many = (0..MOST_NAMES + 3).map(|n| format!("model-{n}"));
        let tried = Tried::of(many);
        assert_eq!(tried.models().len(), MOST_NAMES);
        assert!(!tried.is_all());
        let strings = in_english();
        let caveats = tried.caveats(&strings);
        assert_eq!(caveats.len(), 1);
        assert!(
            caveats
                .first()
                .is_some_and(|line| line.text().contains("was cut")),
            "{caveats:?}"
        );
    }

    /// **Two things to say are two lines, not one sentence with a dash in it.**
    /// The order is the order they are worth reading, and each one is whole —
    /// which is what lets a translator write a sentence rather than a fragment
    /// somebody else's punctuation will be glued to.
    #[test]
    fn two_caveats_are_two_lines_and_the_answer_is_a_third() {
        let long = "m".repeat(LONGEST_NAME + 1);
        let mut many: Vec<String> = (0..MOST_NAMES + 3).map(|n| format!("model-{n}")).collect();
        many.push(long);
        let tried = Tried::of(many);
        let strings = in_english();
        let caveats = tried.caveats(&strings);
        assert_eq!(caveats.len(), 2);
        for line in &caveats {
            assert!(!line.text().contains('—'), "{line}");
        }
        assert!(!tried.said(&strings).text().contains('—'), "{tried:?}");
    }

    /// A provider that answers with an empty list has still answered, and the
    /// key was still accepted. Saying "that worked" and nothing else would
    /// leave somebody looking for a model list that is not going to appear.
    #[test]
    fn a_provider_that_offers_nothing_is_a_working_provider_that_says_so() {
        let tried = names(&[]);
        assert!(tried.models().is_empty());
        assert!(tried.is_all());
        assert!(
            tried.said(&in_english()).text().contains("no models"),
            "{tried:?}"
        );
    }

    /// The refusals are read by somebody who has just typed something in, so
    /// they say what to do about it — and the two that get confused with each
    /// other say plainly which one this is.
    #[test]
    fn the_refusals_say_what_to_do_and_which_one_this_is() {
        let strings = in_english();
        let said = |not: &NotTried| not.said(&strings).into_text();
        assert!(said(&NotTried::KeyNotAccepted).contains("check it is the whole key"));
        assert!(said(&NotTried::NeedsAKey).contains("add the one it"));
        assert!(said(&NotTried::Unreachable).contains("check the address"));
        assert!(said(&NotTried::Redirected).contains("nobody agreed to"));
        assert!(said(&NotTried::NotUnderstood).contains("rather than of the website"));
        assert_eq!(
            said(&NotTried::NotWell(503)),
            "the provider answered 503, which is a problem at their end — try again in a moment"
        );
    }

    /// The policy's refusal is carried rather than reworded, so the machine
    /// cannot explain the same rule two ways — and since it is carried as a
    /// value, both explanations are in whichever language the person reads.
    #[test]
    fn a_policy_refusal_is_the_policys_own_words() {
        let refusal = NotAllowed::NotThisMachine {
            source: crate::source::InferenceSource::PairedMachine {
                machine: "the studio workstation".to_owned(),
            },
        };
        let strings = in_english();
        assert_eq!(
            NotTried::Forbidden(refusal.clone()).said(&strings).text(),
            refusal.said(&strings).text(),
            "the policy's words are the message, not a summary of them"
        );
    }

    /// **A test that never happened is said in the reader's own language**, and
    /// the status a provider answered with is not translated because it is not
    /// a word.
    #[test]
    fn a_refusal_is_said_in_the_language_the_person_reads() {
        let strings = translated(&[(
            words::PROVIDER_NOT_WELL,
            "der Anbieter antwortete {status}, das ist ein Problem auf seiner Seite — versuchen \
             Sie es gleich noch einmal",
        )]);
        let said = NotTried::NotWell(503).said(&strings);
        assert!(said.is_translated());
        assert!(
            said.text().starts_with("der Anbieter antwortete 503"),
            "{said}"
        );
    }

    /// **A refusal never depends on a string table.** With no words at all it
    /// says the same thing about what happened and names the rule by its key.
    #[test]
    fn a_refusal_without_the_words_still_names_the_rule() {
        let nothing = Strings::of(alo_strings::Vocabulary::empty());
        let said = NotTried::KeyNotAccepted.said(&nothing);
        assert!(said.is_a_bug());
        assert!(
            said.text().contains("models.not-tried.key-not-accepted"),
            "{said}"
        );
    }
}
