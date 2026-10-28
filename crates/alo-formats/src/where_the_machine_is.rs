//! **Where the machine is, from what it honestly knows** — the two signals a
//! machine has about its region before anybody has named one.
//!
//! The owner's decision of 2026-10-28: when nobody has picked a region, a
//! person's dates are written the way **the region the machine is in** writes
//! them — never the United States for every English reader who is somewhere
//! else. In order, what a machine knows:
//!
//! 1. the region picked on the first-start sheet ([`crate::set_up_in`]);
//! 2. **the country of its time zone**, where a zone is known —
//!    [`the_region_of_the_zone`];
//! 3. and only then the region CLDR writes the language in by default —
//!    [`the_usual_region_of`], which is what the first-start sheet offers
//!    before anything else is known, and is offered there for the person to
//!    see and change rather than used silently.
//!
//! Both answers here are CLDR's, through `icu`'s stable API, and neither is a
//! table of ours.

use icu::locale::LanguageIdentifier;
use icu::locale::LocaleExpander;
use icu::time::zone::IanaParser;

use crate::region_code::RegionCode;
use crate::regions::EVERY_REGION;

/// **The country a time zone is in**: `Europe/Berlin` is `DE`.
///
/// CLDR names every zone with a BCP 47 id whose first two letters are the
/// zone's country (`deber`, `usnyc`, `chzrh`) and whose next three are a place
/// in it — five letters in all. Ids that are not a place have another shape
/// (`utc`, `gmt`, `unk`, `utcw05`, `est5edt`) and name no country, so this
/// answers [`None`] for them, and for a zone CLDR does not know: a machine
/// keeping universal time is not in any region, and saying it was would be a
/// guess.
#[must_use]
pub fn the_region_of_the_zone(iana: &str) -> Option<RegionCode> {
    let zone = IanaParser::new().parse(iana);
    let id = zone.0.as_str();
    if id.len() != 5 || !id.chars().all(|letter| letter.is_ascii_lowercase()) {
        return None;
    }
    let code = id.get(..2)?.to_ascii_uppercase();
    if !EVERY_REGION.contains(&code.as_str()) {
        return None;
    }
    RegionCode::named(&code).ok()
}

/// **The region CLDR writes a language in where nothing else is said**: `de`
/// is `DE`, `pt` is `BR`, `en` is `US`.
///
/// That last is why it comes last: it is the answer for an English reader
/// nobody knows anything about, and the first-start sheet shows it to them by
/// name, where they can pick where they really are.
#[must_use]
pub fn the_usual_region_of(language: &str) -> Option<RegionCode> {
    let mut asked: LanguageIdentifier = language.parse().ok()?;
    if asked.region.is_none() {
        let _ = LocaleExpander::new_common().maximize(&mut asked);
    }
    let region = asked.region?;
    let code = region.as_str();
    if !EVERY_REGION.contains(&code) {
        return None;
    }
    RegionCode::named(code).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(region: Option<RegionCode>) -> Option<String> {
        region.map(|it| it.code().to_owned())
    }

    /// **A zone is in its country.**
    #[test]
    fn a_zone_is_in_its_country() {
        for (zone, country) in [
            ("Europe/Berlin", "DE"),
            ("Europe/Brussels", "BE"),
            ("Europe/Lisbon", "PT"),
            ("Atlantic/Azores", "PT"),
            ("America/New_York", "US"),
            ("Europe/Dublin", "IE"),
            ("Europe/Malta", "MT"),
        ] {
            assert_eq!(
                code(the_region_of_the_zone(zone)).as_deref(),
                Some(country),
                "{zone}"
            );
        }
    }

    /// **A zone that is not a place names no country**: universal time, an
    /// offset, and a name CLDR does not know are not guessed at.
    #[test]
    fn a_zone_that_is_not_a_place_names_no_country() {
        for zone in [
            "UTC",
            "Etc/UTC",
            "Etc/GMT",
            "GMT",
            "Etc/GMT+5",
            "EST5EDT",
            "Made/Up",
            "",
        ] {
            assert_eq!(code(the_region_of_the_zone(zone)), None, "{zone}");
        }
    }

    /// **Each language's usual region is CLDR's**, and the two that are not
    /// European are named here so nobody is surprised by them.
    #[test]
    fn each_language_has_cldrs_usual_region() {
        assert_eq!(code(the_usual_region_of("de")).as_deref(), Some("DE"));
        assert_eq!(code(the_usual_region_of("ga")).as_deref(), Some("IE"));
        assert_eq!(code(the_usual_region_of("mt")).as_deref(), Some("MT"));
        assert_eq!(code(the_usual_region_of("en")).as_deref(), Some("US"));
        assert_eq!(code(the_usual_region_of("pt")).as_deref(), Some("BR"));
        assert_eq!(code(the_usual_region_of("pt-PT")).as_deref(), Some("PT"));
        assert_eq!(code(the_usual_region_of("not a language")), None);
        for official in alo_strings::union::OFFICIAL {
            assert!(
                the_usual_region_of(official.tag).is_some(),
                "{} has no usual region",
                official.tag
            );
        }
    }
}
