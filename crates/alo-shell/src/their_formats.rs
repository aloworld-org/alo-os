//! **How one person's machine writes a moment**: what they chose, the language
//! they read, and the zone they are in — and the one rule for what happens
//! where they chose nothing.
//!
//! Every surface that shows a time or a day asks this rather than deciding: the
//! status area's clock, the record's times and the days over them, and the
//! lock screen's clock once a running machine draws one. One answer, so the
//! clock and the record cannot disagree about what time it was.
//!
//! # The words and the region
//!
//! **The words are always the language the person reads now**; where they
//! have named none, the machine's source language, English.
//!
//! **The region is the first of these that answers** — the owner's decision of
//! 2026-10-28, *the region the machine is in*, and never the United States for
//! every English reader who is somewhere else:
//!
//! 1. the region the person chose in Settings (`formats.toml`);
//! 2. the region picked on the machine's first-start sheet
//!    (`alo_formats::set_up_in`);
//! 3. the country of the time zone the machine keeps, where it keeps one that
//!    is a place (`alo_formats::where_the_machine_is::the_region_of_the_zone`);
//! 4. and only then CLDR's own default for the language — `de` as Germany
//!    writes it, `en` as CLDR writes English with no region named.
//!
//! [`TheirFormats::the_region`] says which of the four answered.
//!
//! It is never *English as written in Great Britain for everybody*, which is
//! what the status area's clock was until 2026-10-28: a figure typed into one
//! binary, true for one country.
//!
//! # Where they are
//!
//! The zone they chose, where they chose one; otherwise the machine's own. A
//! zone that is named and that the zone database does not have is not guessed
//! at: the machine's own zone is used and [`TheirFormats::zone_not_found`] says
//! which name it was, so whoever starts the session can say so.

use std::time::SystemTime;

use alo_formats::where_the_machine_is::the_region_of_the_zone;
use alo_formats::{Chosen, RegionCode, Regionally, THE_SOURCE_LANGUAGE, Timezone, words};
use alo_strings::{Filling, Strings};
use jiff::Timestamp;
use jiff::civil::Date;
use jiff::tz::TimeZone;

/// **One person's way of writing a moment.**
#[derive(Debug, Clone)]
pub struct TheirFormats {
    /// What they chose, as their file says.
    chosen: Chosen,
    /// The language they read, best first, where they said.
    reading: Option<String>,
    /// The zone moments are turned into days and times in.
    zone: TimeZone,
    /// The zone they named, where the database did not have it.
    zone_not_found: Option<Timezone>,
    /// The region the machine was set up in, on its first-start sheet.
    set_up_in: Option<RegionCode>,
}

/// **Which of the four answered** the question of a person's region, in the
/// order [`TheirFormats::the_region`] asks them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereTheRegionCameFrom {
    /// The person chose it in Settings.
    TheyChoseIt,
    /// It was picked on the machine's first-start sheet.
    TheMachineWasSetUpThere,
    /// The machine's time zone is in it.
    TheTimeZoneIsThere,
    /// Nothing named one, so the language's own way is used, with no region.
    NothingNamedOne,
}

impl PartialEq for TheirFormats {
    /// Two are the same when what was chosen and read is: the zone is worked
    /// out from those and from the machine.
    fn eq(&self, other: &Self) -> bool {
        self.chosen == other.chosen
            && self.reading == other.reading
            && self.set_up_in == other.set_up_in
    }
}

impl TheirFormats {
    /// **What a person chose, for somebody reading `reading`.**
    #[must_use]
    pub fn of(chosen: Chosen, reading: Option<&str>) -> Self {
        let (zone, zone_not_found) = match &chosen.timezone {
            Some(named) => match TimeZone::get(named.name()) {
                Ok(zone) => (zone, None),
                Err(_) => (TimeZone::system(), Some(named.clone())),
            },
            None => (TimeZone::system(), None),
        };
        Self {
            chosen,
            reading: reading.map(str::to_owned),
            zone,
            zone_not_found,
            set_up_in: None,
        }
    }

    /// The same person, on a machine set up in `region` on its first-start
    /// sheet — or on one where nobody answered that, for [`None`].
    #[must_use]
    pub fn where_the_machine_was_set_up(mut self, region: Option<RegionCode>) -> Self {
        self.set_up_in = region;
        self
    }

    /// The same person, after they chose something else.
    #[must_use]
    pub fn with(&self, chosen: Chosen) -> Self {
        Self::of(chosen, self.reading.as_deref()).where_the_machine_was_set_up(self.set_up_in.clone())
    }

    /// The same person in a zone given rather than looked up — for a test, or
    /// a machine that has been told its zone some other way.
    #[must_use]
    pub fn in_zone(mut self, zone: TimeZone) -> Self {
        self.zone = zone;
        self.zone_not_found = None;
        self
    }

    /// What they chose.
    #[must_use]
    pub const fn chosen(&self) -> &Chosen {
        &self.chosen
    }

    /// The language they read, where they said — or the machine's source.
    #[must_use]
    pub fn reading(&self) -> &str {
        self.reading.as_deref().unwrap_or(THE_SOURCE_LANGUAGE)
    }

    /// The zone they named that the zone database does not have, if they did.
    #[must_use]
    pub const fn zone_not_found(&self) -> Option<&Timezone> {
        self.zone_not_found.as_ref()
    }

    /// **The language and region their dates and times are written in**: the
    /// language they read **now**, in the order of the region they chose — or,
    /// where they chose none, the rule in this file's header.
    ///
    /// The language is never the file's. The owner's decision of 2026-10-28:
    /// a person in Germany who reads English reads *28 Oct 2026, 17:38*, and the
    /// day they change to French the month is French and the order is still
    /// Germany's.
    #[must_use]
    pub fn regionally(&self) -> Regionally {
        let reading =
            Regionally::reading(self.reading()).unwrap_or_else(|_| Regionally::the_source());
        match self.the_region().0 {
            Some(region) => reading.clone().in_region(&region).unwrap_or(reading),
            None => reading,
        }
    }

    /// **The region their dates and times are written like, and which of the
    /// four in this file's header answered.**
    #[must_use]
    pub fn the_region(&self) -> (Option<String>, WhereTheRegionCameFrom) {
        if let Some(chosen) = self.chosen.region() {
            return (Some(chosen.to_owned()), WhereTheRegionCameFrom::TheyChoseIt);
        }
        if let Some(set_up) = &self.set_up_in {
            return (
                Some(set_up.code().to_owned()),
                WhereTheRegionCameFrom::TheMachineWasSetUpThere,
            );
        }
        if let Some(zone) = self.zone.iana_name().and_then(the_region_of_the_zone) {
            return (
                Some(zone.code().to_owned()),
                WhereTheRegionCameFrom::TheTimeZoneIsThere,
            );
        }
        (None, WhereTheRegionCameFrom::NothingNamedOne)
    }

    /// The day `at` falls on, where they are.
    fn day(&self, at: SystemTime) -> Option<Date> {
        let instant = Timestamp::try_from(at).ok()?;
        Some(instant.to_zoned(self.zone.clone()).date())
    }

    /// **The time of day at `at`**, written as they write one.
    #[must_use]
    pub fn time_of(&self, at: SystemTime) -> Option<String> {
        let instant = Timestamp::try_from(at).ok()?;
        let here = instant.to_zoned(self.zone.clone());
        let hour = u8::try_from(here.hour()).ok()?;
        let minute = u8::try_from(here.minute()).ok()?;
        self.regionally().time(hour, minute)
    }

    /// **Which day `at` was, seen from `now`**: today, yesterday, or a date.
    ///
    /// A moment after today — a clock that was wrong when it was written — is
    /// a date like any other, never *today*.
    #[must_use]
    pub fn day_of(&self, at: SystemTime, now: SystemTime) -> Option<TheDay> {
        let then = self.day(at)?;
        let today = self.day(now)?;
        if then == today {
            return Some(TheDay::Today);
        }
        if today.yesterday().ok() == Some(then) {
            return Some(TheDay::Yesterday);
        }
        let month = u8::try_from(then.month()).ok()?;
        let day = u8::try_from(then.day()).ok()?;
        Some(TheDay::On(self.regionally().date(
            i32::from(then.year()),
            month,
            day,
        )?))
    }
}

/// **A day, as a heading names it.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheDay {
    /// The day it is now, where the person is.
    Today,
    /// The day before.
    Yesterday,
    /// Any other day, written as the person writes a date.
    On(String),
}

impl TheDay {
    /// What the heading says, in the person's language.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> String {
        let nothing = Filling::nothing();
        match self {
            Self::Today => strings.say(&words::TODAY.key(), &nothing).into_text(),
            Self::Yesterday => strings.say(&words::YESTERDAY.key(), &nothing).into_text(),
            Self::On(written) => written.clone(),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// 2026-10-28 17:38:00 in Brussels, which is 16:38 universal time.
    fn half_past_five() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_793_205_480)
    }

    fn brussels(chosen: Chosen, reading: Option<&str>) -> TheirFormats {
        TheirFormats::of(chosen, reading).in_zone(TimeZone::get("Europe/Brussels").unwrap())
    }

    fn in_region(region: &str) -> Chosen {
        let mut chosen = Chosen::default();
        chosen.written_as_in(Some(region)).unwrap();
        chosen
    }

    fn in_universal_time(chosen: Chosen, reading: Option<&str>) -> TheirFormats {
        TheirFormats::of(chosen, reading).in_zone(TimeZone::UTC)
    }

    fn code(region: &str) -> Option<RegionCode> {
        RegionCode::named(region).ok()
    }

    /// **The region is the first of the four that answers**, in the order the
    /// owner decided on 2026-10-28: what the person chose, where the machine
    /// was set up, the country of its time zone, and only then the language's
    /// own default.
    #[test]
    fn the_region_is_the_first_of_the_four_that_answers() {
        // Nothing at all: the language's own way, with no region.
        let nothing = in_universal_time(Chosen::default(), Some("en"));
        assert_eq!(
            nothing.the_region(),
            (None, WhereTheRegionCameFrom::NothingNamedOne)
        );
        assert_eq!(nothing.regionally(), Regionally::reading("en").unwrap());
        // A zone that is a place: its country.
        let in_brussels = brussels(Chosen::default(), Some("en"));
        assert_eq!(
            in_brussels.the_region(),
            (
                Some("BE".to_owned()),
                WhereTheRegionCameFrom::TheTimeZoneIsThere
            )
        );
        // The first-start sheet, over the zone.
        let set_up = in_brussels.where_the_machine_was_set_up(code("IE"));
        assert_eq!(
            set_up.the_region(),
            (
                Some("IE".to_owned()),
                WhereTheRegionCameFrom::TheMachineWasSetUpThere
            )
        );
        // The person's own choice, over everything.
        let chosen = set_up.with(in_region("DE"));
        assert_eq!(
            chosen.the_region(),
            (Some("DE".to_owned()), WhereTheRegionCameFrom::TheyChoseIt)
        );
    }

    /// **An English reader in Ireland is not written to as an American**: the
    /// machine set up in Ireland writes Ireland's twenty-four-hour clock even
    /// though its zone is universal time and English's own default is the
    /// United States.
    #[test]
    fn an_english_reader_where_the_machine_is_set_up_is_not_written_to_as_an_american() {
        let irish = in_universal_time(Chosen::default(), Some("en"))
            .where_the_machine_was_set_up(code("IE"));
        assert_eq!(irish.time_of(half_past_five()).as_deref(), Some("16:38"));
        let nobody_knows = in_universal_time(Chosen::default(), Some("en"));
        assert_ne!(
            nobody_knows.time_of(half_past_five()),
            irish.time_of(half_past_five())
        );
    }

    /// **Choosing a region changes how the same moment is written.**
    #[test]
    fn choosing_a_region_changes_the_time_written() {
        let american = brussels(in_region("US"), Some("en"));
        let british = brussels(in_region("GB"), Some("en"));
        let at = half_past_five();
        assert_eq!(british.time_of(at).as_deref(), Some("17:38"));
        assert_ne!(american.time_of(at), british.time_of(at));
    }

    /// **The zone they are in decides the hour**, not universal time.
    #[test]
    fn the_hour_is_the_one_where_they_are() {
        let british = in_region("GB");
        let in_brussels = brussels(british.clone(), Some("en"));
        let in_london =
            TheirFormats::of(british, Some("en")).in_zone(TimeZone::get("Europe/London").unwrap());
        assert_eq!(
            in_brussels.time_of(half_past_five()).as_deref(),
            Some("17:38")
        );
        assert_eq!(
            in_london.time_of(half_past_five()).as_deref(),
            Some("16:38")
        );
    }

    /// **Today, yesterday, and a date** in the region chosen.
    #[test]
    fn a_day_is_today_yesterday_or_a_date_in_the_region_chosen() {
        let now = half_past_five();
        let day = Duration::from_secs(24 * 60 * 60);
        let german = brussels(in_region("DE"), Some("de"));
        assert_eq!(german.day_of(now, now), Some(TheDay::Today));
        assert_eq!(german.day_of(now - day, now), Some(TheDay::Yesterday));
        assert_eq!(
            german.day_of(now - 3 * day, now),
            Some(TheDay::On("25.10.2026".to_owned()))
        );
        let american = brussels(in_region("US"), Some("en"));
        assert_eq!(
            american.day_of(now - 3 * day, now),
            Some(TheDay::On("Oct 25, 2026".to_owned()))
        );
    }

    /// **A region writes in the language read now** — the owner's decision of
    /// 2026-10-28. Germany chosen by somebody reading English is Germany's
    /// day-month order and twenty-four-hour clock in English words, and the
    /// day they change to French the month is French and the order is still
    /// Germany's. `formats.toml` was not touched between the two.
    #[test]
    fn a_region_writes_in_the_language_read_now() {
        let germany = in_region("DE");
        let in_english = brussels(germany.clone(), Some("en"));
        assert_eq!(
            in_english.time_of(half_past_five()).as_deref(),
            Some("17:38"),
            "not Germany's twenty-four-hour clock"
        );
        assert_eq!(
            in_english.regionally().date(2026, 10, 28).as_deref(),
            Some("28 Oct 2026"),
            "not day before month, in English"
        );
        let in_french = brussels(germany, Some("fr"));
        assert_eq!(in_french.regionally().language(), "fr");
        assert_eq!(in_french.regionally().region(), Some("DE"));
        assert_eq!(
            in_french.regionally().date(2026, 10, 28).as_deref(),
            Some("28 oct. 2026"),
            "the month did not follow the language"
        );
    }

    /// **The region, not the language, decides the order**: the same English
    /// reader, the United States chosen instead, is month first and a
    /// twelve-hour clock.
    #[test]
    fn the_region_not_the_language_decides_the_order() {
        let american = brussels(in_region("US"), Some("en"));
        let german = brussels(in_region("DE"), Some("en"));
        assert_eq!(
            american.regionally().date(2026, 10, 28).as_deref(),
            Some("Oct 28, 2026")
        );
        assert_ne!(
            american.time_of(half_past_five()),
            german.time_of(half_past_five())
        );
    }

    /// **Midnight where they are is the line**, not midnight in universal time:
    /// 00:30 in Brussels is today, though it is still yesterday in London.
    #[test]
    fn midnight_where_they_are_divides_the_days() {
        let formats = brussels(in_region("GB"), Some("en"));
        // 2026-10-28 00:30 in Brussels is 2026-10-27 23:30 universal time.
        let just_after_midnight = half_past_five() - Duration::from_secs((17 * 60 + 8) * 60);
        let just_before = just_after_midnight - Duration::from_secs(60 * 60);
        assert_eq!(
            formats.day_of(just_after_midnight, half_past_five()),
            Some(TheDay::Today)
        );
        assert_eq!(
            formats.day_of(just_before, half_past_five()),
            Some(TheDay::Yesterday)
        );
    }

    /// **A moment after now is a date, never today.**
    #[test]
    fn a_moment_after_today_is_a_date() {
        let formats = brussels(in_region("GB"), Some("en"));
        let now = half_past_five();
        let tomorrow = now + Duration::from_secs(24 * 60 * 60);
        assert!(matches!(formats.day_of(tomorrow, now), Some(TheDay::On(_))));
    }

    /// **A zone the database does not have is said, not guessed.**
    #[test]
    fn a_zone_nobody_has_is_said() {
        let chosen = Chosen {
            timezone: Some(Timezone::named("Made/Up").unwrap()),
            ..Chosen::default()
        };
        let formats = TheirFormats::of(chosen, Some("en"));
        assert_eq!(
            formats.zone_not_found().map(Timezone::name),
            Some("Made/Up")
        );
    }

    /// **The headings are words, the date is CLDR's.**
    #[test]
    fn the_heading_is_said_in_the_vocabulary() {
        let strings = Strings::of(alo_formats::formats_words().unwrap());
        assert_eq!(TheDay::Today.said(&strings), "Today");
        assert_eq!(TheDay::Yesterday.said(&strings), "Yesterday");
        assert_eq!(
            TheDay::On("25.10.2026".to_owned()).said(&strings),
            "25.10.2026"
        );
    }
}
