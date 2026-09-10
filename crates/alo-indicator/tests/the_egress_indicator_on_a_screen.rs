//! The plan's acceptance for *the egress indicator, on a screen*, one test per
//! criterion.
//!
//! `docs/autonomy/v0-01-delivery-plan.md`, task 9: *the indicator is drawn from
//! `alo_egress::Indicator` and nothing else; a local answer leaves it dark; a
//! provider answer lights it while the question is in flight; and it cannot be
//! drawn from a value that was not a departure.*
//!
//! These tests reach the machine the way a shell does, and not through a seam
//! of the test's own: what is leaving comes off an `alo_egress::Indicator` that
//! really permitted something, the picture reaches a compositor through the
//! port `alo-shell` will implement, and the strings come from
//! `alo_saying::everything_this_machine_can_say` — the vocabulary a shell
//! really holds — so a string this crate declares and nobody collects fails
//! here rather than on somebody's screen.
//!
//! Nothing here draws anything, and nothing asserts a pixel. `ROADMAP.md`'s
//! exit gate is about a certified machine, and a green suite on a developer's
//! laptop is not that.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_capability::Grantee;
use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
use alo_indicator::words::{EVERY_COUNTED, EVERY_WORD};
use alo_indicator::{Compositor, Drawn, Drew, Indicating, NotShown, SurfaceRefused};
use alo_models::{InferenceSource, Region};
use alo_saying::everything_this_machine_can_say;
use alo_strings::Strings;

/// The moment every test here is written against.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// The agent asking the questions below.
fn the_agent() -> Grantee {
    Grantee::named("@mail")
}

/// The strings a shell really holds: everything the machine can say.
fn what_this_machine_says() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// A question put to a provider, which is a departure.
fn asking_a_provider() -> Leaving {
    Leaving::asking(
        &the_agent(),
        &InferenceSource::Hosted {
            provider: "alo".to_owned(),
            region: Region::Declared("the EU".to_owned()),
        },
    )
    .unwrap()
}

/// A screen, as this crate sees one: it keeps whatever it was last given and
/// counts how often it was asked, so *one change, one redraw* is a number.
#[derive(Default)]
struct Screen {
    /// The last picture it was given.
    showing: Option<Drawn>,
    /// How many pictures arrived.
    shown: usize,
}

impl Compositor for Screen {
    fn show(&mut self, drawn: Drawn) -> Result<(), SurfaceRefused> {
        self.shown += 1;
        self.showing = Some(drawn);
        Ok(())
    }
}

/// A compositor with nothing to draw on.
#[derive(Default)]
struct NoScreen {
    /// How many pictures were offered to it and refused.
    asked: usize,
}

impl Compositor for NoScreen {
    fn show(&mut self, _drawn: Drawn) -> Result<(), SurfaceRefused> {
        self.asked += 1;
        Err(SurfaceRefused::NothingToShowOn)
    }
}

/// **The indicator is drawn from `alo_egress::Indicator` and nothing else.**
///
/// Everything that reaches the screen — whether the light is lit, how many
/// things it is lit for, and the sentence on every line — is the machine's own
/// indicator read back. The lines are worded by the crate that decided the
/// egress, in the vocabulary the whole machine holds, so nothing on this
/// surface is a second opinion about what left.
#[test]
fn the_indicator_is_drawn_from_the_machines_own_indicator_and_nothing_else() {
    let mut indicator = Indicator::default();
    let departing = indicator
        .beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon())
        .unwrap();

    let mut screen = Screen::default();
    let mut indicating = Indicating::nowhere();
    assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);

    let drawn = screen.showing.as_ref().unwrap();
    assert_eq!(drawn.lamp().how_many(), indicator.showing().len());
    assert_eq!(drawn.lines(), indicator.showing());

    // Every sentence on the surface is one the machine really says, and none of
    // them was written here.
    let strings = what_this_machine_says();
    let lamp = drawn.lamp_said(&strings);
    assert!(!lamp.is_a_bug(), "{lamp}");
    assert_eq!(lamp.text(), "One thing is leaving this machine right now");

    let lines = drawn.lines_said(&strings);
    assert_eq!(lines.len(), 1);
    let line = lines.first().unwrap();
    assert!(!line.is_a_bug(), "{line}");
    assert_eq!(line.text(), "@mail is asking a question of alo, in the EU");
    assert_eq!(
        line.text(),
        indicator.showing().first().unwrap().said(&strings).text(),
        "the screen and the machine's own indicator worded one departure differently"
    );

    // And every string this crate declares is in the machine's vocabulary, so
    // a shell cannot be shown a key where a sentence belongs.
    let vocabulary = everything_this_machine_can_say().unwrap();
    for word in EVERY_WORD {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "the machine cannot say {}",
            word.named()
        );
    }
    for counted in EVERY_COUNTED {
        assert!(
            vocabulary.plural(&counted.key()).is_some(),
            "the machine cannot count {}",
            counted.named()
        );
    }

    assert!(indicator.ended(departing));
}

/// **A local answer leaves it dark.**
///
/// A working day of questions answered on this machine, and the light never
/// comes on — because a question answered here never becomes a departure at
/// all, so there is nothing to show. This is `CLAUDE.md`'s *zero inference
/// egress* seen from the screen a person glances at; the honest measurement is
/// still taken at the network boundary, and the two agreeing is the claim.
#[test]
fn a_local_answer_leaves_the_indicator_dark() {
    // Never `mut`: a day of local answers gives nobody anything to add to it,
    // and the compiler saying so is part of what this test claims.
    let indicator = Indicator::default();
    let mut screen = Screen::default();
    let mut indicating = Indicating::nowhere();

    // The indicator goes up dark rather than not going up: a person cannot
    // tell a dark light from a light that is not there.
    assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);
    assert!(screen.showing.as_ref().unwrap().lamp().is_dark());

    for _ in 0..8 {
        assert!(
            Leaving::asking(&the_agent(), &InferenceSource::ThisMachine).is_err(),
            "a question answered here became a departure"
        );
        assert_eq!(
            indicating.show(Some(&mut screen), &indicator),
            Drew::Unchanged
        );
    }

    assert_eq!(screen.shown, 1, "a quiet day redrew the indicator");
    let drawn = screen.showing.as_ref().unwrap();
    assert!(drawn.lamp().is_dark());
    assert!(drawn.lines().is_empty());
    assert_eq!(
        drawn.lamp_said(&what_this_machine_says()).text(),
        "Nothing is leaving this machine right now"
    );
}

/// **A provider answer lights it while the question is in flight** — while, and
/// not after.
///
/// The light is on from the moment the egress is permitted, because permitting
/// and showing are one act inside `alo-egress`, and it is off again the moment
/// the connection ends. There is no window in which a question is on its way to
/// a company and the screen says the machine is quiet.
#[test]
fn a_provider_answer_lights_it_while_the_question_is_in_flight() {
    let mut indicator = Indicator::default();
    let mut screen = Screen::default();
    let mut indicating = Indicating::nowhere();
    assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);
    assert!(screen.showing.as_ref().unwrap().lamp().is_dark());

    let departing = indicator
        .beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon())
        .unwrap();
    assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);
    let in_flight = screen.showing.as_ref().unwrap();
    assert!(in_flight.lamp().is_lit());
    assert_eq!(in_flight.lamp().how_many(), 1);
    assert_eq!(
        in_flight
            .lines_said(&what_this_machine_says())
            .first()
            .unwrap()
            .text(),
        "@mail is asking a question of alo, in the EU"
    );

    // While it is still in flight the screen does not change back.
    assert_eq!(
        indicating.show(Some(&mut screen), &indicator),
        Drew::Unchanged
    );
    assert!(screen.showing.as_ref().unwrap().lamp().is_lit());

    // The answer comes back, and the light goes out.
    assert!(indicator.ended(departing));
    assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);
    let afterwards = screen.showing.as_ref().unwrap();
    assert!(afterwards.lamp().is_dark());
    assert!(afterwards.lines().is_empty());
}

/// **It cannot be drawn from a value that was not a departure.**
///
/// The refusal path of the whole surface, and the one that decides whether the
/// indicator is worth looking at. An egress a policy stopped never reaches the
/// list `alo-egress` keeps, so it never reaches the screen: nothing left, and a
/// light that lit for the connections a rule stopped would teach people to
/// ignore the one thing on the screen that matters.
///
/// The other half of this criterion is held by the compiler rather than here —
/// there is no constructor for a [`alo_indicator::Lamp`] or a [`Drawn`] that
/// takes anything but an `alo_egress::Indicator`, and the compile-fail examples
/// on both types are what say so.
#[test]
fn it_cannot_be_drawn_from_a_value_that_was_not_a_departure() {
    let mut indicator = Indicator::default();
    let mut screen = Screen::default();
    let mut indicating = Indicating::nowhere();
    assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);

    // An organisation that lets nothing leave: the agent's question is refused.
    let refused = indicator.beginning(&EgressPolicy::NothingLeaves, asking_a_provider(), noon());
    assert!(
        refused.is_err(),
        "the policy permitted it, so this proves nothing"
    );

    // The screen is not asked again, because nothing about the machine changed.
    assert_eq!(
        indicating.show(Some(&mut screen), &indicator),
        Drew::Unchanged
    );
    assert_eq!(screen.shown, 1);
    let drawn = screen.showing.as_ref().unwrap();
    assert!(drawn.lamp().is_dark());
    assert!(drawn.lines().is_empty());

    // The same is true of an egress refused for being outside the building: a
    // second policy, so this is a rule about refusals rather than about one.
    let outside = indicator.beginning(
        &EgressPolicy::InTheBuilding,
        Leaving::because(
            &the_agent(),
            Why::Fetching,
            Destination::at("alo.example").unwrap(),
        ),
        noon(),
    );
    assert!(outside.is_err(), "the policy permitted it");
    assert_eq!(
        indicating.show(Some(&mut screen), &indicator),
        Drew::Unchanged
    );
    assert!(screen.showing.as_ref().unwrap().lamp().is_dark());
}

/// **Nowhere to show it is a sentence a person can read**, in the vocabulary
/// the machine really holds.
///
/// A machine that cannot put what is leaving in front of a person, and says
/// nothing about that, looks exactly like a machine on which nothing is
/// leaving. Both ways of having nowhere are refused in words, and neither
/// leaves anything believing an indicator is on a screen.
#[test]
fn nowhere_to_show_it_is_refused_in_words_rather_than_in_silence() {
    let strings = what_this_machine_says();
    let indicator = Indicator::default();
    let mut indicating = Indicating::nowhere();

    // Nothing is drawing a screen at all.
    let refused = indicating.show(None, &indicator);
    assert_eq!(refused, Drew::Refused(NotShown::NoCompositor));
    assert!(!indicating.is_showing());
    let said = NotShown::NoCompositor.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("Sign in to the desktop"), "{said}");

    // Something is drawing, and there is no display to draw on.
    let mut nowhere = NoScreen::default();
    let refused = indicating.show(Some(&mut nowhere), &indicator);
    assert_eq!(
        refused,
        Drew::Refused(NotShown::Surface(SurfaceRefused::NothingToShowOn))
    );
    assert_eq!(nowhere.asked, 1);
    assert!(!indicating.is_showing());
    let said = NotShown::Surface(SurfaceRefused::NothingToShowOn).said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("Connect a screen"), "{said}");

    // A screen arrives, and the indicator is drawn without waiting for
    // anything about the machine to change.
    let mut screen = Screen::default();
    assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);
    assert!(indicating.is_showing());
    assert!(screen.showing.as_ref().unwrap().lamp().is_dark());
}
