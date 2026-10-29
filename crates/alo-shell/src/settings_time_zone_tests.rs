//! The time-zone section, against a service that says yes, and one that says
//! no.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;

use alo_formats::RegionCode;
use alo_strings::Strings;

use super::TimeZoneSection;
use crate::settings_kept::SettingsKept;
use crate::the_zone_through_timedated::SetsTheZone;

/// A time-zone service that answers as it is told, and remembers what it was
/// asked.
struct AService {
    answers: Result<(), String>,
    asked: RefCell<Vec<String>>,
}

impl SetsTheZone for AService {
    fn set(&self, zone: &str) -> Result<(), String> {
        self.asked.borrow_mut().push(zone.to_owned());
        self.answers.clone()
    }
}

fn words() -> Strings {
    Strings::of(alo_formats::formats_words().unwrap())
}

fn american(now: &str) -> TimeZoneSection {
    TimeZoneSection::of(
        RegionCode::named("US").ok().as_ref(),
        "en",
        Some(now.to_owned()),
    )
}

/// **The region's cities are offered, the machine's zone ticked**, and
/// nothing is said before anything is changed.
#[test]
fn the_regions_cities_are_offered_with_the_machines_ticked() {
    let section = american("America/New_York");
    assert!(section.choices().contains(&"America/Chicago".to_owned()));
    assert_eq!(section.called("America/Chicago"), "Chicago");
    assert!(section.is_chosen("America/New_York"));
    assert!(section.said().is_empty());
}

/// **The person's change lands**: the service is asked once, for exactly
/// the zone chosen, and the tick moves.
#[test]
fn the_persons_change_lands() {
    let mut section = american("America/New_York");
    let service = AService {
        answers: Ok(()),
        asked: RefCell::new(Vec::new()),
    };
    assert_eq!(
        section.choose("America/Chicago", &service, &words()),
        SettingsKept::Written
    );
    assert_eq!(service.asked.borrow().as_slice(), ["America/Chicago"]);
    assert!(section.is_chosen("America/Chicago"));
}

/// **A service that says no moves nothing**, and the section says so — which
/// is what the agent's account meets, and what any account meets on a machine
/// whose rule is missing.
#[test]
fn a_service_that_says_no_moves_nothing_and_is_said() {
    let mut section = american("America/New_York");
    let service = AService {
        answers: Err("org.freedesktop.PolicyKit1.Error.NotAuthorized".to_owned()),
        asked: RefCell::new(Vec::new()),
    };
    assert_eq!(
        section.choose("America/Chicago", &service, &words()),
        SettingsKept::NotWritten
    );
    assert!(
        section.is_chosen("America/New_York"),
        "the tick moved on a no"
    );
    assert!(
        section
            .said()
            .iter()
            .any(|it| it.contains("was not changed")),
        "{:?}",
        section.said()
    );
}

/// **A zone that is not one of the region's is refused before the service is
/// asked**, so no row can ask the service for anything a person was not shown.
#[test]
fn a_zone_not_offered_is_refused_before_anything_is_asked() {
    let mut section = american("America/New_York");
    let service = AService {
        answers: Ok(()),
        asked: RefCell::new(Vec::new()),
    };
    assert_eq!(
        section.choose("Europe/Berlin", &service, &words()),
        SettingsKept::Refused
    );
    assert!(service.asked.borrow().is_empty());
}

/// **A region changed above offers its own cities.**
#[test]
fn a_region_changed_offers_its_own_cities() {
    let mut section = american("America/New_York");
    section.follows(RegionCode::named("DE").ok().as_ref(), "de");
    assert_eq!(section.choices(), ["Europe/Berlin"]);
    assert!(!section.is_chosen("Europe/Berlin"));
}
