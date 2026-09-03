//! Why alo OS itself reaches the network — the whole list, and the promise it is.
//!
//! `docs/features.md` makes one promise in this area that has nothing to do
//! with agents: **★ No telemetry. Not "anonymised telemetry". None — and the
//! policy lives in a Rust service, not a checkbox.** Everything else in this
//! crate is about egress an *agent* causes, decided against
//! [`EgressPolicy`](crate::EgressPolicy) and shown on the indicator. Telemetry
//! is the other kind: egress with **no agent behind it**, which until this file
//! existed could not be named here at all — and a thing that cannot be named
//! cannot be shown either.
//!
//! # A closed list is the mechanism, and there is nothing measuring on it
//!
//! This is law 2's shape applied to what the system itself does. A verb cannot
//! run an arbitrary command because [`alo_capability::Takes`] has no free-text
//! kind for one to arrive in; alo OS cannot phone home about you because
//! [`Errand`] has no member for it. There is no `Errand::Other(String)`, no
//! diagnostics, no crash report, no *usage* anything — not because somebody
//! remembered to leave them out, but because adding one is an edit to a public
//! enum, in a repository whose scope gate is `docs/features.md`, failing the
//! test at the bottom of this file on the way.
//!
//! A checkbox is the thing that promise is contrasted with, and the difference
//! is exactly this: a checkbox has the code behind it either way.
//!
//! # Three reasons, and each one is somebody's business rather than ours
//!
//! - **Signing in** — `docs/features.md` v0.01, *sign-in with an alo identity*.
//! - **Fetching a model** — v0.01, *model lifecycle: pull*. A machine that
//!   cannot download a model cannot answer a question on its own hardware,
//!   which is the product.
//! - **Checking for an update** — v0.5, *atomic updates with rollback*. Named
//!   here now rather than when it is built, because a list that is complete
//!   later is not a guarantee today, and an update check is the classic place
//!   telemetry rides along.
//!
//! Naming a reason is not building it: the two that are v0.01 have code in this
//! repository, the third has none, and what this file guarantees is about the
//! list rather than about what is behind each line of it.
//!
//! # What is not on the list, and why that is not a hole
//!
//! **Finding machines on the local network** (ADR 0003, v0.5) announces and
//! listens rather than reaching a named destination, so there is nothing here
//! for it to be — it has no [`crate::Destination`]. When it is built it is
//! either an errand of its own or a documented exception, and that is written
//! into the queue rather than left for somebody to notice.
//!
//! **A crash report or a support bundle a person deliberately sends** is not on
//! this list either, and the reason is the interesting one: the moment a person
//! reads what is in something and sends it, it is their egress and not the
//! machine's. What makes telemetry telemetry is that nobody asked and nobody
//! saw. Neither is true of an errand here — every one of them appears on the
//! same indicator as an agent's egress, which is the half of this that a person
//! can actually check.
//!
//! # The organisation's egress policy is not asked about these
//!
//! [`EgressPolicy`](crate::EgressPolicy) is `From<&SourcePolicy>` — it is a
//! rule an organisation stated about *where a question may be answered*,
//! widened to everything an agent can cause. Applying it to an errand would
//! mean a machine set to `ThisMachineOnly` could never download the model it is
//! set to answer with: a policy that defeats the setting it came from. So an
//! errand is decided by being on this list, and by nothing else — there is no
//! rule to turn off, because there is no reason here anybody would want
//! stopped. What an organisation controls about updates is *where they come
//! from* (v1, an update mirror they host), which is a destination and not a
//! permission.

use serde::{Deserialize, Serialize};

use crate::words;

/// A reason alo OS itself reaches the network.
///
/// A closed list, and that is the whole of the *no telemetry* promise as code
/// rather than as a sentence. Adding a member widens what this machine does
/// with nobody having asked, so it belongs in `docs/features.md` before it
/// belongs here — the same rule [`Why`](crate::Why) states for what an agent
/// can cause.
///
/// Read back as well as written down, like [`Why`](crate::Why): what alo OS
/// itself did is a question asked at the end of a week as well as in the second
/// it happened. Reading one back decides nothing and permits nothing — it names
/// a reason, and naming a reason is all it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Errand {
    /// Signing a person in against their alo identity.
    SigningIn,
    /// Downloading a model from the catalogue, so this machine can answer on
    /// its own hardware (ADR 0006, ADR 0007).
    FetchingAModel,
    /// Asking whether there is a newer deployment than the one that is running.
    CheckingForAnUpdate,
}

impl Errand {
    /// Every reason there is, in the order this file declares them.
    ///
    /// What a settings panel lists and what the test at the bottom of this file
    /// walks. A reason left out of it would be one nothing can show.
    pub const EVERY: [Self; 3] = [
        Self::SigningIn,
        Self::FetchingAModel,
        Self::CheckingForAnUpdate,
    ];

    /// The string this crate declares for this reason.
    #[must_use]
    pub fn word(self) -> words::Word {
        match self {
            Self::SigningIn => words::ALO_IS_SIGNING_YOU_IN,
            Self::FetchingAModel => words::ALO_IS_FETCHING_A_MODEL,
            Self::CheckingForAnUpdate => words::ALO_IS_CHECKING_FOR_AN_UPDATE,
        }
    }

    /// The promise itself, in the language the person reads.
    ///
    /// *alo OS reaches the network for these reasons and no others, and never
    /// to say anything about how you use this machine.* Shown beside the list —
    /// in settings, and wherever somebody goes to find out what their machine
    /// has been doing — because a promise nobody is told about is not a
    /// feature. It is `alo-answering`'s *nothing was sent anywhere* made about
    /// the machine rather than about one failed question, and it is here rather
    /// than in a README for the same reason that one is: the person it is for
    /// does not read this repository, and may not read English.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given [`crate::egress_words`] answers with the
    /// key, marked, and `Said::is_a_bug`.
    #[must_use]
    pub fn nothing_else(strings: &alo_strings::Strings) -> alo_strings::Said {
        strings.say(
            &words::ALO_REACHES_NOTHING_ELSE.key(),
            &alo_strings::Filling::nothing(),
        )
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
    use alo_strings::{Strings, Vocabulary};

    /// **★ No telemetry, as a closed list rather than as a promise.**
    ///
    /// This is a tripwire and is meant to be one. A fourth reason for alo OS to
    /// reach the network cannot be added without the exhaustive match below
    /// failing to compile and this count failing to hold, so whoever adds one
    /// reads this file first — which is where they find out that measurement,
    /// diagnostics, crash reports and *usage* anything are not scope decisions
    /// somebody may revisit but the promise the product is sold on.
    #[test]
    fn there_is_no_reason_on_the_list_that_is_about_how_this_machine_is_used() {
        assert_eq!(Errand::EVERY.len(), 3);
        for errand in Errand::EVERY {
            match errand {
                // Somebody signing in, somebody's model, and the release this
                // machine is running. Nothing about what they did with it.
                Errand::SigningIn | Errand::FetchingAModel | Errand::CheckingForAnUpdate => {}
            }
        }
    }

    /// A reason names one string, and every reason has one. A member with no
    /// word would be a line on the indicator nobody could read.
    #[test]
    fn every_reason_has_a_sentence_of_its_own() {
        let mut named: Vec<&str> = Errand::EVERY
            .iter()
            .map(|errand| errand.word().named())
            .collect();
        named.sort_unstable();
        named.dedup();
        assert_eq!(named.len(), Errand::EVERY.len());
    }

    /// **The promise is a string a person reads**, in their own language, and
    /// it says both halves: these reasons and no others, and nothing about how
    /// the machine is used. A translation that kept only the first half would
    /// leave out the half that is the feature.
    #[test]
    fn the_promise_is_said_in_the_language_the_person_reads() {
        let english = Errand::nothing_else(&in_english());
        assert!(english.text().contains("no others"), "{english}");
        assert!(
            english.text().contains("how you use this machine"),
            "{english}"
        );

        let german = Errand::nothing_else(&translated(&[(
            words::ALO_REACHES_NOTHING_ELSE,
            "alo OS geht aus diesen Gründen ins Netz und aus keinen anderen, und niemals, um etwas \
             darüber zu sagen, wie Sie diesen Rechner benutzen",
        )]));
        assert!(german.is_translated(), "{german}");
        assert!(german.text().contains("keinen anderen"), "{german}");
        assert!(!german.text().contains("no others"), "{german}");
    }

    /// **The promise does not go blank because nobody declared the words.** A
    /// machine whose shell forgot this crate's vocabulary still says something
    /// and still names the key, which is how the person who has to fix it finds
    /// out.
    #[test]
    fn the_promise_without_the_words_still_names_its_key() {
        let said = Errand::nothing_else(&Strings::of(Vocabulary::empty()));
        assert!(said.is_a_bug());
        assert!(said.text().contains("egress.itself.nothing-else"), "{said}");
    }

    /// What alo OS did on its own is a question asked afterwards as well as
    /// while it happens, so a reason survives being written down and read back.
    #[test]
    fn a_reason_survives_being_written_down_and_read_back() {
        for errand in Errand::EVERY {
            let written = serde_json::to_string(&errand).unwrap();
            assert_eq!(serde_json::from_str::<Errand>(&written).ok(), Some(errand));
            assert!(!written.contains('_'), "{written}");
        }
    }
}
