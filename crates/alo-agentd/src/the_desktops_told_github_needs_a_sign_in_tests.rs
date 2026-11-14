//! **A command that needs a GitHub sign-in is told once, anything else is told
//! to nobody, and a desktop that stops reading is dropped rather than waited
//! for.** Over real socket pairs.

use std::io::{BufRead as _, BufReader, Read as _};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use alo_protocol::{Done, ToAPerson};

use alo_publishing::NotPublished;
use alo_turn::NotDone;

use super::{
    AT_MOST, TheDesktopsToldGitHubNeedsASignIn, a_refusal_needs_a_github_sign_in,
    needs_a_github_sign_in,
};

/// **A publication refused because nobody is signed in tells the desktop
/// once** (#1213's refusal); a publication refused for any other reason, and
/// any other refusal, tell it nothing.
#[test]
fn a_publication_nobody_signed_in_for_is_told_once() {
    let watching = TheDesktopsToldGitHubNeedsASignIn::none_yet();
    let mut desktop = a_desktop(&watching);
    let others = [
        NotDone::CouldNotPublish(NotPublished::NoGh),
        NotDone::CouldNotPublish(NotPublished::NoGit),
        NotDone::TurnClosed,
    ];
    for why in &others {
        assert!(!a_refusal_needs_a_github_sign_in(why), "{why:?}");
        watching.told_of_a_refusal(why);
    }
    let not_signed_in = NotDone::CouldNotPublish(NotPublished::NotSignedIn);
    assert!(a_refusal_needs_a_github_sign_in(&not_signed_in));
    watching.told_of_a_refusal(&not_signed_in);
    assert_eq!(
        everything_told(&mut desktop),
        [ToAPerson::AGitHubSignInIsNeeded]
    );
}

/// What `git` prints when it has an `https` remote on GitHub and nobody to
/// type a name — what alo's own runs get.
const NOBODY_SIGNED_IN: &str =
    "fatal: could not read Username for 'https://github.com': terminal prompts disabled\n";

/// A command's answer, as the wire carries it.
fn a_run(returned: Option<i32>, said: &str, took_too_long: bool) -> Done {
    Done::RanACommand {
        returned,
        said: said.to_owned(),
        cut_short: false,
        took_too_long,
        sent_bytes: Some(0),
    }
}

/// A desktop's end of a watching connection, reading with a short wait.
fn a_desktop(watching: &TheDesktopsToldGitHubNeedsASignIn) -> BufReader<UnixStream> {
    let (ours, theirs) = UnixStream::pair().unwrap();
    watching.watching(ours);
    theirs
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    BufReader::new(theirs)
}

/// Every line a desktop has been told, until nothing more arrives.
fn everything_told(desktop: &mut BufReader<UnixStream>) -> Vec<ToAPerson> {
    let mut told = Vec::new();
    loop {
        let mut line = String::new();
        match desktop.read_line(&mut line) {
            Ok(0) | Err(_) => return told,
            Ok(_) => told.push(ToAPerson::read(&line).unwrap()),
        }
    }
}

/// **A command that failed because nobody is signed in is told once**, to
/// every desktop watching, and carries nothing.
#[test]
fn a_command_that_needs_a_sign_in_is_told_once_to_each_desktop() {
    let watching = TheDesktopsToldGitHubNeedsASignIn::none_yet();
    let mut first = a_desktop(&watching);
    let mut second = a_desktop(&watching);

    watching.told_of(&a_run(Some(128), NOBODY_SIGNED_IN, false));

    for desktop in [&mut first, &mut second] {
        assert_eq!(everything_told(desktop), [ToAPerson::AGitHubSignInIsNeeded]);
    }
}

/// **Two failures are two, and never more**: one line per command that needed
/// a sign-in, whatever the desktop does with them.
#[test]
fn each_failure_is_told_once_and_only_once() {
    let watching = TheDesktopsToldGitHubNeedsASignIn::none_yet();
    let mut desktop = a_desktop(&watching);
    watching.told_of(&a_run(Some(128), NOBODY_SIGNED_IN, false));
    watching.told_of(&a_run(Some(128), NOBODY_SIGNED_IN, false));
    assert_eq!(
        everything_told(&mut desktop),
        [
            ToAPerson::AGitHubSignInIsNeeded,
            ToAPerson::AGitHubSignInIsNeeded
        ]
    );
}

/// **Any other end is told to nobody**: a command that worked though it
/// printed git's line, one that failed for another reason, one stopped by a
/// signal, one that ran too long, and an answer that is not a command at all.
#[test]
fn any_other_end_is_told_to_nobody() {
    let watching = TheDesktopsToldGitHubNeedsASignIn::none_yet();
    let mut desktop = a_desktop(&watching);
    let others = [
        a_run(Some(0), NOBODY_SIGNED_IN, false),
        a_run(
            Some(1),
            "mkdir: cannot create directory '/x': Permission denied\n",
            false,
        ),
        a_run(None, NOBODY_SIGNED_IN, false),
        a_run(Some(128), NOBODY_SIGNED_IN, true),
        Done::Renamed { now_at: None },
    ];
    for done in &others {
        assert!(!needs_a_github_sign_in(done), "{done:?}");
        watching.told_of(done);
    }
    assert_eq!(everything_told(&mut desktop), Vec::<ToAPerson>::new());
    assert!(needs_a_github_sign_in(&a_run(
        Some(128),
        NOBODY_SIGNED_IN,
        false
    )));
}

/// **A desktop watching what leaves is never told it** — two lists, so a
/// desktop written before this answer existed never meets it.
#[test]
fn a_desktop_watching_what_leaves_is_never_told_it() {
    let leaving = crate::watchers::Watchers::none_yet();
    let (ours, mut theirs) = UnixStream::pair().unwrap();
    leaving.watching(ours);
    theirs
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();

    let watching = TheDesktopsToldGitHubNeedsASignIn::none_yet();
    watching.told_of(&a_run(Some(128), NOBODY_SIGNED_IN, false));

    let mut heard = [0_u8; 64];
    assert!(
        !matches!(theirs.read(&mut heard), Ok(read) if read > 0),
        "a desktop watching what leaves was told something"
    );
}

/// **A desktop that stops reading is dropped, never waited for**: the command's
/// answer is not held up by a screen that froze.
#[test]
fn a_desktop_that_stops_reading_is_dropped() {
    let watching = TheDesktopsToldGitHubNeedsASignIn::none_yet();
    let (ours, theirs) = UnixStream::pair().unwrap();
    watching.watching(ours);
    // Never read: the socket fills, and the next write cannot be taken.
    let mut told = 0_usize;
    while watching.how_many() == 1 && told < 1_000_000 {
        watching.told_of(&a_run(Some(128), NOBODY_SIGNED_IN, false));
        told += 1;
    }
    assert_eq!(
        watching.how_many(),
        0,
        "a desktop that stopped reading is kept"
    );
    drop(theirs);
}

/// **At most so many desktops are told, and the oldest goes.**
#[test]
fn the_oldest_desktop_goes() {
    let watching = TheDesktopsToldGitHubNeedsASignIn::none_yet();
    let mut kept = Vec::new();
    for _ in 0..=AT_MOST {
        kept.push(a_desktop(&watching));
    }
    assert_eq!(watching.how_many(), AT_MOST);
}

/// **Clones share one list**, and two lists made apart are never the same.
#[test]
fn clones_share_one_list() {
    let one = TheDesktopsToldGitHubNeedsASignIn::none_yet();
    let other = one.clone();
    let _desktop = a_desktop(&other);
    assert_eq!(one.how_many(), 1);
    assert_eq!(one, other);
    assert_ne!(one, TheDesktopsToldGitHubNeedsASignIn::none_yet());
}
