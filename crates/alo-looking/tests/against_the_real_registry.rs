//! The whole of a check, measured against the real place `image/pinned.toml`
//! names — with the machine's proxy on the road, a refusal exercised, and the
//! indicator read while it happens and after.
//!
//! The sixth criterion of task 6 of
//! `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`. Everything else in
//! this crate is measured on a machine with no network at all
//! (`finding_out_there_is_an_update.rs`); this is the one that says the road
//! is really there.
//!
//! # Why it is not in the suite
//!
//! It reaches the public internet, and no other test in this workspace's
//! default suite does. A network test in the nine gates is a gate that fails
//! for a reason that has nothing to do with the change being gated, on a
//! machine whose connection dropped for a minute — and the cost of that lands
//! on whoever is trying to merge something unrelated. So it is `#[ignore]`d,
//! named in the handoff's evidence (which the supervisor runs with
//! `--include-ignored`), and its numbers are written into the report.
//!
//! Run it by hand with:
//!
//! ```text
//! cargo test -p alo-looking --test against_the_real_registry -- --ignored --nocapture
//! ```
//!
//! # What it measures, and what it does not
//!
//! It measures that the place the pin names answers this machine, that what it
//! answers is a whole build, that `alo_looking::look` reaches the same answer
//! the two questions do on their own, that the indicator carries the check
//! while the requests are in flight, and that three of the five refusals are
//! the ones the real place actually produces.
//!
//! It does not measure a machine behind a proxy, because this one is not
//! behind one: what it shows is that the road is **decided** by
//! `alo_proxy::the_way` for `alo_proxy::Road::CheckingForAnUpdate` and handed
//! to the client explicitly, and that a machine with a proxy set carries it. A
//! company network is owed a measurement of its own, as every report in this
//! workstream has said about hardware.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_egress::{Errand, Indicator};
use alo_image::{THE_PIN, ThePin};
use alo_keeping_up::{Digest, Offered, Running, Standing, a_check_at};
use alo_looking::{Because, NoAnswer, Place, Release, ThePlace, TheRegistry, look};
use alo_proxy::{
    Carried, ConfigurationAddress, NotEvaluated, ProxyAddress, Reaching, Road, Scheme, SpokenTo,
    TheEvaluator, TheProxy, the_way,
};
use alo_saying::everything_this_machine_can_say;
use alo_strings::Strings;

/// An evaluator nothing here may ask: this machine's proxy setting is read
/// rather than worked out by a script, and a decision that consulted one would
/// be a road nobody chose.
struct NeverAsked;

impl TheEvaluator for NeverAsked {
    fn asked(
        &self,
        _: &ConfigurationAddress,
        address: &str,
        _: &str,
    ) -> Result<String, NotEvaluated> {
        unreachable!("nothing should have been asked about {address}")
    }
}

/// What a check was written into, so that the one departure it makes can be
/// read back here the way the machine's record reads it back on a booted one.
#[derive(Debug, Default)]
struct WhatWasWrittenDown {
    /// Each check that left, as the errand and the place it was made to.
    departures: Vec<(Errand, alo_egress::Destination)>,
}

impl alo_looking::Noting for WhatWasWrittenDown {
    fn the_check_left(&mut self, underway: &alo_egress::Underway) {
        self.departures
            .push((underway.errand(), underway.destination().clone()));
    }
}

/// A moment: nothing in this crate reads a clock.
fn a_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

/// The pin this repository ships.
fn the_pin() -> ThePin {
    let text = std::fs::read_to_string(std::path::Path::new(alo_image::THE_IMAGE).join(THE_PIN))
        .expect("the pin");
    ThePin::read(&text).expect("the pin this repository ships")
}

/// The way out this machine decided for the road a check takes.
fn decided(proxy: &TheProxy, place: &Place) -> Carried {
    let going_to = Reaching::over(Scheme::Https, place.host()).expect("a host");
    Carried::of(
        the_way(proxy, Road::CheckingForAnUpdate, &going_to, &NeverAsked)
            .expect("nothing refuses it"),
    )
}

/// The place some registry names, for the two refusals measured against a real
/// place that is not ours.
fn a_place_at(registry: &str) -> Place {
    let text = std::fs::read_to_string(std::path::Path::new(alo_image::THE_IMAGE).join(THE_PIN))
        .expect("the pin");
    let written: String = text
        .lines()
        .map(|line| {
            if line.starts_with("registry = ") {
                format!("registry = \"{registry}\"")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");
    Place::the_pin_names(&ThePin::read(&written).expect("a pin")).expect("a place")
}

/// **The real place, asked, with the indicator up for the whole of it.**
///
/// The two questions are asked here rather than through
/// [`alo_looking::look`] so that the indicator can be read *between* them —
/// which is the only way to show, from outside, that one line covers both.
/// `look` is then called for the same check and has to reach the same answer.
#[test]
#[ignore = "reaches the real place this repository's builds come from"]
fn a_check_against_the_place_this_repository_pins() {
    let pin = the_pin();
    let place = Place::on_this_machine().expect("the pin this repository ships");
    let strings = Strings::of(everything_this_machine_can_say().expect("the machine's words"));
    let running =
        Running::reported(Digest::read(pin.digest()).expect("the pinned build is a whole build"));

    let taking = decided(&TheProxy::None, &place);
    println!(
        "asking {} about releases, going {}",
        place.host(),
        if taking.is_straight() {
            "straight out".to_owned()
        } else {
            format!("through {}", taking.shown().unwrap_or_default())
        }
    );
    let asking = TheRegistry::at(&place).taking(taking);

    let mut indicator = Indicator::default();

    assert!(indicator.is_quiet());
    let underway =
        indicator.beginning_on_its_own(a_check_at(place.destination().clone()), a_moment());

    // **The line is up before a single packet goes anywhere**, and it says
    // what this machine is doing and where.
    assert_eq!(indicator.showing().len(), 1);
    assert_eq!(
        indicator
            .showing()
            .first()
            .expect("the check is showing")
            .said(&strings)
            .text(),
        format!("alo OS is checking for an update at {}", place.host())
    );
    assert_eq!(underway.errand(), Errand::CheckingForAnUpdate);

    let held = asking
        .every_name()
        .expect("the place answered with its names");
    println!("it holds {} names: {held:?}", held.len());
    assert!(
        held.iter().any(|name| name == pin.version()),
        "the place does not hold the release the pin names"
    );

    // **Still up, between the two questions.** One check, one line.
    assert_eq!(indicator.showing().len(), 1);

    let newest = Release::newest_of(held.iter().map(String::as_str))
        .expect("the place holds at least one release");
    println!("the newest release it offers is {}", newest.named_as());
    assert!(&newest >= place.not_before());

    let named = asking
        .the_build_of(&newest)
        .expect("the place named the build that release is");
    println!("it says {} is {named}", newest.named_as());
    let build = Digest::read(&named).expect("a whole build");
    assert_eq!(indicator.showing().len(), 1);

    // **Whether the place vouches for what it offers**, out of the names it
    // has already answered with — no third question and no second departure.
    let vouching = alo_looking::vouched_for(&build, &held);
    println!(
        "it holds {} for that build: {vouching:?}",
        alo_looking::the_name_vouching_for(&build)
    );

    let offered =
        Offered::heard(&underway, build.clone(), vouching).expect("an offer heard during a check");
    let standing = Standing::between(&running, &offered);
    println!("this machine stands: {}", standing.said(&strings).text());

    indicator.ended_on_its_own(underway);
    assert!(indicator.is_quiet(), "the check was left on the indicator");
    assert!(indicator.showing().is_empty());

    // **And the whole act reaches the same answer.** The same two questions,
    // through the function a machine actually calls.
    let mut again = Indicator::default();
    let mut written = WhatWasWrittenDown::default();
    let found = look(
        &mut again,
        &mut written,
        a_moment(),
        &place,
        &running,
        Because::ThePersonAsked,
        &TheRegistry::at(&place).taking(decided(&TheProxy::None, &place)),
    )
    .expect("the real place answered a whole check");
    assert_eq!(found.standing(), &standing);
    assert_eq!(found.about(), running.digest());
    assert!(again.is_quiet());
    println!(
        "look() answered the same: ready={} about={}",
        found.is_ready(),
        found.about().as_str()
    );

    // The release the pin names is the build the pin names — which is what
    // makes *what this repository published* and *what a machine is offered*
    // one fact rather than two.
    let pinned = asking
        .the_build_of(&Release::named(pin.version()).expect("the pinned release"))
        .expect("the place named the pinned release's build");
    assert_eq!(pinned, pin.digest(), "the place disagrees with the pin");
    println!("and the pinned release {} is {pinned}", pin.version());
}

/// **What this machine is offered today, and what it is told about it** — the
/// first half of task 7's acceptance, against the real place.
///
/// It asserts nothing about *which* answer the place gives, because that is
/// the release process's to change and this is not the place to freeze it. It
/// asserts the two things that must hold whatever the place holds: that the
/// sentence a person reads matches what the place actually vouches for, and
/// that a build nothing vouches for is still **offered** rather than hidden —
/// the choice a person has is *when*, never *whether to be told*.
///
/// What it prints is the measurement the report carries.
#[test]
#[ignore = "reaches the real place this repository's builds come from"]
fn what_this_machine_is_offered_today_and_what_it_is_told_about_it() {
    let place = Place::on_this_machine().expect("the pin this repository ships");
    let strings = Strings::of(everything_this_machine_can_say().expect("the machine's words"));
    let running = Running::reported(
        Digest::read(the_pin().digest()).expect("the pinned build is a whole build"),
    );

    let asking = TheRegistry::at(&place).taking(decided(&TheProxy::None, &place));
    let held = asking.every_name().expect("the place answered");
    let mut indicator = Indicator::default();
    let mut written = WhatWasWrittenDown::default();
    let found = look(
        &mut indicator,
        &mut written,
        a_moment(),
        &place,
        &running,
        Because::ThePersonAsked,
        &TheRegistry::at(&place).taking(decided(&TheProxy::None, &place)),
    )
    .expect("the real place answered a whole check");
    assert!(indicator.is_quiet());

    let said = found.said(&strings);
    println!(
        "running {}\nit holds {held:?}\na person reads: {}",
        running.digest().as_str(),
        said.text()
    );
    assert!(!said.is_a_bug(), "{said}");

    match found.standing() {
        Standing::UpToDate => {
            println!("this machine is up to date, so nothing is offered to act on");
        }
        Standing::Ready(ready) => {
            println!(
                "offered {} — {:?}",
                ready.offered().as_str(),
                ready.vouching()
            );
            // The sentence says what the place actually holds, either way.
            assert_eq!(
                ready.vouching(),
                alo_looking::vouched_for(ready.offered(), &held),
                "the offer disagrees with the names the place holds"
            );
            if ready.vouching().is_vouched_for() {
                assert!(said.text().starts_with("An update is ready"), "{said}");
            } else {
                assert!(said.text().contains("cannot confirm"), "{said}");
            }
        }
    }

    // **And the newest release is offered whether or not anything vouches for
    // it.** A newer version that exists is told about; what changes is the
    // sentence.
    let newest = Release::newest_of(held.iter().map(String::as_str)).expect("a release");
    let newest_is = Digest::read(
        &asking
            .the_build_of(&newest)
            .expect("the place named that release's build"),
    )
    .expect("a whole build");
    if &newest_is != running.digest() {
        assert!(
            found.is_ready(),
            "the place holds {} and this machine was told nothing about it",
            newest.named_as()
        );
    }
    println!(
        "the newest release {} is {}, vouched for: {:?}",
        newest.named_as(),
        newest_is.as_str(),
        alo_looking::vouched_for(&newest_is, &held)
    );
}

/// **Three refusals, from the real place rather than from a stand-in.**
///
/// A repository nobody publishes, a release the place does not hold, and a
/// host that does not exist — each the refusal a person would actually read,
/// and each leaving the machine as it was.
#[test]
#[ignore = "reaches the real place this repository's builds come from"]
fn the_refusals_the_real_place_produces() {
    let strings = Strings::of(everything_this_machine_can_say().expect("the machine's words"));

    // A repository at the real place that nobody publishes: it answers, and
    // what it answers is a refusal.
    let nobodys = a_place_at("ghcr.io/aloworld-org/alo-os-nothing-is-published-here");
    let refused = TheRegistry::at(&nobodys)
        .taking(decided(&TheProxy::None, &nobodys))
        .every_name()
        .expect_err("a repository nobody publishes is not answered");
    println!("a repository nobody publishes: {refused}");
    assert_eq!(refused, NoAnswer::ItRefused);

    // A release the real place does not hold.
    let ours = Place::on_this_machine().expect("the pin this repository ships");
    let refused = TheRegistry::at(&ours)
        .taking(decided(&TheProxy::None, &ours))
        .the_build_of(&Release::named("999.999.999").expect("a release"))
        .expect_err("a release the place does not hold is not named");
    println!("a release it does not hold: {refused}");
    assert_eq!(refused, NoAnswer::ItRefused);

    // A place that does not exist at all, which is the one said once.
    let nowhere = a_place_at("nowhere.invalid/aloworld-org/alo-os");
    let mut indicator = Indicator::default();
    let mut written = WhatWasWrittenDown::default();
    let refused = look(
        &mut indicator,
        &mut written,
        a_moment(),
        &nowhere,
        &Running::reported(
            Digest::read(the_pin().digest()).expect("the pinned build is a whole build"),
        ),
        Because::ThisMachineStarted,
        &TheRegistry::at(&nowhere).taking(decided(&TheProxy::None, &nowhere)),
    )
    .expect_err("a place that does not exist is not reached");
    println!("a place that does not exist: {refused}");
    assert_eq!(refused, NoAnswer::NoWayOut);
    assert!(indicator.is_quiet());

    for refusal in [NoAnswer::ItRefused, NoAnswer::NoWayOut] {
        let said = refusal.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        println!("a person reads: {}", said.text());
    }
}

/// **A machine with a proxy set carries it onto this road.**
///
/// Measured against a proxy that is not there, because what is being shown is
/// that the road out is the machine's decision and is handed to the client
/// explicitly — a client left to read the environment of whatever process it
/// happens to be in is taking a road nobody chose.
#[test]
#[ignore = "reaches the real place this repository's builds come from"]
fn a_machines_proxy_is_on_the_road_a_check_takes() {
    let place = Place::on_this_machine().expect("the pin this repository ships");
    let proxy = ProxyAddress::checked(SpokenTo::Http, "127.0.0.1", 9).expect("an address");
    let through = decided(&TheProxy::one(proxy), &place);
    assert!(!through.is_straight());
    println!("going through {}", through.shown().unwrap_or_default());

    let asking = TheRegistry::at(&place).taking(through);
    let refused = asking
        .every_name()
        .expect_err("a proxy that is not there cannot answer");
    println!("through a proxy that is not there: {refused}");
    assert!(
        matches!(refused, NoAnswer::NoWayOut | NoAnswer::NothingCameBack),
        "{refused:?}"
    );

    // And the same place, straight out, answers — so the refusal above is the
    // proxy having been taken rather than the place being unreachable.
    let straight = TheRegistry::at(&place).taking(decided(&TheProxy::None, &place));
    assert!(
        straight.every_name().is_ok(),
        "the place did not answer straight out either, so nothing was shown"
    );
}
