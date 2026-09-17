//! Every sentence a person meets in a working day's devices — pairing
//! headphones, taking a call on them, covering the camera, running the battery
//! down — walked through the real values and held to the table in the report
//! that records it.
//!
//! Task 6 of `docs/autonomy/v0-5-devices-and-media-plan.md`. The five tasks
//! before it each end in sentences somebody reads at a moment when something is
//! already going on: a call, a cable, a battery. Each crate's own tests hold its
//! sentences one at a time. What none of them can show is **the sequence** —
//! whether these read as one account when they arrive one after another, or as
//! four crates talking past each other.
//!
//! # The table is the report's, and this test reads it
//!
//! The sequence is recorded in [`THE_REPORT`] under [`THE_WALK`], and this test
//! parses that table rather than a copy of it. A sentence that changes without
//! the table changing fails here, and so does a table edited to say something
//! the machine does not say. A later change that moves a sentence publishes the
//! table again in a follow-up report and points [`THE_REPORT`] at it, because a
//! published report is never rewritten.
//!
//! # Where the walk is quiet, it says so
//!
//! Three of its moments produce **no sentence at all** — a call starting, a
//! headset taking it, and the pinned device being the one chosen — and they are
//! rows in the table like the rest, reading `(nothing is said)`. A walk that
//! listed only the sentences would hide the best thing about this part of the
//! machine, which is how little of it a person has to read.
//!
//! # It is the decisions, not the hardware
//!
//! Every sentence here comes out of the real code that would produce it on a
//! machine — a real grant, a real pairing, a real file — but the devices are
//! values rather than a headset somebody plugged in. The hardware halves are
//! held by each crate's own on-a-machine tests, which say what they measured.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_bluetooth::testing::{AService, a_device};
use alo_bluetooth::{Asked, Devices, Kind as DeviceKind, granting, pair_with};
use alo_cameras::{Seen, TheSwitches, Which, every_camera_let_go};
use alo_capability::{Applicant, Ask, Facility, Grant, Grants, Reach};
use alo_measuring::{Number, Process, Source};
use alo_opening::ThisMachine;
use alo_power::{Charge, Charging, Reading, Telling, TheModel, what_is_draining_it};
use alo_sound::device::{Identity, Kind as SoundKind, OneDevice};
use alo_sound::keeping::Kept;
use alo_sound::testing::{AMachine, a_machine};
use alo_sound::{Mute, Pinned, bring_into_line};
use alo_strings::{Filling, Strings, Word};

/// The report this walk is recorded in.
///
/// The first table was published in
/// `docs/autonomy/updates/the-walk-through-a-working-days-devices.md` and stands
/// as it was: step 9 has since changed, because `alo-opening` learned the
/// containers people are sent and a film stopped reading as a file nothing
/// recognises. A published report is never rewritten, so the new table is a
/// follow-up and this points at it.
const THE_REPORT: &str =
    "docs/autonomy/updates/the-walk-through-a-working-days-devices-after-media-kinds.md";

/// The heading its table is under.
const THE_WALK: &str = "## The walk, sentence by sentence";

/// What a moment with nothing to say is written as.
const NOTHING: &str = "(nothing is said)";

/// Everything this machine can say, in the language this test reads.
fn strings() -> Strings {
    Strings::of(
        alo_saying::everything_this_machine_can_say().expect("this machine's own vocabulary"),
    )
}

/// One sentence, as a person reads it.
fn said(strings: &Strings, word: Word) -> String {
    strings
        .say(&word.key(), &Filling::nothing())
        .text()
        .to_owned()
}

/// **The walk**: each moment, and what a person reads at it.
fn the_walk() -> Vec<(&'static str, String)> {
    let strings = strings();
    let mut walk: Vec<(&'static str, String)> = Vec::new();

    // 1 and 2. A person pairs headphones they chose out of what was found, and
    // the device asks them to compare six digits.
    let headphones = a_device("AA:BB:CC:DD:EE:01", "Headphones", DeviceKind::Audio, false);
    let service = AService::with(vec![headphones.clone()]).asking(Asked::TheSameOnBoth(123_456));
    let found = service.now().expect("the service answered");
    let asked = Asked::TheSameOnBoth(123_456);
    walk.push((
        "The headphones ask a person to compare six digits — 123456",
        said(&strings, asked.word()),
    ));
    pair_with(&service, &found, headphones.address()).expect("a person chose it and said yes");

    // The first device this machine has ever paired with, and the one thing a
    // person is told about what pairing it did.
    let paired = alo_bluetooth::TheDevices::reported(
        vec![a_device(
            "AA:BB:CC:DD:EE:01",
            "Headphones",
            DeviceKind::Audio,
            true,
        )],
        alo_bluetooth::Radio::On,
        true,
    );
    walk.push((
        "and it is the first device this machine has paired with",
        said(
            &strings,
            granting::what_a_person_is_told(&paired).expect("the first device is worth a sentence"),
        ),
    ));

    // 3. A call starts, on the laptop's own speakers, with nothing pinned.
    let speakers = "laptop-speakers";
    let headset = "the-headphones";
    let mut machine = AMachine::answering(a_machine(
        &[(speakers, SoundKind::Output)],
        Some(speakers),
        None,
    ));
    let brought = bring_into_line(&mut machine, &Kept::default()).expect("a machine that answers");
    walk.push((
        "A call starts, on the laptop's own speakers",
        sentences(&strings, brought.say()),
    ));

    // 4. The headphones are connected mid-call, and a person had pinned them.
    let pinned = Kept {
        pinned: Pinned::nothing().and(&OneDevice::reported(
            Identity::reported(headset).expect("named"),
            "Headphones",
            SoundKind::Output,
        )),
        ..Kept::default()
    };
    let mut machine = AMachine::answering(a_machine(
        &[(speakers, SoundKind::Output), (headset, SoundKind::Output)],
        Some(speakers),
        None,
    ));
    let brought = bring_into_line(&mut machine, &pinned).expect("a machine that answers");
    walk.push((
        "The headphones connect mid-call, and take the call",
        sentences(&strings, brought.say()),
    ));

    // 5. They are taken off and put away, and the machine chooses the speakers.
    let mut machine = AMachine::answering(a_machine(
        &[(speakers, SoundKind::Output)],
        Some(speakers),
        None,
    ));
    let brought = bring_into_line(&mut machine, &pinned).expect("a machine that answers");
    walk.push((
        "The headphones are put away, and the call comes back to the speakers",
        sentences(&strings, brought.say()),
    ));

    // 6. The person mutes their microphone.
    walk.push((
        "The person mutes their microphone",
        said(&strings, Mute::On.word()),
    ));

    // 7. And turns the camera off for everything.
    let done = every_camera_let_go(&Seen::default());
    walk.push((
        "The person turns the camera off for everything",
        said(&strings, done.word()),
    ));

    // 8. An application that really was granted the camera asks for it anyway.
    // The grant is a real one and it really does allow: what a person reads
    // next is the switch, not a missing permission.
    let application = Applicant::named("world.alo.example.VideoCall");
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked_for(
            &application.grantee(),
            Reach::Facility(Facility::Camera),
            SystemTime::UNIX_EPOCH,
            Duration::from_secs(60 * 60),
        )
        .expect("an application may be granted a facility"),
    );
    assert!(
        grants
            .allowing(
                &application,
                &Ask::Facility(Facility::Camera),
                SystemTime::UNIX_EPOCH
            )
            .is_ok(),
        "the walk's application does not hold the grant the row says it holds"
    );
    let switches = TheSwitches::both_on().switching(Which::Camera);
    let turned_off = switches
        .may_open(Which::Camera)
        .expect_err("a person turned it off");
    walk.push((
        "An application that was granted the camera asks for it",
        said(&strings, turned_off.word()),
    ));

    // 9. A video file arrives and is opened.
    walk.push((
        "A video file is opened",
        what_this_machine_says_about_a_video_file(&strings),
    ));

    // 10 and 11. The battery reaches a fifth, and the model is what is using
    // the machine.
    let telling = Telling::nothing_yet();
    let (telling, low) = telling.about(&a_reading(20, Charging::Discharging));
    walk.push((
        "The battery reaches a fifth",
        sentences(&strings, &low.into_iter().collect::<Vec<_>>()),
    ));
    let draining = what_is_draining_it(
        &[a_process(42, "the model", 800)],
        &TheModel::running_as(vec![42]),
    );
    walk.push((
        "and what is using the machine is the person's own model",
        sentences(&strings, &draining.word().into_iter().collect::<Vec<_>>()),
    ));

    // 12. And it is nearly gone.
    let (_, nearly) = telling.about(&a_reading(4, Charging::Discharging));
    walk.push((
        "The battery is nearly gone",
        sentences(&strings, &nearly.into_iter().collect::<Vec<_>>()),
    ));

    walk
}

/// The sentences of a moment, or [`NOTHING`] where a moment says nothing.
fn sentences(strings: &Strings, words: &[Word]) -> String {
    if words.is_empty() {
        return NOTHING.to_owned();
    }
    words
        .iter()
        .map(|word| said(strings, *word))
        .collect::<Vec<_>>()
        .join(" ")
}

/// What this machine says about a video file today, through the real road a
/// file takes.
fn what_this_machine_says_about_a_video_file(strings: &Strings) -> String {
    let at = std::env::temp_dir().join(format!("alo-walk-{}.mkv", std::process::id()));
    let mut file = File::create(&at).expect("a file for this test");
    // The first bytes of a Matroska file, which is what a video a person is
    // sent usually is.
    file.write_all(&[0x1A, 0x45, 0xDF, 0xA3, 0x01, 0x00, 0x00, 0x00])
        .expect("written");
    file.write_all(&[0; 512]).expect("written");
    drop(file);

    let mut reading = File::open(&at).expect("the file this test wrote");
    let decided = alo_opening::decide(
        &mut reading,
        at.file_name().expect("a name"),
        &ThisMachine::with_nothing(),
    )
    .expect("the file could be read");
    drop(fs::remove_file(&at));

    decided
        .said(strings)
        .iter()
        .map(|said| said.text().to_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

/// One reading of a battery.
fn a_reading(per_cent: u8, charging: Charging) -> Reading {
    Reading::taken(
        Charge::reported(per_cent).expect("a charge"),
        charging,
        Some(-1_000),
        SystemTime::UNIX_EPOCH,
    )
}

/// One running process, as a reading of a machine holds them.
fn a_process(pid: u32, name: &str, processor: u64) -> Process {
    let from = || Source::of("/proc/stat", "cpu");
    Process {
        pid,
        name: name.to_owned(),
        memory: Number::known(0, from()),
        processor: Number::known(processor, from()),
        read: Number::known(0, from()),
        written: Number::known(0, from()),
        received: Number::known(0, from()),
        sent: Number::known(0, from()),
        network: alo_measuring::Network {
            namespace: None,
            shared_with: 0,
        },
    }
}

/// This repository.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
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
/// heading — each as its moment and its sentence, without the header.
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
fn a_working_days_devices_read_as_the_table_in_the_report() {
    let walk = the_walk();
    assert!(
        as_read(&walk) == the_table(),
        "the walk and the table in {THE_REPORT} differ. The walk reads:\n\n{}\n",
        as_a_table(&walk)
    );
}

/// **Every sentence these four crates can say is in the machine's vocabulary,
/// with a note for whoever translates it.**
#[test]
fn every_sentence_in_these_crates_is_collected_and_carries_a_note() {
    let machine = alo_saying::everything_this_machine_can_say().expect("this machine's words");
    for (crate_named, words) in EVERY_SENTENCE {
        for word in words {
            assert!(
                machine.phrase(&word.key()).is_some(),
                "{crate_named} says {} and this machine does not know it",
                word.named()
            );
            assert!(
                word.note().is_some(),
                "{crate_named}'s {} has no note for whoever translates it",
                word.named()
            );
            assert!(
                !word.says().trim().is_empty(),
                "{crate_named}'s {} says nothing",
                word.named()
            );
        }
    }
}

/// **No sentence names what runs underneath.**
///
/// The plan's own list, and the reason is one sentence: a person pairing
/// headphones is pairing headphones. Every name here is a thing alo OS rents
/// and could replace, and a sentence that named one would be a sentence that
/// had to change when it did.
#[test]
fn no_sentence_names_the_rented_stack() {
    const NEVER: [&str; 12] = [
        "pipewire",
        "wireplumber",
        "bluez",
        "libcamera",
        "v4l2",
        "alsa",
        "gstreamer",
        "ffmpeg",
        "libaom",
        "dav1d",
        "power-profiles",
        "upower",
    ];
    for (crate_named, words) in EVERY_SENTENCE {
        for word in words {
            let said = word.says().to_lowercase();
            for never in NEVER {
                assert!(
                    !said.contains(never),
                    "{crate_named}'s {} names {never}",
                    word.named()
                );
            }
        }
    }
}

/// Every sentence the four crates of this plan can say.
const EVERY_SENTENCE: [(&str, &[Word]); 4] = [
    ("alo-sound", &alo_sound::words::EVERY_WORD),
    ("alo-bluetooth", &alo_bluetooth::words::EVERY_WORD),
    ("alo-cameras", &alo_cameras::words::EVERY_WORD),
    ("alo-power", &alo_power::words::EVERY_WORD),
];
