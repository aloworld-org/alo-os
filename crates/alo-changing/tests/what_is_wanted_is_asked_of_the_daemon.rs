//! **The person's desktop asks which applications the turn wanted, and is
//! told exactly what the daemon said** — on a real Unix socket, a thread
//! standing where `alo-agentd` stands.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use alo_changing::{AskingWhatIsWanted, TheDaemonsDoor};
use alo_protocol::{FromAPerson, ToAPerson};

/// A folder of this test's own, empty, on the real disk.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir()
        .join("alo-changing-wanted-tests")
        .join(format!("{}-{what}", std::process::id()));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// A daemon that reads one line, holds it to being `wanted`, and answers.
fn a_daemon_at(socket: &Path, answer: Option<ToAPerson>) -> JoinHandle<()> {
    let listener = UnixListener::bind(socket).unwrap();
    std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line).unwrap();
        assert_eq!(
            FromAPerson::read(line.trim_end()).unwrap(),
            FromAPerson::Wanted,
            "the desktop asked something other than `wanted`"
        );
        if let Some(answer) = answer {
            let written = answer.written().unwrap();
            (&stream).write_all(written.as_bytes()).unwrap();
            (&stream).write_all(b"\n").unwrap();
        }
    })
}

/// What the desktop is told, for this answer from the daemon.
fn told(named: &str, answer: Option<ToAPerson>) -> Option<Vec<String>> {
    let folder = a_folder_of_our_own(named);
    let socket = folder.join("agentd.sock");
    let daemon = a_daemon_at(&socket, answer);
    let wanted = TheDaemonsDoor::at(&socket).wanted();
    daemon.join().unwrap();
    wanted
}

/// **The applications the daemon names are the ones the desktop is told**, in
/// its order, and an empty list is an answer rather than nothing.
#[test]
fn the_desktop_is_told_what_the_daemon_said() {
    let two = vec![
        "com.vscodium.codium".to_owned(),
        "org.mozilla.firefox".to_owned(),
    ];
    assert_eq!(
        told("two", Some(ToAPerson::wanted(two.clone()))),
        Some(two)
    );
    assert_eq!(told("none", Some(ToAPerson::wanted(Vec::new()))), Some(Vec::new()));
}

/// **Any other answer, or none, is not a list**: the desktop draws no sheet
/// from an answer `wanted` cannot earn.
#[test]
fn another_answer_or_none_is_no_list() {
    assert_eq!(told("granted", Some(ToAPerson::granted(1))), None);
    assert_eq!(told("silent", None), None);
}

/// **No daemon at all is no list.**
#[test]
fn no_daemon_is_no_list() {
    let folder = a_folder_of_our_own("nobody");
    assert_eq!(TheDaemonsDoor::at(&folder.join("agentd.sock")).wanted(), None);
}
