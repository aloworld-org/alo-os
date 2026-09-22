//! One laptop, four resumes: at another desk, at a desk nobody has arranged,
//! at the same desk, at none — and back at the first one.
//!
//! Task 11 of `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`: *one
//! test suspends at one desk and resumes at another, at the same one, and at
//! none.*
//!
//! A machine that was asleep saw no cable go in or come out. Every other road
//! into `alo_displays::Attached` is an event — this screen went, this one
//! arrived — and a resume has none of them to work from, so it asks the whole
//! question again from what is reported now. This walks the four answers that
//! asking can give, in the order one person meets them, through a real sleep:
//! a seat really locked, a machine really asked, and the screens really laid
//! out.
//!
//! # What is real here, and what is not
//!
//! The session is a real `alo_accounts::Session` — a password that verified —
//! and every sentence comes out of the machine's one assembled vocabulary
//! through the value that produces it. **What is not here is the machine.** No
//! lid has closed and no screen has been plugged into anything: [`TheLaptop`]
//! is this file's stand-in for what alo OS asks of the base, counting what it
//! was asked. That is the boundary this crate's own tests draw, and the plan
//! says so.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_accounts::{Accounts, Session};
use alo_capability::Grants;
use alo_displays::{
    Arrangement, Attached, Changes as Screens, Identity, NotArranged, Note, Panel, Placed,
    Position, Reported, Resumed, Scale, Socket, Support,
};
use alo_locking::Seat;
use alo_overlay::Summoning;
use alo_sleeping::{
    Decided, Displays, Holding, Inhibit, LockedFirst, Logind, NotHeld, NotSlept, Settings, Slept,
    Why, Woke, asked,
};
use alo_strings::{Said, Strings};

/// The number this machine's person runs as.
const PERSON: u32 = 1000;

/// Anna's password, and it is right.
const ANNAS: &str = "correct horse battery staple";

/// A fixed moment: the evening she closes the lid at the office.
fn evening() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// The machine's one vocabulary, as a shell holds it.
fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("alo OS's own words"))
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

/// The laptop's own panel: thirteen inches, and it says nothing about itself.
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

/// The screen at home: 24 inches at 1920 by 1080, with no serial number.
fn the_home_screen() -> Reported {
    Reported::of(
        Socket::named("HDMI-1").unwrap(),
        Some(Panel::of("Acme", "P24", None).unwrap()),
        (1920, 1080),
        Some((531, 299)),
    )
    .unwrap()
}

/// Which screen the laptop's own panel is: it says nothing, so it is the
/// socket it is in.
fn which_laptop() -> Identity {
    Identity::Socket(Socket::named("eDP-1").unwrap())
}

/// Which screen the office monitor is.
fn which_office_screen() -> Identity {
    Identity::Panel(Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap())
}

/// Which screen the one at home is.
fn which_home_screen() -> Identity {
    Identity::Panel(Panel::of("Acme", "P24", None).unwrap())
}

/// The arrangement Anna made at the office: the big screen on the left of the
/// laptop, at a size she chose herself.
fn as_she_arranged_the_office() -> Arrangement {
    Arrangement::of(vec![
        (
            which_laptop(),
            Placed::at(Position::the_origin(), Scale::per_cent(175).unwrap()).as_the_main_screen(),
        ),
        (
            which_office_screen(),
            Placed::at(Position::at(-2560, 0), Scale::per_cent(150).unwrap()),
        ),
    ])
    .unwrap()
}

/// The machine underneath: it takes every hold and sleeps when asked.
///
/// This is where the walk stops being real. `alo-sleeping`'s own
/// `TheMachinesLogind` speaks to the base over the system bus on Linux; this
/// says yes and counts.
#[derive(Default)]
struct TheLaptop {
    /// How many times it was asked to sleep.
    slept: usize,
}

impl Logind for TheLaptop {
    type Held = ();

    fn hold(&mut self, _inhibit: Inhibit, why: &Said) -> Result<(), NotHeld> {
        assert!(!why.is_a_bug(), "a hold described by a key: {why}");
        Ok(())
    }

    fn sleep(&mut self, _locked_first: LockedFirst) -> Result<(), NotSlept> {
        self.slept = self.slept.saturating_add(1);
        Ok(())
    }
}

/// Close the lid on a seat of Anna's, let the machine sleep, and open it again
/// an hour later. The seat is locked before anything reaches the machine, and
/// it is still locked when it wakes.
fn slept_and_woke(laptop: &mut TheLaptop, at: SystemTime) -> Woke<String> {
    let mut holding: Holding<()> = Holding::none();
    let Decided::Sleeps(going) = asked(
        Why::LidClosed,
        &Settings::shipped(),
        Displays::OnlyItsOwn,
        &mut holding,
        &Grants::default(),
        at,
    ) else {
        panic!("a closed lid with nothing else attached sleeps");
    };
    let Slept::Asleep(asleep) =
        going.carried_out(Seat::opened(anna()), &mut Summoning::closed(), laptop, at)
    else {
        panic!("the machine sleeps");
    };
    assert!(asleep.seat().is_locked(), "it locked before it slept");
    let woke = asleep.woke(at + Duration::from_secs(60 * 60));
    assert!(woke.seat().is_locked(), "and it woke locked");
    woke
}

/// Every sentence a resume produced, each checked for coming out whole and for
/// naming nothing the person never chose to learn.
fn what_she_reads(resumed: &Resumed, attached: &Attached, strings: &Strings) -> Vec<String> {
    let mut said: Vec<Said> = Vec::new();
    said.extend(resumed.note().map(|note| note.said(strings)));
    said.extend(resumed.moved().map(|moved| moved.said(strings)));
    said.extend(resumed.came_back().filter_map(|back| back.said(strings)));
    said.extend(attached.notes().iter().map(|note| note.said(strings)));
    said.into_iter()
        .map(|sentence| {
            assert!(!sentence.is_a_bug(), "{sentence}");
            assert!(sentence.unfilled().is_empty(), "{sentence}");
            for connector in ["eDP-1", "DP-1", "HDMI-1"] {
                assert!(
                    !sentence.text().contains(connector),
                    "a person was told the name of a connector: {sentence}"
                );
            }
            sentence.text().to_owned()
        })
        .collect()
}

/// **A laptop suspended at one desk and opened at another is set up for the
/// desk it was opened at** — and at the same desk, and at none, it does the
/// two other things a person needs it to do.
///
/// Four resumes, in the order one person meets them:
///
/// 1. she closes the lid at the office and opens it at home, where a screen
///    she has never used with this machine is plugged in instead;
/// 2. she closes it at home and opens it at the same desk;
/// 3. she closes it there and opens it where nothing is plugged in;
/// 4. and back at the office, where the arrangement she made is restored and
///    what was open on the office screen goes back to it.
#[expect(
    clippy::too_many_lines,
    reason = "the four answers a resume can give are one sequence, and splitting them into \
              functions that hand each other a set of screens and a machine would hide the thing \
              this test is for, which is that one laptop meets all four"
)]
#[test]
fn a_laptop_suspended_at_one_desk_and_opened_at_another() {
    let strings = in_english();
    let mut laptop = TheLaptop::default();
    let mut remembered = Screens::untouched();

    // At the office, with the big screen on the left where she put it.
    remembered.remember(as_she_arranged_the_office());
    let mut attached = Attached::now(
        vec![the_laptop(), the_office_screen()],
        &remembered,
        Support::Fractional,
    )
    .unwrap();
    assert!(attached.notes().contains(&Note::AsYouLeftThem));
    assert_eq!(
        attached
            .on(&which_office_screen())
            .unwrap()
            .placed()
            .position(),
        Position::at(-2560, 0)
    );

    // 1. She closes the lid and opens it at home, where another screen is
    //    plugged in — one this machine has never seen.
    let woke = slept_and_woke(&mut laptop, evening());
    let resumed = woke
        .the_desk(
            &mut attached,
            vec![the_laptop(), the_home_screen()],
            &remembered,
        )
        .unwrap();

    assert!(!resumed.the_same_desk());
    assert_eq!(resumed.note(), Some(&Note::TheDeskChanged));
    assert_eq!(
        resumed.moved().count(),
        1,
        "the office screen is not at home"
    );
    let moved = resumed.moved().next().unwrap();
    assert_eq!(moved.from(), &which_office_screen());
    assert_eq!(
        moved.onto(),
        attached.main_screen(),
        "what was open on it belongs on the main screen of what remains"
    );
    assert_eq!(attached.main_screen(), &which_laptop());
    assert!(
        attached
            .notes()
            .contains(&Note::NewHere(which_home_screen())),
        "a set nobody has arranged is laid out side by side, each at its own size"
    );
    assert_eq!(
        attached.on(&which_laptop()).unwrap().placed().position(),
        Position::the_origin()
    );
    assert_eq!(
        attached
            .on(&which_home_screen())
            .unwrap()
            .placed()
            .position(),
        Position::at(
            i32::try_from(Scale::per_cent(175).unwrap().laid_out(1920)).unwrap(),
            0,
        ),
        "beside the laptop, at the width the laptop takes once it is drawn"
    );
    let read = what_she_reads(&resumed, &attached, &strings);
    assert!(
        read.iter().any(|sentence| sentence.contains("Dell U2720Q")),
        "{read:?}"
    );
    assert!(
        read.iter()
            .any(|sentence| sentence.contains("went to sleep")),
        "{read:?}"
    );
    remembered.remember(attached.arrangement().clone());

    // 2. She closes it at home and opens it at the same desk. Nothing moved,
    //    and she is told nothing at all.
    let before = attached.clone();
    let woke = slept_and_woke(&mut laptop, evening() + Duration::from_secs(4 * 60 * 60));
    let resumed = woke
        .the_desk(
            &mut attached,
            vec![the_laptop(), the_home_screen()],
            &remembered,
        )
        .unwrap();

    assert!(resumed.the_same_desk());
    assert_eq!(resumed.note(), None);
    assert_eq!(resumed.moved().count(), 0);
    assert_eq!(resumed.came_back().count(), 0);
    assert_eq!(attached, before, "nothing about the desk changed");

    // 3. She closes it there and opens it where nothing at all reports itself.
    //    It is refused, and the screens it went to sleep with are still its
    //    screens — the next resume is compared against them.
    let before = attached.clone();
    let woke = slept_and_woke(&mut laptop, evening() + Duration::from_secs(8 * 60 * 60));
    assert_eq!(
        woke.the_desk(&mut attached, Vec::new(), &remembered),
        Err(NotArranged::NoScreens)
    );
    assert_eq!(
        attached, before,
        "what it slept with was not thrown away, down to the places in it and what she was last \
         told"
    );
    assert_eq!(
        attached.each().count(),
        2,
        "it still has the two screens it went to sleep with"
    );

    // 4. And back at the office, where the arrangement she made is restored
    //    and what was open on the office screen goes back to it.
    let woke = slept_and_woke(&mut laptop, evening() + Duration::from_secs(20 * 60 * 60));
    let resumed = woke
        .the_desk(
            &mut attached,
            vec![the_laptop(), the_office_screen()],
            &remembered,
        )
        .unwrap();

    assert!(!resumed.the_same_desk());
    assert!(attached.notes().contains(&Note::AsYouLeftThem));
    assert_eq!(
        attached
            .on(&which_office_screen())
            .unwrap()
            .placed()
            .position(),
        Position::at(-2560, 0),
        "the arrangement she made for this set, on the left where she put it"
    );
    assert_eq!(
        resumed.moved().count(),
        1,
        "the home screen is not at the office"
    );
    assert_eq!(resumed.moved().next().unwrap().from(), &which_home_screen());
    assert_eq!(resumed.came_back().count(), 1);
    let back = resumed.came_back().next().unwrap();
    assert_eq!(back.back(), &which_office_screen());
    assert!(
        back.anything_goes_back(),
        "what was open on it before goes back to it"
    );
    let read = what_she_reads(&resumed, &attached, &strings);
    assert!(
        read.iter().any(|sentence| sentence.contains("is back")),
        "{read:?}"
    );

    assert_eq!(laptop.slept, 4, "it really slept between each of them");
}
