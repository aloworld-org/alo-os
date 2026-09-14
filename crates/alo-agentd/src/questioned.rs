//! A question from a paired machine, at the door.
//!
//! `alo-asking`'s corridor puts a question to a paired machine at
//! [`alo_asking::THE_QUESTION_PATH`] on the port presence advertises, with a
//! proof in [`alo_asking::THE_PROOF_HEADER`]; task 5 measured the day with a
//! studio that checked the proof and answered from a fixture. This is the
//! daemon's door for it, and it judges a question in the order every door on
//! this port judges everything: **the proof first**, through the one
//! `alo_corridor::Doorway` the verb wire uses — so a proof spent on a
//! question is refused as a verb and the other way round — then **the
//! pairing's own list**: a pairing that does not permit asking this machine's
//! models (`alo_nearby::MayAskIts::Models`) permits nothing here, however
//! good the proof.
//!
//! # And then it is not answered, and says so
//!
//! What answers a proven, permitted question is this machine's own model,
//! recorded as `alo_record::Entry::answered_for` and leaving under a
//! departure — and that is a door `alo_turn::Machine` does not have yet:
//! `alo_turn::Arriving` says in its own header that a remote turn puts no
//! question, so a remote question has to be a door of the machine's, and
//! which of the person's models may answer for another machine is a setting
//! the person has not been given (ADR 0008). Both are decisions for the crates
//! that own them rather than side effects of the daemon binding a port, so the
//! plan names them as the task after this one, and until then a proven
//! question is answered [`NOT_ANSWERED_HERE`] with nothing written — nothing
//! happened on this machine — and the asking machine's person is told the
//! machine down the corridor could not answer, which is true.
//!
//! **Nothing here is a fallback.** A question this machine cannot answer is
//! not put anywhere else, and a question from a machine that is not paired,
//! or whose pairing does not say *models*, is refused before anybody knows
//! what it asked.

use std::io::Write as _;
use std::net::TcpStream;
use std::time::SystemTime;

use alo_asking::THE_PROOF_HEADER;
use alo_corridor::{AtTheDoor, Doorway, Naming, Replying};
use alo_nearby::http::{self, Message};
use alo_nearby::{MachineId, MayAskIts, NotNearby, NotProven, Pairings, Proof};

/// What the wire says for a question it proved and did not answer.
pub const NOT_ANSWERED_HERE: &str = "not-answered-here";

/// What the wire says for a question from a pairing that does not permit
/// asking this machine's models.
pub const NOT_PERMITTED: &str = "not-permitted";

/// What became of a question at the door.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Questioned {
    /// It carried no proof, and was refused before anything was asked.
    NoProof,
    /// Its proof did not hold, and was refused before anything was asked.
    NotProven(NotProven),
    /// It proved where it came from, and that pairing does not permit asking
    /// this machine's models.
    NotPermitted(MachineId),
    /// It proved where it came from, the pairing permits it, and this machine
    /// has no door to answer it through yet.
    NotAnsweredHere(MachineId),
}

/// Judge one question and answer it, at `now`.
///
/// `pairings` is this machine's at the moment; `doorway` holds the proofs
/// seen on this port; `naming` is what the person here called the other
/// machines and decides nothing.
///
/// # Errors
///
/// [`NotNearby::TheNetwork`] if the reply could not be written; what was
/// decided has been decided, and nothing on this machine changed for it.
pub fn answered(
    mut stream: TcpStream,
    message: &Message,
    doorway: &mut Doorway<'_, '_>,
    pairings: &Pairings,
    naming: &dyn Naming,
    now: SystemTime,
) -> Result<Questioned, NotNearby> {
    let (questioned, written) = judged(message, doorway, pairings, naming, now);
    stream
        .write_all(written.as_bytes())
        .map_err(|why| NotNearby::TheNetwork(why.to_string()))?;
    Ok(questioned)
}

/// The judgement, and the bytes that go back for it, with no socket in
/// sight.
fn judged(
    message: &Message,
    doorway: &mut Doorway<'_, '_>,
    pairings: &Pairings,
    naming: &dyn Naming,
    now: SystemTime,
) -> (Questioned, String) {
    let proof = match message.header(THE_PROOF_HEADER).map(Proof::read) {
        None | Some(Err(_)) => {
            return (
                Questioned::NoProof,
                Replying::before_the_door(AtTheDoor::NoProof).written(),
            );
        }
        Some(Ok(proof)) => proof,
    };
    let origin = match doorway.proven(&proof, &message.body, pairings, naming, now) {
        Ok(origin) => origin,
        Err(why) => {
            return (
                Questioned::NotProven(why),
                Replying::before_the_door(AtTheDoor::NotProven(why)).written(),
            );
        }
    };
    let from = origin.machine().clone();
    if !pairings.permits(&from, MayAskIts::Models, now) {
        return (
            Questioned::NotPermitted(from),
            http::a_reply(403, "Forbidden", &format!("{NOT_PERMITTED}\n")),
        );
    }
    (
        Questioned::NotAnsweredHere(from),
        http::a_reply(
            503,
            "Service Unavailable",
            &format!("{NOT_ANSWERED_HERE}\n"),
        ),
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use alo_asking::{THE_PROOF_HEADER, THE_QUESTION_PATH};
    use alo_corridor::Doorway;
    use alo_egress::Indicator;
    use alo_files::OnThisMachine;
    use alo_nearby::http::Message;
    use alo_nearby::{MayAskIts, NotProven, Pairing, Pairings, Proof};
    use alo_record::Record;
    use alo_turn::Machine;

    use super::{Questioned, judged};
    use crate::terms::NoNameYet;
    use crate::testing::{
        NothingIsBounded, in_english, noon, paired_between, reception, the_studio,
    };

    /// A question as the corridor puts one, with these headers.
    fn a_question(headers: Vec<(String, String)>) -> Message {
        Message {
            first: format!("POST {THE_QUESTION_PATH} HTTP/1.1"),
            headers,
            body: r#"{"model":"m","messages":[{"role":"user","content":"hello"}]}"#.to_owned(),
        }
    }

    /// The same question, proven by reception at this moment.
    fn proven_by(on_reception: &Pairing, at: std::time::SystemTime) -> Message {
        let unproven = a_question(Vec::new());
        let proof = Proof::made(on_reception, &reception(), unproven.body.as_bytes(), at);
        a_question(vec![(THE_PROOF_HEADER.to_owned(), proof.said())])
    }

    /// The studio, with a doorway on it, judging what `doing` puts through
    /// and handing back its record afterwards.
    fn on_the_studio(
        doing: impl FnOnce(&mut Doorway<'_, '_>) -> Vec<(Questioned, String)>,
    ) -> (Vec<(Questioned, String)>, usize) {
        let strings = in_english();
        let mut indicator = Indicator::default();
        let mut bounding = NothingIsBounded;
        let mut record = Record::default();
        let judged = {
            let mut machine = Machine::carrying_out_file_verbs(
                &strings,
                &OnThisMachine,
                &mut bounding,
                &mut indicator,
                &mut record,
            )
            .unwrap();
            let mut doorway = Doorway::at(
                the_studio(),
                &mut machine,
                Duration::from_secs(3_600),
                Duration::from_secs(3_600),
            )
            .unwrap();
            doing(&mut doorway)
        };
        (judged, record.len())
    }

    /// **The proof is judged first and a question without one, or with one
    /// that does not hold, is refused before anything is asked** — and a
    /// proof that held once is a replay the second time, through the same
    /// memory the verb wire refuses replays with. Nothing is written.
    #[test]
    fn a_question_is_proven_before_anything_else_and_a_replay_is_refused() {
        let (on_reception, on_studio) =
            paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        let mut pairings = Pairings::none();
        pairings.keep(on_studio);
        let (judged, written) = on_the_studio(|doorway| {
            let first = judged(
                &a_question(Vec::new()),
                doorway,
                &pairings,
                &NoNameYet,
                noon(),
            );
            let proven = proven_by(&on_reception, noon());
            let second = judged(&proven, doorway, &pairings, &NoNameYet, noon());
            let replayed = judged(
                &proven,
                doorway,
                &pairings,
                &NoNameYet,
                noon() + Duration::from_secs(1),
            );
            vec![first, second, replayed]
        });
        assert_eq!(judged.first().unwrap().0, Questioned::NoProof);
        assert!(judged.first().unwrap().1.starts_with("HTTP/1.1 4"));
        assert_eq!(
            judged.get(1).unwrap().0,
            Questioned::NotAnsweredHere(reception())
        );
        assert!(judged.get(1).unwrap().1.contains("not-answered-here"));
        assert_eq!(
            judged.get(2).unwrap().0,
            Questioned::NotProven(NotProven::AlreadySeen)
        );
        assert_eq!(written, 0, "a question wrote something down");
    }

    /// **A pairing that does not say models permits no question**, however
    /// good the proof — the list is what it says.
    #[test]
    fn a_pairing_for_something_else_does_not_open_this_door() {
        let (on_reception, on_studio) =
            paired_between(reception(), the_studio(), &[MayAskIts::Workspace], noon());
        let mut pairings = Pairings::none();
        pairings.keep(on_studio);
        let (judged, written) = on_the_studio(|doorway| {
            vec![judged(
                &proven_by(&on_reception, noon()),
                doorway,
                &pairings,
                &NoNameYet,
                noon(),
            )]
        });
        assert_eq!(
            judged.first().unwrap().0,
            Questioned::NotPermitted(reception())
        );
        assert!(judged.first().unwrap().1.starts_with("HTTP/1.1 403"));
        assert_eq!(written, 0);
    }

    /// **An unpaired machine's question is refused before anybody knows what
    /// it asked**: the proof names a pairing this machine does not hold.
    #[test]
    fn an_unpaired_machines_question_is_refused_at_the_proof() {
        let (on_reception, _) =
            paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        let (judged, written) = on_the_studio(|doorway| {
            vec![judged(
                &proven_by(&on_reception, noon()),
                doorway,
                &Pairings::none(),
                &NoNameYet,
                noon(),
            )]
        });
        assert_eq!(
            judged.first().unwrap().0,
            Questioned::NotProven(NotProven::NotWithThatMachine)
        );
        assert_eq!(written, 0);
    }
}
