//! What this crate says survives being put beside everybody else's.
//!
//! The crate's own list is tested where it is declared; this asks the one
//! question that list cannot answer — whether `alo-saying` collects it at
//! all. Task 3 records a crate declaring nine strings that reached nothing,
//! and `alo-collected` now refuses that shape repository-wide; this is the
//! same fact measured from this crate's side, so the failure names the crate
//! whose words went missing.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

/// **Everything this crate says is in the machine's one vocabulary** — the
/// vocabulary a translation is checked against, and the one every sentence a
/// person reads comes out of.
#[test]
fn everything_this_crate_says_is_collected_by_the_machine() {
    let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    for word in alo_changing::words::EVERY_WORD {
        let key = word.key();
        assert!(
            vocabulary.phrase(&key).is_some(),
            "{} is not collected: the machine cannot say it",
            word.named()
        );
    }
}
