//! A moment, written the one way that reads the same everywhere.
//!
//! A place kept before a command carries the moment it was taken and **nothing
//! about the command** — alo deliberately never learns which one it was
//! ([ADR 0130](../../../docs/decisions/0130-what-takes-the-bracket-before-a-persons-own-command.md)).
//! So the moment is the only thing that tells one place from another, and it has
//! to be printed.
//!
//! # Why ISO 8601 in universal time, and not the person's own format
//!
//! `alo_formats::Regionally::date_and_time` exists and is the right road for a
//! date on a **screen**: a window, a panel, a row in Settings. This is a
//! terminal command whose output a person pastes, sorts and greps, and
//! `2026-10-22 09:41Z` is unambiguous in every region on earth while
//! `22/10/2026` and `10/22/2026` are the same six characters meaning two days.
//! Universal time for the same reason: a place kept at 00:30 local and read
//! after a timezone change must not appear to have been kept tomorrow.
//!
//! # This arithmetic exists twice already, and that is a finding
//!
//! `alo_capturing::OnThisDay` and `alo_files`' zip writer each hold a private
//! copy of the same conversion, and this is the third. The other two are not
//! reachable from here without taking a dependency on a screenshot crate or a
//! zip writer, and moving them into a shared home touches three crates for a
//! reason that is tidiness rather than correctness, so it is **written down here
//! instead of done quietly**: a date crate is wanted, nobody has decided where
//! it lives, and whoever decides has three callers waiting. Noted 2026-10-22.

use std::time::{SystemTime, UNIX_EPOCH};

/// How many seconds a day is.
const A_DAY: i64 = 24 * 60 * 60;

/// How many seconds an hour is.
const AN_HOUR: i64 = 60 * 60;

/// How many seconds a minute is.
const A_MINUTE: i64 = 60;

/// This moment as `YYYY-MM-DD HH:MMZ`, in universal time.
///
/// A moment before the epoch comes back as the date it is, with a negative year
/// where it is that old — this is for a clock on a running machine, and a
/// machine whose clock says 1823 has a problem this function is not about.
#[must_use]
pub fn as_text(at: SystemTime) -> String {
    let seconds = match at.duration_since(UNIX_EPOCH) {
        Ok(since) => i64::try_from(since.as_secs()).unwrap_or(i64::MAX),
        Err(before) => i64::try_from(before.duration().as_secs())
            .unwrap_or(i64::MAX)
            .saturating_neg(),
    };
    let (year, month, day) = the_date_of(seconds.div_euclid(A_DAY));
    let through_the_day = seconds.rem_euclid(A_DAY);
    let hour = through_the_day / AN_HOUR;
    let minute = (through_the_day % AN_HOUR) / A_MINUTE;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}Z")
}

/// The year, month and day this many days after the first of January 1970.
///
/// Howard Hinnant's `civil_from_days`, which is exact for every year the
/// Gregorian calendar is defined over and has no table of month lengths to get
/// wrong.
fn the_date_of(days_since_the_epoch: i64) -> (i64, i64, i64) {
    // Shift the epoch to the first of March 0000, so that a leap day is the last
    // day of the year rather than one in the middle of it.
    let shifted = days_since_the_epoch + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{UNIX_EPOCH, as_text};

    /// A moment this crate's own test fixtures use, so the expected text is one
    /// a reader can check against a calendar.
    fn at(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    /// **The epoch itself**, which is the one date every reader knows.
    #[test]
    fn the_epoch_is_the_first_of_january_nineteen_seventy() {
        assert_eq!(as_text(at(0)), "1970-01-01 00:00Z");
    }

    /// **A leap day**, which is what the shifted-era arithmetic above exists
    /// for. 29 February 2024 is 19,782 days after the epoch.
    #[test]
    fn a_leap_day_is_the_twenty_ninth_of_february() {
        assert_eq!(as_text(at(19_782 * 86_400)), "2024-02-29 00:00Z");
    }

    /// **The day after a leap day**, because an off-by-one in the month
    /// arithmetic shows here and not on the leap day itself.
    #[test]
    fn the_day_after_a_leap_day_is_the_first_of_march() {
        assert_eq!(as_text(at(19_783 * 86_400)), "2024-03-01 00:00Z");
    }

    /// **The last day of a century that is not a leap year.** 1900 was not a
    /// leap year and 2000 was; a conversion that divides by four and stops gets
    /// one of the two wrong. 2000-02-29 is 11,016 days after the epoch.
    #[test]
    fn the_year_two_thousand_had_a_leap_day() {
        assert_eq!(as_text(at(11_016 * 86_400)), "2000-02-29 00:00Z");
    }

    /// **The time of day**, to the minute and no finer: a second is noise on a
    /// row a person is choosing from.
    #[test]
    fn the_time_is_the_hour_and_the_minute() {
        assert_eq!(
            as_text(at(1_761_125_000)),
            "2025-10-22 09:23Z",
            "the hour and minute are taken from the seconds through the day"
        );
    }

    /// **A moment before the epoch is still a date**, rather than a panic or a
    /// saturated year. A machine whose clock has not been set yet reports one.
    #[test]
    fn a_moment_before_the_epoch_is_still_a_date() {
        let before = UNIX_EPOCH - Duration::from_secs(86_400);
        assert_eq!(as_text(before), "1969-12-31 00:00Z");
    }
}
