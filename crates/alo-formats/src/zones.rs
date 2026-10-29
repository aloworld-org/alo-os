//! **The time zones a region has, named in the person's language** — so a
//! machine set up in a country with more than one can ask which.
//!
//! The owner's decision of 2026-10-28: picking a region on the first-start
//! sheets sets the machine's time zone, and **where a country has several —
//! the United States, Russia, Brazil — the person is asked which**, from
//! CLDR's and the zone database's own data, in their language, offline.
//!
//! Both answers are CLDR's, through `icu`'s stable API:
//!
//! - **Which zones a region has**: every canonical IANA zone CLDR knows, with
//!   its BCP 47 id, whose first two letters are the zone's country
//!   (`usnyc`, `uschi`, `ruvvo`) — the rule [`crate::where_the_machine_is`]
//!   already reads the other way round.
//! - **What each is called**: its *exemplar city* in the person's language —
//!   `New York`, `Chicago`; in Russian `Нью-Йорк`, `Чикаго` — which is how
//!   every CLDR-based system names a zone to a person.
//!
//! Nothing here leaves the machine: both are compiled into the program.

use icu::collator::Collator;
use icu::collator::options::CollatorOptions;
use icu::datetime::NoCalendarFormatter;
use icu::datetime::fieldsets::zone::ExemplarCity;
use icu::locale::Locale;
use icu::time::zone::iana::IanaParserExtended;

use crate::region_code::RegionCode;

/// One zone a region has, as a person picks it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedZone {
    /// The zone database's name, `America/Chicago`, which is what the machine
    /// is set to.
    pub iana: String,
    /// What it is called in the person's language: `Chicago`.
    pub name: String,
}

/// **Every zone `region` has**, named in `language` and sorted the way it
/// sorts — empty for a region CLDR names no zone in.
#[must_use]
pub fn the_zones_in(region: &RegionCode, language: &str) -> Vec<NamedZone> {
    let prefix = region.code().to_ascii_lowercase();
    let locale: Locale = language.parse().unwrap_or(Locale::UNKNOWN);
    let formatter = NoCalendarFormatter::try_new(locale.clone().into(), ExemplarCity).ok();
    let mut zones: Vec<NamedZone> = IanaParserExtended::new()
        .iter()
        .filter(|it| {
            let id = it.time_zone.0.as_str();
            id.len() == 5
                && id.chars().all(|letter| letter.is_ascii_lowercase())
                && id.starts_with(prefix.as_str())
        })
        .map(|it| {
            let name = formatter
                .as_ref()
                .map(|formatter| formatter.format(&it.time_zone.without_offset()).to_string())
                .unwrap_or_else(|| it.canonical.to_owned());
            NamedZone {
                iana: it.canonical.to_owned(),
                name,
            }
        })
        .collect();
    zones.dedup_by(|one, other| one.iana == other.iana);
    match Collator::try_new(locale.into(), CollatorOptions::default()) {
        Ok(collator) => zones.sort_by(|one, other| collator.compare(&one.name, &other.name)),
        Err(_) => zones.sort_by(|one, other| one.name.cmp(&other.name)),
    }
    zones
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn zones(region: &str, language: &str) -> Vec<NamedZone> {
        the_zones_in(&RegionCode::named(region).unwrap(), language)
    }

    /// **A country with one zone has one**, and it is the one its clock keeps.
    #[test]
    fn a_country_with_one_zone_has_one() {
        let germany = zones("DE", "de");
        assert_eq!(
            germany
                .iter()
                .map(|it| it.iana.as_str())
                .collect::<Vec<_>>(),
            ["Europe/Berlin"],
            "Büsingen keeps Zurich's rules and is filed under Switzerland in CLDR"
        );
        assert_eq!(zones("BE", "fr").len(), 1);
    }

    /// **The countries the owner named have several**, each named in the
    /// language read, and none of them is the zone database's own spelling.
    #[test]
    fn the_united_states_russia_and_brazil_have_several() {
        for (region, at_least) in [("US", 10), ("RU", 10), ("BR", 5)] {
            let named = zones(region, "en");
            assert!(named.len() >= at_least, "{region}: {named:?}");
        }
        let american = zones("US", "en");
        assert!(
            american
                .iter()
                .any(|it| it.iana == "America/Chicago" && it.name == "Chicago")
        );
        let in_german = zones("US", "de");
        assert!(in_german.iter().any(|it| it.iana == "America/New_York"));
        assert!(
            in_german.iter().all(|it| !it.name.contains('/')),
            "a zone was named by its database spelling"
        );
    }

    /// **The list is in the language's order.**
    #[test]
    fn the_zones_are_in_the_languages_order() {
        let named = zones("RU", "ru");
        let locale: Locale = "ru".parse().unwrap();
        let collator = Collator::try_new(locale.into(), CollatorOptions::default()).unwrap();
        for pair in named.windows(2) {
            if let [one, other] = pair {
                assert_ne!(
                    collator.compare(&one.name, &other.name),
                    std::cmp::Ordering::Greater,
                    "{} before {}",
                    one.name,
                    other.name
                );
            }
        }
    }
}
