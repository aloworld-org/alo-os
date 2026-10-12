//! What this crate says survives being put beside everybody else's.
//!
//! The crate's own list is tested where it is declared; this asks the one
//! question that list cannot answer — **whether `alo-saying` collects it at
//! all.** A crate can declare nine perfectly good strings that reach nothing,
//! and this repository has done exactly that before; measuring it from this
//! side means the failure names the crate whose words went missing rather than
//! a total that is short by nine.
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

    for word in alo_tasks::words::EVERY_WORD {
        let key = word.key();
        assert!(
            vocabulary.phrase(&key).is_some(),
            "{} is not collected: the machine cannot say it",
            word.named()
        );
    }
}

/// **And a window's own words are not the person's.**
///
/// A task is named by what somebody asked for, and this crate must never hold
/// a sentence that stands in for that. The check is that nothing here says
/// anything about a *particular* task — no example request, no file name — so
/// that the design's illustrative content cannot leak into the shipped
/// vocabulary and be translated as though it were a label.
#[test]
fn nothing_this_crate_says_pretends_to_be_a_persons_own_words() {
    for word in alo_tasks::words::EVERY_WORD {
        let says = word.says().to_lowercase();
        for example in [
            "launch presentation",
            "invoices",
            "launch brief",
            "product notes",
            "six-slide",
            ".pdf",
            ".odp",
            ".md",
        ] {
            assert!(
                !says.contains(example),
                "{} says `{}`, which is the design's example content rather than \
                 a word this machine owns",
                word.named(),
                word.says()
            );
        }
    }
}
