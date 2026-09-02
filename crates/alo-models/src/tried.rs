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
//! whatever shows them once that exists.

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

    /// The line a person reads when the test comes back.
    #[must_use]
    pub fn describe(&self) -> String {
        let mut said = if self.models.is_empty() {
            "that worked, and this provider offers no models to choose from".to_owned()
        } else {
            "that worked, and this provider says what it offers".to_owned()
        };
        if !self.all_of_them {
            said.push_str(" — the list is longer than this and was cut");
        }
        if self.unshowable > 0 {
            said.push_str(" — some names could not be shown and were left out");
        }
        said
    }
}

/// Why a provider was not tested, or was tested and did not work.
///
/// Every message says what to do, and none of them repeats the provider's own
/// words: an error surface that quotes whatever a remote service said is a way
/// for somebody else's text to arrive on a person's screen wearing ours.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NotTried {
    /// This machine's policy does not permit reaching that provider at all, so
    /// nothing was sent. Carries the policy's own words
    /// ([`SourcePolicy::refusal`](crate::SourcePolicy::refusal)) rather than
    /// composing a second explanation that could disagree with the first.
    #[error("{0}")]
    Forbidden(String),
    /// Nothing answered.
    #[error(
        "nothing answered at that address — check the address, and that this machine is online"
    )]
    Unreachable,
    /// The address sent us somewhere else. Refused rather than followed: the
    /// address the policy answered about is the address that gets reached.
    #[error(
        "that address sends this machine somewhere else, and a key is not carried to an address nobody agreed to — use the address the provider documents"
    )]
    Redirected,
    /// The provider will not answer without a key, and none was given.
    #[error("this provider will not answer without a key — add the one it gave you")]
    NeedsAKey,
    /// A key was given and the provider did not accept it. **The whole reason
    /// this feature exists**: found while somebody is looking at the settings
    /// panel they typed it into, rather than in the middle of a question.
    #[error(
        "that key was not accepted — check it is the whole key, and that it is this provider's"
    )]
    KeyNotAccepted,
    /// Something answered, but not like a provider this system can talk to.
    #[error(
        "that address answered, but not like a provider this system can use — check it is the address of the API rather than of the website"
    )]
    NotUnderstood,
    /// The provider answered, and said it was having trouble.
    #[error("the provider answered {0}, which is a problem at their end — try again in a moment")]
    NotWell(u16),
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(tried.describe().starts_with("that worked"), "{tried:?}");
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
        assert!(
            tried.describe().contains("could not be shown"),
            "{}",
            tried.describe()
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
        assert!(tried.describe().contains("was cut"), "{}", tried.describe());
    }

    /// A provider that answers with an empty list has still answered, and the
    /// key was still accepted. Saying "that worked" and nothing else would
    /// leave somebody looking for a model list that is not going to appear.
    #[test]
    fn a_provider_that_offers_nothing_is_a_working_provider_that_says_so() {
        let tried = names(&[]);
        assert!(tried.models().is_empty());
        assert!(tried.is_all());
        assert!(tried.describe().contains("no models"), "{tried:?}");
    }

    /// The refusals are read by somebody who has just typed something in, so
    /// they say what to do about it — and the two that get confused with each
    /// other say plainly which one this is.
    #[test]
    fn the_refusals_say_what_to_do_and_which_one_this_is() {
        assert!(
            NotTried::KeyNotAccepted
                .to_string()
                .contains("check it is the whole key")
        );
        assert!(NotTried::NeedsAKey.to_string().contains("add the one it"));
        assert!(
            NotTried::Unreachable
                .to_string()
                .contains("check the address")
        );
        assert!(
            NotTried::Redirected
                .to_string()
                .contains("nobody agreed to")
        );
        assert!(
            NotTried::NotUnderstood
                .to_string()
                .contains("rather than of the website")
        );
        assert_eq!(
            NotTried::NotWell(503).to_string(),
            "the provider answered 503, which is a problem at their end — try again in a moment"
        );
    }

    /// The policy's refusal is carried rather than reworded, so the machine
    /// cannot explain the same rule two ways.
    #[test]
    fn a_policy_refusal_is_the_policys_own_words() {
        let said = "this machine is set to answer only on itself";
        assert_eq!(
            NotTried::Forbidden(said.to_owned()).to_string(),
            said,
            "the policy's words are the message, not a summary of them"
        );
    }
}
