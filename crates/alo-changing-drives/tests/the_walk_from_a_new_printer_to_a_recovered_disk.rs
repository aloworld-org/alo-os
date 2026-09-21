//! Every sentence a person meets from *set up this printer* to *that does not
//! open this machine* — walked through the real values, and held to the table
//! in the report that records it.
//!
//! Task 7 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`: *one walk —
//! the agent proposes adding a printer, the person approves, the machine joins
//! a network, a USB drive mounts and ejects, an update is applied through the
//! broker — produces the exact sequence a person meets, recorded as a table and
//! held by one test.*
//!
//! The six tasks before it each end in sentences a person reads, and each
//! crate's own tests hold its sentences one at a time. What none of them can
//! show is **the sequence** — whether the sentences read as one account when
//! they arrive one after another, or as five crates talking past each other. So
//! this walks one person's week:
//!
//! 1. the agent proposes setting up a printer the machine found;
//! 2. the person approves, and it is set up;
//! 3. the agent proposes joining a network, and says what that does to the
//!    conversation they are having;
//! 4. the person approves, and the machine joins;
//! 5. a drive they plug in is opened;
//! 6. the same drive is finished with, and safe to unplug;
//! 7. they pull it out and ask for it again;
//! 8. an update they approved is applied through the broker;
//! 9. the same approval is offered a second time, and the door refuses it;
//! 10. the machine starts on the new version, and what they type does not open
//!     its disk;
//! 11. the key they wrote down is typed with one character wrong.
//!
//! And then the key is typed correctly and the machine starts, saying nothing —
//! which is what a machine that opens does, and why step 10's sentence has to
//! name the key in the same breath as the refusal.
//!
//! # The table is the report's, and this test reads it
//!
//! The exact sequence is recorded in [`THE_REPORT`] under [`THE_WALK`], and
//! this test parses that table rather than a copy of it. A sentence that
//! changes without the table changing fails here, and so does a table edited to
//! say something the machine does not. A later change that moves a sentence
//! publishes the table again in a follow-up report and points [`THE_REPORT`] at
//! it, because a published report is never rewritten.
//!
//! # What is real here, and what is not
//!
//! Every sentence comes out of the machine's one assembled vocabulary
//! (`alo_saying::everything_this_machine_can_say`) through the value that
//! really produces it: an `alo_capability::Call`'s own sentence, a
//! `changed_said`, a `NotChanged`, an `alo_encrypting::TheDiskRefused`. **What
//! is not here is the wire.** Each verb's road from an approval to the broker's
//! real door, and out to the rented service on the other side, is held by the
//! test the task that built it wrote —
//! `alo-changing-printers`' and `alo-changing-network`'s end-to-end tests, and
//! this crate's `only_the_drive_approved_is_mounted_or_ejected.rs` and
//! `only_the_update_approved_is_carried_out.rs`. Walking them again here would
//! test the wire twice and the sequence once; this tests the sequence, which is
//! the thing nothing else looks at.
//!
//! # Why this walk is in this crate
//!
//! It is the workstream's rather than this crate's, and it was written in
//! `alo-brokerd`. It cannot live there: the sentence a person approves is an
//! `alo_capability::Call`'s own, and `alo-brokerd`'s
//! `the_broker_can_make_no_grant` reads that manifest **as text** and refuses
//! `alo-capability` wherever it appears, dev-dependencies included — which is
//! the stricter reading of *the process that mounts a drive can make no grant*
//! and the right one. So it lives in the crate this walk found missing.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_broker::{Answer, Identity, SystemVerb};
use alo_capability::{Approvals, Authorised, Call, Given, Grantee, Grants, Proposal};
use alo_changing_network::{Answered, JOIN_NETWORK, network_verbs, would};
use alo_changing_printers::{ADD_PRINTER, printer_verbs};
use alo_drives::{Drive, Filesystem, Health, Plugged, TheDrives};
use alo_encrypting::{ASecretOnItsWay, TheDisk, TheDiskRefused, TheSequence, TheVolume};
use alo_networks::{NetworkName, Primary, Saved, TheNetworks};
use alo_printing::Called;
use alo_record::AtTheBroker;
use alo_strings::{Said, Strings};

/// The report the walk is recorded in, relative to the repository.
const THE_REPORT: &str =
    "docs/autonomy/updates/every-sentence-and-the-walk-from-a-new-printer-to-a-recovered-disk.md";

/// The heading the table is under.
const THE_WALK: &str = "## The walk, sentence by sentence";

/// The printer the machine found, by the name it gave itself.
const THE_PRINTER: &str = "Brother HL-L2350DW series";

/// The network the agent proposes joining.
const THE_NETWORK: &str = "Café Central";

/// The network the machine is on while it proposes that.
const AT_HOME: &str = "Home";

/// The drive a person plugs in, by the name the disk service keeps for it.
const THE_DRIVE: &str = "Kingston-DataTraveler-1C1B";

/// The filesystem on it.
const ON_THE_DRIVE: &str = "1234-ABCD";

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long an approval lasts.
fn five_minutes() -> Duration {
    Duration::from_secs(300)
}

/// The agent the person works with.
fn agent() -> Grantee {
    Grantee::named("@assistant")
}

/// The machine's one vocabulary, as a shell holds it.
fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("alo OS's own words"))
}

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// A call proposed by the agent, approved once, and redeemed — and the
/// approval's number, so that offering it a second time can be shown refused.
fn approved(call: &Call) -> Authorised {
    let grants = Grants::default();
    let mut approvals = Approvals::default();
    let id = approvals
        .propose(Proposal::checked(call, &agent(), &grants, noon(), five_minutes()).unwrap());
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&grants, noon())
        .unwrap();
    assert!(
        approvals.approve(id, noon()).is_err(),
        "one approval was answered twice"
    );
    authorised
}

/// The machine's networks while it is on the wireless network at home.
fn on_the_wireless_at_home() -> TheNetworks {
    let named = |name: &str| NetworkName::announced(name.as_bytes()).unwrap();
    TheNetworks {
        visible: Vec::new(),
        saved: vec![Saved::reported(named(AT_HOME), "home")],
        wireless_on: true,
        primary: Some(Primary::reported(true, "home")),
    }
}

/// The drive a person plugged in, with one thing on it, open or not.
fn a_drive_plugged_in(open: bool) -> Drive {
    Drive::reported(THE_DRIVE, Plugged::Removable, Health::NotKnown)
        .unwrap()
        .with(Filesystem::reported(ON_THE_DRIVE, open).unwrap())
}

/// What the walk has met so far: each sentence with the moment it was met at.
#[derive(Default)]
struct Met {
    /// Every sentence, in order.
    sentences: Vec<(&'static str, String)>,
}

impl Met {
    /// These sentences, met at this moment. Each one came out of the
    /// vocabulary whole: no key, no unfilled gap.
    fn at(&mut self, moment: &'static str, said: impl IntoIterator<Item = Said>) {
        for sentence in said {
            assert!(!sentence.is_a_bug(), "{moment}: {sentence}");
            assert!(sentence.unfilled().is_empty(), "{moment}: {sentence}");
            self.sentences.push((moment, sentence.text().to_owned()));
        }
    }
}

/// The walk, carried out: every sentence a person meets, in order.
fn the_walk() -> Vec<(&'static str, String)> {
    let strings = in_english();
    let mut met = Met::default();

    // 1. The agent proposes setting up a printer the machine found. The
    //    sentence is the call's own, which is the one a person is shown.
    let setting_up = printer_verbs()
        .unwrap()
        .call(
            ADD_PRINTER,
            &[(
                alo_changing_printers::words::PRINTER,
                Given::text(THE_PRINTER),
            )],
        )
        .unwrap();
    met.at(
        "The agent proposes setting up a printer this machine found",
        [setting_up.sentence(&strings)],
    );

    // 2. The person approves, the broker carries it out, and the printer is
    //    set up. The approval is redeemed once and cannot be answered twice.
    let authorised = approved(&setting_up);
    assert_eq!(authorised.call().verb(), ADD_PRINTER);
    let printer = Identity::of_what_was_reported(THE_PRINTER.as_bytes());
    let called = Called::announced(Some(THE_PRINTER));
    assert_eq!(
        alo_changing_printers::NotChanged::from_the_brokers(Answer::Carried, &called),
        Ok(())
    );
    met.at(
        "The person approves, and it is set up",
        alo_changing_printers::changed_said(&SystemVerb::AddPrinter(printer), &called, &strings),
    );

    // 3. The agent proposes joining a network — and the sentence says what that
    //    does to the conversation it is being proposed in, because the machine
    //    is answering over the wireless network it would leave.
    let here = on_the_wireless_at_home();
    let joining = alo_changing_network::Change::Join(THE_NETWORK.to_owned());
    let says = would(&joining, &here, Answered::OverTheNetwork);
    assert_eq!(
        says,
        alo_changing_network::ThisConversation::LosesItsConnection
    );
    let join = network_verbs()
        .unwrap()
        .call(
            JOIN_NETWORK,
            &[
                (
                    alo_changing_network::words::NETWORK,
                    Given::text(THE_NETWORK),
                ),
                (
                    alo_changing_network::words::THIS_CONVERSATION,
                    Given::text(says.option()),
                ),
            ],
        )
        .unwrap();
    met.at(
        "The agent proposes joining a network, and says what it does to this conversation",
        [join.sentence(&strings)],
    );

    // 4. The person approves, and the machine joins.
    let authorised = approved(&join);
    assert_eq!(authorised.call().verb(), JOIN_NETWORK);
    assert_eq!(
        alo_changing_network::NotChanged::from_the_brokers(
            Answer::Carried,
            alo_changing_network::NotChanged::CouldNotFinish,
        ),
        Ok(())
    );
    met.at(
        "The person approves, and the machine joins",
        [alo_changing_network::changed_said(&joining, &strings)],
    );

    // 5. A drive is plugged in, and opened for the person.
    let drives = TheDrives {
        drives: vec![a_drive_plugged_in(false)],
    };
    let drive = alo_changing_drives::the_drive_called(&drives, THE_DRIVE).unwrap();
    let on_it = alo_changing_drives::what_can_be_opened(drive)
        .unwrap()
        .first()
        .unwrap();
    let opening = alo_changing_drives::to_open(drive, on_it).unwrap();
    assert_eq!(
        alo_changing_drives::NotChanged::from_the_brokers(Answer::Carried, THE_DRIVE),
        Ok(())
    );
    met.at(
        "A drive is plugged in, and opened",
        alo_changing_drives::changed_said(
            &alo_changing_drives::Change::Open.to(opening),
            THE_DRIVE,
            &strings,
        ),
    );

    // 6. The person finishes with it, and it is safe to unplug.
    let finishing = alo_changing_drives::to_finish_with(drive).unwrap();
    assert_ne!(finishing, opening, "one identity for two different things");
    met.at(
        "The person finishes with it",
        alo_changing_drives::changed_said(
            &alo_changing_drives::Change::FinishWith.to(finishing),
            THE_DRIVE,
            &strings,
        ),
    );

    // 7. They pull it out, and ask for it again. Nothing is guessed at.
    let nothing_plugged_in = TheDrives::default();
    let gone = alo_changing_drives::the_drive_called(&nothing_plugged_in, THE_DRIVE).unwrap_err();
    met.at(
        "They pull it out, and ask for it again",
        [gone.said(&strings)],
    );

    // 8. An update they approved is applied through the broker. Nothing has
    //    happened to the machine yet, and what they read says so.
    let build = Identity::of_what_was_reported(b"the build they approved");
    assert_eq!(
        alo_changing_updates::NotChanged::from_the_brokers(
            Answer::Carried,
            alo_changing_updates::Change::Apply,
        ),
        Ok(())
    );
    met.at(
        "An update they approved is applied",
        alo_changing_updates::changed_said(
            &alo_changing_updates::Change::Apply.to(build),
            &strings,
        ),
    );

    // 9. The same approval is offered a second time. One approval, one
    //    execution: the door has already spent it.
    let spent = alo_changing_updates::NotChanged::from_the_brokers(
        Answer::Refused(AtTheBroker::ApprovalSpent),
        alo_changing_updates::Change::Apply,
    )
    .unwrap_err();
    met.at(
        "The same approval is offered a second time",
        [spent.said(&strings)],
    );

    // 10. The machine starts on the new version, and what they type does not
    //     open its disk.
    met.at(
        "The machine starts, and what they type does not open its disk",
        [alo_enrolling::in_the_persons_language(
            alo_enrolling::about_what_the_disk_refused(TheDiskRefused::WhatWasGivenDoesNotOpenIt),
            &strings,
        )],
    );

    // 11. The key they wrote down, typed with one character wrong. The machine
    //     says the same thing, because which secret was wrong is a fact about
    //     somebody's own key and it never guesses at one.
    met.at(
        "The key they wrote down, typed with one character wrong",
        [alo_enrolling::in_the_persons_language(
            alo_enrolling::about_what_the_disk_refused(TheDiskRefused::WhatWasGivenDoesNotOpenIt),
            &strings,
        )],
    );

    met.sentences
}

/// The table under [`THE_WALK`] in [`THE_REPORT`].
fn the_table() -> Vec<(String, String)> {
    let report = fs::read_to_string(the_repository().join(THE_REPORT))
        .unwrap_or_else(|why| panic!("{THE_REPORT} could not be read: {why}"));
    let rows = rows_under(&report);
    assert!(
        !rows.is_empty(),
        "{THE_REPORT} has no table under {THE_WALK:?}"
    );
    rows
}

/// The rows of the table under [`THE_WALK`] in this text, before the next
/// heading — each as its moment and its sentence, without the header and the
/// line under it.
fn rows_under(report: &str) -> Vec<(String, String)> {
    report
        .lines()
        .skip_while(|line| line.trim() != THE_WALK)
        .skip(1)
        .take_while(|line| !line.starts_with("## "))
        .skip_while(|line| !line.starts_with('|'))
        .take_while(|line| line.starts_with('|'))
        .skip(2)
        .map(|row| {
            let cells: Vec<&str> = row
                .trim()
                .trim_matches('|')
                .split(" | ")
                .map(str::trim)
                .collect();
            let [_, moment, sentence] = cells.as_slice() else {
                panic!("a row is a step, a moment and a sentence: {row}");
            };
            ((*moment).to_owned(), (*sentence).to_owned())
        })
        .collect()
}

/// The walk, written as the table's rows.
fn as_a_table(walk: &[(&str, String)]) -> String {
    walk.iter()
        .enumerate()
        .map(|(step, (moment, sentence))| format!("| {} | {moment} | {sentence} |", step + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The walk as the table's rows are read.
fn as_read(walk: &[(&str, String)]) -> Vec<(String, String)> {
    walk.iter()
        .map(|(moment, sentence)| ((*moment).to_owned(), sentence.clone()))
        .collect()
}

/// **The walk produces exactly the table in the report**, sentence by sentence
/// and in order — so no sentence along it changes without the table changing.
#[test]
fn the_walk_from_a_new_printer_to_a_recovered_disk_reads_as_the_table() {
    let walk = the_walk();
    assert!(
        as_read(&walk) == the_table(),
        "the walk and the table in {THE_REPORT} differ. The walk reads:\n\n{}\n",
        as_a_table(&walk)
    );
}

/// **Only the walk's own table is read, and a changed sentence is a
/// difference.** Not a table under another heading, not rows after the table
/// ends, not a table under the heading after it, and not a heading with no
/// table at all. Held against text, so the check is shown refusing without
/// editing the report.
#[test]
fn only_the_walks_own_table_is_read_and_a_changed_sentence_is_a_difference() {
    let walk = [
        ("A printer is proposed", "set up the printer".to_owned()),
        ("A drive is opened", "it is ready".to_owned()),
    ];
    let header = "| Step | Moment | What a person reads |\n|---|---|---|";
    let report = format!(
        "# A report\n\n## Another table\n\n{header}\n| 1 | Elsewhere | Not the walk |\n\n\
         {THE_WALK}\n\nThe walk.\n\n{header}\n{}\n\nAfter the table.\n\n\
         | 9 | A second table | Not the walk either |\n",
        as_a_table(&walk)
    );
    let walked = as_read(&walk);
    assert_eq!(rows_under(&report), walked);

    let mut reworded = walked.clone();
    if let Some(last) = reworded.last_mut() {
        last.1 = "it is ready now".to_owned();
    }
    assert_ne!(rows_under(&report), reworded, "a reworded sentence");

    let mut reordered = walked;
    reordered.reverse();
    assert_ne!(rows_under(&report), reordered, "a reordered walk");

    let no_table = format!(
        "# A report\n\n{THE_WALK}\n\nNothing yet.\n\n## Next\n\n{header}\n| 1 | Elsewhere | Not the walk |\n"
    );
    assert!(
        rows_under(&no_table).is_empty(),
        "a table under the next heading"
    );
    assert!(rows_under("# A report with no walk\n").is_empty());
}

/// **No sentence on the walk names the machinery.** The same rule the
/// workstream's vocabulary is held to, applied to the sequence a person
/// actually meets — because a sentence can be innocent in a list and arrive
/// beside four others that together read as a log.
#[test]
fn no_sentence_on_the_walk_names_the_machinery() {
    for (moment, sentence) in the_walk() {
        let said = sentence.to_lowercase();
        for never in [
            "luks",
            "tpm",
            "cups",
            "networkmanager",
            "socket",
            "root",
            "cryptsetup",
            "udisks",
            "bootc",
            "ostree",
            "systemd",
            "dbus",
            "/dev/",
            "daemon",
            "capability",
        ] {
            assert!(
                !said.contains(never),
                "{moment} says \"{never}\": {sentence}"
            );
        }
    }
}

/// **The sentence at the end of the walk keeps its promise.** It tells somebody
/// at a machine that will not open to use the key they wrote down, and the key
/// they wrote down really is what opens it: the recovery road is one run of the
/// volume's own tool, the same run an ordinary start is but for which secret it
/// reads — which is what makes *use the key you wrote down* a true sentence
/// rather than a kind one.
#[test]
fn the_key_the_last_sentence_names_is_the_one_that_opens_the_disk() {
    let walk = the_walk();
    let (_, last) = walk.last().expect("the walk says something");
    assert!(last.contains("key you wrote down"), "{last}");

    let disk = TheDisk::named("wwn-0x5002538e40b1a2c3").unwrap();
    let volume = TheVolume::the_partition_of(&disk, 3).unwrap();
    let recovering = TheSequence::opening(&volume, ASecretOnItsWay::TheRecoveryKey);
    let starting = TheSequence::opening(&volume, ASecretOnItsWay::ThePersonsSecret);
    assert_eq!(recovering.runs().len(), 1, "recovering is one run");
    let recovering = recovering.runs().first().expect("recovering is one run");
    let starting = starting.runs().first().expect("starting is one run");
    assert_eq!(recovering.given(), Some(ASecretOnItsWay::TheRecoveryKey));
    assert_ne!(
        recovering.arguments(),
        starting.arguments(),
        "recovering and starting read the same secret"
    );
    assert_eq!(
        recovering.tool(),
        starting.tool(),
        "recovering is not the same tool as starting"
    );
}
