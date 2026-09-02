//! One egress an agent is about to cause: who, where to, and why.
//!
//! This is the thing the policy decides about and the indicator shows. It
//! exists before the connection does, which is the whole of law 1's first half:
//! *visible at the moment it happens* is only achievable if what is about to
//! happen can be named before it happens.
//!
//! **Why is a closed list, and that is deliberate.** A free-text purpose would
//! be text the model composed, shown to a person on the indicator as though the
//! system were saying it — the same objection [`alo_capability::Takes`] makes to
//! a free-text argument, applied to the one surface a person is expected to
//! trust. Three reasons cover what an agent can cause: it is asking a question,
//! it is fetching something, or it is sending something. A fourth is a change to
//! what an agent can do, and belongs in ADR 0001 before it belongs here.
//!
//! **The sentence lives here and only here.** One place composes the words a
//! person reads on the indicator, so there is one thing to translate and one
//! thing to review, rather than a phrase in the shell and a different one in the
//! settings panel.

use std::fmt;

use alo_capability::Grantee;
use alo_models::InferenceSource;
use serde::{Deserialize, Serialize};

use crate::destination::{Destination, DestinationError};

/// Why an agent is causing something to leave.
///
/// A closed list. Adding to it widens what an agent can cause, so it belongs in
/// [ADR 0001](../../../docs/decisions/0001-the-capability-model.md) before it
/// belongs here.
///
/// Read back as well as written down, like [`Destination`] and unlike
/// [`Leaving`]: a record of what left this machine says *why* it left, and a
/// record read at the end of a week has to be able to say it. Reading one back
/// decides nothing and permits nothing — it names a reason, and naming a reason
/// is all it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Why {
    /// A question put to a model somewhere other than this machine
    /// ([ADR 0008](../../../docs/decisions/0008-where-inference-happens.md)).
    /// The largest egress this product will ever cause.
    Asking,
    /// Retrieving something from an address a verb named.
    Fetching,
    /// Handing something to a service outside this machine.
    Sending,
}

/// One egress, about to happen.
///
/// Serialises so the indicator's contents can be shown or written down, and
/// deliberately does **not** deserialise — like [`alo_capability::Call`], and
/// for the same reason. One read back off a disk would be an egress nothing had
/// decided about, holding a destination nobody checked, and the type would
/// become a promise instead of a fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Leaving {
    /// Whose authority this is under.
    agent: Grantee,
    /// Why it is leaving.
    why: Why,
    /// Where it is going.
    destination: Destination,
}

impl Leaving {
    /// An egress this agent is about to cause, for this reason, to here.
    #[must_use]
    pub fn because(agent: &Grantee, why: Why, destination: Destination) -> Self {
        Self {
            agent: agent.clone(),
            why,
            destination,
        }
    }

    /// A question this agent is about to put to a model somewhere else.
    ///
    /// The common case, and the one with a source rather than a bare address:
    /// where a question is answered is already a decided property in
    /// `alo-models`, so it is turned into a destination here instead of being
    /// described a second time.
    ///
    /// # Errors
    /// [`DestinationError::NothingLeaves`] when the source is this machine.
    /// A question answered here causes no egress, so there is no [`Leaving`]
    /// for it — which is the shape law 1's zero-egress claim takes in code.
    pub fn asking(agent: &Grantee, source: &InferenceSource) -> Result<Self, DestinationError> {
        Ok(Self::because(agent, Why::Asking, Destination::of(source)?))
    }

    /// Whose authority this is under.
    #[must_use]
    pub fn agent(&self) -> &Grantee {
        &self.agent
    }

    /// Why it is leaving.
    #[must_use]
    pub fn why(&self) -> Why {
        self.why
    }

    /// Where it is going.
    #[must_use]
    pub fn destination(&self) -> &Destination {
        &self.destination
    }

    /// The line a person reads at the moment it happens.
    ///
    /// One sentence, naming all three things, because an indicator that said
    /// only *something is leaving* would be a diagnostic rather than a feature.
    #[must_use]
    pub fn describe(&self) -> String {
        let agent = self.agent.as_str();
        let destination = self.destination.describe();
        match self.why {
            Why::Asking => format!("{agent} is asking a question of {destination}"),
            Why::Fetching => format!("{agent} is fetching something from {destination}"),
            Why::Sending => format!("{agent} is sending something to {destination}"),
        }
    }
}

impl fmt::Display for Leaving {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.describe())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_models::Region;

    fn mail() -> Grantee {
        Grantee::named("@mail")
    }

    /// **A working day with a local model has nothing to show.** There is no
    /// [`Leaving`] to be made from an answer given here, so the indicator has
    /// nothing to put on it — which is the zero-egress claim as a type rather
    /// than as a promise.
    #[test]
    fn a_question_answered_on_this_machine_is_not_a_departure_at_all() {
        assert_eq!(
            Leaving::asking(&mail(), &InferenceSource::ThisMachine).unwrap_err(),
            DestinationError::NothingLeaves
        );
    }

    /// **Law 1: the corridor is egress too.** An answer from a machine in the
    /// next room is a departure with a line of its own, and the difference from
    /// a hosted provider is said in words rather than by staying silent.
    #[test]
    fn a_question_answered_in_the_next_room_is_a_departure_with_a_line_of_its_own() {
        let leaving = Leaving::asking(
            &mail(),
            &InferenceSource::PairedMachine {
                machine: "the studio workstation".to_owned(),
            },
        )
        .unwrap();
        assert_eq!(leaving.why(), Why::Asking);
        assert!(leaving.destination().stays_in_the_building());
        assert_eq!(
            leaving.describe(),
            "@mail is asking a question of the studio workstation, on your network"
        );
    }

    /// The line names all three things: who, why, and where. An indicator that
    /// said only that something was leaving would be a diagnostic.
    #[test]
    fn the_line_a_person_reads_names_who_why_and_where() {
        let asking = Leaving::asking(
            &mail(),
            &InferenceSource::Hosted {
                provider: "alo".to_owned(),
                region: Region::Declared("the EU".to_owned()),
            },
        )
        .unwrap();
        assert_eq!(
            asking.to_string(),
            "@mail is asking a question of alo, in the EU"
        );

        let fetching = Leaving::because(
            &Grantee::named("@files"),
            Why::Fetching,
            Destination::at("alo.example").unwrap(),
        );
        assert_eq!(
            fetching.describe(),
            "@files is fetching something from alo.example"
        );

        let sending = Leaving::because(
            &Grantee::named("@files"),
            Why::Sending,
            Destination::at("alo.example").unwrap(),
        );
        assert_eq!(
            sending.describe(),
            "@files is sending something to alo.example"
        );
    }

    /// Whose authority an egress is under is an identity, matched exactly like
    /// every other identity in the capability model.
    #[test]
    fn the_agent_an_egress_is_under_is_kept_exactly_as_it_was_named() {
        let leaving = Leaving::because(
            &mail(),
            Why::Sending,
            Destination::at("alo.example").unwrap(),
        );
        assert_eq!(leaving.agent(), &Grantee::named("@mail"));
        assert_ne!(leaving.agent(), &Grantee::named("@Mail"));
    }

    /// The indicator's contents can be shown and written down, and cannot be
    /// read back into an egress nothing decided about.
    #[test]
    fn what_is_leaving_can_be_written_down_and_not_read_back() {
        let leaving = Leaving::because(
            &mail(),
            Why::Asking,
            Destination::at("alo.example").unwrap(),
        );
        let written = serde_json::to_string(&leaving).unwrap();
        assert!(written.contains("alo.example"), "{written}");
        assert!(written.contains("asking"), "{written}");
        // There is no `Deserialize` for `Leaving`, so this is as far as it
        // goes: the destination reads back, and the decision does not.
        assert!(serde_json::from_str::<Destination>(&written).is_err());

        // The parts a record keeps do read back, because "what left, and why"
        // is a question asked at the end of a week and not only in the second
        // the indicator lit up.
        for why in [Why::Asking, Why::Fetching, Why::Sending] {
            let alone = serde_json::to_string(&why).unwrap();
            assert_eq!(
                serde_json::from_str::<Why>(&alone).ok(),
                Some(why),
                "{alone}"
            );
        }
    }
}
