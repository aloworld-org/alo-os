//! Every sentence a person meets from *this machine is locked* to *what was
//! open on it has gone back to it* — walked through the real values, and held
//! to the table in the report that records it.
//!
//! Task 7 of `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`: *one
//! walk — lock, suspend with the lid, resume, unlock, dock to a second display,
//! undock — produces the exact sequence a person meets, recorded in the report
//! as a table and held by one test that fails if a sentence changes without the
//! table; no sentence names `logind`, DRM, EDID, a connector name or any other
//! part of the machinery.*
//!
//! The six tasks before it each end in sentences a person reads, and each
//! crate's own tests hold its sentences one at a time. What none of them can
//! show is **the sequence** — whether the sentences read as one account when
//! they arrive one after another, or as five crates talking past each other. So
//! this walks one person's day and a half:
//!
//! 1. Anna signs in, and alo OS takes the two holds it keeps for a session;
//! 2. her screens are laid out — a laptop panel that says nothing about itself;
//! 3. she asks the agent for something and steps away, and the machine does not
//!    sleep on its own;
//! 4. she locks the screen;
//! 5. she comes back for a moment and presses the key that opens the agent;
//! 6. she closes the lid, and the machine locks and sleeps;
//! 7. she opens it the next morning, and it wakes locked;
//! 8. she types a password that is not hers;
//! 9. she types her own, and what the agent had been working on is over;
//! 10. at the office she plugs in the screen she uses there;
//! 11. she closes the lid with it attached, as she chose;
//! 12. at the end of the day she pulls the cable out;
//! 13. and puts it back.
//!
//! Steps 6 and 7's sleep itself says **nothing**, which is what a machine that
//! goes to sleep and comes back does; the table has no row for it, and
//! inventing one would have been the easiest dishonest thing in this task.
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
//! really produces it: a `Keeper`, a `NotWhileLocked`, an `AfterSleep`, a
//! `Note`, a `Moved`, a `CameBack`, the `Greeted` a wrong password answers
//! with. The session under it is a real `alo_accounts::Session` — a password
//! that really verified — and the turn is a real `alo_turn::Turning` writing to
//! a real record.
//!
//! **What is not here is the machine.** No lid has closed, nothing has
//! suspended, and no screen has been plugged into anything: [`TheLaptop`] is
//! this file's stand-in for what alo OS would ask of the base, counting what it
//! was asked and keeping the sentence it was asked with. That is the same
//! boundary `alo-sleeping`'s own tests draw, and the plan says so: a lid that
//! has never closed on certified hardware is code and nothing more.
//!
//! # Why this walk is in this crate
//!
//! It belongs to the workstream rather than to `alo-sleeping`, and there is
//! nowhere else it can be. `alo-sleeping`, `alo-leaving` and `alo-notifying`
//! each hold a test that reads every manifest in the workspace and refuses any
//! dependant but `alo-saying` and `alo-shell` — so no crate can name all five,
//! and this crate is the only one of the three whose own rule does not stop it
//! reaching `alo-locking` and `alo-displays`, which are the other two the walk
//! goes through. Widening one of those lists to give the walk a tidier home
//! would have been paying for a test with the guarantee the test exists to
//! keep. `tests/every_sentence_this_workstream_says.rs` is the same argument
//! for the audit beside it.

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
use alo_appearance::Appearance;
use alo_capability::Grants;
use alo_context::Context;
use alo_displays::{Attached, Changes as Screens, Panel, Reported, Socket, Support};
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Reaching};
use alo_locking::{Arrived, NotWhileLocked, Seat, Unlocking};
use alo_overlay::{Compositor, Summoning, SurfaceRefused, SurfaceRequest};
use alo_record::Record;
use alo_sleeping::{
    AfterSleep, Decided, Displays, Holding, Inhibit, Lid, LockedFirst, Logind, NotHeld, NotSlept,
    Settings, Slept, TheLidIsOurs, UntilLocked, Why, asked,
};
use alo_strings::{Said, Strings};
use alo_turn::{Doing, Done, Machine, NoBoundary, Turning};

/// The report the walk is recorded in, relative to the repository.
const THE_REPORT: &str =
    "docs/autonomy/updates/every-sentence-and-the-walk-from-lock-to-resume-to-a-new-desk.md";

/// The heading the table is under.
const THE_WALK: &str = "## The walk, sentence by sentence";

/// The number this machine's person runs as.
const PERSON: u32 = 1000;

/// Anna's password, and it is right.
const ANNAS: &str = "correct horse battery staple";

/// A password that is not.
const NOT_ANNAS: &str = "correct horse battery stapler";

/// A fixed moment: the evening she locks the machine.
fn evening() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the turn she asked for lasts.
fn an_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The next morning, long after that turn's hour was up.
fn morning() -> SystemTime {
    evening() + Duration::from_secs(13 * 60 * 60)
}

/// The machine's one vocabulary, as a shell holds it.
fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("alo OS's own words"))
}

/// The machine's egress indicator with nothing leaving, which is what it says
/// on this walk.
///
/// The lock screen carries a `Lamp` made from this: dark, or lit for a number
/// and naming nothing. **Nothing on this walk leaves the machine** — no verb is
/// carried out, nothing is asked of a provider — so the lamp is dark, and the
/// lock screen has no row in the table for it. That the light still fires while
/// locked is `alo-locking`'s own test's subject and is not restated here.
fn nothing_is_leaving() -> Indicator {
    Indicator::default()
}

/// The machine's accounts: Anna, at this machine's number.
fn the_accounts() -> Accounts {
    let mut accounts = Accounts::none().unwrap();
    accounts.created("anna", PERSON, ANNAS).unwrap();
    accounts
}

/// Anna's session, as a sign-in really opens one.
fn anna() -> Session {
    let signed_in = the_accounts().signs_in("anna", ANNAS).unwrap();
    Session::opened(signed_in, PERSON).unwrap()
}

/// The laptop's own panel: thirteen inches, and it says nothing about itself,
/// which is what most built-in screens do.
fn the_laptop() -> Reported {
    Reported::of(
        Socket::named("eDP-1").unwrap(),
        None,
        (1920, 1080),
        Some((294, 165)),
    )
    .unwrap()
}

/// The screen at the office: 27 inches at 3840 by 2160, with a serial number.
fn the_office_screen() -> Reported {
    Reported::of(
        Socket::named("DP-1").unwrap(),
        Some(Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap()),
        (3840, 2160),
        Some((596, 336)),
    )
    .unwrap()
}

/// A compositor with room for the agent's overlay, counting how often it was
/// asked — so *the compositor was asked nothing while locked* is a number.
#[derive(Default)]
struct TheShell {
    /// How many surface requests arrived.
    asked: usize,
}

impl Compositor for TheShell {
    fn honour(&mut self, _asked: SurfaceRequest) -> Result<(), SurfaceRefused> {
        self.asked = self.asked.saturating_add(1);
        Ok(())
    }
}

/// What the machine underneath was asked, shared between it and its holds.
#[derive(Debug, Default)]
struct Asked {
    /// How many holds are alive.
    alive: usize,
    /// Every hold taken, with the sentence alo OS gave for it.
    holds: Vec<(Inhibit, Said)>,
    /// How many times it was asked to sleep.
    slept: usize,
}

/// A window onto that.
#[derive(Debug, Clone, Default)]
struct Told(Rc<RefCell<Asked>>);

/// One hold, counted alive until it is dropped — so *alo OS let go of the
/// machine* is a number going down rather than a flag somebody set.
#[derive(Debug)]
struct Held(Told);

impl Drop for Held {
    fn drop(&mut self) {
        let mut asked = (self.0).0.borrow_mut();
        asked.alive = asked.alive.saturating_sub(1);
    }
}

/// The machine underneath: it takes every hold and sleeps when asked.
///
/// This is where the walk stops being real. `alo-sleeping`'s own
/// `TheMachinesLogind` speaks to the base over the system bus on Linux; what
/// this does is record what it was asked and say yes, so that the walk is a
/// walk through alo OS's decisions and its sentences rather than a claim about
/// a machine that has slept.
struct TheLaptop(Told);

impl Logind for TheLaptop {
    type Held = Held;

    fn hold(&mut self, inhibit: Inhibit, why: &Said) -> Result<Held, NotHeld> {
        assert!(!why.is_a_bug(), "a hold described by a key: {why}");
        let mut asked = (self.0).0.borrow_mut();
        asked.alive = asked.alive.saturating_add(1);
        asked.holds.push((inhibit, why.clone()));
        drop(asked);
        Ok(Held(self.0.clone()))
    }

    fn sleep(&mut self, _locked_first: LockedFirst) -> Result<(), NotSlept> {
        let mut asked = (self.0).0.borrow_mut();
        asked.slept = asked.slept.saturating_add(1);
        Ok(())
    }
}

/// A machine with nothing in front of a turn, which is not a machine alo OS
/// ships — `alo_turn::bounding` says why no library here has one.
struct NothingIsBounded;

impl alo_turn::Bounding for NothingIsBounded {
    fn carrying_out(&mut self, _reaching: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary> {
        Ok(doing.done())
    }

    fn carrying_out_a_departure(
        &mut self,
        _to: &[std::net::SocketAddr],
        doing: &mut dyn FnMut(),
    ) -> Result<(), NoBoundary> {
        doing();
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
#[expect(
    clippy::too_many_lines,
    reason = "the walk is one person's day and a half, and splitting it into steps that hand \
              each other a seat, a turn, a screen set and a machine would hide the one thing it \
              is for, which is the order"
)]
fn the_walk() -> Vec<(&'static str, String)> {
    let strings = in_english();
    let mut met = Met::default();
    let told = Told::default();
    let mut laptop = TheLaptop(told.clone());
    let mut shell = TheShell::default();
    let mut summoning = Summoning::closed();
    let mut grants = Grants::default();
    let mut record = Record::default();
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let appearance = Appearance::shipped();

    // 1. Anna signs in. alo OS takes the two holds it keeps for as long as
    //    somebody is signed in, and each carries the sentence a person would
    //    read in the machine's own list of what is holding sleep off.
    let lid_is_ours = TheLidIsOurs::taken(&mut laptop, &strings).unwrap();
    let until_locked = UntilLocked::taken(&mut laptop, &strings).unwrap();
    met.at(
        "Anna signs in, and alo OS says what it is holding sleep off for",
        told.0
            .borrow()
            .holds
            .iter()
            .map(|(_, why)| why.clone())
            .collect::<Vec<_>>(),
    );

    // 2. Her screens are laid out. The laptop's own panel says nothing about
    //    itself, so it is remembered by the socket it is in — and named to her
    //    by a word alo OS has for that socket, never by the socket's own name.
    let mut remembered = Screens::untouched();
    let mut attached = Attached::now(vec![the_laptop()], &remembered, Support::Fractional).unwrap();
    met.at(
        "She opens the screens section of Settings for the first time",
        attached.notes().iter().map(|note| note.said(&strings)),
    );
    remembered.remember(attached.arrangement().clone());

    // 3. She asks the agent for something and steps away. Nobody touches the
    //    machine, and it does not sleep on its own: the turn is holding it
    //    awake, and it is named.
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        &mut record,
    )
    .unwrap();
    let turning = Turning::beginning(
        Context::at_invocation(evening()),
        "@files",
        an_hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    let mut holding = Holding::none();
    holding
        .a_turn(&turning, evening(), &mut laptop, &strings)
        .unwrap();
    let Decided::StaysAwake(awake) = asked(
        Why::NobodyIsUsingIt,
        &Settings::shipped(),
        Displays::OnlyItsOwn,
        &mut holding,
        &grants,
        evening(),
    ) else {
        panic!("a turn that is running holds the machine awake");
    };
    met.at(
        "Nobody touches the machine, and the agent is still working",
        awake.said(&strings),
    );

    // 4. She locks the screen and leaves. Everything carries on behind it.
    let seat = Seat::<String>::opened(anna()).locked(&mut summoning);
    let display = the_laptop().named_for_the_shell().clone();
    let lock_screen = seat
        .lock_screen(
            evening(),
            &appearance,
            &display,
            None,
            &nothing_is_leaving(),
        )
        .expect("a locked seat has a lock screen");
    met.at(
        "She locks the screen and leaves the desk",
        [lock_screen.locked_said(&strings)],
    );

    // 5. She comes back for a moment and presses the key that opens the agent.
    //    The compositor is asked nothing, and what she reads is the one thing
    //    the lock screen already says.
    let mut seat = seat;
    assert_eq!(
        seat.press_the_agents_key(&mut summoning, Some(&mut shell)),
        Err(NotWhileLocked)
    );
    assert_eq!(shell.asked, 0, "the compositor was asked while locked");
    met.at(
        "She presses the key that opens the agent, at the locked screen",
        [NotWhileLocked.said(&strings)],
    );

    // A message arrives while she is away. It is held, and nothing is drawn:
    // there is no sentence here, which is the point of it.
    assert_eq!(
        seat.arrives("a message from the school".to_owned()),
        Arrived::Held
    );

    // 6. She closes the lid. The seat is locked before anything reaches the
    //    machine, and the machine sleeps. It says nothing.
    let Decided::Sleeps(going) = asked(
        Why::LidClosed,
        &Settings::shipped(),
        Displays::OnlyItsOwn,
        &mut holding,
        &grants,
        evening(),
    ) else {
        panic!("closing the lid with no other screen attached sleeps");
    };
    assert_eq!(
        going.overriding().len(),
        1,
        "what her own sleep overrode is named"
    );
    let Slept::Asleep(asleep) = going.carried_out(seat, &mut summoning, &mut laptop, evening())
    else {
        panic!("the machine sleeps");
    };
    assert!(asleep.seat().is_locked());
    assert_eq!(told.0.borrow().slept, 1);

    // 7. She opens the lid the next morning. It wakes locked, and the lock
    //    screen says what it said last night.
    let woke = asleep.woke(morning());
    assert!(woke.seat().is_locked(), "a resume lands on the lock screen");
    met.at(
        "She opens the lid the next morning, and the machine wakes",
        [woke
            .seat()
            .lock_screen(
                morning(),
                &appearance,
                &display,
                None,
                &nothing_is_leaving(),
            )
            .expect("a machine that woke is locked")
            .locked_said(&strings)],
    );

    // What became of the agent's work is decided here, written down, and shown
    // to her once she is in — step 9.
    let AfterSleep::Stopped(stopped) = woke.a_turn(turning, &mut grants, &strings) else {
        panic!("a turn whose hour ran out while the machine slept is stopped");
    };

    // 8. She types a password that is not hers. The seat stays locked, and
    //    what she reads is the sign-in's own refusal, unchanged.
    let Unlocking::StillLocked { seat, refused } =
        woke.into_seat().unlocks(the_accounts(), "anna", NOT_ANNAS)
    else {
        panic!("a wrong password does not unlock");
    };
    assert!(seat.is_locked());
    met.at(
        "She types a password that is not hers",
        refused.said(&strings),
    );

    // 9. She types her own. The seat opens, what arrived while she was away is
    //    handed back, and the agent's work is over.
    let Unlocking::Unlocked { held, .. } = seat.unlocks(the_accounts(), "anna", ANNAS) else {
        panic!("her own password unlocks");
    };
    assert_eq!(held.len(), 1, "what arrived while locked is handed back");
    met.at("She types her own, and the machine unlocks", [stopped]);

    // 10. At the office she plugs in the screen she uses there. It has never
    //     been used with this machine before.
    let back = attached
        .plugged_in(the_office_screen(), &remembered)
        .unwrap();
    assert!(
        back.said(&strings).is_none(),
        "a screen arriving for the first time carries nothing home"
    );
    met.at(
        "At the office she plugs in the screen she uses there",
        attached.notes().iter().map(|note| note.said(&strings)),
    );
    remembered.remember(attached.arrangement().clone());

    // 11. She closes the lid with it attached, having chosen that the laptop
    //     keeps running in that case.
    let chosen = Settings {
        lid: Lid::StaysAwakeWithADisplay,
        keep_awake: false,
    };
    let Decided::StaysAwake(awake) = asked(
        Why::LidClosed,
        &chosen,
        Displays::AnotherAttached,
        &mut holding,
        &grants,
        morning(),
    ) else {
        panic!("she chose that a closed lid with another screen stays awake");
    };
    met.at(
        "She closes the lid with the office screen attached, as she chose",
        awake.said(&strings),
    );

    // 12. At the end of the day she pulls the cable out. What was open on the
    //     office screen belongs on the one that remains.
    let moved = attached
        .unplugged(the_office_screen().socket(), &remembered)
        .unwrap();
    let mut said = vec![moved.said(&strings)];
    said.extend(attached.notes().iter().map(|note| note.said(&strings)));
    met.at("At the end of the day she pulls the cable out", said);

    // 13. And puts it back, which is the whole of what hotplug has to mean.
    let back = attached
        .plugged_in(the_office_screen(), &remembered)
        .unwrap();
    let mut said = vec![back.said(&strings).expect("what was on it goes back to it")];
    said.extend(attached.notes().iter().map(|note| note.said(&strings)));
    met.at("She puts it back a moment later", said);

    // She signs out at last, and alo OS lets go of the two holds it keeps for a
    // session — which is the other half of taking them at step 1.
    let alive = told.0.borrow().alive;
    drop(until_locked);
    drop(lid_is_ours);
    assert_eq!(
        told.0.borrow().alive,
        alive - 2,
        "alo OS did not let go of what it was holding the machine for"
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
fn the_walk_from_lock_to_resume_to_a_new_desk_reads_as_the_table() {
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
        ("She locks the screen", "This machine is locked".to_owned()),
        (
            "She pulls the cable out",
            "it is on the other one".to_owned(),
        ),
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
        last.1 = "it is on the other one now".to_owned();
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

/// **No sentence on the walk names the machinery**, and that includes the
/// **connector name** — which is the one a list of words cannot catch, because
/// it arrives through a gap rather than in the sentence.
///
/// The laptop on this walk is a panel that says nothing about itself, plugged
/// into `eDP-1`, which is exactly the screen that used to put a connector name
/// in front of a person. What she reads instead is *Built-in screen*.
#[test]
fn no_sentence_on_the_walk_names_the_machinery() {
    let walk = the_walk();
    for (moment, sentence) in &walk {
        let said = sentence.to_lowercase();
        for never in [
            "logind",
            "systemd",
            "drm",
            "edid",
            "wayland",
            "smithay",
            "compositor",
            "inhibit",
            "dbus",
            "d-bus",
            "/sys/",
            "/dev/",
            "daemon",
            "edp-",
            "lvds-",
            "dsi-",
            "dp-1",
            "hdmi-a",
            "hdmi-b",
        ] {
            assert!(
                !said.contains(never),
                "{moment} says \"{never}\": {sentence}"
            );
        }
    }
    assert!(
        walk.iter()
            .any(|(_, sentence)| sentence.contains("Built-in screen")),
        "the walk never names the screen that would have leaked a connector name"
    );
}

/// **The machine was locked before it was asked to sleep, and it wakes
/// locked.** The walk's own steps 6 and 7 assert it as they go; this asks the
/// compiler's half of it, which is that a sleep cannot be asked for without
/// proof of a lock.
///
/// `alo_sleeping::Logind::sleep` takes a [`LockedFirst`], and the only thing
/// that makes one is `Going::carried_out` out of a seat it has just locked.
/// There is no constructor:
///
/// ```compile_fail
/// let first = alo_sleeping::LockedFirst(());
/// ```
#[test]
fn nothing_on_this_walk_could_have_slept_unlocked() {
    let told = Told::default();
    let mut laptop = TheLaptop(told.clone());
    let mut summoning = Summoning::closed();
    let mut holding: Holding<Held> = Holding::none();
    let Decided::Sleeps(going) = asked(
        Why::YouAsked,
        &Settings::shipped(),
        Displays::OnlyItsOwn,
        &mut holding,
        &Grants::default(),
        evening(),
    ) else {
        panic!("choosing Sleep sleeps");
    };
    let Slept::Asleep(asleep) = going.carried_out(
        Seat::<String>::opened(anna()),
        &mut summoning,
        &mut laptop,
        evening(),
    ) else {
        panic!("the machine sleeps");
    };
    assert!(asleep.seat().is_locked());
    assert!(asleep.woke(morning()).seat().is_locked());
    assert_eq!(told.0.borrow().slept, 1);
}
