//! The entry, as a journal listening on a socket receives it — and the refusal
//! a caller relies on when no journal is there.

use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::UnixDatagram;

use super::{the_entry, written_to_the_journal};

/// A folder of this test's own, private to it.
fn a_folder(named: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-try-{named}-{}", std::process::id()));
    std::fs::create_dir_all(&folder).expect("a folder");
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o700))
        .expect("a private folder");
    folder
}

/// **A journal listening receives the account whole, found by `alo-try`.**
#[test]
fn a_listening_journal_receives_the_account() {
    let folder = a_folder("journal");
    let socket = folder.join("socket");
    let journal = UnixDatagram::bind(&socket).expect("a journal of the test's own");

    written_to_the_journal(&socket, "alo-try: the box was joined").expect("it was taken");

    let mut received = [0_u8; 512];
    let length = journal.recv(&mut received).expect("an entry arrived");
    assert_eq!(
        received.get(..length),
        Some(&b"SYSLOG_IDENTIFIER=alo-try\nPRIORITY=4\nMESSAGE=alo-try: the box was joined\n"[..])
    );
    std::fs::remove_dir_all(&folder).ok();
}

/// **The refusal path: with no journal there, the caller is told**, so the
/// account can be said somewhere else rather than lost.
#[test]
fn no_journal_is_an_error_the_caller_hears() {
    let folder = a_folder("no-journal");
    assert!(written_to_the_journal(&folder.join("nobody-listens"), "an account").is_err());
    std::fs::remove_dir_all(&folder).ok();
}

/// **An account cannot add a field of its own**: a line break in it is a
/// space, so the entry keeps exactly three fields.
#[test]
fn an_account_cannot_add_a_field() {
    let entry = the_entry("one\nPRIORITY=0\rtwo");
    let entry = String::from_utf8(entry).expect("text");
    assert_eq!(entry.lines().count(), 3, "{entry:?}");
    assert!(entry.ends_with("MESSAGE=one PRIORITY=0 two\n"), "{entry:?}");
}
