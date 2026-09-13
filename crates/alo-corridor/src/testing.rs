//! Two machines, paired the way two machines would be, and a receiving
//! machine with a folder of its own, for the tests in this crate.
//!
//! Compiled only under `cfg(test)`: nothing here ships, and nothing here is
//! a second constructor for a pairing — it walks the one road
//! `alo_nearby::Deliberating` offers, on both sides, so that a test holding
//! both rows holds what two real machines would.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::Given;
use alo_files::{OnThisMachine, Reaching, Resolving as _};
use alo_nearby::{Deliberating, Keying, MachineId, MayAskIts, Pairing, Proposal, Side};
use alo_strings::Strings;
use alo_turn::{Bounding, Doing, Done, NoBoundary};

/// The machine asking.
pub(crate) fn reception() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// The machine asked.
pub(crate) fn studio() -> MachineId {
    MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
}

/// A moment to reason from.
pub(crate) fn a_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// Any two machines, paired for a day for the asked one's models, as each
/// side keeps it: the asking machine's row first, the asked machine's second.
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

/// What the studio's person called reception, and nothing else.
pub(crate) fn naming() -> impl Fn(&MachineId) -> Option<String> {
    |machine: &MachineId| (*machine == reception()).then(|| "the reception machine".to_owned())
}

/// The words a machine on either side of the corridor reads.
///
/// Every crate's list a refusal met at the door can be worded by, so that a
/// missing string is a failing test and not a passing one.
pub(crate) fn in_english() -> Strings {
    let mut vocabulary = alo_files::file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_keeping::declare_into(&mut vocabulary).unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_answering::declare_into(&mut vocabulary).unwrap();
    alo_asking::declare_into(&mut vocabulary).unwrap();
    alo_nearby::words::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    crate::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A machine with nothing in front of a turn, which is not a machine alo OS
/// ships — `alo_turn::bounding` says why no library here has one, and why a
/// test writes these lines where its reader can see them.
#[derive(Debug)]
pub(crate) struct NothingIsBounded;

impl Bounding for NothingIsBounded {
    fn carrying_out(&mut self, _reaching: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary> {
        Ok(doing.done())
    }
}

/// A folder of this test's own with one file in it, both resolved — a grant
/// is over a place, and on Windows a resolved path carries a prefix the
/// typed one does not (`docs/quirks.md`).
pub(crate) fn a_folder_with_an_invoice(what: &str) -> (PathBuf, PathBuf) {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-corridor-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    let folder = OnThisMachine.real(&folder).unwrap().into_path_buf();
    let invoice = folder.join("march.pdf");
    fs::write(&invoice, "March, 4180.00").unwrap();
    (folder, invoice)
}

/// A path as a verb's argument arrives: text.
pub(crate) fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}
