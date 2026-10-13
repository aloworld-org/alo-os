//! The application door's place, and the five things it refuses rather than
//! serve.
//!
//! **The refusals are the point of this file.** Everything protecting this door
//! is the mode of the path it lives at — ADR 0108 puts no token on the wire —
//! so a `prepared` that succeeded where it should not is a socket the agent can
//! reach, and the whole decision comes apart quietly. Each refusal below is
//! therefore tested by making the machine really be in that state, never by
//! calling a checking function directly.
//!
//! **None of them needs privilege**, which is why they run everywhere rather
//! than only in the gate: `prepared` is told which user to hold the directory
//! to, so *somebody else's directory* is a directory of our own and a uid that
//! is not ours.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, Write as _};
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

use crate::the_application_doors_place::{NoDoor, THE_SOCKET, ThePlace};

/// The user this test is really running as.
fn us() -> u32 {
    rustix::process::geteuid().as_raw()
}

/// A root of this test's own, with nothing in it.
///
/// Named from the caller so two tests never share one, and under the system
/// temporary directory rather than anywhere deeper: a Unix socket path is
/// bounded at about 108 bytes by the kernel, and a long root would fail for
/// that reason while looking like a refusal this file is about.
fn a_root_of_our_own(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-door-{}-{what}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// That root, with the person's own directory already in it — which on a real
/// machine `alo-agentd` made and this crate never does.
fn with_the_persons_directory(what: &str) -> (PathBuf, ThePlace) {
    let root = a_root_of_our_own(what);
    let place = ThePlace::beneath(&root, us());
    std::fs::create_dir_all(place.directory()).unwrap();
    (root, place)
}

/// **A door is served, it is shut to everybody but the person, and the person
/// can talk on it.**
///
/// The mode is read off the socket on the disk rather than from the constant
/// that set it, because the constant is not what a kernel consults.
#[test]
fn a_door_is_served_and_only_the_person_may_reach_it() {
    let (root, place) = with_the_persons_directory("served");

    let listening = place.prepared(us()).unwrap();

    assert!(place.socket().exists(), "no socket at {:?}", place.socket());
    assert_eq!(
        place.socket().file_name().unwrap().to_string_lossy(),
        THE_SOCKET,
        "the socket is not named what the contract publishes"
    );
    let mode = std::fs::metadata(place.socket())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(
        mode,
        0o600,
        "the socket is {mode:o}, not 0600 — the agent's group can reach {:o}",
        mode & 0o060
    );

    // And a connection really crosses it, so *a socket exists* is not standing
    // in for *a door works*.
    let mut speaking = std::os::unix::net::UnixStream::connect(place.socket()).unwrap();
    let (answered, _) = listening.accept().unwrap();
    speaking.write_all(b"open org.gnome.Geary\n").unwrap();
    let mut heard = String::new();
    std::io::BufReader::new(answered)
        .read_line(&mut heard)
        .unwrap();
    assert_eq!(heard, "open org.gnome.Geary\n");

    let _ = std::fs::remove_dir_all(root);
}

/// **It does not block**, because the one place it is read from is a frame
/// loop: a window system that stopped drawing to wait for a caller is a window
/// system a caller can freeze.
#[test]
fn a_door_with_nobody_at_it_does_not_wait() {
    let (root, place) = with_the_persons_directory("nonblocking");

    let listening = place.prepared(us()).unwrap();
    let nobody = listening.accept();

    match nobody {
        Err(why) if why.kind() == std::io::ErrorKind::WouldBlock => {}
        other => panic!("accept on an empty door answered {other:?} instead of not waiting"),
    }

    let _ = std::fs::remove_dir_all(root);
}

/// **The person's directory is the daemon's, so a missing one is said and not
/// made.** ADR 0017 gives it to `alo-agentd`; a compositor that made it would
/// be choosing the mode of the directory the agent's own door lives in.
#[test]
fn a_missing_directory_is_the_daemons_and_is_not_made() {
    let root = a_root_of_our_own("no-directory");
    let place = ThePlace::beneath(&root, us());

    match place.prepared(us()) {
        Err(NoDoor::NoDirectory { at }) => assert_eq!(at, place.directory()),
        other => panic!("{other:?}"),
    }
    assert!(
        !place.directory().exists(),
        "the compositor made the daemon's directory"
    );
    assert!(!place.socket().exists());

    let _ = std::fs::remove_dir_all(root);
}

/// **A symbolic link where the directory belongs is refused, unfollowed.**
/// Whoever can replace the link can replace the socket, and every caller on the
/// machine would then be talking to them.
#[test]
fn a_link_where_the_directory_belongs_is_refused() {
    let root = a_root_of_our_own("a-link");
    let elsewhere = root.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let place = ThePlace::beneath(&root, us());
    std::os::unix::fs::symlink(&elsewhere, place.directory()).unwrap();

    match place.prepared(us()) {
        Err(NoDoor::ALink { at }) => assert_eq!(at, place.directory()),
        other => panic!("{other:?}"),
    }
    // And nothing was bound inside what it pointed at, which is the thing
    // following the link would have done.
    assert!(!elsewhere.join(THE_SOCKET).exists());

    let _ = std::fs::remove_dir_all(root);
}

/// Something that is not a directory at all.
#[test]
fn a_file_where_the_directory_belongs_is_refused() {
    let root = a_root_of_our_own("not-a-directory");
    let place = ThePlace::beneath(&root, us());
    std::fs::write(place.directory(), b"not a directory").unwrap();

    match place.prepared(us()) {
        Err(NoDoor::NotADirectory { at }) => assert_eq!(at, place.directory()),
        other => panic!("{other:?}"),
    }

    let _ = std::fs::remove_dir_all(root);
}

/// **A directory belonging to somebody else is refused**, and this needs no
/// privilege to test: the directory is ours and the person asked about is not.
///
/// That is the real shape of the fault rather than a convenience — a
/// compositor serving a door in a directory the signed-in person does not own
/// is a compositor that has been handed somebody else's place, however the
/// machine got into that state.
#[test]
fn a_directory_belonging_to_somebody_else_is_refused() {
    let (root, place) = with_the_persons_directory("somebody-elses");
    let somebody_else = us().wrapping_add(1);

    match place.prepared(somebody_else) {
        Err(NoDoor::SomebodyElses { at, owner }) => {
            assert_eq!(at, place.directory());
            assert_eq!(owner, us());
        }
        other => panic!("{other:?}"),
    }
    assert!(!place.socket().exists(), "a socket was bound anyway");

    let _ = std::fs::remove_dir_all(root);
}

/// **A socket this crate left behind is cleared**, because a compositor that
/// was killed leaves one and `bind` refuses a path that exists.
#[test]
fn a_socket_left_behind_by_a_killed_compositor_is_cleared() {
    let (root, place) = with_the_persons_directory("stale");

    let first = place.prepared(us()).unwrap();
    // Killed: the listener goes, `taken_away` never runs, the socket stays.
    drop(first);
    assert!(place.socket().exists(), "the fixture is not a stale socket");

    let again = place.prepared(us()).unwrap();
    assert!(place.socket().exists());
    drop(again);

    let _ = std::fs::remove_dir_all(root);
}

/// **Anything that is not a socket is refused rather than removed**, and that
/// is the point of looking instead of unlinking. A compositor that cleared
/// whatever it found at a path in a directory is a worse thing to own than a
/// door that will not open.
#[test]
fn something_that_is_not_a_socket_is_refused_and_left_alone() {
    let (root, place) = with_the_persons_directory("in-the-way");
    std::fs::write(place.socket(), b"somebody's file").unwrap();

    match place.prepared(us()) {
        Err(NoDoor::SomethingElseIsThere { at }) => assert_eq!(at, place.socket()),
        other => panic!("{other:?}"),
    }
    assert_eq!(
        std::fs::read(place.socket()).unwrap(),
        b"somebody's file",
        "the compositor removed something that was not its socket"
    );

    let _ = std::fs::remove_dir_all(root);
}

/// A link at the socket's own path is refused too, and unfollowed: removing it
/// would unlink the link and bind over it, leaving whatever it pointed at
/// untouched and unexplained.
#[test]
fn a_link_where_the_socket_belongs_is_refused_unfollowed() {
    let (root, place) = with_the_persons_directory("socket-link");
    let elsewhere = root.join("pointed-at");
    std::fs::write(&elsewhere, b"pointed at").unwrap();
    std::os::unix::fs::symlink(&elsewhere, place.socket()).unwrap();

    match place.prepared(us()) {
        Err(NoDoor::SomethingElseIsThere { at }) => assert_eq!(at, place.socket()),
        other => panic!("{other:?}"),
    }
    assert!(elsewhere.exists(), "what the link pointed at was removed");

    let _ = std::fs::remove_dir_all(root);
}

/// **Taking the door away takes the socket and leaves the directory**, which is
/// the daemon's. A compositor that removed it would take the agent's own door
/// with it.
#[test]
fn taking_the_door_away_leaves_the_daemons_directory() {
    let (root, place) = with_the_persons_directory("taken-away");

    let listening = place.prepared(us()).unwrap();
    drop(listening);
    place.taken_away();

    assert!(!place.socket().exists(), "the socket outlived the door");
    assert!(
        place.directory().is_dir(),
        "the compositor removed the daemon's directory"
    );

    let _ = std::fs::remove_dir_all(root);
}
