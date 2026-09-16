//! *The microphone is on* and *something left this machine* are two warnings,
//! and neither is ever drawn as the other.
//!
//! The plan's constraint for this task says it outright, and it is not a
//! stylistic preference. Law 1's indicator answers *has anything left this
//! machine* and is the one guarantee alo OS is sold on; this one answers *is
//! anything in this room watching or listening to me*. A person who learned to
//! read one as the other would be reassured by the wrong light at the wrong
//! moment, and each of the two would have quietly stopped meaning what it says.
//!
//! So this file asks the question neither crate's own tests can. `alo-egress`
//! and `alo-indicator` are dev-dependencies here and nothing else — nothing on
//! this indicator can be drawn from what is leaving, because the crate that
//! knows what is leaving is not linked into it — and what is left to check is
//! the half that a person actually meets: the words, and the two indicators'
//! behaviour side by side on one machine.

#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_applications::Application;
use alo_capability::Grantee;
use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
use alo_in_use::{By, InUse, NotHeard, Streams, Use, UseId, Used};
use alo_indicator::Lamp;
use alo_saying::everything_this_machine_can_say;
use alo_strings::Strings;
use std::time::{Duration, SystemTime};

/// A media server answering with whatever it was handed.
struct AMachine(Vec<Use>);

impl Streams for AMachine {
    fn in_use_now(&mut self) -> Result<Vec<Use>, NotHeard> {
        Ok(self.0.clone())
    }
}

/// The moment every departure here is made at.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A question going somewhere else, which is what lights the other indicator.
fn something_leaving() -> Leaving {
    Leaving::because(
        &Grantee::named("@files"),
        Why::Fetching,
        Destination::at("alo.example").unwrap(),
    )
}

/// A camera being used by a video-call application, which is what lights this
/// one.
fn something_watching() -> Use {
    Use::of(
        UseId::recorded(42),
        Used::Camera,
        By::an_application(Application::called("com.example.VideoCall", "Video Call").unwrap()),
    )
}

/// Everything this machine can say, which is the vocabulary both indicators'
/// sentences are answered from.
fn the_machines_words() -> Strings {
    let Ok(vocabulary) = everything_this_machine_can_say() else {
        panic!("alo OS's own words do not collect");
    };
    Strings::of(vocabulary)
}

/// **A machine with something leaving and nothing watching says exactly that,
/// twice.** The lamp is lit, the room is quiet, and a person reading either
/// line learns nothing false about the other.
#[test]
fn something_leaving_does_not_light_what_is_watching_or_listening() {
    let strings = the_machines_words();
    let mut leaving = Indicator::default();
    let departing = leaving
        .beginning(&EgressPolicy::Anywhere, something_leaving(), noon())
        .unwrap();
    assert!(Lamp::of(&leaving).is_lit());

    let in_use = InUse::read_from(&mut AMachine(Vec::new())).unwrap();
    assert!(in_use.is_quiet());
    assert_eq!(
        in_use.reads_as(&strings).first().map(|said| said.text()),
        Some("Nothing is watching or listening right now")
    );

    leaving.ended(departing);
}

/// **And a machine with something watching and nothing leaving says exactly
/// that, twice.** The mirror of the case above, which is the one that would be
/// missed: it is the room that is not quiet, and law 1's indicator has to stay
/// dark through it or the measurement `CLAUDE.md` publishes stops being worth
/// anything.
#[test]
fn something_watching_does_not_light_what_is_leaving() {
    let strings = the_machines_words();
    let leaving = Indicator::default();
    assert!(Lamp::of(&leaving).is_dark());

    let in_use = InUse::read_from(&mut AMachine(vec![something_watching()])).unwrap();
    assert!(!in_use.is_quiet());
    assert_eq!(
        in_use.reads_as(&strings).first().map(|said| said.text()),
        Some("The camera is in use by Video Call (com.example.VideoCall)")
    );
    assert!(Lamp::of(&leaving).is_dark(), "the egress indicator lit");
}

/// **Neither indicator's sentences are the other's.** Two lists, two areas, no
/// key in common and no sentence in common — so a translator meets them as two
/// different things to say and a shell cannot put one where the other belongs
/// by looking up a key.
#[test]
fn neither_indicators_sentences_are_the_others() {
    let leaving: Vec<&str> = alo_indicator::words::EVERY_WORD
        .iter()
        .map(|word| word.says())
        .collect();
    let watching: Vec<&str> = alo_in_use::EVERY_WORD
        .iter()
        .map(|word| word.says())
        .collect();

    for says in &watching {
        assert!(
            !leaving.contains(says),
            "both indicators say {says}, so one can be drawn as the other"
        );
    }
    for word in alo_in_use::EVERY_WORD {
        assert_eq!(word.key().area(), "in-use", "{}", word.named());
    }
    for word in alo_indicator::words::EVERY_WORD {
        assert_eq!(word.key().area(), "indicator", "{}", word.named());
    }
}

/// **Each indicator's sentences are about its own question, in words.** The
/// egress indicator talks about leaving this machine and never about something
/// being in use; this one talks about watching and listening and never about
/// what left. A sentence that drifted across would be the first step in the two
/// becoming one light nobody reads.
#[test]
fn each_indicators_sentences_are_about_its_own_question() {
    for word in alo_in_use::EVERY_WORD {
        assert!(
            !word.says().contains("leaving this machine"),
            "{} is the egress indicator's sentence",
            word.named()
        );
    }
    for word in alo_indicator::words::EVERY_WORD {
        assert!(
            !word.says().contains("watching or listening"),
            "{} is the in-use indicator's sentence",
            word.named()
        );
        assert!(
            !word.says().contains("is in use"),
            "{} is the in-use indicator's sentence",
            word.named()
        );
    }
    for counted in alo_indicator::words::EVERY_COUNTED {
        for says in [counted.one(), counted.other()] {
            assert!(
                !says.contains("watching or listening") && !says.contains("is in use"),
                "{} is the in-use indicator's sentence",
                counted.named()
            );
        }
    }
}

/// **A quiet room and a dark lamp read differently.** The two states a machine
/// is in for most of a day, and the one place a person could be told the same
/// thing twice and think they had checked both.
#[test]
fn a_quiet_room_and_a_dark_lamp_do_not_read_the_same() {
    let strings = the_machines_words();
    let dark = Lamp::of(&Indicator::default()).said(&strings);
    let quiet = InUse::read_from(&mut AMachine(Vec::new()))
        .unwrap()
        .reads_as(&strings);

    assert_eq!(quiet.len(), 1);
    let quiet = quiet.first().unwrap();
    assert_ne!(dark.text(), quiet.text());
    assert!(!dark.is_a_bug());
    assert!(!quiet.is_a_bug());
}

/// **A machine that cannot say what is watching still says what is leaving.**
/// Two indicators, two failures: one of them going silent must not take the
/// other with it, and the refusal that arrives in its place is this crate's own
/// sentence rather than the other's.
#[test]
fn one_indicator_failing_does_not_take_the_other_with_it() {
    /// A media server that cannot be reached at all.
    struct NoServer;
    impl Streams for NoServer {
        fn in_use_now(&mut self) -> Result<Vec<Use>, NotHeard> {
            Err(NotHeard::NothingHandlesSoundAndVideo {
                said: "nothing to run".to_owned(),
            })
        }
    }

    let strings = the_machines_words();
    let mut leaving = Indicator::default();
    let departing = leaving
        .beginning(&EgressPolicy::Anywhere, something_leaving(), noon())
        .unwrap();

    let refused = InUse::read_from(&mut NoServer).unwrap_err();
    let said = refused.said(&strings);
    assert!(!said.is_a_bug());
    assert!(said.text().contains("watching or listening"));
    assert!(!said.text().contains("leaving this machine"));

    // And what is leaving is still on its own indicator, unaffected.
    assert!(Lamp::of(&leaving).is_lit());
    assert_eq!(Lamp::of(&leaving).how_many(), 1);
    leaving.ended(departing);
}
