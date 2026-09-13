//! Two machines, paired the way two machines would be, for the tests in this
//! crate that need both sides of one pairing.
//!
//! Compiled only under `cfg(test)`: nothing here ships, and nothing here is a
//! second constructor for a pairing — it walks the one road `deliberating.rs`
//! offers, on both sides, so that a test holding both rows holds what two real
//! machines would.

use std::time::{Duration, SystemTime};

use crate::deliberating::{Deliberating, Proposal, Side};
use crate::keying::Keying;
use crate::machine::MachineId;
use crate::pairing::Pairing;
use crate::permitting::MayAskIts;

/// The machine asking.
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
pub(crate) fn reception() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// The machine asked.
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
pub(crate) fn studio() -> MachineId {
    MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
}

/// A machine nobody paired with.
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
pub(crate) fn a_stranger() -> MachineId {
    MachineId::read("99998888777766665555444433332222").unwrap()
}

/// A moment to reason from.
pub(crate) fn a_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::new(1_760_000_000, 123_456_789)
}

/// Reception and the studio, paired for a day for the studio's models, as each
/// side keeps it: reception's row first, the studio's second.
pub(crate) fn paired() -> (Pairing, Pairing) {
    paired_between(reception(), studio())
}

/// Reception proposing to the studio, for a day, for the studio's models,
/// with the keying whose offer the proposal carries.
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
pub(crate) fn a_proposal() -> (Proposal, Keying) {
    let keying = Keying::fresh().unwrap();
    let proposal = Proposal::checked(
        reception(),
        studio(),
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
        keying.offer().clone(),
    )
    .unwrap();
    (proposal, keying)
}

/// Both sides of one proposal from reception to the studio, each holding the
/// other's offer and nobody having agreed yet: reception's first, the
/// studio's second.
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
pub(crate) fn both_sides() -> (Deliberating, Deliberating) {
    let (proposal, at_reception) = a_proposal();
    let at_studio = Deliberating::asked(proposal.clone(), Keying::fresh().unwrap());
    let at_reception = Deliberating::asking(proposal, at_reception)
        .unwrap()
        .answered_with(at_studio.answered().unwrap().clone())
        .unwrap();
    (at_reception, at_studio)
}

/// Any two machines, paired for a day for the asked one's models, as each side
/// keeps it: the asking machine's row first, the asked machine's second.
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
pub(crate) fn paired_between(asking: MachineId, asked: MachineId) -> (Pairing, Pairing) {
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
