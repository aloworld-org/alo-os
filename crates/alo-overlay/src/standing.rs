//! Where the machine stands when the overlay opens: one of three, and the
//! sentence for each.
//!
//! This is the value the plan's task 3 asks for — *the overlay's state, with a
//! case for each of nothing granted, nothing chosen and ready*. It is derived
//! from the readings beside it ([`crate::WouldAnswer`] and [`crate::Granted`])
//! and never set by a caller, so there is no way for the sentence in front of
//! a person to disagree with the machine underneath it.
//!
//! # Why *nothing chosen* wins when both are true
//!
//! A machine nobody has set up is in both states at once: nothing answers
//! questions and nothing has been granted. One sentence has to come first, and
//! the order is not arbitrary.
//!
//! **Choosing comes first because granting without it changes nothing.** A
//! person who grants a folder on a machine with no model still cannot ask
//! anything; a person who chooses a model on a machine with no grants can ask
//! questions immediately, and only the changes their agent proposes are
//! blocked. So the first instruction is the one that makes the next press of
//! the key useful, and *nothing granted* is what they read the second time.
//!
//! Both cases still exist and both are reachable — this is an order, not a
//! collapse — and [`crate::AtRest`] shows the readings underneath either way,
//! so nothing about the other half of the machine is hidden by the choice of
//! which sentence leads.
//!
//! # It says what to do, and that is a requirement rather than a courtesy
//!
//! An empty state that reports emptiness has told somebody they are stuck.
//! Both sentences a person can act on name the action — choose a model or add
//! a provider; grant a folder — and `crate::words` has the test that keeps
//! them from shrinking back into bare reports.

use alo_strings::{Filling, Said, Strings};

use crate::answering::WouldAnswer;
use crate::granted::Granted;
use crate::words::{self, Word};

/// What the overlay says first, before anybody has asked it anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Nothing has been chosen to answer questions. Says what to do.
    NothingChosen,
    /// Something would answer, and the agent has been granted nothing. Says
    /// what to do.
    NothingGranted,
    /// Something would answer and something is granted: ask away.
    Ready,
}

impl Standing {
    /// Where the machine stands, from what it holds.
    ///
    /// The only constructor. A caller cannot name a state — it follows from
    /// the two readings, which follow from the person's own settings and the
    /// grants on the machine.
    #[must_use]
    pub const fn of(answering: &WouldAnswer, granted: Granted) -> Self {
        if !answering.is_chosen() {
            return Self::NothingChosen;
        }
        if granted.is_nothing() {
            return Self::NothingGranted;
        }
        Self::Ready
    }

    /// Whether there is something for the person to do before the agent is
    /// useful.
    #[must_use]
    pub const fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }

    /// The string this crate declares for it.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::NothingChosen => words::NOTHING_CHOSEN,
            Self::NothingGranted => words::NOTHING_GRANTED,
            Self::Ready => words::READY,
        }
    }

    /// What to put in front of the person, in the language they read.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    /// A `Strings` that was never given [`crate::overlay_words`] answers with
    /// the key, marked `Said::is_a_bug` — the honest answer to *the shell
    /// forgot to declare what this crate can say*, and never a blank line.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
// No `expect(clippy::unwrap_used)` here, unlike its neighbours: nothing in
// this file's tests builds a value that can fail to be built, so there is
// nothing to unwrap. An exemption nobody needs is an exemption somebody
// later reaches for.
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use alo_models::InferenceSource;

    /// Every state, so no test below quietly skips one.
    const EVERY_STANDING: [Standing; 3] = [
        Standing::NothingChosen,
        Standing::NothingGranted,
        Standing::Ready,
    ];

    /// A choice that would answer on this machine.
    fn chosen() -> WouldAnswer {
        WouldAnswer::Something {
            source: InferenceSource::ThisMachine,
            model: "mistral-small".to_owned(),
        }
    }

    /// **A machine nobody has chosen for stands at *nothing chosen*.**
    #[test]
    fn nothing_chosen_is_a_machine_nobody_has_chosen_for() {
        assert_eq!(
            Standing::of(&WouldAnswer::Nothing, Granted::Nothing),
            Standing::NothingChosen
        );
        assert!(!Standing::NothingChosen.is_ready());
    }

    /// **A machine that can answer and can reach nothing stands at *nothing
    /// granted*.**
    #[test]
    fn nothing_granted_is_a_machine_that_can_answer_and_reach_nothing() {
        assert_eq!(
            Standing::of(&chosen(), Granted::Nothing),
            Standing::NothingGranted
        );
    }

    /// **And a machine with both is ready.**
    #[test]
    fn ready_is_a_machine_with_both() {
        let standing = Standing::of(&chosen(), Granted::of_how_many(1));
        assert_eq!(standing, Standing::Ready);
        assert!(standing.is_ready());
    }

    /// **Choosing comes first when neither has been done**, because granting
    /// a folder on a machine with no model changes nothing a person can do.
    /// The order is the decision this file documents; this is the test that
    /// stops it being reversed by accident.
    #[test]
    fn a_machine_with_neither_is_told_to_choose_first() {
        assert_eq!(
            Standing::of(&WouldAnswer::Nothing, Granted::Nothing),
            Standing::NothingChosen
        );
        // And grants on a machine with nothing chosen do not make it ready:
        // the agent would have somewhere to reach and nothing to think with.
        assert_eq!(
            Standing::of(&WouldAnswer::Nothing, Granted::of_how_many(3)),
            Standing::NothingChosen
        );
    }

    /// **Every state says something a person could read**, and no two of them
    /// read the same — three states that shared a sentence would be one state
    /// wearing three names.
    #[test]
    fn every_state_reads_and_no_two_read_the_same() {
        let strings = in_english();
        let mut seen = Vec::new();
        for standing in EVERY_STANDING {
            let said = standing.said(&strings);
            assert!(!said.text().is_empty(), "{standing:?} says nothing");
            assert!(!said.is_a_bug(), "{standing:?} is not declared");
            assert!(
                !seen.contains(&said.text().to_owned()),
                "two states say {said}"
            );
            seen.push(said.text().to_owned());
        }
    }

    /// **The two states a person can act on tell them what to do**, which is
    /// the plan's acceptance for the empty overlay read at the level a person
    /// meets it: a sentence, in their language, with an instruction in it.
    #[test]
    fn the_states_that_are_not_ready_say_what_to_do() {
        let strings = in_english();
        let nothing_chosen = Standing::NothingChosen.said(&strings);
        assert!(
            nothing_chosen.text().contains("Choose a model"),
            "{nothing_chosen}"
        );
        assert!(
            nothing_chosen.text().contains("add a provider"),
            "{nothing_chosen}"
        );

        let nothing_granted = Standing::NothingGranted.said(&strings);
        assert!(
            nothing_granted.text().contains("Grant it a folder"),
            "{nothing_granted}"
        );
    }

    /// **A shell that forgot to declare this crate's words is told so**,
    /// rather than putting a blank line where the state should be. This is
    /// the refusal path of externalisation itself.
    #[test]
    fn a_machine_that_never_declared_these_words_says_it_is_a_bug() {
        let nothing = Strings::of(alo_strings::Vocabulary::empty());
        for standing in EVERY_STANDING {
            let said = standing.said(&nothing);
            assert!(said.is_a_bug(), "{standing:?} answered from nowhere");
            assert!(
                !said.text().is_empty(),
                "{standing:?} answered with a blank"
            );
            assert!(
                said.text().contains(standing.word().named()),
                "{standing:?} does not name the string nobody declared"
            );
        }
    }

    /// **The state arrives in the language the person reads**, which is the
    /// whole of what declaring these through `alo-strings` buys.
    #[test]
    fn the_state_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(
            words::NOTHING_CHOSEN,
            "Es wurde noch nichts ausgewählt, das Fragen beantwortet. Wählen Sie ein Modell für \
             dieses Gerät oder fügen Sie einen Anbieter hinzu",
        )]);
        let said = Standing::NothingChosen.said(&strings);
        assert!(said.is_translated());
        assert!(said.text().starts_with("Es wurde"));

        // The ones nobody translated are still English, and say they are.
        let ready = Standing::Ready.said(&strings);
        assert!(!ready.is_translated());
        assert!(!ready.is_a_bug());
    }
}
