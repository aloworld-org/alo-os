//! One file on a real disk, and the machine it turns out to be.
//!
//! `crates/alo-agentd`'s own tests ask each rule on its own: what has to be true
//! of the file, what the format number does, what each value refuses. This is
//! the join — a description written the way `docs/contracts/machine-description.md`
//! writes it, on the disk the tests are running on, and then the three things a
//! service does with it.
//!
//! **The socket really opens**, from the two logins the file named. **The record
//! really starts**, at the path the file named, and reads back as a record.
//! Neither is a fixture standing in for the file: what is passed to
//! `Listening::at` and to `alo_keeping::Writing::opening` came off the disk.
//!
//! What cannot be reached here is a description belonging to a **third** user,
//! because making one takes a privilege this test does not have on every machine
//! it runs on. That rule is asked of the decision instead, in
//! `crate::trusting`'s own tests, and it is the same limit the rest of this
//! crate has: telling two logins apart takes two logins, and a test process
//! has one.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use alo_agentd::side::Side;
use alo_agentd::unix::{our_group, us};
use alo_agentd::{Described, Listening, NotDescribed, Place, THE_DESCRIPTION, THE_FORMAT};
use alo_keeping::{Reading, Writing};

/// The login this test gives the agent, which is not the one it runs as.
const AN_AGENT: u32 = 989;

/// The one it gives the agent if this test happens to run as [`AN_AGENT`].
const ANOTHER_AGENT: u32 = 990;

/// A folder of this test's own, on the disk the tests are running on.
fn a_directory_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-agentd-described-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// The login this test would give an agent on this machine.
fn an_agent() -> u32 {
    if us().unwrap().raw() == AN_AGENT {
        ANOTHER_AGENT
    } else {
        AN_AGENT
    }
}

/// A description of the machine these tests are running on, with the record
/// going into a folder of this test's own.
fn describing_this_machine(record: &Path) -> String {
    format!(
        r#"format = {THE_FORMAT}

[logins]
person = {person}
agent = {agent}
group = {group}

[agent]
name = "alo"
turn-seconds = 900
proposal-seconds = 300

[record]
path = "{record}"
keeping = {{ for-days = 90 }}
"#,
        person = us().unwrap().raw(),
        agent = an_agent(),
        group = our_group().unwrap().raw(),
        record = record.display(),
    )
}

/// Write a description into a folder of this test's own, at this mode.
fn described(what: &str, said: &str, mode: u32) -> PathBuf {
    let at = a_directory_of_our_own(what).join("agentd.toml");
    std::fs::write(&at, said).unwrap();
    std::fs::set_permissions(&at, Permissions::from_mode(mode)).unwrap();
    at
}

/// **A description on a disk is a machine**, and every value in it arrives where
/// the service reads it.
#[test]
fn a_file_on_a_disk_is_a_machine() {
    let folder = a_directory_of_our_own("whole");
    let record = folder.join("record");
    let at = described("whole-file", &describing_this_machine(&record), 0o600);

    let machine = Described::at(&at, us().unwrap()).unwrap();

    assert_eq!(machine.sides().person(), us().unwrap());
    assert_eq!(machine.sides().agent().raw(), an_agent());
    assert_eq!(machine.agent(), "alo");
    assert_eq!(machine.turn().duration().as_secs(), 900);
    assert_eq!(machine.proposal().duration().as_secs(), 300);
    assert_eq!(machine.record(), record);
    assert_eq!(machine.keeping().days(), Some(90));
}

/// **The socket opens from the file.** The two logins that decide which door a
/// caller is on came off a disk, and a real client connecting to the real socket
/// lands on the person's door — because whoever is running the tests is the
/// person the description names.
#[test]
fn the_two_logins_in_the_file_open_the_real_socket() {
    let folder = a_directory_of_our_own("socket-from-the-file");
    let record = folder.join("record");
    let at = described("socket-file", &describing_this_machine(&record), 0o600);
    let machine = Described::at(&at, us().unwrap()).unwrap();

    let place = Place::beneath(&folder, us().unwrap());
    let listening = Listening::at(place.clone(), machine.sides()).unwrap();

    let client = std::os::unix::net::UnixStream::connect(place.socket()).unwrap();
    let accepted = listening.next().unwrap();
    assert_eq!(accepted.side(), Side::Person);
    assert_eq!(accepted.caller().user(), machine.sides().person());
    drop(client);
}

/// **The record starts where the file said.** This is the half of queue item 20
/// that was waiting on a machine describing itself: the path is read off the
/// disk, `alo-keeping` opens it, and what comes back reads as a record rather
/// than as a file that happens to be there.
#[test]
fn the_record_starts_where_the_file_said_it_would() {
    let folder = a_directory_of_our_own("record-from-the-file");
    let record = folder.join("record");
    let at = described("record-file", &describing_this_machine(&record), 0o600);
    let machine = Described::at(&at, us().unwrap()).unwrap();

    let writing = Writing::opening(machine.record()).unwrap();
    assert_eq!(writing.path(), machine.record());
    drop(writing);

    assert!(machine.record().is_file(), "the record is on the disk");
    let read_back = Reading::at(machine.record()).unwrap();
    assert_eq!(read_back.head().format(), alo_keeping::THE_FORMAT);
}

/// **A description anybody can rewrite describes nothing.** The file names which
/// login is the agent, so a mode that lets somebody else write it is somebody
/// else naming this machine's agent.
#[test]
fn a_description_the_world_can_write_is_refused() {
    let folder = a_directory_of_our_own("loose");
    let record = folder.join("record");
    let at = described("loose-file", &describing_this_machine(&record), 0o666);

    let refused = Described::at(&at, us().unwrap()).unwrap_err();
    assert!(matches!(refused, NotDescribed::Loose { .. }));
    assert!(!record.exists(), "and nothing was started");
}

/// **A description written for a newer alo OS is refused rather than guessed
/// at**, even when everything else in it is a machine this service could serve.
#[test]
fn a_description_from_a_newer_alo_os_is_refused_off_the_disk() {
    let folder = a_directory_of_our_own("newer");
    let record = folder.join("record");
    let said = describing_this_machine(&record).replace("format = 1", "format = 2");
    let at = described("newer-file", &said, 0o600);

    assert!(matches!(
        Described::at(&at, us().unwrap()).unwrap_err(),
        NotDescribed::AnotherFormat {
            format: 2,
            reads: 1,
            ..
        }
    ));
}

/// **A link where the description belongs is refused even when it points at a
/// description we own**: what alo-agentd would really read would be decided by
/// whoever can change the link.
#[test]
fn a_link_where_the_description_belongs_is_refused() {
    let folder = a_directory_of_our_own("linked");
    let record = folder.join("record");
    let really = described("linked-really", &describing_this_machine(&record), 0o600);

    let at = folder.join("agentd.toml");
    std::os::unix::fs::symlink(&really, &at).unwrap();

    assert!(matches!(
        Described::at(&at, us().unwrap()).unwrap_err(),
        NotDescribed::ALink { .. }
    ));
}

/// **A machine with no description does not start**, and the refusal names the
/// file rather than answering as though the machine had said nothing in
/// particular.
#[test]
fn a_machine_with_no_description_does_not_start() {
    let at = a_directory_of_our_own("absent").join("agentd.toml");
    let refused = Described::at(&at, us().unwrap()).unwrap_err();
    assert!(matches!(refused, NotDescribed::Unreadable { .. }));
    assert!(refused.to_string().contains("agentd.toml"), "{refused}");
}

/// The path in the contract is the path the service looks at, and it is one
/// string: a client's installer and this service reading two different places
/// would be a machine described twice.
#[test]
fn the_contract_names_where_the_description_is() {
    assert_eq!(THE_DESCRIPTION, "/etc/alo/agentd.toml");
    assert!(Path::new(THE_DESCRIPTION).is_absolute());
}
