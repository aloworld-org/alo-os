//! *The machine that asks is the machine that paired.*
//!
//! [ADR 0031]: a pairing leaves each machine holding a key nobody else has, a
//! message from a paired machine carries a proof made with it, and the
//! receiving machine judges the proof before it asks anything else of itself.
//! This file holds task 6's acceptance from outside the crate, and — as every
//! file in this plan does — carries the refusals beside the agreement, each
//! under its own name.
//!
//! | The acceptance | The test |
//! |---|---|
//! | a pairing leaves each machine holding something made when both confirmed and held by nobody else | [`a_pairing_leaves_each_machine_holding_the_same_key_and_nobody_else_holding_it`] |
//! | being discovered confers nothing | [`being_discovered_confers_no_key`] |
//! | sharing the network confers nothing | [`sharing_the_network_and_reading_both_offers_confers_no_key`] |
//! | having read a `MachineId` off the wire confers nothing | [`a_stranger_presenting_a_paired_machines_identity_is_refused_before_any_grant_is_asked`] |
//! | a message is accepted only when it proves it comes from the identity the pairing names | [`a_message_is_accepted_only_when_its_proof_holds`] |
//! | a proof replayed from an earlier exchange is refused | [`a_proof_replayed_from_an_earlier_exchange_is_refused`] |
//! | a proof from a pairing since revoked or expired is refused at the moment | [`a_proof_from_a_pairing_since_revoked_or_expired_is_refused_at_the_moment`] |
//! | `Found` carries the address a machine answered from | [`what_is_found_carries_the_address_it_answered_from`] |
//!
//! [ADR 0031]: https://github.com/aloworld-org/alo-os/blob/main/docs/decisions/0031-the-pairing-is-the-key.md

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};

use alo_nearby::{
    Answering, Deliberating, Found, Keying, Looking, MachineId, MayAskIts, NotProven, Origin,
    Pairing, Pairings, Presence, Proof, Proposal, Proven, Seen, Side, WHILE_A_PROOF_STANDS,
    nearby_words,
};
use alo_strings::Strings;

/// The machine at reception, with no model of its own.
fn reception() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// The studio machine, with the GPU in it.
fn studio() -> MachineId {
    MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
}

/// Somebody on the office WiFi who nobody paired with.
fn a_stranger() -> MachineId {
    MachineId::read("99998888777766665555444433332222").unwrap()
}

/// A moment to reason from.
fn a_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// Two machines paired the way two machines are: the asking one's row first,
/// the asked one's second, and the same key on both.
fn paired_between(asking: MachineId, asked: MachineId) -> (Pairing, Pairing) {
    let at_asking = Keying::fresh().unwrap();
    let at_asked = Keying::fresh().unwrap();
    let proposal = Proposal::checked(
        asking,
        asked,
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
        at_asking.offer().clone(),
    )
    .unwrap();
    let asked_side = Deliberating::asked(proposal.clone(), at_asked);
    let asking_side = Deliberating::asking(proposal, at_asking)
        .unwrap()
        .answered_with(asked_side.answered().unwrap().clone())
        .unwrap();
    // Both people read the same six digits before either says yes.
    assert_eq!(asking_side.code(), asked_side.code());
    let on_asking = asking_side
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(a_moment())
        .unwrap();
    let on_asked = asked_side
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(a_moment())
        .unwrap();
    (on_asking, on_asked)
}

/// The studio's pairings, holding its row about reception, and reception's
/// own row about the studio.
fn at_the_studio() -> (Pairings, Pairing) {
    let (on_reception, on_studio) = paired_between(reception(), studio());
    let mut pairings = Pairings::none();
    pairings.keep(on_studio);
    (pairings, on_reception)
}

/// The studio judging one message, with nothing remembered beforehand.
fn judged(
    pairings: &Pairings,
    proof: &Proof,
    about: &[u8],
    now: SystemTime,
) -> Result<Proven, NotProven> {
    Proven::checked(pairings, &studio(), proof, about, now, &mut Seen::nothing())
}

/// **Each machine holds the same key, made when both confirmed, and nobody
/// else holds it.** A proof reception makes verifies at the studio; one the
/// studio makes verifies at reception; and the same agreement walked by a
/// third machine — even one that read both offers — verifies at neither.
#[test]
fn a_pairing_leaves_each_machine_holding_the_same_key_and_nobody_else_holding_it() {
    let (on_reception, on_studio) = paired_between(reception(), studio());
    let mut at_reception = Pairings::none();
    at_reception.keep(on_reception.clone());
    let mut at_studio = Pairings::none();
    at_studio.keep(on_studio.clone());

    let from_reception = Proof::made(&on_reception, &reception(), b"a question", a_moment());
    assert!(judged(&at_studio, &from_reception, b"a question", a_moment()).is_ok());

    let from_studio = Proof::made(&on_studio, &studio(), b"an answer", a_moment());
    assert!(
        Proven::checked(
            &at_reception,
            &reception(),
            &from_studio,
            b"an answer",
            a_moment(),
            &mut Seen::nothing()
        )
        .is_ok()
    );

    // And a pairing of the same two machines made again is a different key:
    // what is held was made at the moment both confirmed, not derived from
    // who they are.
    let (again, _) = paired_between(reception(), studio());
    let with_the_other_key = Proof::made(&again, &reception(), b"a question", a_moment());
    assert_eq!(
        judged(&at_studio, &with_the_other_key, b"a question", a_moment()).unwrap_err(),
        NotProven::NotFromThatMachine
    );
}

/// **Being discovered confers no key.** A machine the studio found — and that
/// found the studio — has nothing a proof can be made with, and the studio has
/// nothing to check one against.
#[test]
fn being_discovered_confers_no_key() {
    let (at_studio, _) = at_the_studio();
    // The stranger was found, and found the studio, and holds a pairing of
    // its own with some third machine — so it has a key, just not this one.
    let (strangers_row, _) = paired_between(a_stranger(), studio());
    let found = Found::seen(a_stranger(), 7_610, Ipv4Addr::new(192, 168, 1, 40).into());
    assert!(!at_studio.paired_with(&found.machine, a_moment()));

    let its_own = Proof::made(&strangers_row, &a_stranger(), b"a question", a_moment());
    assert_eq!(
        judged(&at_studio, &its_own, b"a question", a_moment()).unwrap_err(),
        NotProven::NotWithThatMachine
    );
}

/// **Sharing the network confers no key**, even for a machine that read both
/// offers off it while the pairing was made: the offers are public by design
/// and the key is not derivable from them.
#[test]
fn sharing_the_network_and_reading_both_offers_confers_no_key() {
    let at_reception = Keying::fresh().unwrap();
    let at_studio = Keying::fresh().unwrap();
    let proposal = Proposal::checked(
        reception(),
        studio(),
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
        at_reception.offer().clone(),
    )
    .unwrap();
    let studio_side = Deliberating::asked(proposal.clone(), at_studio);
    // What crossed the wire, and what the stranger therefore read.
    let reception_offer = proposal.offered().clone();
    let studio_offer = studio_side.answered().unwrap().clone();

    let reception_side = Deliberating::asking(proposal.clone(), at_reception)
        .unwrap()
        .answered_with(studio_offer.clone())
        .unwrap();
    let on_reception = reception_side
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(a_moment())
        .unwrap();
    let mut pairings = Pairings::none();
    pairings.keep(
        studio_side
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(a_moment())
            .unwrap(),
    );

    // The stranger walks the same agreement with both public halves and a
    // keying of its own, pretending to be reception.
    let strangers_keying = Keying::fresh().unwrap();
    let pretending = Proposal::checked(
        reception(),
        studio(),
        proposal.may(),
        proposal.lasting(),
        strangers_keying.offer().clone(),
    )
    .unwrap();
    let as_if_reception = Deliberating::asking(pretending, strangers_keying)
        .unwrap()
        .answered_with(studio_offer)
        .unwrap()
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(a_moment())
        .unwrap();
    let forged = Proof::made(&as_if_reception, &reception(), b"a question", a_moment());
    assert_eq!(
        judged(&pairings, &forged, b"a question", a_moment()).unwrap_err(),
        NotProven::NotFromThatMachine
    );
    // And neither offer alone is a key: a keying answered with reception's
    // own offer is refused as reflected before any key is made.
    assert!(
        Deliberating::asking(
            Proposal::checked(
                reception(),
                studio(),
                proposal.may(),
                proposal.lasting(),
                reception_offer.clone(),
            )
            .unwrap(),
            Keying::fresh().unwrap()
        )
        .is_err()
    );
    // While the real pairing goes on working.
    let real = Proof::made(&on_reception, &reception(), b"a question", a_moment());
    assert!(judged(&pairings, &real, b"a question", a_moment()).is_ok());
}

/// **A stranger presenting reception's `MachineId` is refused before any grant
/// is asked.** The identity is read off a discovery packet by design, and
/// [`Origin`] — the only thing a remote turn can begin from — has no
/// constructor that takes one without a proof.
#[test]
fn a_stranger_presenting_a_paired_machines_identity_is_refused_before_any_grant_is_asked() {
    let (at_studio, _) = at_the_studio();
    let (strangers_row, _) = paired_between(a_stranger(), studio());
    let presenting = Proof::made(&strangers_row, &reception(), b"list_folder", a_moment());
    assert_eq!(presenting.from(), &reception());

    let refused = Origin::proven(
        &at_studio,
        &studio(),
        &presenting,
        b"list_folder",
        "the reception machine",
        a_moment(),
        &mut Seen::nothing(),
    )
    .unwrap_err();
    assert_eq!(refused, NotProven::NotFromThatMachine);

    let strings = Strings::of(nearby_words().unwrap());
    let said = refused.said(&strings);
    assert!(said.text().contains("could not prove"), "{said}");
    assert!(
        said.text().contains("nothing it asked for was considered"),
        "{said}"
    );
}

/// **A message is accepted only when its proof holds** — over exactly the
/// bytes that arrived, for this machine, from the pairing's own key.
#[test]
fn a_message_is_accepted_only_when_its_proof_holds() {
    let (at_studio, on_reception) = at_the_studio();
    let proof = Proof::made(
        &on_reception,
        &reception(),
        b"may the tenant sublet?",
        a_moment(),
    );

    let proven = judged(&at_studio, &proof, b"may the tenant sublet?", a_moment()).unwrap();
    assert_eq!(proven.machine(), &reception());

    // The same proof over altered bytes.
    assert_eq!(
        judged(
            &at_studio,
            &proof,
            b"may the tenant keep a dog?",
            a_moment()
        )
        .unwrap_err(),
        NotProven::NotFromThatMachine
    );
    // The same proof at a machine it was not made for.
    assert_eq!(
        Proven::checked(
            &at_studio,
            &a_stranger(),
            &proof,
            b"may the tenant sublet?",
            a_moment(),
            &mut Seen::nothing()
        )
        .unwrap_err(),
        NotProven::NotForThisMachine
    );
    // And the same proof, read back off the wire, still holds.
    let travelled = Proof::read(&proof.said()).unwrap();
    assert!(
        judged(
            &at_studio,
            &travelled,
            b"may the tenant sublet?",
            a_moment()
        )
        .is_ok()
    );
}

/// **A proof replayed from an earlier exchange is refused**, however soon
/// after, and a fresh proof over the same bytes is not.
#[test]
fn a_proof_replayed_from_an_earlier_exchange_is_refused() {
    let (at_studio, on_reception) = at_the_studio();
    let mut seen = Seen::nothing();
    let proof = Proof::made(&on_reception, &reception(), b"a question", a_moment());
    assert!(
        Proven::checked(
            &at_studio,
            &studio(),
            &proof,
            b"a question",
            a_moment(),
            &mut seen
        )
        .is_ok()
    );

    let replayed = Proof::read(&proof.said()).unwrap();
    let a_little_later = a_moment() + Duration::from_secs(10);
    assert_eq!(
        Proven::checked(
            &at_studio,
            &studio(),
            &replayed,
            b"a question",
            a_little_later,
            &mut seen
        )
        .unwrap_err(),
        NotProven::AlreadySeen
    );

    let fresh = Proof::made(&on_reception, &reception(), b"a question", a_little_later);
    assert!(
        Proven::checked(
            &at_studio,
            &studio(),
            &fresh,
            b"a question",
            a_little_later,
            &mut seen
        )
        .is_ok()
    );
}

/// **A proof from a pairing since revoked or expired is refused at the
/// moment**, not at the next restart: the key is on the row, and the row is
/// gone.
#[test]
fn a_proof_from_a_pairing_since_revoked_or_expired_is_refused_at_the_moment() {
    let (mut at_studio, on_reception) = at_the_studio();
    let before = Proof::made(&on_reception, &reception(), b"a question", a_moment());
    assert!(judged(&at_studio, &before, b"a question", a_moment()).is_ok());

    // Expired: a day later, the same key and a fresh proof.
    let a_day_later = a_moment() + Duration::from_secs(86_400);
    let after_the_end = Proof::made(&on_reception, &reception(), b"a question", a_day_later);
    assert_eq!(
        judged(&at_studio, &after_the_end, b"a question", a_day_later).unwrap_err(),
        NotProven::NotWithThatMachine
    );

    // Revoked, in one action, and the very next proof is refused.
    assert!(at_studio.revoke(&reception()));
    let the_next_moment = a_moment() + Duration::from_secs(1);
    let after_revoking = Proof::made(&on_reception, &reception(), b"a question", the_next_moment);
    assert_eq!(
        judged(&at_studio, &after_revoking, b"a question", the_next_moment).unwrap_err(),
        NotProven::NotWithThatMachine
    );
}

/// A proof from a clock more than the window away is refused, and the
/// sentence tells the person what to check.
#[test]
fn a_proof_from_another_moment_is_refused_and_says_what_to_check() {
    let (at_studio, on_reception) = at_the_studio();
    let proof = Proof::made(&on_reception, &reception(), b"a question", a_moment());
    let refused = judged(
        &at_studio,
        &proof,
        b"a question",
        a_moment() + WHILE_A_PROOF_STANDS + Duration::from_secs(1),
    )
    .unwrap_err();
    assert_eq!(refused, NotProven::NotNow);
    let strings = Strings::of(nearby_words().unwrap());
    assert!(refused.said(&strings).text().contains("clocks"));
}

/// **What is found carries the address it answered from**, so the next hop is
/// dialled from what discovery measured rather than from what a test typed.
#[test]
fn what_is_found_carries_the_address_it_answered_from() {
    let there = UdpSocket::bind("127.0.0.1:0").unwrap();
    there
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let at = there.local_addr().unwrap();
    let answering = Answering::on(there, Presence::of(studio(), 7_610));
    let answered = std::thread::spawn(move || answering.answer_one());

    let here = UdpSocket::bind("127.0.0.1:0").unwrap();
    here.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let looking = Looking::from(here);
    looking.ask(at).unwrap();
    let found = looking.found(Duration::from_secs(5)).unwrap();
    assert!(answered.join().unwrap().unwrap().is_some());

    let one = found.first().unwrap();
    assert_eq!(one.address, at.ip());
    assert_eq!(one.where_it_answers(), SocketAddr::new(at.ip(), 7_610));
}
