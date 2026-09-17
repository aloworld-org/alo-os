//! The broker opens its door only once it knows who the door is for, has
//! somewhere to write down every answer, and has handed its key to the person's
//! group and not the agent's — and what it answers through the door it opened
//! is in the record file before the answer arrives.
//!
//! Task 3 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` inherits the
//! broker's process from task 1: *its binary, its unit, the machine's record it
//! writes to, and how its approving key reaches the turn.* This is the process's
//! start, against paths of the test's own.
//!
//! # Whoever runs it
//!
//! The kernel names this test as whoever runs it, and that is root on the
//! machine the gates run on. A broker told the door is for root refuses to
//! start, by design, so every broker here is started for somebody who is **not**
//! this test — and what it answers this test through the real door is the
//! refusal of a stranger, written down first. That the door hears the person
//! and carries their request out is `alo-broker`'s own socket test and
//! `crates/alo-changing-network`'s end-to-end test; this is the part neither can
//! reach: the machine's record file, the key's file, and the order.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
use std::path::PathBuf;
use std::time::SystemTime;

use alo_broker::asking::ask;
use alo_broker::handing_over::the_key_handed_over;
use alo_broker::{
    Answer, Carrying, Identity, NotCarried, Request, SystemVerb, our_group, our_user,
};
use alo_brokerd::{NotStarted, Places, Started, started};
use alo_keeping::Reading;
use alo_record::{AtTheBroker, Happened};

/// Counts what was carried out.
#[derive(Debug, Default)]
struct Counts(usize);

impl Carrying for Counts {
    fn carry(&mut self, _: SystemVerb, _: u64) -> Result<(), NotCarried> {
        self.0 += 1;
        Ok(())
    }
}

/// A group this test may hand a file to and that is not root's: its own, or —
/// running as root, who may hand a file to any group — one nobody is in.
fn a_group() -> u32 {
    if our_group() == 0 { 4242 } else { our_group() }
}

/// Somebody who is not this test, and not root.
fn somebody_else() -> u32 {
    if our_user() == 0 {
        1000
    } else {
        our_user().wrapping_add(1)
    }
}

/// A machine of this test's own, whose description names `person` and whose
/// agent is in `agents_group`.
fn a_machine(what: &str, person: u32, agents_group: u32) -> Places {
    let root = std::env::temp_dir().join(format!("alo-brokerd-{}-{what}", std::process::id()));
    drop(std::fs::remove_dir_all(&root));
    std::fs::create_dir_all(root.join("run")).unwrap();
    std::fs::create_dir_all(root.join("state")).unwrap();
    let description = root.join("agentd.toml");
    std::fs::write(
        &description,
        format!(
            "format = 1\n\n[logins]\nperson = {person}\nagent = 60989\ngroup = {agents_group}\n"
        ),
    )
    .unwrap();
    Places {
        description,
        record: root.join("state").join("record.jsonl"),
        key: root.join("run").join("approving.key"),
        door: root.join("run").join("door.sock"),
        wanted: root.join("run").join("wanted"),
    }
}

/// Nothing a refused start could have left behind is there.
fn nothing_opened(places: &Places) {
    for left in [&places.key, &places.door, &places.wanted] {
        assert!(
            std::fs::symlink_metadata(left).is_err(),
            "{} was left behind by a start that was refused",
            left.display()
        );
    }
}

/// **Started as it must be, the door opens to its group, the key is that
/// group's to read, the folder a proxy is handed over in is that group's to
/// write, the record exists — and what the door answers through the
/// real socket is in the record file before the answer arrives.**
#[test]
fn a_broker_started_as_it_must_be_writes_down_what_it_answers_on_the_disk() {
    let places = a_machine("started", somebody_else(), a_group().wrapping_add(1));
    let Started {
        listening,
        mut broker,
    } = started(&places, a_group(), |_| Counts::default()).unwrap();

    let door = std::fs::metadata(&places.door).unwrap();
    assert_eq!(door.permissions().mode() & 0o777, 0o660);
    assert_eq!(door.gid(), a_group());
    let key = std::fs::metadata(&places.key).unwrap();
    assert_eq!(key.permissions().mode() & 0o7777, 0o440);
    assert_eq!(key.gid(), a_group());
    let wanted = std::fs::metadata(&places.wanted).unwrap();
    assert!(wanted.is_dir());
    assert_eq!(wanted.permissions().mode() & 0o7777, 0o770);
    assert_eq!(wanted.gid(), a_group());
    assert_eq!(Reading::at(&places.record).unwrap().record().len(), 0);

    let (door, key) = (places.door.clone(), places.key.clone());
    let asking = std::thread::spawn(move || {
        let verb = SystemVerb::JoinNetwork(Identity::of_what_was_reported(b"Home"));
        let token =
            the_key_handed_over(&key, our_user())
                .unwrap()
                .issue(&verb, 12, SystemTime::now());
        ask(&door, &Request::of(verb, token)).unwrap()
    });
    let answered = listening.answer_one(&mut broker).unwrap();
    assert_eq!(answered, Answer::Refused(AtTheBroker::NotTheAgentService));
    assert_eq!(asking.join().unwrap(), answered);
    assert_eq!(broker.carrying().0, 0);

    let kept: Vec<Happened> = Reading::at(&places.record)
        .unwrap()
        .into_record()
        .everything()
        .map(|entry| entry.happened().clone())
        .collect();
    assert!(
        matches!(
            kept.as_slice(),
            [Happened::Brokered {
                verb: None,
                from_approval: None,
                refused: Some(AtTheBroker::NotTheAgentService),
            }]
        ),
        "{kept:?}"
    );
}

/// **A restarted broker hands over a new key**, and the record it opens again
/// is the same record, added to rather than replaced.
#[test]
fn a_restarted_broker_hands_over_a_new_key_and_keeps_the_same_record() {
    let places = a_machine("restarted", somebody_else(), a_group().wrapping_add(1));
    let verb = SystemVerb::ForgetNetwork(Identity::of_what_was_reported(b"home-uuid"));

    let first = started(&places, a_group(), |_| Counts::default()).unwrap();
    let old =
        the_key_handed_over(&places.key, our_user())
            .unwrap()
            .issue(&verb, 1, SystemTime::now());
    drop(first);

    let second = started(&places, a_group(), |_| Counts::default()).unwrap();
    let new =
        the_key_handed_over(&places.key, our_user())
            .unwrap()
            .issue(&verb, 1, SystemTime::now());
    assert_ne!(old, new, "a restarted broker handed over the same key");
    assert!(Reading::at(&places.record).is_ok());
    drop(second);
}

/// **A description that says `alo-agentd` runs as root opens no door.**
#[test]
fn a_description_naming_root_opens_no_door() {
    let places = a_machine("root", 0, 60989);
    assert!(matches!(
        started(&places, a_group(), |_| Counts::default()),
        Err(NotStarted::NotADoor(_))
    ));
    nothing_opened(&places);
}

/// **A process in root's group, or in the agent's own, opens no door and hands
/// over no key** — the agent's login must never be the one that can read it.
#[test]
fn a_broker_in_roots_group_or_the_agents_opens_no_door_and_hands_over_no_key() {
    let places = a_machine("roots-group", somebody_else(), 60989);
    assert!(matches!(
        started(&places, 0, |_| Counts::default()),
        Err(NotStarted::InRootsGroup)
    ));
    nothing_opened(&places);

    let places = a_machine("agents-group", somebody_else(), a_group());
    assert!(matches!(
        started(&places, a_group(), |_| Counts::default()),
        Err(NotStarted::InTheAgentsGroup { .. })
    ));
    nothing_opened(&places);
}

/// **No description, or one that does not say, opens no door.**
#[test]
fn a_machine_that_does_not_say_who_the_door_is_for_opens_none() {
    let places = a_machine("undescribed", somebody_else(), 60989);
    std::fs::write(&places.description, "format = 1\n").unwrap();
    assert!(matches!(
        started(&places, a_group(), |_| Counts::default()),
        Err(NotStarted::NotDescribed(_))
    ));
    nothing_opened(&places);

    std::fs::remove_file(&places.description).unwrap();
    assert!(matches!(
        started(&places, a_group(), |_| Counts::default()),
        Err(NotStarted::NotDescribed(_))
    ));
    nothing_opened(&places);
}

/// **A broker with nowhere to keep its record opens no door and hands over no
/// key**: a folder that is not there, and a file that is not a record, which is
/// left exactly as it was.
#[test]
fn a_broker_that_cannot_keep_its_record_opens_no_door() {
    let places = a_machine("no-record", somebody_else(), a_group().wrapping_add(1));
    let nowhere = Places {
        record: PathBuf::from("/nonexistent-alo-brokerd-folder/record.jsonl"),
        ..places.clone()
    };
    assert!(matches!(
        started(&nowhere, a_group(), |_| Counts::default()),
        Err(NotStarted::NoRecord(_))
    ));
    nothing_opened(&nowhere);

    std::fs::write(&places.record, "somebody else's file\n").unwrap();
    assert!(matches!(
        started(&places, a_group(), |_| Counts::default()),
        Err(NotStarted::NoRecord(_))
    ));
    nothing_opened(&places);
    assert_eq!(
        std::fs::read_to_string(&places.record).unwrap(),
        "somebody else's file\n"
    );
}
