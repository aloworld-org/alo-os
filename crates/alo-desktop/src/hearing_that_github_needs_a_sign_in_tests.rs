//! **Each knock the service gives reaches the desktop once, a service that does
//! not offer it is told apart, and a door that is not the person's is never
//! asked.** A stand-in service on a real socket, in a directory of the test's
//! own.

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;

use alo_indicator::watching::StoppedListening;
use alo_protocol::{FromAPerson, ToAPerson};

use super::{AGitHubSignInIsNeeded, StoppedHearing, heard_once};

/// Who is running the test.
fn me() -> u32 {
    rustix::process::getuid().as_raw()
}

/// A door in a directory of the test's own, with this mode.
fn a_door(mode: u32) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let place = root.path().join("1000");
    std::fs::create_dir(&place).unwrap();
    std::fs::set_permissions(&place, std::fs::Permissions::from_mode(mode)).unwrap();
    (root, place.join("agentd.sock"))
}

/// A service that answers one desktop with these lines and hangs up, after
/// checking it was asked to watch for a GitHub sign-in.
fn a_service_saying(door: &PathBuf, said: Vec<ToAPerson>) -> std::thread::JoinHandle<()> {
    let listener = UnixListener::bind(door).unwrap();
    std::thread::spawn(move || {
        let (connection, _) = listener.accept().unwrap();
        let mut reading = BufReader::new(connection.try_clone().unwrap());
        let mut asked = String::new();
        reading.read_line(&mut asked).unwrap();
        assert_eq!(
            FromAPerson::read(&asked).unwrap(),
            FromAPerson::WatchWhatNeedsAGitHubSignIn
        );
        // **A desktop may hang up before the last line**, which is what some
        // of these tests are about, so a write that finds nobody is not a
        // failure of the stand-in.
        let mut writing = connection;
        for told in said {
            let line = format!("{}\n", told.written().unwrap());
            if writing.write_all(line.as_bytes()).is_err() {
                return;
            }
        }
    })
}

/// **Each knock is handed over once**, after the service said the desktop is
/// watching — two failures, two knocks, and the connection's end said.
#[test]
fn each_knock_reaches_the_desktop_once() {
    let (_root, door) = a_door(0o700);
    let service = a_service_saying(
        &door,
        vec![
            ToAPerson::WatchingWhatNeedsAGitHubSignIn,
            ToAPerson::AGitHubSignInIsNeeded,
            ToAPerson::AGitHubSignInIsNeeded,
        ],
    );
    let mut heard = Vec::new();
    let stopped = heard_once(&door, me(), &mut |knock| {
        heard.push(knock);
        true
    });
    service.join().unwrap();
    assert_eq!(stopped, StoppedHearing::Ended);
    assert_eq!(heard, [AGitHubSignInIsNeeded, AGitHubSignInIsNeeded]);
}

/// **A service that does not offer it is told apart, and nothing is heard** —
/// an older `alo-agentd` refuses the request, and a refusal is never a knock.
#[test]
fn a_service_that_does_not_offer_it_knocks_nothing() {
    let (_root, door) = a_door(0o700);
    let service = a_service_saying(
        &door,
        vec![
            ToAPerson::Leaving { lines: Vec::new() },
            ToAPerson::AGitHubSignInIsNeeded,
        ],
    );
    let mut heard = Vec::new();
    let stopped = heard_once(&door, me(), &mut |knock| {
        heard.push(knock);
        true
    });
    service.join().unwrap();
    assert_eq!(stopped, StoppedHearing::NotOffered);
    assert!(heard.is_empty(), "{heard:?}");
}

/// **Anything but a knock ends the hearing**: what leaves, arriving on this
/// connection, is never read as a sign-in being needed.
#[test]
fn anything_but_a_knock_ends_it() {
    let (_root, door) = a_door(0o700);
    let service = a_service_saying(
        &door,
        vec![
            ToAPerson::WatchingWhatNeedsAGitHubSignIn,
            ToAPerson::Leaving { lines: Vec::new() },
            ToAPerson::AGitHubSignInIsNeeded,
        ],
    );
    let mut heard = Vec::new();
    let stopped = heard_once(&door, me(), &mut |knock| {
        heard.push(knock);
        true
    });
    service.join().unwrap();
    assert_eq!(stopped, StoppedHearing::Ended);
    assert!(heard.is_empty(), "{heard:?}");
}

/// **A door in a directory others may write in is never asked** — the
/// indicator's own check, through the one function both use.
#[test]
fn a_door_others_may_write_beside_is_never_asked() {
    let (_root, door) = a_door(0o777);
    let _listener = UnixListener::bind(&door).unwrap();
    let stopped = heard_once(&door, me(), &mut |_| true);
    assert_eq!(
        stopped,
        StoppedHearing::NotReached(StoppedListening::NotThePersonsAlone)
    );
}

/// **A desktop that stopped listening stops the hearing** at the first knock.
#[test]
fn a_desktop_that_stopped_listening_stops_it() {
    let (_root, door) = a_door(0o700);
    let service = a_service_saying(
        &door,
        vec![
            ToAPerson::WatchingWhatNeedsAGitHubSignIn,
            ToAPerson::AGitHubSignInIsNeeded,
        ],
    );
    let stopped = heard_once(&door, me(), &mut |_| false);
    service.join().unwrap();
    assert_eq!(stopped, StoppedHearing::NobodyToTell);
}
