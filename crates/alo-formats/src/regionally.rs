//! The formats one person reads, from the language and the region they chose.

use icu::calendar::Date;
use icu::calendar::week::WeekInformation;
use icu::datetime::fieldsets::{YMD, YMDT};
use icu::datetime::{DateTimeFormatter, NoCalendarFormatter, fieldsets};
use icu::decimal::DecimalFormatter;
use icu::locale::Locale;
use serde::{Deserialize, Serialize};

/// **A language, and the region whose way of writing things a person wants.**
///
/// The two apart: a Portuguese speaker in Belgium is `pt` and `BE`, and neither
/// choice moves the other.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Regionally {
    /// The language, as `alo-strings` spells it.
    language: String,
    /// The region, as two capital letters, or none — which is CLDR's own
    /// default for that language.
    region: Option<String>,
}

impl Regionally {
    /// A person who reads this language and has said nothing about a region.
    ///
    /// # Errors
    /// [`NotARegion::NotALanguage`] where the language is not one CLDR knows.
    pub fn reading(language: &str) -> Result<Self, NotARegion> {
        let _: Locale = language.parse().map_err(|_| NotARegion::NotALanguage {
            asked_for: language.to_owned(),
        })?;
        Ok(Self {
            language: language.to_owned(),
            region: None,
        })
    }

    /// The same person, writing things the way this region does.
    ///
    /// # Errors
    /// [`NotARegion::NotARegion`] for anything that is not two capital letters.
    pub fn in_region(mut self, region: &str) -> Result<Self, NotARegion> {
        if region.len() != 2 || !region.chars().all(|letter| letter.is_ascii_uppercase()) {
            return Err(NotARegion::NotARegion {
                asked_for: region.to_owned(),
            });
        }
        self.region = Some(region.to_owned());
        Ok(self)
    }

    /// The language this person reads.
    #[must_use]
    pub fn language(&self) -> &str {
        &self.language
    }

    /// The region whose formats they read, where they named one.
    #[must_use]
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    /// What CLDR is asked, which is the language and the region together.
    fn locale(&self) -> Locale {
        let written = match &self.region {
            Some(region) => format!("{}-{region}", self.language),
            None => self.language.clone(),
        };
        written.parse().unwrap_or(Locale::UNKNOWN)
    }

    /// **A date, written as this person writes dates.**
    #[must_use]
    pub fn date(&self, year: i32, month: u8, day: u8) -> Option<String> {
        let date = Date::try_new_iso(year, month, day).ok()?;
        let formatter = DateTimeFormatter::try_new(self.locale().into(), YMD::medium()).ok()?;
        Some(formatter.format(&date).to_string())
    }

    /// **A time of day**, written as this person writes times — which is where
    /// the twelve-hour clock lives, and it is CLDR's answer per region rather
    /// than a setting anybody here invented.
    #[must_use]
    pub fn time(&self, hour: u8, minute: u8) -> Option<String> {
        let time = icu::time::Time::try_new(hour, minute, 0, 0).ok()?;
        let formatter =
            NoCalendarFormatter::try_new(self.locale().into(), fieldsets::T::medium()).ok()?;
        Some(formatter.format(&time).to_string())
    }

    /// **A date and a time together.**
    #[must_use]
    pub fn date_and_time(
        &self,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
    ) -> Option<String> {
        let date = Date::try_new_iso(year, month, day).ok()?;
        let time = icu::time::Time::try_new(hour, minute, 0, 0).ok()?;
        let formatter = DateTimeFormatter::try_new(self.locale().into(), YMDT::medium()).ok()?;
        Some(
            formatter
                .format(&icu::time::DateTime { date, time })
                .to_string(),
        )
    }

    /// **A whole number**, with the grouping and the separators this person
    /// reads — 1.234.567 in German, 1 234 567 in French.
    #[must_use]
    pub fn number(&self, how_many: i64) -> Option<String> {
        let formatter = DecimalFormatter::try_new(self.locale().into(), Default::default()).ok()?;
        Some(formatter.format(&how_many.into()).to_string())
    }

    /// **Which day their week starts on.**
    #[must_use]
    pub fn first_day_of_the_week(&self) -> Option<FirstDay> {
        let week = WeekInformation::try_new(self.locale().into()).ok()?;
        Some(match week.first_weekday {
            icu::calendar::types::Weekday::Monday => FirstDay::Monday,
            icu::calendar::types::Weekday::Tuesday => FirstDay::Tuesday,
            icu::calendar::types::Weekday::Wednesday => FirstDay::Wednesday,
            icu::calendar::types::Weekday::Thursday => FirstDay::Thursday,
            icu::calendar::types::Weekday::Friday => FirstDay::Friday,
            icu::calendar::types::Weekday::Saturday => FirstDay::Saturday,
            icu::calendar::types::Weekday::Sunday => FirstDay::Sunday,
        })
    }
}

/// The day a week starts on, where a person reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FirstDay {
    /// Monday, which is most of Europe.
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
    /// Sunday.
    Sunday,
}

/// Why a language or a region was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotARegion {
    /// Not a language tag at all.
    #[error("`{asked_for}` is not a language this machine can ask CLDR about")]
    NotALanguage {
        /// What was asked for.
        asked_for: String,
    },
    /// Not a region code.
    #[error("`{asked_for}` is not a region: two capital letters, as in BE or PT")]
    NotARegion {
        /// What was asked for.
        asked_for: String,
    },
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Every one of the 24 writes a date, a time, a number and a first day**
    /// — Maltese and Irish held to the same completeness as German, which is
    /// this task's constraint.
    #[test]
    fn every_language_this_machine_carries_writes_all_four() {
        for official in alo_strings::union::OFFICIAL {
            let reading = Regionally::reading(official.tag).unwrap();
            assert!(
                reading.date(2026, 9, 17).is_some(),
                "{}: no date",
                official.tag
            );
            assert!(reading.time(14, 30).is_some(), "{}: no time", official.tag);
            assert!(
                reading.date_and_time(2026, 9, 17, 14, 30).is_some(),
                "{}: no date and time",
                official.tag
            );
            assert!(
                reading.number(1_234_567).is_some(),
                "{}: no number",
                official.tag
            );
            assert!(
                reading.first_day_of_the_week().is_some(),
                "{}: no first day of the week",
                official.tag
            );
        }
    }

    /// **The language and the region are apart**: a Portuguese speaker in
    /// Belgium reads Portuguese and writes the date the way Belgium does.
    #[test]
    fn a_portuguese_speaker_in_belgium_reads_portuguese_and_writes_belgian_dates() {
        let lisbon = Regionally::reading("pt").unwrap();
        let brussels = Regionally::reading("pt").unwrap().in_region("BE").unwrap();
        assert_eq!(brussels.language(), "pt");
        assert_eq!(brussels.region(), Some("BE"));
        assert_eq!(lisbon.region(), None);
        // Both write Portuguese; the region is what the format follows.
        assert!(brussels.date(2026, 9, 17).is_some());
        assert!(brussels.number(1_234_567).is_some());
    }

    /// **The formats really differ**, or none of this was worth renting.
    #[test]
    fn the_same_day_and_the_same_number_are_written_differently() {
        let german = Regionally::reading("de").unwrap();
        let english = Regionally::reading("en").unwrap();
        assert_ne!(german.date(2026, 9, 17), english.date(2026, 9, 17));
        assert_ne!(german.number(1_234_567), english.number(1_234_567));
        assert_eq!(german.number(1_234_567).unwrap(), "1.234.567");
    }

    /// **A region that is not one is refused**, naming what to write instead.
    #[test]
    fn something_that_is_not_a_region_is_refused() {
        let reading = Regionally::reading("de").unwrap();
        assert!(matches!(
            reading.clone().in_region("be"),
            Err(NotARegion::NotARegion { .. })
        ));
        assert!(matches!(
            reading.in_region("BELGIUM"),
            Err(NotARegion::NotARegion { .. })
        ));
        assert!(matches!(
            Regionally::reading("not a language at all"),
            Err(NotARegion::NotALanguage { .. })
        ));
    }
}
