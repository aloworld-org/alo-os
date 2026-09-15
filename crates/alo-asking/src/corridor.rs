//! The machine down the corridor: a question that leaves this machine and does
//! not leave the building.
//!
//! *A machine without a GPU discovers the one with it, and the agents just
//! work. The inference never leaves the building; it moves down the corridor.*
//!
//! # The sentence this file exists to refuse
//!
//! *"It only went to the machine down the hall."*
//!
//! That is the shape every guarantee dies in — not a decision to break it, but
//! an exception that sounds too small to matter. ADR 0003 refuses it in
//! advance: a paired machine **is still egress, and the indicator still
//! fires.** `alo_models::InferenceSource::causes_egress` has been true for
//! `PairedMachine` since item 18a, and this file is the first thing that could
//! have quietly disagreed with it.
//!
//! So this door is built exactly like [`crate::Asking::to_a_provider`] and
//! deliberately not at all like [`crate::Asking::to_a_service_on_this_machine`]:
//! it reaches the wire holding an `alo_egress::Departing`, which only
//! `alo_egress::Indicator::beginning` makes, so the question cannot leave
//! without a person having been shown it leaving. The two doors that skip the
//! indicator are the two where nothing goes anywhere, and this is not one of
//! them however short the corridor is.
//!
//! # The question travels; the grant does not
//!
//! ADR 0003 again, and it is the half a reader will most want proof of. What
//! this machine's pairing decides is that this machine **may ask**. What the
//! machine down the corridor does about the question — which of its own grants
//! it evaluates, whose approval it asks for, what it writes down — is that
//! machine's, decided by that machine's person, and nothing constructed here
//! travels with the question to influence it.
//!
//! [`DownTheCorridor::paired`] is therefore about *this* machine's side only:
//! whether a pairing made by two people permits asking that machine's models,
//! at the moment of asking. An unpaired machine offering inference reaches this
//! constructor and is refused, however convenient it would be — which is
//! `an_unpaired_machine_offering_inference_is_not_used`.
//!
//! # And the question proves where it came from
//!
//! ADR 0031. What does travel with the question is a proof made with the key
//! the pairing holds — over the exact bytes of the request, for the machine
//! it is going to, at the moment — carried in [`THE_PROOF_HEADER`]. The machine
//! down the corridor checks it against its own pairings before it does
//! anything else, and writes down *the reception machine* because the
//! connection proved it rather than because a pairing happened to name one. A
//! stranger who read this machine's identity off a discovery packet can put a
//! question in the same shape at the same address and is refused there, before
//! any grant is asked. The proof is made here and nowhere else in this crate,
//! because this is the one door with a pairing in hand.
//!
//! # And it is never a fallback
//!
//! Nothing here substitutes for a model that was not there.
//! `docs/features.md` refuses silent substitution four times, and a paired
//! machine is the most tempting substitution in the document: the answer would
//! be better, the wait would be shorter, and the person would never know their
//! question left. A question reaches this door because the person's permission
//! named this machine, and for no other reason.

use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::time::{Duration, SystemTime};

use alo_answering::WentWrong;
use alo_models::{InferenceSource, Secret};
use alo_nearby::{MachineId, MayAskIts, Pairing, Pairings, Proof};

use crate::openai;
use crate::question::Question;
use crate::refusing::Miswired;

/// How long this machine waits for the one down the corridor.
///
/// The same as for a provider. It is a machine somebody owns rather than a
/// service somebody sells, and a model of the size worth sharing thinks for
/// about as long either way.
const WHILE_A_MODEL_THINKS: Duration = Duration::from_secs(120);

/// The header a question down the corridor carries its proof in.
///
/// One spelling, here, for the door that sends it and whatever receives it.
/// The value is [`alo_nearby::Proof::said`], one line.
pub const THE_PROOF_HEADER: &str = "alo-pairing";

/// The path a question down the corridor is put to, on the port a machine
/// advertises.
///
/// The OpenAI-compatible completions path, because a question is put to a
/// paired machine in the same shape it is put to a provider (`crate::openai`
/// says why there is one shape rather than two). One spelling, here, for the
/// door that sends it and the daemon that tells a question apart from a
/// proposal and a verb by path; `openai::tests` holds that the URL this crate
/// really puts to ends in it.
pub const THE_QUESTION_PATH: &str = "/v1/chat/completions";

/// A paired machine, as a place a question may be put.
///
/// Made only by [`paired`](Self::paired), which asks this machine's own
/// pairings. There is no other constructor, so holding one of these is evidence
/// that two people agreed, that their agreement had not ended, and that this
/// machine holds the key to prove a question is its.
#[derive(Debug)]
pub struct DownTheCorridor<'a> {
    /// The name the person gave it when they paired with it, which is what
    /// goes on the indicator and into the record.
    called: &'a str,
    /// Where to put the question.
    endpoint: String,
    /// A key, if that machine asks for one. Usually nothing: it is a machine
    /// somebody owns, and what stands in for a key is the pairing.
    key: Option<&'a Secret>,
    /// The pairing, which holds the key a proof is made with and names the
    /// machine the proof is for.
    pairing: Pairing,
    /// This machine, which the proof names as the sender.
    here: MachineId,
    /// The interface of the network the machine was found on, where a question
    /// to it is held to that network (ADR 0042), and `None` where it is dialled
    /// by its address alone.
    held_to: Option<NonZeroU32>,
}

impl<'a> DownTheCorridor<'a> {
    /// The machine down the corridor, if a pairing permits asking its models.
    ///
    /// `here` is this machine, which every question from here names as its
    /// sender. `now` is passed rather than read, as it is everywhere a pairing
    /// is asked about, so that *at the moment of asking* is a moment the
    /// caller names.
    ///
    /// # Errors
    ///
    /// [`Miswired::NotPairedWithIt`] when no pairing on this machine permits
    /// asking that machine's models at `now` — which covers a machine that was
    /// merely discovered, one whose pairing has expired, and one whose pairing
    /// was revoked. All three are the same fact about this machine, and there
    /// is no arm that distinguishes them, because telling a caller *which* kind
    /// of not-paired a machine is would be telling them how to become paired.
    pub fn paired(
        pairings: &Pairings,
        here: &MachineId,
        machine: &MachineId,
        called: &'a str,
        at: SocketAddr,
        key: Option<&'a Secret>,
        now: SystemTime,
    ) -> Result<Self, Miswired> {
        let Some(pairing) = pairings.with(machine, now) else {
            return Err(Miswired::NotPairedWithIt);
        };
        if !pairing.permits(MayAskIts::Models, now) {
            return Err(Miswired::NotPairedWithIt);
        }
        Ok(Self {
            called,
            endpoint: format!("http://{at}"),
            key,
            pairing: pairing.clone(),
            here: here.clone(),
            held_to: None,
        })
    }

    /// The same machine, on the network behind the interface the kernel numbers
    /// `interface` — so a question to it connects from a socket held to that
    /// interface, and is registered as a departure held to it.
    ///
    /// [ADR 0042](../../../docs/decisions/0042-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md):
    /// a private IPv4 address is a different machine on each network that hands
    /// it out, and what discovery measured is the address **on the network it
    /// answered on**. `None` is a machine dialled by its address alone, as every
    /// one was before.
    #[must_use]
    pub const fn on_the_network(mut self, interface: Option<NonZeroU32>) -> Self {
        self.held_to = interface;
        self
    }

    /// The interface a question to this machine is held to, if it is held to
    /// one — which is what the boundary around the turn must be shown it on.
    #[must_use]
    pub const fn held_to(&self) -> Option<NonZeroU32> {
        self.held_to
    }

    /// Where an answer from here came from, in the words the rest of the
    /// machine already uses.
    ///
    /// The **name the person gave it**, not its identity and not its address. A
    /// person reading their record wants *the studio machine*, and an identity
    /// would be the machine talking to itself in front of them.
    #[must_use]
    pub fn source(&self) -> InferenceSource {
        InferenceSource::PairedMachine {
            machine: self.called.to_owned(),
        }
    }

    /// The host and port this would connect to, for somebody to resolve and
    /// register before the boundary is entered (ADR 0020).
    #[must_use]
    pub fn where_it_would_connect(&self) -> Option<(String, u16)> {
        alo_models::address::where_it_connects(&self.endpoint)
    }

    /// Put the question down the corridor, with a proof that it is this
    /// machine's, made at `now`.
    ///
    /// `pub(crate)`, and the only caller is
    /// [`crate::Asking::to_a_paired_machine`], which holds an
    /// `alo_egress::Departing` by the time it gets here. That is the whole of
    /// why this is not public: a public method here would be a way to reach
    /// another machine with law 1 having shown nothing.
    ///
    /// # Errors
    ///
    /// [`WentWrong`], as `openai::put` answers it. The machine down the
    /// corridor is asked in the same shape a provider is, because it is running
    /// the same kind of thing and inventing a second shape for it would be two
    /// wire formats to keep true rather than one.
    pub(crate) fn ask(
        &self,
        question: &Question,
        to: &[SocketAddr],
        now: SystemTime,
    ) -> Result<String, WentWrong> {
        let vouching = |body: &[u8]| Proof::made(&self.pairing, &self.here, body, now).said();
        openai::put_on(
            (to, self.held_to),
            &self.endpoint,
            self.key,
            question,
            WHILE_A_MODEL_THINKS,
            Some(&vouching),
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::time::{Duration, SystemTime};

    use alo_nearby::{MachineId, MayAskIts, Pairings};

    use super::DownTheCorridor;
    use crate::refusing::Miswired;
    use crate::testing::paired_as_two_machines;

    /// This machine.
    fn here() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// The one with the GPU in it.
    fn the_studio() -> MachineId {
        MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
    }

    /// A moment to reason from.
    fn a_moment() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// Where the studio machine would answer.
    fn its_address() -> std::net::SocketAddr {
        "192.168.1.20:7610".parse().unwrap()
    }

    /// A pairing two people made, for a day, as this machine keeps it.
    fn paired_for_a_day() -> Pairings {
        let mut pairings = Pairings::none();
        pairings
            .keep(paired_as_two_machines(here(), the_studio(), &[MayAskIts::Models], a_moment()).0);
        pairings
    }

    /// **An unpaired machine offering inference is not used**, however
    /// convenient. There is no constructor that skips the pairing.
    #[test]
    fn an_unpaired_machine_offering_inference_is_not_used() {
        let refused = DownTheCorridor::paired(
            &Pairings::none(),
            &here(),
            &the_studio(),
            "the studio machine",
            its_address(),
            None,
            a_moment(),
        )
        .unwrap_err();
        assert!(matches!(refused, Miswired::NotPairedWithIt));
    }

    /// A pairing that has ended permits nothing, which is the same refusal:
    /// this machine is not paired with that one *now*.
    #[test]
    fn a_pairing_that_has_ended_does_not_open_the_door() {
        let a_week_later = a_moment() + Duration::from_secs(7 * 86_400);
        let refused = DownTheCorridor::paired(
            &paired_for_a_day(),
            &here(),
            &the_studio(),
            "the studio machine",
            its_address(),
            None,
            a_week_later,
        )
        .unwrap_err();
        assert!(matches!(refused, Miswired::NotPairedWithIt));
    }

    /// A pairing for the workspace and not for the models does not open this
    /// door either — the list is what it says.
    #[test]
    fn a_pairing_for_something_else_does_not_open_this_door() {
        let mut pairings = Pairings::none();
        pairings.keep(
            paired_as_two_machines(here(), the_studio(), &[MayAskIts::Workspace], a_moment()).0,
        );

        let refused = DownTheCorridor::paired(
            &pairings,
            &here(),
            &the_studio(),
            "the studio machine",
            its_address(),
            None,
            a_moment(),
        )
        .unwrap_err();
        assert!(matches!(refused, Miswired::NotPairedWithIt));
    }

    /// **It says where an answer came from by the name the person gave it**,
    /// not by an identity and not by an address.
    #[test]
    fn an_answer_from_here_names_the_machine_the_person_named() {
        let corridor = DownTheCorridor::paired(
            &paired_for_a_day(),
            &here(),
            &the_studio(),
            "the studio machine",
            its_address(),
            None,
            a_moment(),
        )
        .unwrap();
        assert_eq!(
            corridor.source(),
            alo_models::InferenceSource::PairedMachine {
                machine: "the studio machine".to_owned()
            }
        );
        assert!(
            corridor.source().causes_egress(),
            "a question down the corridor stopped being an egress"
        );
        assert!(corridor.source().stays_in_the_building());
    }

    /// Where it would connect is answerable before anything is asked, which is
    /// what ADR 0020 needs of every door in this crate.
    #[test]
    fn where_it_would_connect_is_known_before_anything_is_asked() {
        let corridor = DownTheCorridor::paired(
            &paired_for_a_day(),
            &here(),
            &the_studio(),
            "the studio machine",
            its_address(),
            None,
            a_moment(),
        )
        .unwrap();
        assert_eq!(
            corridor.where_it_would_connect(),
            Some(("192.168.1.20".to_owned(), 7_610))
        );
    }
}
