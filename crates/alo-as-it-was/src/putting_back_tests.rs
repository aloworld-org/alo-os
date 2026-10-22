//! Putting a folder back, and every way it is refused instead.
//!
//! The refusals are the point of this file. A person asking to go back is
//! asking for the one act in this product that changes their files without an
//! agent anywhere near it, so *and it was stopped* is the case that matters —
//! and each one is reached here by building the machine that causes it rather
//! than by constructing the error.

use std::path::Path;

use super::{BEING_PUT_BACK, NotGoingBack, go_back, what_would_go_back};
use crate::testing::{AMachine, at};

/// The moment a go-back happens in these tests.
const NOW: u64 = 1_700_010_000;

/// **It goes back, and what was there is beside it.**
///
/// The whole act on an ordinary filesystem: the folder holds what it held when
/// the place was kept, and what it held a moment ago is under its own name next
/// to it. Nothing is deleted, which is how
/// [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s *an
/// undo never overwrites a later change* is met where nothing compares the two
/// sides.
#[test]
fn the_folder_goes_back_and_what_was_there_is_set_aside() {
    let machine = AMachine::called("goes-back");
    let folder = machine.a_folder("src", "as it was");
    let place = machine.a_place_kept("1000-7-their-own", at(1_700_000_000));
    std::fs::write(folder.join("a-file"), "changed since").unwrap();
    std::fs::write(folder.join("made-since"), "new").unwrap();

    let goes = what_would_go_back(
        &place.join("before"),
        &folder,
        &machine.home(),
        at(1_700_000_000),
        at(NOW),
    )
    .unwrap();
    go_back(&goes).unwrap();

    assert_eq!(
        machine.what_it_says(&folder.join("a-file")),
        "as it was",
        "the file is as it was before the command"
    );
    assert!(
        !folder.join("made-since").exists(),
        "a file made after the place was kept is not in the folder any more"
    );
    let aside = goes.what_was_here();
    assert_eq!(
        machine.what_it_says(&aside.join("a-file")),
        "changed since",
        "and what was in the folder is beside it, not thrown away"
    );
    assert!(
        aside.join("made-since").exists(),
        "including the file that was made since, which nothing else holds a copy of"
    );
}

/// **A second go-back of one folder does not overwrite the first one's set-aside
/// copy.**
///
/// The name carries the moment for exactly this reason. Without it the second
/// rename would meet a directory that is not empty and fail *after* the swap,
/// which is the one outcome where a person has been told something happened and
/// cannot tell what.
#[test]
fn a_second_go_back_sets_its_copy_aside_somewhere_else() {
    let machine = AMachine::called("twice");
    let folder = machine.a_folder("src", "as it was");
    let place = machine.a_place_kept("1000-7-their-own", at(1_700_000_000));

    let before = place.join("before");
    let first = what_would_go_back(&before, &folder, &machine.home(), at(1), at(NOW)).unwrap();
    go_back(&first).unwrap();
    let second = what_would_go_back(&before, &folder, &machine.home(), at(1), at(NOW + 1)).unwrap();
    assert_ne!(
        first.what_was_here(),
        second.what_was_here(),
        "two go-backs set aside under two names"
    );
    go_back(&second).unwrap();
    assert!(first.what_was_here().is_dir(), "the first is still there");
    assert!(second.what_was_here().is_dir(), "and so is the second");
}

/// **A folder outside the home is refused.**
///
/// There is no grant to `/` and there is no road here to one. A place kept is a
/// copy of this person's home, so there is nothing in it to put anywhere else.
#[test]
fn a_folder_outside_the_home_is_refused() {
    let machine = AMachine::called("outside");
    machine.a_folder("src", "as it was");
    let place = machine.a_place_kept("1000-7-their-own", at(1));
    let elsewhere = std::env::temp_dir();

    let refused = what_would_go_back(
        &place.join("before"),
        &elsewhere,
        &machine.home(),
        at(1),
        at(NOW),
    );
    assert!(
        matches!(refused, Err(NotGoingBack::NotInTheirHome { .. })),
        "{refused:?}"
    );
}

/// **A link inside the home that points out of it is refused.**
///
/// The one shape of this mistake that a textual check lets through, and the
/// reason both paths are resolved before they are compared. A test written with
/// ordinary directories would pass either way, which is why this one is written
/// with a link.
#[test]
fn a_link_out_of_the_home_is_refused_rather_than_followed() {
    let machine = AMachine::called("a-link-out");
    machine.a_folder("src", "as it was");
    let place = machine.a_place_kept("1000-7-their-own", at(1));
    let outside = std::env::temp_dir().join(format!("alo-outside-{}", std::process::id()));
    std::fs::create_dir_all(&outside).unwrap();
    let pointing = machine.home().join("escape");
    std::os::unix::fs::symlink(&outside, &pointing).unwrap();

    let refused = what_would_go_back(
        &place.join("before"),
        &pointing,
        &machine.home(),
        at(1),
        at(NOW),
    );
    let _ = std::fs::remove_dir_all(&outside);
    assert!(
        matches!(refused, Err(NotGoingBack::NotInTheirHome { .. })),
        "{refused:?}"
    );
}

/// **The whole home is refused, by name.**
///
/// ADR 0127's second road, which happens at the next sign-in because nothing
/// can swap a directory somebody is living in. Refused rather than attempted:
/// attempting it would half-work.
#[test]
fn the_whole_home_is_refused() {
    let machine = AMachine::called("whole-home");
    machine.a_folder("src", "as it was");
    let place = machine.a_place_kept("1000-7-their-own", at(1));

    let refused = what_would_go_back(
        &place.join("before"),
        &machine.home(),
        &machine.home(),
        at(1),
        at(NOW),
    );
    assert!(
        matches!(refused, Err(NotGoingBack::TheWholeHome { .. })),
        "{refused:?}"
    );
}

/// **A file rather than a folder is refused**, and so is a path that is not
/// there at all.
#[test]
fn a_file_and_a_path_that_is_not_there_are_both_refused() {
    let machine = AMachine::called("not-a-folder");
    machine.a_folder("src", "as it was");
    let place = machine.a_place_kept("1000-7-their-own", at(1));
    let before = place.join("before");
    let a_file = machine.home().join("notes.txt");
    std::fs::write(&a_file, b"mine").unwrap();

    for named in [a_file, machine.home().join("never-existed")] {
        let refused = what_would_go_back(&before, &named, &machine.home(), at(1), at(NOW));
        assert!(
            matches!(refused, Err(NotGoingBack::NotAFolder { .. })),
            "{}: {refused:?}",
            named.display()
        );
    }
}

/// **A folder that did not exist when the place was kept is refused, never
/// deleted.**
///
/// Putting it *back* would mean removing it, which is a different and larger
/// act. alo does not do it on the strength of a request to go back.
#[test]
fn a_folder_the_place_never_held_is_refused_and_still_there() {
    let machine = AMachine::called("not-in-place");
    let place = machine.a_place_kept("1000-7-their-own", at(1));
    let made_since = machine.a_folder("made-since", "after the place was kept");

    let refused = what_would_go_back(
        &place.join("before"),
        &made_since,
        &machine.home(),
        at(1),
        at(NOW),
    );
    assert!(
        matches!(refused, Err(NotGoingBack::NotInThatPlace { .. })),
        "{refused:?}"
    );
    assert!(
        made_since.join("a-file").exists(),
        "and the folder is untouched, which is the half a person cares about"
    );
}

/// **A link where the copy should be is refused rather than followed.**
///
/// A link in a kept copy still points wherever it pointed, so following one
/// would read from outside the copy — and the copy is the only thing in this
/// act that is trusted.
#[test]
fn a_link_in_the_place_is_refused() {
    let machine = AMachine::called("a-link-inside");
    let folder = machine.a_folder("src", "as it was");
    let place = machine.a_place_kept("1000-7-their-own", at(1));
    let inside = place.join("before").join("src");
    std::fs::remove_dir_all(&inside).unwrap();
    std::os::unix::fs::symlink(machine.home(), &inside).unwrap();

    let refused = what_would_go_back(
        &place.join("before"),
        &folder,
        &machine.home(),
        at(1),
        at(NOW),
    );
    assert!(
        matches!(refused, Err(NotGoingBack::ALinkRatherThanAFolder { .. })),
        "{refused:?}"
    );
}

/// **A go-back that did not finish is refused, and what it left is not
/// removed.**
///
/// Whatever is under the working name was copied out of one of this person's own
/// kept places. It is not alo's to throw away, so the refusal asks them.
#[test]
fn a_half_done_go_back_is_refused_and_left_alone() {
    let machine = AMachine::called("half-done");
    let folder = machine.a_folder("src", "as it was");
    let place = machine.a_place_kept("1000-7-their-own", at(1));
    let left_behind = machine.home().join(format!("src.{BEING_PUT_BACK}"));
    std::fs::create_dir_all(&left_behind).unwrap();
    std::fs::write(left_behind.join("a-file"), "half copied").unwrap();

    let refused = what_would_go_back(
        &place.join("before"),
        &folder,
        &machine.home(),
        at(1),
        at(NOW),
    );
    assert!(
        matches!(refused, Err(NotGoingBack::OneIsHalfDone { .. })),
        "{refused:?}"
    );
    assert_eq!(
        machine.what_it_says(&left_behind.join("a-file")),
        "half copied",
        "and alo did not remove it"
    );
}

/// **A taken set-aside name is refused before anything is copied.**
///
/// The alternative is finding out after the swap, when the folder has already
/// changed and the old contents have nowhere to go.
#[test]
fn a_taken_set_aside_name_is_refused_before_anything_is_touched() {
    let machine = AMachine::called("taken-aside");
    let folder = machine.a_folder("src", "as it was");
    let place = machine.a_place_kept("1000-7-their-own", at(1));
    let goes = what_would_go_back(
        &place.join("before"),
        &folder,
        &machine.home(),
        at(1),
        at(NOW),
    )
    .unwrap();
    std::fs::create_dir_all(goes.what_was_here()).unwrap();

    let refused = what_would_go_back(
        &place.join("before"),
        &folder,
        &machine.home(),
        at(1),
        at(NOW),
    );
    assert!(
        matches!(
            refused,
            Err(NotGoingBack::NowhereToSetAsideWhatIsThere { .. })
        ),
        "{refused:?}"
    );
}

/// **A copy that cannot be made refuses, and the folder is exactly as it was.**
///
/// The clause that matters is the second one. A failed copy that left the folder
/// half-replaced would be the worst outcome this act has, so the test asserts
/// the folder's contents rather than only the error.
///
/// # The cause is the snapshot going away, and not a permission
///
/// It was a permission until 2026-10-22, and **that version passed as a person
/// and failed on the gate's runner, which runs as root**: root ignores a mode,
/// so `cp` succeeded and no refusal arrived. The cause here is the source being
/// gone between the plan and the act, which is a real race rather than a
/// contrivance — nothing holds a place open, and `crate::putting_back`
/// deliberately plans first and copies second. **It denies root exactly as it
/// denies anybody**, so what this measures no longer depends on who runs it.
#[test]
fn a_copy_that_cannot_be_made_changes_nothing() {
    let machine = AMachine::called("cannot-copy");
    let folder = machine.a_folder("src", "as it was");
    let place = machine.a_place_kept("1000-7-their-own", at(1));
    std::fs::write(folder.join("a-file"), "changed since").unwrap();
    let goes = what_would_go_back(
        &place.join("before"),
        &folder,
        &machine.home(),
        at(1),
        at(NOW),
    )
    .unwrap();

    // The place is let go of between the plan and the act — which is what the
    // privileged remover does on a timer, to the oldest place, under disk
    // pressure (ADR 0045 terms 1 and 5).
    std::fs::remove_dir_all(place.join("before")).unwrap();
    let refused = go_back(&goes);

    assert!(
        matches!(refused, Err(NotGoingBack::NotCopied { .. })),
        "{refused:?}"
    );
    assert!(
        !refused.as_ref().err().unwrap().the_folder_went_back(),
        "and it says the folder did not go back"
    );
    assert_eq!(
        machine.what_it_says(&folder.join("a-file")),
        "changed since",
        "the folder is exactly as it was"
    );
    assert!(!goes.what_was_here().exists(), "and nothing was set aside");
    assert!(
        !machine
            .home()
            .join(format!("src.{BEING_PUT_BACK}"))
            .exists(),
        "and no half-copy was left behind, so a second attempt is not refused for it"
    );
}

/// **Only one refusal says the folder went back.**
///
/// A caller that reported every `Err` as *nothing changed* would be telling a
/// person the opposite of what their disk says.
#[test]
fn only_the_last_rename_failing_says_the_folder_went_back() {
    let paths = || Path::new("/tmp/anywhere").to_owned();
    let every = [
        NotGoingBack::NotInTheirHome {
            folder: paths(),
            home: paths(),
        },
        NotGoingBack::TheWholeHome { home: paths() },
        NotGoingBack::NotAFolder { folder: paths() },
        NotGoingBack::NotInThatPlace { folder: paths() },
        NotGoingBack::ALinkRatherThanAFolder { at: paths() },
        NotGoingBack::OneIsHalfDone { at: paths() },
        NotGoingBack::NowhereToSetAsideWhatIsThere { at: paths() },
        NotGoingBack::NotCopied {
            why: "no room".to_owned(),
        },
        NotGoingBack::NotSwapped {
            why: "no room".to_owned(),
        },
        NotGoingBack::SetAsideUnderTheWorkingName {
            folder: paths(),
            at: paths(),
            why: "no room".to_owned(),
        },
    ];
    let went_back: Vec<bool> = every
        .iter()
        .map(NotGoingBack::the_folder_went_back)
        .collect();
    assert_eq!(
        went_back.iter().filter(|said| **said).count(),
        1,
        "exactly one of these means the folder changed: {went_back:?}"
    );
    assert_eq!(
        went_back.last(),
        Some(&true),
        "and it is the one where only the last rename failed, which is last on the list"
    );
}
