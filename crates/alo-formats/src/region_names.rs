//! **What each region is called, in the language the person reads, and in the
//! order that language sorts them.**
//!
//! Both answers are CLDR's: the names from `icu::experimental::displaynames`,
//! the order from `icu::collator`. A person reading Swedish finds Österrike
//! where a Swedish reader looks for it — after Ö, not beside Ostrava — and a
//! person reading Maltese reads Maltese names, without anybody here having
//! typed one.
//!
//! **This is the only file that touches `icu`'s unstable half**, which is why
//! it is a file of its own: the API is the one CLDR's own implementers have not
//! yet promised to keep, so an upgrade that moves it should break here and
//! nowhere else. [ADR 0136](../../../docs/decisions/0136-the-region-names-come-from-icus-unstable-half.md)
//! records why, and what to check on every upgrade of `icu`;
//! `tests/the_region_names_keep_their_shape.rs` fails if an upgrade compiles
//! and quietly answers something else.
//!
//! # A name CLDR does not have
//!
//! Where the person's language has no name for a region, its English name is
//! used, and where English has none either, the code itself. Neither is a
//! translation of ours. The tests hold English to naming every region on the
//! list, so the code is a last resort that the shipped data never reaches.

use icu::collator::Collator;
use icu::collator::options::CollatorOptions;
use icu::experimental::displaynames::DisplayNamesOptions;
use icu::experimental::displaynames::multi::RegionDisplayNames;
use icu::locale::Locale;
use icu::locale::subtags::Region;

use crate::regions::EVERY_REGION;

/// One region, as a person picks it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedRegion {
    /// The two-letter code, as `formats.toml` keeps it.
    pub code: &'static str,
    /// What it is called in the language the person reads.
    pub name: String,
}

/// **Every region, named in `language` and sorted the way it sorts.**
///
/// A language CLDR cannot be asked about gets the English names, sorted the
/// English way — never an empty list, which would leave a person nothing to
/// choose.
#[must_use]
pub fn the_regions_named_in(language: &str) -> Vec<NamedRegion> {
    let locale: Locale = language.parse().unwrap_or(Locale::UNKNOWN);
    let english: Locale = "en".parse().unwrap_or(Locale::UNKNOWN);
    let theirs =
        RegionDisplayNames::try_new(locale.clone().into(), DisplayNamesOptions::default()).ok();
    let fallback = RegionDisplayNames::try_new(english.into(), DisplayNamesOptions::default()).ok();
    let mut named: Vec<NamedRegion> = EVERY_REGION
        .into_iter()
        .map(|code| {
            let asked = Region::try_from_str(code).ok();
            let in_theirs = asked.and_then(|region| theirs.as_ref()?.of(region));
            let in_english = asked.and_then(|region| fallback.as_ref()?.of(region));
            NamedRegion {
                code,
                name: in_theirs.or(in_english).unwrap_or(code).to_owned(),
            }
        })
        .collect();
    match Collator::try_new(locale.clone().into(), CollatorOptions::default()) {
        Ok(collator) => named.sort_by(|one, other| collator.compare(&one.name, &other.name)),
        Err(_) => named.sort_by(|one, other| one.name.cmp(&other.name)),
    }
    named
}

/// The name of one region in `language`, by the same rule as the list.
#[must_use]
pub fn the_name_of(code: &str, language: &str) -> Option<String> {
    the_regions_named_in(language)
        .into_iter()
        .find(|region| region.code == code)
        .map(|region| region.name)
}

/// Whether two names are in the order `language` sorts them, for a test.
#[cfg(test)]
fn in_order(language: &str, one: &str, other: &str) -> bool {
    let locale: Locale = language.parse().unwrap_or(Locale::UNKNOWN);
    Collator::try_new(locale.clone().into(), CollatorOptions::default())
        .map(|collator| collator.compare(one, other) != std::cmp::Ordering::Greater)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **English names every region on the list**, so the code is never what
    /// a person reads.
    #[test]
    fn english_names_every_region() {
        let named = the_regions_named_in("en");
        assert_eq!(named.len(), EVERY_REGION.len());
        for region in &named {
            assert_ne!(
                region.name, region.code,
                "{} has no English name",
                region.code
            );
        }
    }

    /// **Every one of the 24 lists every region**, in its own language where
    /// CLDR has it — Maltese and Irish held to the same test as German.
    #[test]
    fn every_language_this_machine_carries_lists_every_region() {
        for official in alo_strings::union::OFFICIAL {
            let named = the_regions_named_in(official.tag);
            assert_eq!(named.len(), EVERY_REGION.len(), "{}", official.tag);
            for pair in named.windows(2) {
                if let [one, other] = pair {
                    assert!(
                        in_order(official.tag, &one.name, &other.name),
                        "{}: {} before {}",
                        official.tag,
                        one.name,
                        other.name
                    );
                }
            }
        }
    }

    /// **The names are the language's, not English's.**
    #[test]
    fn a_region_is_called_what_the_language_calls_it() {
        assert_eq!(the_name_of("DE", "de").as_deref(), Some("Deutschland"));
        assert_eq!(the_name_of("DE", "en").as_deref(), Some("Germany"));
        assert_eq!(the_name_of("BE", "fr").as_deref(), Some("Belgique"));
    }

    /// **A language nobody can ask about still gets a list.**
    #[test]
    fn a_language_that_is_not_one_still_gets_every_region() {
        assert_eq!(
            the_regions_named_in("not a language").len(),
            EVERY_REGION.len()
        );
    }
}
