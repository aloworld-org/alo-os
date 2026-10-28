//! **What the service says reaches the desktop, and nothing else does.** A
//! stand-in service on a real socket, in a directory of the test's own.

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;

use alo_capability::Grantee;
use alo_egress::{Destination, Reported, Why};
use alo_protocol::{FromAPerson, ToAPerson};

use super::{StoppedListening, kept_listening, listened_once};
use crate::told::FromTheDaemon;

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

fn asking_openai() -> Reported {
    Reported::Agent {
        agent: Grantee::named("@alo"),
        why: Why::Asking,
        destination: Destination::at("api.openai.com").unwrap(),
    }
}

/// A service that answers one watcher with these lines, one message each, and
/// then hangs up. It checks it was asked the right thing.
fn a_service_saying(door: &PathBuf, said: Vec<String>) -> std::thread::JoinHandle<()> {
    let listener = UnixListener::bind(door).unwrap();
    std::thread::spawn(move || {
        let (connection, _) = listener.accept().unwrap();
        let mut reading = BufReader::new(connection.try_clone().unwrap());
        let mut asked = String::new();
        reading.read_line(&mut asked).unwrap();
        assert_eq!(
            FromAPerson::read(&asked).unwrap(),
            FromAPerson::WatchWhatLeaves
        );
        let mut writing = connection;
        for line in said {
            writing.write_all(line.as_bytes()).unwrap();
            writing.write_all(b"\n").unwrap();
        }
    })
}

#[test]
fn what_the_service_says_reaches_the_desktop_and_then_it_cannot_tell() {
    let (_root, door) = a_door(0o700);
    let service = a_service_saying(
        &door,
        vec![
            ToAPerson::Leaving {
                lines: vec![asking_openai()],
            }
            .written()
            .unwrap(),
            ToAPerson::Leaving { lines: Vec::new() }.written().unwrap(),
        ],
    );
    let mut heard = Vec::new();
    let stopped = listened_once(&door, me(), &mut |told| {
        heard.push(told);
        true
    });
    service.join().unwrap();
    assert_eq!(stopped, StoppedListening::Ended);
    let lines: Vec<usize> = heard.iter().map(|told| told.lines().len()).collect();
    assert_eq!(lines, [1, 0]);
    assert!(heard.iter().all(|told| !told.cannot_tell()));
}

/// **A directory anybody may write in is not believed**: another account
/// could have put a socket there. Nothing is read and nothing is told.
#[test]
fn a_door_in_a_directory_others_may_write_in_is_not_believed() {
    let (_root, door) = a_door(0o777);
    let _listener = UnixListener::bind(&door).unwrap();
    let mut told = 0;
    assert_eq!(
        listened_once(&door, me(), &mut |_| {
            told += 1;
            true
        }),
        StoppedListening::NotThePersonsAlone
    );
    assert_eq!(told, 0);
}

/// **A directory that is somebody else's is not believed either.**
#[test]
fn a_door_in_somebody_elses_directory_is_not_believed() {
    let (_root, door) = a_door(0o700);
    let _listener = UnixListener::bind(&door).unwrap();
    assert_eq!(
        listened_once(&door, me().wrapping_add(1), &mut |_| true),
        StoppedListening::NotThePersonsAlone
    );
}

#[test]
fn nothing_listening_is_not_there() {
    let (_root, door) = a_door(0o700);
    assert_eq!(
        listened_once(&door, me(), &mut |_| true),
        StoppedListening::NotThere
    );
}

/// **A refusal is not a list.** A service that will not tell this desktop is a
/// desktop that cannot tell, and nothing is handed over as though it were.
#[test]
fn a_refusal_is_not_told_as_a_list() {
    let (_root, door) = a_door(0o700);
    let refused = ToAPerson::refused(
        &alo_strings::Strings::of(alo_strings::Vocabulary::empty()).say(
            &crate::words::CANNOT_TELL.key(),
            &alo_strings::Filling::nothing(),
        ),
    )
    .written()
    .unwrap();
    let service = a_service_saying(&door, vec![refused]);
    let mut told = 0;
    assert_eq!(
        listened_once(&door, me(), &mut |_| {
            told += 1;
            true
        }),
        StoppedListening::Ended
    );
    service.join().unwrap();
    assert_eq!(told, 0);
}

/// **When the service hangs up, the desktop is told *cannot tell* at once**,
/// never left showing the last list; and listening stops when the desktop
/// drops its end.
#[test]
fn a_service_that_hung_up_is_unreachable_until_it_answers_again() {
    let (_root, door) = a_door(0o700);
    let service = a_service_saying(
        &door,
        vec![
            ToAPerson::Leaving {
                lines: vec![asking_openai()],
            }
            .written()
            .unwrap(),
        ],
    );
    let (send, hear) = std::sync::mpsc::channel();
    let listening = std::thread::spawn(move || kept_listening(door, me(), &send));
    let first = hear.recv().unwrap();
    assert_eq!(first.lines().len(), 1);
    assert_eq!(hear.recv().unwrap(), FromTheDaemon::Unreachable);
    service.join().unwrap();
    drop(hear);
    listening.join().unwrap();
}
