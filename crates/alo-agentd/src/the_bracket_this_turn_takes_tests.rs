//! The per-turn bracket is held to the things this shape gets wrong: a session
//! with no home treated as a machine that cannot snapshot, a bracket taken
//! twice, the second sentence kept instead of the first, and a turn that
//! changed nothing leaving a directory behind.
//!
//! **No real snapshot is made here.** A temporary directory is not a subvolume,
//! so every bracket in this file is *not taken* — which is the case these tests
//! are about. That it really is taken on a real filesystem is
//! `alo-turn`'s `tests/a_bracket_on_a_real_btrfs_machine.rs`, run as root.

use super::*;
use std::ffi::OsString;

/// Noon, for a turn that began at a knowable moment.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000)
}

/// Where brackets would go, under a temporary folder.
fn somewhere(under: &Path, home: &Path) -> WhereBracketsGo {
    WhereBracketsGo::of_a_session(under, 1000, Some(&OsString::from(home)))
        .expect("an absolute home is a session this can serve")
}

/// **A session with a home has somewhere to put a bracket.**
#[test]
fn a_session_with_a_home_has_somewhere_for_a_bracket() {
    let under = Path::new("/var/lib/alo-undo");
    let goes = somewhere(under, Path::new("/home/ada"));

    assert_eq!(goes.whose(), under.join("1000"));
    assert_eq!(goes.home(), Path::new("/home/ada"));
}

/// **A session with no home, or a relative one, has nowhere** — and that is a
/// person's state rather than a fault.
///
/// A relative `$HOME` would be followed from wherever the unit happened to
/// start, which is the same rule `alo-kept-undo` holds a settings folder to.
#[test]
fn a_session_with_no_absolute_home_has_nowhere() {
    let under = Path::new("/var/lib/alo-undo");
    assert!(WhereBracketsGo::of_a_session(under, 1000, None).is_none());
    assert!(
        WhereBracketsGo::of_a_session(under, 1000, Some(&OsString::from("home/ada"))).is_none(),
        "a relative home was accepted"
    );
    assert!(
        WhereBracketsGo::of_a_session(under, 1000, Some(&OsString::from(""))).is_none(),
        "an empty home was accepted"
    );
}

/// **A session with nowhere to snapshot says so, and does not say *not this
/// turn*.**
///
/// The two are different sentences to a person: *nothing on this machine keeps
/// what your files were* against *this one turn could not be kept*. A session
/// with no home is the first, and reporting the second would blame the turn for
/// the machine.
#[test]
fn a_session_with_nowhere_says_nothing_on_this_machine() {
    let mut bracket = TheTurnsBracket::none_yet(None, noon(), 42, 1);
    bracket.before_a_change("archive march's invoices");

    assert_eq!(
        bracket.what_this_turn_kept(),
        WhatThisTurnKept::NowhereToKeepIt
    );
    assert!(bracket.taken_at().is_none());
}

/// **A machine that could snapshot and did not says *not this turn***, which is
/// ADR 0045's third term.
#[test]
fn a_bracket_that_could_not_be_taken_says_not_this_turn() {
    let under = tempfile::tempdir().expect("a temporary folder");
    let home = tempfile::tempdir().expect("a temporary home");
    let goes = somewhere(under.path(), home.path());

    let mut bracket = TheTurnsBracket::none_yet(Some(goes), noon(), 42, 1);
    bracket.before_a_change("archive march's invoices");

    assert_eq!(
        bracket.what_this_turn_kept(),
        WhatThisTurnKept::NotThisTurn,
        "a temporary directory was snapshotted, which this test is not about"
    );
    assert!(
        bracket.why_not().is_some(),
        "nothing said why the bracket was not taken"
    );
}

/// **The first sentence is the one kept**, not the last.
///
/// `alo_kept_undo::TheTurn` holds one sentence and a turn may approve more than
/// one change. This records the first, and the module's header says why joining
/// several would be this file inventing wording.
#[test]
fn the_first_sentence_approved_is_the_one_kept() {
    let under = tempfile::tempdir().expect("a temporary folder");
    let home = tempfile::tempdir().expect("a temporary home");
    let goes = somewhere(under.path(), home.path());

    let mut bracket = TheTurnsBracket::none_yet(Some(goes), noon(), 42, 1);
    bracket.before_a_change("archive march's invoices");
    bracket.before_a_change("and then delete the originals");

    assert_eq!(
        bracket.the_first_sentence_for_tests(),
        Some("archive march's invoices"),
        "the second approval replaced the first turn's sentence"
    );
}

/// **A turn that changed nothing leaves nothing behind.**
///
/// The ordinary case: most turns answer questions. A directory made for every
/// turn would be a directory per question, and the privileged remover would walk
/// them for ever.
#[test]
fn a_turn_that_changed_nothing_leaves_nothing() {
    let under = tempfile::tempdir().expect("a temporary folder");
    let home = tempfile::tempdir().expect("a temporary home");
    let goes = somewhere(under.path(), home.path());
    let whose = goes.whose().to_path_buf();

    let mut bracket = TheTurnsBracket::none_yet(Some(goes), noon(), 42, 1);
    bracket.as_the_turn_ends(noon());

    assert!(
        !whose.exists(),
        "a turn that approved nothing made the person's directory"
    );
    assert_eq!(
        bracket.what_this_turn_kept(),
        WhatThisTurnKept::NotThisTurn,
        "a turn that never asked for a bracket should not claim one"
    );
}

/// **A turn's name is one `alo-turn` will accept**, which is the join between
/// the two halves.
///
/// Built here and validated there: a name this file invented and that crate
/// refused would mean no bracket was ever taken, and nothing would say why —
/// the taking answers *why not* rather than failing, so a mismatch here is
/// silent. Three turns of one process, and one of another, to show the name
/// carries both.
#[test]
fn the_name_a_turn_is_given_is_one_the_taker_accepts() {
    let under = tempfile::tempdir().expect("a temporary folder");
    let mut named = Vec::new();
    for (process, which) in [(42, 1), (42, 2), (42, 3), (7, 1)] {
        let name = a_name_for(noon(), process, which);
        alo_turn::the_bracket_a_turn_takes::WhereThisTurnIsKept::made(under.path(), &name)
            .unwrap_or_else(|why| panic!("{name:?} is not a name the taker accepts: {why:?}"));
        named.push(name);
    }

    // And every one differs, because two turns sharing a directory would have
    // the second overwrite the first's note.
    let how_many = named.len();
    named.sort();
    named.dedup();
    assert_eq!(how_many, named.len(), "two turns were given one name");
}

/// **The name carries the moment, the process and the turn**, so a counter
/// reset by a restart cannot collide with a turn in the same second.
#[test]
fn the_name_separates_turns_across_a_restart() {
    assert_ne!(
        a_name_for(noon(), 42, 1),
        a_name_for(noon(), 7, 1),
        "two processes' first turns in one second share a name"
    );
    assert_ne!(
        a_name_for(noon(), 42, 1),
        a_name_for(noon(), 42, 2),
        "two turns of one process share a name"
    );
}
