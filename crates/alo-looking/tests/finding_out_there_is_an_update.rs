//! Finding out there is an update — the acceptance of task 6 of
//! `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, one criterion at a
//! time.
//!
//! | The acceptance | The test |
//! |---|---|
//! | where this machine checks is read rather than guessed, from the registry and release `image/pinned.toml` pins, through `alo-image` | [`where_this_machine_checks_is_read_from_the_pin_and_written_nowhere_else`] |
//! | a check is one act on the indicator for the whole of it, an `Offered` can be heard no other way, and it fetches the answer and never the build | [`a_check_is_one_act_that_fetches_an_answer_and_never_a_build`] |
//! | when a check happens is somebody's act and never a watcher's: two occasions, no thread, no timer | [`when_a_check_happens_is_somebodys_act_and_never_a_watchers`] |
//! | what a check answers is an offer or one of a closed set of refusals with a sentence each, and a machine with no way out says so once | [`every_refusal_is_a_sentence_and_no_way_out_is_said_once`] |
//! | the answer is kept where a surface reads it back without asking again, and a kept answer about a build this machine no longer runs is refused | [`the_answer_is_kept_and_a_machine_that_has_moved_on_is_not_shown_it`] |
//!
//! The sixth criterion — the whole of it measured against the real place
//! `image/pinned.toml` names, with the proxy honoured and the indicator read
//! afterwards — is `against_the_real_registry.rs`, which reaches the network
//! and is therefore not in the suite that runs on a machine with none.
//!
//! # Why the sentences are read out of the machine's own vocabulary
//!
//! `alo_saying::everything_this_machine_can_say` rather than this crate's own
//! list, because a crate whose words nothing collects compiles, tests, ships
//! and says nothing to anybody in any language — which has happened here
//! before (`crates/alo-collected`). Every sentence in this file is therefore
//! also a check that this crate was collected.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::Cell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_egress::{Destination, Errand, Indicator, OnItsOwn};
use alo_image::{THE_PIN, ThePin};
use alo_keeping_up::{Digest, Offered, Running, Vouching, a_check_at};
use alo_looking::{
    Because, Kept, NoAnswer, NoLongerTrue, NotKept, Place, Release, SaidOnce, Say, ThePlace, look,
};
use alo_saying::everything_this_machine_can_say;
use alo_strings::{Said, Strings};

/// Where this crate's own source is, for the tests that read it.
const THE_SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src");

/// This crate's own manifest.
const THE_MANIFEST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");

/// A moment: nothing in this crate reads a clock.
fn a_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

/// What a check was written into, for a test that reads it back.
///
/// On a booted machine this is the machine's record
/// (`alo_record::Entry::left_on_its_own`, written by `alo-looking-once`); here
/// it keeps what that entry would be made from, so that this file can say *one
/// check, one departure written down* without the record itself.
#[derive(Debug, Default)]
struct WhatWasWrittenDown {
    /// Each check that left, as the errand and the place it was made to.
    departures: Vec<(Errand, Destination)>,
}

impl alo_looking::Noting for WhatWasWrittenDown {
    fn the_check_left(&mut self, underway: &alo_egress::Underway) {
        self.departures
            .push((underway.errand(), underway.destination().clone()));
    }
}

/// A whole build, from one repeated pair.
fn build(pair: &str) -> Digest {
    Digest::read(&format!("sha256:{}", pair.repeat(32))).unwrap()
}

/// The machine's one vocabulary, in English, which is the only place any of
/// these sentences comes from.
fn the_machine() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// The pin this repository ships, read the way the boot environment reads its
/// copy.
fn the_pin() -> ThePin {
    let text =
        std::fs::read_to_string(Path::new(alo_image::THE_IMAGE).join(THE_PIN)).expect("the pin");
    ThePin::read(&text).expect("the pin this repository ships")
}

/// The place this machine's build was pinned to come from.
fn the_place() -> Place {
    Place::on_this_machine().expect("the pin this repository ships")
}

/// Every file of this crate's own source, by path and text.
fn every_source_file() -> Vec<(PathBuf, String)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(THE_SOURCE).expect("this crate's source") {
        let path = entry.expect("a source file").path();
        let text = std::fs::read_to_string(&path).expect("a source file");
        read.push((path, text));
    }
    assert!(
        read.len() >= 10,
        "only {} source files were read",
        read.len()
    );
    read
}

/// A folder of this machine's own, emptied first.
fn a_folder(named: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!(
        "alo-looking-acceptance-{named}-{}",
        std::process::id()
    ));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).expect("a folder");
    folder
}

/// A place that answers whatever this file needs it to, and counts.
#[derive(Debug)]
struct APlaceThatAnswers {
    /// Every name it holds.
    names: Vec<String>,
    /// What it says each release is.
    builds: Vec<(String, String)>,
    /// What it refuses with instead of answering at all.
    refusing: Option<NoAnswer>,
    /// How often it has been asked anything.
    asked: Cell<usize>,
}

impl APlaceThatAnswers {
    /// A place holding this release, saying it is this build, and vouching for
    /// it — which is the ordinary case every test here that is not about
    /// vouching wants.
    fn offering(release: &str, build: &Digest) -> Self {
        Self {
            names: vec![
                release.to_owned(),
                alo_looking::the_name_vouching_for(build),
            ],
            builds: vec![(release.to_owned(), build.as_str().to_owned())],
            refusing: None,
            asked: Cell::new(0),
        }
    }

    /// A place that refuses every question this way.
    fn refusing(refusal: NoAnswer) -> Self {
        Self {
            names: Vec::new(),
            builds: Vec::new(),
            refusing: Some(refusal),
            asked: Cell::new(0),
        }
    }

    /// How often it has been asked anything.
    fn how_often_it_was_asked(&self) -> usize {
        self.asked.get()
    }
}

impl ThePlace for APlaceThatAnswers {
    fn every_name(&self) -> Result<Vec<String>, NoAnswer> {
        self.asked.set(self.asked.get() + 1);
        self.refusing.map_or_else(|| Ok(self.names.clone()), Err)
    }

    fn the_build_of(&self, release: &Release) -> Result<String, NoAnswer> {
        self.asked.set(self.asked.get() + 1);
        if let Some(refusal) = self.refusing {
            return Err(refusal);
        }
        self.builds
            .iter()
            .find(|(named, _)| named == release.named_as())
            .map(|(_, build)| build.clone())
            .ok_or(NoAnswer::ItRefused)
    }
}

/// **Where this machine checks is read rather than guessed.**
///
/// The registry and the release are `image/pinned.toml`'s, read through
/// `alo_image::ThePin`, and **this crate holds no address of its own** — not
/// the repository, not the host. One repository has one answer about where a
/// build comes from, which is what made the pin one file; an organisation's own
/// mirror then arrives by changing what the pin is read from, rather than by
/// finding a second constant that also has to change.
#[test]
fn where_this_machine_checks_is_read_from_the_pin_and_written_nowhere_else() {
    let pin = the_pin();
    let place = the_place();

    assert_eq!(place.source().as_str(), pin.registry());
    assert_eq!(place.not_before().named_as(), pin.version());
    assert_eq!(
        format!("{}/{}", place.host(), place.repository()),
        pin.registry()
    );
    assert_eq!(
        place.destination(),
        &Destination::at(place.host()).expect("the place is somewhere a person can be shown")
    );

    // The repository itself is deliberately not searched for: every crate in
    // this workspace carries `aloworld-org/alo-os` in its own `html_root_url`,
    // which is the repository this code lives in rather than a place it asks.
    for (path, text) in every_source_file() {
        for named in [pin.registry(), place.host()] {
            assert!(
                !text.contains(named),
                "{} names `{named}`, which is the pin's answer and not this crate's",
                path.display()
            );
        }
    }
}

/// **A check is one act, on the indicator for the whole of it, and it fetches
/// the answer and never the build.**
///
/// Three things together. The errand is on the indicator before anything is
/// asked and off it after the answer arrives, whichever way it ends, and both
/// questions happen inside the one line. An `alo_keeping_up::Offered` can be
/// heard no other way: made only from an `Underway`, and refused during any
/// errand that is not a check. And nothing anywhere in this crate can fetch a
/// build — no container library in its manifest, no path that asks for the
/// bytes of one, and nothing outside the one file that keeps the answer so much
/// as names the filesystem.
#[test]
fn a_check_is_one_act_that_fetches_an_answer_and_never_a_build() {
    let mut indicator = Indicator::default();
    let mut written = WhatWasWrittenDown::default();
    let place = the_place();
    let asked = APlaceThatAnswers::offering(place.not_before().named_as(), &build("bb"));

    assert!(indicator.is_quiet());
    let found = look(
        &mut indicator,
        &mut written,
        a_moment(),
        &place,
        &Running::reported(build("aa")),
        Because::ThePersonAsked,
        &asked,
    )
    .unwrap();
    assert!(found.is_ready());
    assert_eq!(asked.how_often_it_was_asked(), 2, "that was not one act");
    assert!(indicator.is_quiet());
    assert!(indicator.showing().is_empty());

    // The line a person reads while it happens, and the errand it is.
    let errand = a_check_at(place.destination().clone());
    assert_eq!(errand.errand(), Errand::CheckingForAnUpdate);
    let mut showing = Indicator::default();
    let underway = showing.beginning_on_its_own(errand, a_moment());
    assert_eq!(showing.showing().len(), 1);
    assert_eq!(
        showing
            .showing()
            .first()
            .expect("the check is showing")
            .said(&the_machine())
            .text(),
        format!("alo OS is checking for an update at {}", place.host())
    );

    // **An offer can be heard no other way.** During the check it can; during
    // every other errand alo OS runs it cannot, so an answer fetched without
    // being shown has nothing to become.
    assert!(Offered::heard(&underway, build("bb"), Vouching::ThePlaceVouchesForIt).is_ok());
    showing.ended_on_its_own(underway);
    for errand in [
        Errand::SigningIn,
        Errand::FetchingAModel,
        Errand::InstallingAnApplication,
        Errand::CheckingForApplicationUpdates,
        Errand::UpdatingAnApplication,
    ] {
        let mut elsewhere = Indicator::default();
        let underway = elsewhere.beginning_on_its_own(
            OnItsOwn::for_(errand, place.destination().clone()),
            a_moment(),
        );
        let refused =
            Offered::heard(&underway, build("bb"), Vouching::ThePlaceVouchesForIt).unwrap_err();
        assert_eq!(refused.during(), errand);
        elsewhere.ended_on_its_own(underway);
    }

    // **No container library, and nothing else that could fetch a build.**
    let manifest = std::fs::read_to_string(THE_MANIFEST).expect("this crate's manifest");
    let named: Vec<&str> = manifest
        .split("[dependencies]")
        .nth(1)
        .expect("a dependencies section")
        .split("\n[")
        .next()
        .expect("the section's body")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split('=').next().map(str::trim))
        .collect();
    assert_eq!(
        named,
        [
            "alo-keeping-up",
            "alo-egress",
            "alo-image",
            "alo-proxy",
            "alo-strings",
            "ureq",
            "serde",
            "serde_json",
            "thiserror",
        ]
    );

    // **Nothing asks for the bytes of a build**, by any name a place or a tool
    // would answer to.
    for (path, text) in every_source_file() {
        for fetching in [
            "/blobs/",
            "podman",
            "skopeo",
            "docker pull",
            "oci-archive",
            "oci-layout",
            "docker-archive",
        ] {
            assert!(
                !text.contains(fetching),
                "{} names {fetching}",
                path.display()
            );
        }
        // **And nothing writes what a place sent.** One file keeps the answer;
        // nothing else that ships has a way to write anything at all.
        // `testing.rs` is compiled only for this crate's own tests and makes
        // the folder they write into.
        if path
            .file_name()
            .is_some_and(|named| named == "kept.rs" || named == "testing.rs")
        {
            continue;
        }
        for writing in ["std::fs", "File::create", "OpenOptions", "write_all"] {
            assert!(
                !text.contains(writing),
                "{} names {writing}, and only the file that keeps the answer may",
                path.display()
            );
        }
    }
}

/// **When a check happens is somebody's act and never a watcher's.**
///
/// Two occasions, each a call something else makes, and nothing in this crate
/// that could make one by itself: no clock read, no thread, no timer, no sleep
/// and no loop waiting for a moment to come round. The moment a check happens
/// at is handed in, as everything in this workspace that has to say *when*
/// hands it in.
#[test]
fn when_a_check_happens_is_somebodys_act_and_never_a_watchers() {
    assert_eq!(Because::EVERY.len(), 2);
    assert!(Because::ThePersonAsked.somebody_is_waiting());
    assert!(!Because::ThisMachineStarted.somebody_is_waiting());

    // Each of them is an answer a surface can act on, and it is written down
    // with the answer so that what is read back says which produced it.
    let mut indicator = Indicator::default();
    let mut written = WhatWasWrittenDown::default();
    let place = the_place();
    for because in Because::EVERY {
        let found = look(
            &mut indicator,
            &mut written,
            a_moment(),
            &place,
            &Running::reported(build("aa")),
            because,
            &APlaceThatAnswers::offering(place.not_before().named_as(), &build("bb")),
        )
        .unwrap();
        assert_eq!(found.because(), because);
        assert_eq!(found.at(), a_moment());
    }

    // Written as code rather than as prose: this file's own crate argues
    // against watchers in several paragraphs, and a search for the word
    // *timer* would find the sentence saying there is not one.
    for (path, text) in every_source_file() {
        for watching in [
            "SystemTime::now",
            "Instant::now",
            "std::thread",
            "thread::spawn",
            "::sleep(",
            "set_timeout",
            "Interval",
            "async fn",
            ".await",
            "std::sync::mpsc",
        ] {
            assert!(
                !text.contains(watching),
                "{} names {watching}, which is the first line of a watcher",
                path.display()
            );
        }
    }
}

/// **What a check answers is an offer or one of a closed set of refusals with
/// a sentence each — and a machine with no way out at all says so once.**
///
/// Five refusals, no two reading alike, every one of them saying that nothing
/// on the machine changed, and none of them naming the machinery underneath.
/// Then the promise `docs/features.md` makes about nagging, in the shape
/// `alo-telling` already gave it: the first time there is no road out the
/// person is told, and the second and third times they are not.
#[test]
fn every_refusal_is_a_sentence_and_no_way_out_is_said_once() {
    let strings = the_machine();
    assert_eq!(NoAnswer::EVERY.len(), 5);

    let mut read: BTreeSet<String> = BTreeSet::new();
    for refusal in NoAnswer::EVERY {
        let said: Said = refusal.said(&strings);
        assert!(!said.is_a_bug(), "{refusal:?} is not in the vocabulary");
        assert!(said.unfilled().is_empty(), "{said} has a gap left in it");
        let text = said.text().to_lowercase();
        assert!(
            text.contains("nothing on this machine has changed"),
            "{refusal:?} does not say the machine is as it was: {said}"
        );
        for machinery in [
            "bootc",
            "ostree",
            "deployment",
            "digest",
            "registry",
            "container",
            "sha256",
            "manifest",
            "http",
            "proxy",
        ] {
            assert!(!text.contains(machinery), "{refusal:?} says {machinery}");
        }
        read.insert(said.into_text());
    }
    assert_eq!(read.len(), NoAnswer::EVERY.len(), "two refusals read alike");

    // Every one of them is reachable from a check rather than declared and
    // never said.
    let mut indicator = Indicator::default();
    let mut written = WhatWasWrittenDown::default();
    let place = the_place();
    for refusal in NoAnswer::EVERY {
        let answered = look(
            &mut indicator,
            &mut written,
            a_moment(),
            &place,
            &Running::reported(build("aa")),
            Because::ThePersonAsked,
            &APlaceThatAnswers::refusing(refusal),
        );
        assert_eq!(answered.unwrap_err(), refusal);
    }

    // **And it is said once.** A machine with no road out is told so, and then
    // asked again twice and told nothing.
    let mut said_once = SaidOnce::new();
    let no_road_out = || {
        look(
            &mut Indicator::default(),
            &mut WhatWasWrittenDown::default(),
            a_moment(),
            &place,
            &Running::reported(build("aa")),
            Because::ThisMachineStarted,
            &APlaceThatAnswers::refusing(NoAnswer::NoWayOut),
        )
    };
    let Err(Say::This(first)) = said_once.what_to_say(no_road_out()) else {
        panic!("the first time there was no road out, nothing was said");
    };
    assert_eq!(first, NoAnswer::NoWayOut);
    assert!(!first.said(&strings).is_a_bug());
    for _ in 0..2 {
        assert_eq!(
            said_once.what_to_say(no_road_out()),
            Err(Say::SaidAlready),
            "a machine with no way out said so more than once"
        );
    }

    // And a check that was answered forgets it, so the next outage is news.
    assert!(
        said_once
            .what_to_say(look(
                &mut Indicator::default(),
                &mut WhatWasWrittenDown::default(),
                a_moment(),
                &place,
                &Running::reported(build("aa")),
                Because::ThePersonAsked,
                &APlaceThatAnswers::offering(place.not_before().named_as(), &build("aa")),
            ))
            .is_ok()
    );
    assert_eq!(
        said_once.what_to_say(no_road_out()),
        Err(Say::This(NoAnswer::NoWayOut))
    );
}

/// **The answer is kept where a surface reads it back without asking again,
/// and a kept answer names the build it was about.**
///
/// A settings panel that asked again every time somebody opened it would put a
/// line on the indicator for a question answered a minute ago, which is the
/// opposite of what that indicator is for. So one check writes one file and
/// every surface reads it — and the place is asked exactly twice for the whole
/// of this test, which is the once it was checked.
///
/// And the half that matters more: the machine updates, goes back, or is
/// changed by somebody at a root shell, and the kept sentence is **refused**
/// rather than shown. *An update is ready* about a system nobody is running is
/// the kind of stale sentence a person stops trusting a machine for.
#[test]
fn the_answer_is_kept_and_a_machine_that_has_moved_on_is_not_shown_it() {
    let strings = the_machine();
    let folder = a_folder("kept");
    let kept = Kept::in_folder(&folder);
    let place = the_place();
    let was_running = Running::reported(build("aa"));

    assert_eq!(kept.read().unwrap(), None, "a machine that never looked");

    let mut indicator = Indicator::default();

    let mut written = WhatWasWrittenDown::default();
    let asked = APlaceThatAnswers::offering(place.not_before().named_as(), &build("bb"));
    let found = look(
        &mut indicator,
        &mut written,
        a_moment(),
        &place,
        &was_running,
        Because::ThePersonAsked,
        &asked,
    )
    .unwrap();
    kept.keep(&found).unwrap();

    // Read back, twice, with nothing asked of the place in between.
    for _ in 0..2 {
        let answer = kept.read().unwrap().unwrap();
        assert!(answer.is_still_about(&was_running));
        assert!(answer.is_ready(&was_running).unwrap());
        assert_eq!(answer.because(), Because::ThePersonAsked);
        assert_eq!(answer.at(), a_moment());
        assert!(
            answer
                .said(&was_running, &strings)
                .unwrap()
                .text()
                .starts_with("An update is ready"),
        );
    }
    assert_eq!(
        asked.how_often_it_was_asked(),
        2,
        "reading the kept answer asked the place again"
    );

    // **The machine moved on.** The answer is about a version nobody is
    // running, so it is refused — and what the person reads is that the
    // machine changed and to check again.
    let now = Running::reported(build("bb"));
    let answer = kept.read().unwrap().unwrap();
    assert!(!answer.is_still_about(&now));
    assert_eq!(answer.is_ready(&now), Err(NoLongerTrue));
    assert_eq!(answer.said(&now, &strings), Err(NoLongerTrue));
    let said = NoLongerTrue.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert_eq!(
        said.text(),
        "This machine changed after the update was found, so nothing was changed. Check for the \
         update again"
    );

    // **And what is there and is not an answer is refused**, rather than read
    // as a machine that has never looked — which a surface would show as
    // silence about an update that is waiting.
    std::fs::write(kept.path(), "{\"about\":\"sha256:beef\"}").unwrap();
    assert!(matches!(
        kept.read().unwrap_err(),
        NotKept::NotUnderstood { .. }
    ));

    assert_eq!(
        Kept::on_this_machine().path(),
        Path::new("/var/lib/alo/an-update-was-found"),
        "the answer is not kept where an update leaves it alone"
    );
    drop(std::fs::remove_dir_all(&folder));
}
