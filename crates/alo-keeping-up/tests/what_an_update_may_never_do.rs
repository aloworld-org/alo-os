//! What an update is, and what it may never do — the acceptance of task 1 of
//! `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, one criterion at a
//! time.
//!
//! | Criterion | Test |
//! |---|---|
//! | an update is a digest offered, the digest running, and whether they differ | [`an_update_is_a_build_offered_that_differs_from_the_build_running`] |
//! | no clock, no scheduler and no background fetch is decided here | [`nothing_here_keeps_time_schedules_or_reaches_anything`] |
//! | a check for an update is an errand, on the indicator | [`a_check_for_an_update_is_on_the_indicator_while_it_happens`], [`an_answer_heard_during_any_other_errand_is_refused`] |
//! | an update never restarts the machine | [`an_update_never_restarts_the_machine`] |
//! | an update never closes an application | [`an_update_never_closes_an_application`] |
//! | an update never interrupts what a person is doing | [`an_update_never_interrupts_what_a_person_is_doing`] |
//! | there is no *urgent* that could bypass them | [`there_is_no_urgent_update_that_bypasses_the_rule`] |
//! | no setting turns updates off, and nothing applies one unasked | [`the_choice_is_when_and_never_whether`] |
//! | what a person is told is in the vocabulary `alo-saying` collects | [`an_update_ready_is_said_in_words_the_machine_collects`] |

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_egress::{Destination, Errand, Indicator, OnItsOwn};
use alo_keeping_up::{
    Cause, Digest, Disturbance, Offered, Running, Standing, THE_RULE, WhenItApplies, a_check_at,
};
use alo_saying::everything_this_machine_can_say;
use alo_strings::Strings;

/// A moment, handed to the indicator. Here and not in the crate: nothing in
/// `alo-keeping-up` reads or names the time.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

fn updates() -> Destination {
    Destination::at("updates.alo.example").unwrap()
}

fn build(pair: &str) -> Digest {
    Digest::read(&format!("sha256:{}", pair.repeat(32))).unwrap()
}

/// The machine's whole vocabulary, in English.
fn the_machine() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// What the place updates come from offers, heard the only way one can be:
/// during a check that is on the indicator.
fn offered(digest: Digest) -> Offered {
    let mut indicator = Indicator::default();
    let underway = indicator.beginning_on_its_own(a_check_at(updates()), noon());
    let offered = Offered::heard(&underway, digest).unwrap();
    assert!(indicator.ended_on_its_own(underway));
    offered
}

/// **An update is a build offered that differs from the build running**, and
/// nothing else: the same build is no update, a different one is, and what is
/// ready carries the two builds and no priority, severity or deadline.
#[test]
fn an_update_is_a_build_offered_that_differs_from_the_build_running() {
    let running = Running::reported(build("aa"));

    assert_eq!(
        Standing::between(&running, &offered(build("aa"))),
        Standing::UpToDate
    );

    let standing = Standing::between(&running, &offered(build("bb")));
    let Standing::Ready(ready) = &standing else {
        panic!("a different build offered was not an update: {standing:?}");
    };
    assert!(standing.is_ready());
    assert_eq!(ready.running(), &build("aa"));
    assert_eq!(ready.offered(), &build("bb"));

    let written = serde_json::to_value(&standing).unwrap();
    let mut fields: Vec<&str> = written
        .as_object()
        .expect("a standing is written as an object")
        .keys()
        .map(String::as_str)
        .collect();
    fields.sort_unstable();
    assert_eq!(fields, ["offered", "running", "standing"]);
}

/// **No clock, no scheduler and no background fetch is decided here.**
///
/// Read out of the crate itself: its dependencies are exactly the four it
/// needs to decide and to say, and its source never names the time, a thread,
/// a socket, a file or a process. A `SystemTime` here would be the first line
/// of a scheduler; a socket would be a fetch nobody saw.
#[test]
fn nothing_here_keeps_time_schedules_or_reaches_anything() {
    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    let manifest = std::fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
    let dependencies: Vec<&str> = manifest
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
        dependencies,
        ["alo-egress", "alo-strings", "serde", "thiserror"]
    );

    let source = crate_dir.join("src");
    let mut read = 0;
    for entry in std::fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        for forbidden in [
            "SystemTime",
            "Instant",
            "Duration",
            "std::time",
            "std::thread",
            "thread::spawn",
            "std::net",
            "TcpStream",
            "UdpSocket",
            "std::fs",
            "std::process",
            "std::io",
            "async ",
            ".await",
        ] {
            assert!(
                !text.contains(forbidden),
                "{} names {forbidden}",
                path.display()
            );
        }
        read += 1;
    }
    assert!(read >= 7, "only {read} source files were read");
}

/// **A check for an update is an errand, and it is on the indicator while it
/// happens** — in the sentence a person reads, and for as long as the check is
/// under way.
#[test]
fn a_check_for_an_update_is_on_the_indicator_while_it_happens() {
    let mut indicator = Indicator::default();
    assert!(indicator.is_quiet());

    let underway = indicator.beginning_on_its_own(a_check_at(updates()), noon());
    assert!(!indicator.is_quiet());
    assert_eq!(underway.errand(), Errand::CheckingForAnUpdate);
    let shown = indicator.showing().first().expect("the check is shown");
    assert_eq!(shown.id(), underway.shown());
    assert_eq!(
        shown.said(&the_machine()).text(),
        "alo OS is checking for an update at updates.alo.example"
    );

    let offer = Offered::heard(&underway, build("bb")).unwrap();
    assert_eq!(offer.digest(), &build("bb"));

    assert!(indicator.ended_on_its_own(underway));
    assert!(indicator.is_quiet());
}

/// **An answer heard during any other errand is refused**: a line saying the
/// machine is signing someone in, or fetching a model, while it asks about
/// updates would be the indicator saying something untrue.
#[test]
fn an_answer_heard_during_any_other_errand_is_refused() {
    let others = [
        Errand::SigningIn,
        Errand::FetchingAModel,
        Errand::FetchingAnUpdate,
        Errand::InstallingAnApplication,
        Errand::CheckingForApplicationUpdates,
        Errand::UpdatingAnApplication,
    ];
    for errand in others {
        let mut indicator = Indicator::default();
        let underway = indicator.beginning_on_its_own(OnItsOwn::for_(errand, updates()), noon());
        let refused = Offered::heard(&underway, build("bb")).unwrap_err();
        assert_eq!(refused.during(), errand);
    }
    // Every errand there is was considered: the one that may hear an answer,
    // and the others above that may not — an application's update check among
    // them, because an application's update is not this system's.
    //
    // `FetchingAnUpdate` refuses for the sharpest reason of the six. It is the
    // download of a build somebody has already approved, so an offer heard
    // during it is an answer arriving after the decision it was meant to
    // inform. A machine that accepted one could be told to fetch one build and
    // end up holding another.
    assert_eq!(Errand::EVERY.len(), others.len() + 1);
}

/// **An update never restarts the machine** — and the person may, because
/// restarting their own machine is choosing when, not an update choosing for
/// them.
#[test]
fn an_update_never_restarts_the_machine() {
    let refused = THE_RULE
        .allows(Cause::AnUpdate, Disturbance::RestartingTheMachine)
        .unwrap_err();
    assert_eq!(refused.disturbance(), Disturbance::RestartingTheMachine);
    assert_eq!(
        refused.said(&the_machine()).text(),
        "An update never restarts this machine. It applies when you restart"
    );
    assert!(
        THE_RULE
            .allows(Cause::ThePerson, Disturbance::RestartingTheMachine)
            .is_ok()
    );
    for when in WhenItApplies::EVERY {
        assert!(
            THE_RULE
                .allows(
                    when.cause_of_the_restart(),
                    Disturbance::RestartingTheMachine
                )
                .is_ok(),
            "{when:?} is a restart an update causes"
        );
    }
}

/// **An update never closes an application.**
#[test]
fn an_update_never_closes_an_application() {
    let refused = THE_RULE
        .allows(Cause::AnUpdate, Disturbance::ClosingAnApplication)
        .unwrap_err();
    assert_eq!(refused.disturbance(), Disturbance::ClosingAnApplication);
    assert_eq!(
        refused.said(&the_machine()).text(),
        "An update never closes an application you have open"
    );
    assert!(
        THE_RULE
            .allows(Cause::ThePerson, Disturbance::ClosingAnApplication)
            .is_ok()
    );
}

/// **An update never interrupts what a person is doing.**
#[test]
fn an_update_never_interrupts_what_a_person_is_doing() {
    let refused = THE_RULE
        .allows(Cause::AnUpdate, Disturbance::InterruptingThePerson)
        .unwrap_err();
    assert_eq!(refused.disturbance(), Disturbance::InterruptingThePerson);
    assert_eq!(
        refused.said(&the_machine()).text(),
        "An update never interrupts what you are doing"
    );
}

/// **There is no *urgent* update that bypasses the rule.**
///
/// The exhaustive matches are the tripwire: a third cause — an update that
/// matters more — or a fourth disturbance cannot be added without this failing
/// to compile, and whoever adds one reads why here. And for every clause, an
/// update is refused whatever it offers: the rule takes nothing an update could
/// carry to argue with it.
#[test]
fn there_is_no_urgent_update_that_bypasses_the_rule() {
    for cause in [Cause::AnUpdate, Cause::ThePerson] {
        match cause {
            Cause::AnUpdate | Cause::ThePerson => {}
        }
    }
    assert_eq!(Disturbance::EVERY.len(), 3);
    for disturbance in Disturbance::EVERY {
        match disturbance {
            Disturbance::RestartingTheMachine
            | Disturbance::ClosingAnApplication
            | Disturbance::InterruptingThePerson => {}
        }
        assert!(THE_RULE.allows(Cause::AnUpdate, disturbance).is_err());
    }

    // Nothing an update is written as can say it is urgent.
    for word in ["urgent", "critical", "security", "required", "forced"] {
        assert!(
            serde_json::from_value::<WhenItApplies>(serde_json::json!(word)).is_err(),
            "{word} was read as a moment an update applies"
        );
    }
    let ready = serde_json::to_string(&Standing::between(
        &Running::reported(build("aa")),
        &offered(build("bb")),
    ))
    .unwrap();
    for word in ["urgent", "critical", "priority", "severity", "deadline"] {
        assert!(!ready.contains(word), "{ready}");
    }
}

/// **The choice is *when*, never *whether*.** Two members, one of them what
/// happens when the person has chosen nothing; neither of them *never*, and
/// neither of them *by itself* — and a kept choice naming either is refused
/// rather than read as the nearest member.
#[test]
fn the_choice_is_when_and_never_whether() {
    assert_eq!(WhenItApplies::default(), WhenItApplies::AtTheNextRestart);
    assert_eq!(WhenItApplies::EVERY.len(), 2);
    for when in WhenItApplies::EVERY {
        match when {
            WhenItApplies::AtTheNextRestart | WhenItApplies::NowBecauseThePersonAsked => {}
        }
        assert_eq!(when.cause_of_the_restart(), Cause::ThePerson);
        let written = serde_json::to_value(when).unwrap();
        assert_eq!(
            serde_json::from_value::<WhenItApplies>(written).unwrap(),
            when
        );
    }
    for refused in [
        "never",
        "off",
        "disabled",
        "automatically",
        "overnight",
        "immediately",
    ] {
        assert!(
            serde_json::from_value::<WhenItApplies>(serde_json::json!(refused)).is_err(),
            "{refused} was read as a choice"
        );
    }
}

/// **What a person is told is in the vocabulary `alo-saying` collects** — read
/// through the machine's whole vocabulary rather than this crate's own, so a
/// crate nobody collected would fail here.
#[test]
fn an_update_ready_is_said_in_words_the_machine_collects() {
    let strings = the_machine();
    let running = Running::reported(build("aa"));

    let ready = Standing::between(&running, &offered(build("bb"))).said(&strings);
    assert!(!ready.is_a_bug(), "{ready}");
    assert_eq!(
        ready.text(),
        "An update is ready. It will apply when you choose, and nothing you are doing will be \
         interrupted until then"
    );

    let current = Standing::between(&running, &offered(build("aa"))).said(&strings);
    assert!(!current.is_a_bug(), "{current}");
    assert_eq!(current.text(), "This machine is up to date");

    for clause in THE_RULE.said(&strings) {
        assert!(!clause.is_a_bug(), "{clause}");
    }
    for when in WhenItApplies::EVERY {
        let said = strings.say(&when.word().key(), &alo_strings::Filling::nothing());
        assert!(!said.is_a_bug(), "{said}");
    }
    let not_understood = Digest::read("latest").unwrap_err().said(&strings);
    assert!(!not_understood.is_a_bug(), "{not_understood}");
}
