//! What is answered when a place can be kept and when it cannot.
//!
//! **No `btrfs` anywhere in here.** Taking a real snapshot needs a filesystem
//! that can and a machine that may, and `crates/alo-turn/tests/a_bracket_on_a_
//! real_btrfs_machine.rs` is where that is held. What this file is about is the
//! answer a person's shell gets, which is decided before any of that: a session
//! with no home, a folder that cannot be made, and the shape of the reply.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_protocol::ToAPerson;
use alo_strings::Strings;

use super::{a_name_for, kept};
use crate::the_bracket_this_turn_takes::WhereBracketsGo;

/// This crate's own words, which is what the sentence is rendered from.
fn in_english() -> Strings {
    let mut vocabulary = alo_strings::Vocabulary::default();
    crate::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// **A session with no home gets no place, and is told.**
///
/// A login with no home has nothing to snapshot. That is a person's state and
/// not a fault — but from where they sit it is the same thing as a snapshot
/// that failed, because the outcome is identical: the next command cannot be
/// undone. So it is answered the same way rather than silently.
#[test]
fn a_session_with_no_home_keeps_no_place_and_says_so() {
    let answered = kept(None, "whatever", &in_english());
    match answered {
        ToAPerson::APlaceToReturnTo { kept, why_not } => {
            assert!(!kept, "there was nowhere to keep one");
            let said = why_not.expect("a person is told when nothing was kept");
            assert!(
                !said.text().is_empty(),
                "the sentence is the service's and must not be empty"
            );
        }
        other => panic!("answered {other:?}"),
    }
}

/// **A folder that cannot be made keeps no place, and the command still runs.**
///
/// The undo directory is under a path this test owns and makes unusable. What
/// matters is that this is an *answer* and never a refusal: ADR 0045's third
/// term is *a machine too full to snapshot still runs the turn*, and a person
/// whose disk is full still gets to run `ls`.
#[test]
fn a_folder_that_cannot_be_made_keeps_no_place() {
    let at = std::env::temp_dir().join(format!("alo-place-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    // A *file* where the undo folder would go, so making a directory there
    // cannot work and does not depend on permissions a test may not have.
    std::fs::write(&at, b"not a directory").unwrap();

    let goes = WhereBracketsGo::of_a_session(&at, 1000, Some(std::ffi::OsStr::new("/")));
    let answered = kept(goes.as_ref(), "whatever", &in_english());
    match answered {
        ToAPerson::APlaceToReturnTo { kept, why_not } => {
            assert!(!kept, "nothing could be made there");
            assert!(why_not.is_some(), "and the person is told");
        }
        other => panic!("answered {other:?}"),
    }
    let _ = std::fs::remove_file(&at);
}

/// **A relative home is no session at all**, which `WhereBracketsGo` refuses
/// before this is reached — held here because the refusal is what stops a
/// snapshot being taken of whatever directory the service happens to be in.
#[test]
fn a_relative_home_names_no_session() {
    assert!(
        WhereBracketsGo::of_a_session(
            std::path::Path::new("/run/alo-undo"),
            1000,
            Some(std::ffi::OsStr::new("not/absolute")),
        )
        .is_none(),
        "a relative home must not become a place to snapshot"
    );
}

/// **The name says when, whose process, and that it is theirs.**
///
/// Bracket names are opaque by the contract — nothing reads one back — so this
/// holds the shape rather than any part's meaning: two moments are two names,
/// and a person's own is told apart from a turn's.
#[test]
fn two_moments_are_two_names() {
    use std::time::{Duration, SystemTime};
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    let later = now + Duration::from_secs(1);
    assert_ne!(a_name_for(now, 7), a_name_for(later, 7));
    assert_ne!(a_name_for(now, 7), a_name_for(now, 8));
    assert!(
        a_name_for(now, 7).ends_with("their-own"),
        "a person's own place is told apart from a turn's: {}",
        a_name_for(now, 7)
    );
}
