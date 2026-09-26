//! Every sentence a person meets on the morning the desk is not the desk they
//! left — walked through the real values, and held to the table in the report
//! that records it.
//!
//! Task 12 of `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`. Task 7
//! walks one person from locking their machine to docking it at another desk,
//! and its steps 6 and 7 — the sleep and the wake — **say nothing**, which was
//! right when nothing had been decided about a desk that changed while the
//! machine was asleep. Task 11 decided it and added
//! `displays.the-desk-changed`, a sentence **no recorded sequence meets**.
//!
//! This is that sequence. Task 7's walk and its table are not edited: a
//! published report is never rewritten, and that walk is still true.
//!
//! # What this can ask that no other test can
//!
//! Each crate's own tests hold its sentences one at a time. What none of them
//! can ask is whether the sentences a resume at a new desk produces read as
//! **one account** or as several crates talking past each other — and on this
//! morning there are three sources at once: the notes on
//! `alo_displays::Attached`, the screens `alo_displays::Resumed` says have gone,
//! and the ones it says are back.
//!
//! The criterion this walk is read against, and it is a measurement rather than
//! a taste: read in the order the machine emits them, by somebody who does not
//! know which crate said which, **does a reader learn what happened and what,
//! if anything, they should do?** Two ways that fails, and both are findings
//! rather than a rewording — an arbitrary order, because an account has a
//! sequence and a set does not; and any two sentences restating one fact in
//! different words, because a person reads that as two events. What this walk
//! found is in the report.
//!
//! # The table is the report's, and this test reads it
//!
//! The sequence is recorded in [`THE_REPORT`] under [`THE_WALK`], and this test
//! parses that table rather than a copy of it. A sentence that changes without
//! the table fails here, and so does a table edited to say something the
//! machine does not.
//!
//! # What is real here, and what is not
//!
//! Every sentence comes out of the machine's one assembled vocabulary
//! (`alo_saying::everything_this_machine_can_say`) through the value that really
//! produces it: a `Note`, a `Moved`, a `CameBack`. The screens are real
//! `alo_displays::Reported`s and the arrangement is real.
//!
//! **What is not here is the machine.** No lid has closed and nothing has
//! suspended: [`TheLaptop`] is this file's stand-in for what alo OS would ask of
//! the base, and no screen has been plugged into anything. That is the same
//! boundary task 7's walk draws, and the plan says so — a lid that has never
//! closed on certified hardware is code and nothing more.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use alo_accounts::{Accounts, Session};
use alo_capability::Grants;
use alo_displays::{Attached, Changes as Screens, Panel, Reported, Socket, Support};
use alo_locking::Seat;
use alo_overlay::Summoning;
use alo_sleeping::{
    Decided, Displays, Holding, Inhibit, LockedFirst, Logind, NotHeld, NotSlept, Settings, Slept,
    Why, asked,
};
use alo_strings::{Said, Strings};

/// The report the walk is recorded in, relative to the repository.
///
/// It moved when task 13 split `displays.the-desk-changed`: the sequence changed,
/// so the table was **republished in a follow-up report** and this points at that
/// one. Task 12 published the first table and it is not edited — it was true of
/// the sentence as it then was, and a published report is never rewritten. The
/// table this reads is always the newest publication of it.
const THE_REPORT: &str = "docs/autonomy/updates/the-reassurance-said-only-when-it-is-true.md";

/// The heading the table is under.
const THE_WALK: &str = "## The walk, sentence by sentence";

/// The number this machine's person runs as.
const PERSON: u32 = 1000;

/// Anna's password, and it is right.
const ANNAS: &str = "correct horse battery staple";

/// The evening she closes the lid at home.
fn evening() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// The next morning, at the office.
fn morning() -> SystemTime {
    evening() + Duration::from_secs(13 * 60 * 60)
}

/// That evening again, back at her own desk.
fn back_home() -> SystemTime {
    morning() + Duration::from_secs(10 * 60 * 60)
}

/// The machine's one vocabulary, as a shell holds it.
fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("alo OS's own words"))
}

/// Anna's session, as a sign-in really opens one.
fn anna() -> Session {
    let mut accounts = Accounts::none().unwrap();
    accounts.created("anna", PERSON, ANNAS).unwrap();
    let signed_in = accounts.signs_in("anna", ANNAS).unwrap();
    Session::opened(signed_in, PERSON).unwrap()
}

/// The laptop's own panel: it says nothing about itself, which is what most
/// built-in screens do, so it is named by where it is plugged in.
fn the_laptop() -> Reported {
    Reported::of(
        Socket::named("eDP-1").unwrap(),
        None,
        (1920, 1080),
        Some((294, 165)),
    )
    .unwrap()
}

/// The screen at her own desk, which says what it is.
fn the_screen_at_home() -> Reported {
    Reported::of(
        Socket::named("HDMI-1").unwrap(),
        Some(Panel::of("Iiyama", "ProLite XU2793", Some("H1234")).unwrap()),
        (2560, 1440),
        Some((597, 336)),
    )
    .unwrap()
}

/// The screen at the office, which she has never plugged this machine into.
fn the_screen_at_the_office() -> Reported {
    Reported::of(
        Socket::named("DP-1").unwrap(),
        Some(Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap()),
        (3840, 2160),
        Some((596, 336)),
    )
    .unwrap()
}

/// What the machine underneath was asked, shared between it and its holds.
#[derive(Debug, Default)]
struct Asked {
    /// How many times it was asked to sleep.
    slept: usize,
}

/// A window onto that.
#[derive(Debug, Clone, Default)]
struct Told(Rc<RefCell<Asked>>);

/// One hold, which this walk takes and drops without reading.
#[derive(Debug)]
struct Held;

/// The machine underneath: it takes every hold and sleeps when asked.
///
/// This is where the walk stops being real, exactly as task 7's does.
/// `alo-sleeping`'s own `TheMachinesLogind` speaks to the base over the system
/// bus on Linux; this records that it was asked and says yes, so the walk is a
/// walk through alo OS's decisions and its sentences rather than a claim about a
/// machine that has slept.
struct TheLaptop(Told);

impl Logind for TheLaptop {
    type Held = Held;

    fn hold(&mut self, _inhibit: Inhibit, why: &Said) -> Result<Held, NotHeld> {
        assert!(!why.is_a_bug(), "a hold described by a key: {why}");
        Ok(Held)
    }

    fn sleep(&mut self, _locked_first: LockedFirst) -> Result<(), NotSlept> {
        let mut asked = (self.0).0.borrow_mut();
        asked.slept = asked.slept.saturating_add(1);
        Ok(())
    }
}

/// What the walk has met so far: each sentence with the moment it was met at.
#[derive(Default)]
struct Met {
    /// Every sentence, in order.
    sentences: Vec<(&'static str, String)>,
}

impl Met {
    /// These sentences, met at this moment. Each one came out of the vocabulary
    /// whole: no key, no unfilled gap, and no connector name in any gap.
    fn at(&mut self, moment: &'static str, said: impl IntoIterator<Item = Said>) {
        for sentence in said {
            assert!(!sentence.is_a_bug(), "{moment}: {sentence}");
            assert!(
                sentence.unfilled().is_empty(),
                "{moment}: a gap nobody filled in {sentence}"
            );
            for connector in ["eDP-1", "DP-1", "HDMI-1", "eDP", "DVI-", "VGA-"] {
                assert!(
                    !sentence.text().contains(connector),
                    "{moment}: {connector} is the machinery, and a person reads {sentence}"
                );
            }
            self.sentences.push((moment, sentence.text().to_owned()));
        }
    }
}

/// Anna's machine goes to sleep on a closed lid and wakes at `waking`, with the
/// screens `reported` plugged into it — and the desk is asked at the resume.
///
/// Returns every sentence the resume produces, in **the machine's** order, from
/// `Attached::the_account`. The first version of this walk composed that order
/// itself out of three collections, and that was the ordering finding this task
/// reported: half the sequence was decided in `alo-displays` and half was left to
/// whoever drew it. The order is one opinion in one place now, and this walk reads
/// it rather than inventing it — which is what makes the table below a measurement
/// of what a person meets rather than of what this file chose.
fn slept_and_woke_to(
    attached: &mut Attached,
    remembered: &Screens,
    reported: Vec<Reported>,
    sleeping: SystemTime,
    waking: SystemTime,
    strings: &Strings,
) -> Vec<Said> {
    let told = Told::default();
    let mut laptop = TheLaptop(told.clone());
    let mut holding: Holding<Held> = Holding::none();

    let Decided::Sleeps(going) = asked(
        Why::LidClosed,
        &Settings::shipped(),
        Displays::OnlyItsOwn,
        &mut holding,
        &Grants::default(),
        sleeping,
    ) else {
        unreachable!("a closed lid sleeps");
    };
    // `String` is the name a seat carries; the unit tests in `the_desk.rs` name it
    // the same way.
    let Slept::Asleep(asleep): Slept<String> = going.carried_out(
        Seat::opened(anna()),
        &mut Summoning::closed(),
        &mut laptop,
        sleeping,
    ) else {
        unreachable!("the machine sleeps");
    };
    assert_eq!(told.0.borrow().slept, 1, "it was asked to sleep once");

    let woke = asleep.woke(waking);
    assert!(woke.seat().is_locked(), "a machine wakes locked");

    let resumed = woke
        .the_desk(attached, reported, remembered)
        .expect("screens are plugged in");
    assert!(!resumed.the_same_desk(), "this is a different desk");

    attached.the_account(&resumed, strings)
}

/// The walk, carried out: every sentence a person meets, in order.
fn the_walk() -> Vec<(&'static str, String)> {
    let strings = in_english();
    let mut met = Met::default();

    // What this person's machine has kept for them. It is **not**
    // `Changes::untouched()` past the first step, and that matters: a walk whose
    // machine remembers nothing would have alo OS tell her, every single
    // morning, that the screen she has used all along has never been used with
    // this machine before — which would be a finding about this test rather than
    // about alo OS. A machine that keeps a person's settings is the machine
    // `alo-displays` is for, so this walk keeps them, as a session does.
    let mut remembered = Screens::untouched();

    // 1. Anna signs in at her own desk, with her laptop open and her own screen
    //    beside it. Neither has been used with this machine before, so alo OS
    //    places each and says so — and what it worked out is then hers.
    let mut attached = Attached::now(
        vec![the_laptop(), the_screen_at_home()],
        &remembered,
        Support::Fractional,
    )
    .unwrap();
    met.at(
        "Anna signs in at her own desk, with her own screen beside the laptop",
        attached
            .notes()
            .iter()
            .map(|note| note.said(&strings))
            .collect::<Vec<_>>(),
    );
    remembered.remember(attached.arrangement().clone());

    // 2. She closes the lid that evening and opens it at the office, where a
    //    screen this machine has never seen is plugged in and her own is gone.
    met.at(
        "she opens the lid at the office, where the screens are not the ones she left",
        slept_and_woke_to(
            &mut attached,
            &remembered,
            vec![the_laptop(), the_screen_at_the_office()],
            evening(),
            morning(),
            &strings,
        ),
    );
    // The office is a desk of hers now too, so what alo OS worked out there is
    // kept beside what it worked out at home — one arrangement per set of
    // screens, which is what `Changes::for_screens` is keyed by.
    remembered.remember(attached.arrangement().clone());

    // 3. At the end of the day she closes it again and opens it at home, where
    //    her own screen is back and the office's has gone.
    met.at(
        "she opens it again at her own desk, where her own screen is back",
        slept_and_woke_to(
            &mut attached,
            &remembered,
            vec![the_laptop(), the_screen_at_home()],
            back_home(),
            back_home() + Duration::from_secs(60 * 60),
            &strings,
        ),
    );

    met.sentences
}

/// **The walk produces exactly the sequence the report records.**
#[test]
fn the_walk_is_the_sequence_the_report_records() {
    let walked = the_walk();
    let recorded = the_table();

    // How many first, and the whole walk with it. A row-by-row comparison that
    // fires before this one reports the first cell that differs and says nothing
    // about the sequence, which is the thing this test is for — and it leaves
    // whoever is writing the table guessing at the rest of it a row at a time.
    assert_eq!(
        walked.len(),
        recorded.len(),
        "the walk met {} sentences and {THE_REPORT} records {}. What the machine said, \
         in order, so that the table is written from the walk and never the walk from the \
         table:\n{}",
        walked.len(),
        recorded.len(),
        walked
            .iter()
            .map(|(moment, sentence)| format!("| {moment} | {sentence} |"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    for (which, (walked, recorded)) in walked.iter().zip(&recorded).enumerate() {
        assert_eq!(
            (walked.0, walked.1.as_str()),
            (recorded.0.as_str(), recorded.1.as_str()),
            "row {} of {THE_REPORT} is not what the machine said",
            which + 1
        );
    }
}

/// **The fact task 11 decided is met.** That is the whole reason this walk
/// exists: a machine waking to screens it did not sleep with was decided, built
/// and tested one sentence at a time, and no recorded sequence had it.
///
/// The sentence has since changed hands. Task 11 added
/// `displays.the-desk-changed`, which said this **and** promised that the
/// arrangement made elsewhere was waiting — a promise it made to somebody
/// standing at the desk in question, which this walk is what found. Task 13 split
/// it under ADR 0068 and that key retired, so what is held here is
/// `Note::TheDeskChanged`, the fact rather than the wording: whichever sentence
/// carries it, the walk must meet it.
#[test]
fn the_fact_task_eleven_decided_is_met() {
    let strings = in_english();
    let the_desk_changed = alo_displays::Note::TheDeskChanged.said(&strings);
    let walked = the_walk();
    assert!(
        walked
            .iter()
            .any(|(_, sentence)| sentence == the_desk_changed.text()),
        "the walk never met the sentence it was written for: {walked:#?}"
    );
}

/// **Every sentence is met once.** Two sentences the same, word for word, in one
/// walk is a person told the same thing twice — and the second of the two ways
/// the criterion in this file's header fails.
#[test]
fn no_sentence_is_said_twice_at_one_moment() {
    let walked = the_walk();
    for (moment, sentence) in &walked {
        let how_many = walked
            .iter()
            .filter(|(at, said)| at == moment && said == sentence)
            .count();
        assert_eq!(how_many, 1, "{moment} says this twice: {sentence}");
    }
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

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
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
        .filter(|line| !line.contains("---"))
        .filter_map(|line| {
            let mut cell = line.trim_matches('|').split('|').map(str::trim);
            let moment = cell.next()?;
            let sentence = cell.next()?;
            if moment == "When" {
                return None;
            }
            Some((moment.to_owned(), sentence.to_owned()))
        })
        .collect()
}
