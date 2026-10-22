//! What is read out of a person's undo folder, and what is left alone.

use super::{Places, where_this_person_keeps};
use crate::testing::{AMachine, at};

/// **A folder that is not there is no places and nothing wrong.**
///
/// The state of every machine on which nobody has turned this on. A reader that
/// reported it would put a sentence in front of every person who never asked
/// for any of this.
#[test]
fn a_folder_that_is_not_there_is_no_places_and_no_complaint() {
    let places = Places::under(std::path::Path::new("/nowhere/alo-as-it-was/not-a-folder"));
    assert!(places.kept().is_empty(), "there is nothing there");
    assert!(
        places.stepped_over().is_empty(),
        "and nothing to report: {:?}",
        places.stepped_over()
    );
}

/// **A place with a note and a copy is found.**
#[test]
fn a_place_with_a_note_and_a_copy_is_found() {
    let machine = AMachine::called("found");
    machine.a_folder("src", "as it was");
    machine.a_place_kept("1000-7-their-own", at(1_700_000_000));

    let places = Places::under(&machine.whose());
    assert_eq!(places.kept().len(), 1, "{:?}", places.stepped_over());
    let place = places.kept().first().unwrap();
    assert_eq!(place.taken(), at(1_700_000_000));
    assert_eq!(place.did(), "before a command you typed in the terminal");
    assert!(place.before().is_dir(), "the copy is inside it");
}

/// **A place with a copy and no note is not a place, and is reported.**
///
/// This is the fault this file exists for. `alo-agentd` kept a copy before every
/// command a person typed and wrote no note beside it, from the change that
/// built it until the change that added this crate. `alo_letting_go::found`
/// steps such a directory over rather than sweep something it cannot read, so
/// **nothing could ever remove one** — a person who turned the row on would
/// accumulate a read-only copy of their whole home per command, for ever. It is
/// also unfindable: the note is where the moment is written down, and a
/// directory's name is opaque by contract.
///
/// Left out here for the same reason it is swept over there, and **counted**, so
/// that a person looking at the list is told something is there rather than
/// simply not shown it.
#[test]
fn a_place_with_no_note_is_not_a_place_and_is_reported() {
    let machine = AMachine::called("no-note");
    machine.a_place_with_no_note("1000-7-their-own");

    let places = Places::under(&machine.whose());
    assert!(places.kept().is_empty(), "it is not somewhere to return to");
    assert_eq!(
        places.stepped_over().len(),
        1,
        "and it is reported rather than hidden"
    );
}

/// **A place with a note and no copy is not a place either.**
///
/// What an interrupted keeping leaves. Going back to it would put nothing back
/// and report success, which is worse than refusing.
#[test]
fn a_place_with_no_copy_is_not_a_place_and_is_reported() {
    let machine = AMachine::called("no-copy");
    machine.a_place_with_no_copy("1000-7-their-own", at(1_700_000_000));

    let places = Places::under(&machine.whose());
    assert!(places.kept().is_empty(), "there is nothing in it");
    assert_eq!(places.stepped_over().len(), 1, "and it is reported");
}

/// **Newest first, by the note and never by the name.**
///
/// The names here are deliberately in the opposite order to the moments, so a
/// reader that sorted by name would come out backwards. Bracket names are
/// opaque by contract and this is what holds that: the order comes from what was
/// written down.
#[test]
fn newest_first_comes_from_the_note_and_not_the_name() {
    let machine = AMachine::called("order");
    machine.a_folder("src", "as it was");
    machine.a_place_kept("aaa-oldest", at(1_700_000_000));
    machine.a_place_kept("zzz-newest", at(1_700_009_999));

    let places = Places::under(&machine.whose());
    assert_eq!(places.kept().len(), 2, "{:?}", places.stepped_over());
    assert_eq!(
        places.kept().first().unwrap().at().file_name().unwrap(),
        "zzz-newest",
        "the newest is first even though its name sorts last"
    );
}

/// **A place is chosen by name, and a name that is not there is not found.**
///
/// By name rather than by position, because a position means something else the
/// moment another command runs — and a command runs between a person reading
/// the list and typing their answer.
#[test]
fn a_place_is_chosen_by_name() {
    let machine = AMachine::called("named");
    machine.a_folder("src", "as it was");
    machine.a_place_kept("1000-7-their-own", at(1_700_000_000));

    let places = Places::under(&machine.whose());
    assert!(places.named("1000-7-their-own").is_some());
    assert!(
        places.named("1000-8-their-own").is_none(),
        "a name nobody kept is not found"
    );
    assert!(places.named("").is_none(), "and neither is an empty answer");
}

/// **A file in the folder is not a place and is not news.**
///
/// Somebody may leave a file in their own directory. It is theirs.
#[test]
fn a_file_in_the_folder_is_not_a_place_and_is_not_reported() {
    let machine = AMachine::called("a-file");
    std::fs::write(machine.whose().join("notes.txt"), b"mine").unwrap();

    let places = Places::under(&machine.whose());
    assert!(places.kept().is_empty());
    assert!(
        places.stepped_over().is_empty(),
        "a file is not a half-made place: {:?}",
        places.stepped_over()
    );
}

/// **Where a person's places are is their user identifier under the undo
/// folder**, which is what the contract says and what `alo-agentd` writes into.
#[test]
fn a_persons_places_are_under_their_own_identifier() {
    let whose = where_this_person_keeps(1000);
    assert_eq!(whose, std::path::Path::new("/var/lib/alo-undo/1000"));
}
