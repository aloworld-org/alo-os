//! How long what an agent did is kept.
//!
//! `alo-record` says, beside the type it is talking about, why this is not a
//! method on the record:
//!
//! > How long a record is kept, and by what, is a decision for whatever writes
//! > it to a disk, made once and in the open, rather than a method anything
//! > holding this type can reach for.
//!
//! This is that decision, as a value. It is chosen once — by the person whose
//! machine it is, or by the organisation that manages it (ADR 0004) — written
//! down where they can see it, and read by [`crate::Writing::prune`]. Nothing
//! else in this workspace can shorten a record, and nothing at all can shorten
//! one by naming what it wants gone: the only input is a rule and a moment.
//!
//! # Everything, unless somebody says otherwise
//!
//! [`Keeping::Forever`] is the default, and that is a decision rather than the
//! absence of one. A machine that ages evidence out by default would be a
//! machine whose owner has to discover, at the moment they need it, that the
//! afternoon they are asking about is gone. Erring the other way costs disk.
//!
//! What alo OS does **not** ship is a number of days that sounds reasonable.
//! How long an organisation may keep a record of what its staff's machines did
//! is a question with a legal answer in some places and a cultural one in
//! others, and `CLAUDE.md` is explicit that where alo OS lets an organisation
//! set a rule, the rule is theirs to name.
//!
//! # No time at all is not a length of time
//!
//! A record kept for zero days is a record that is deleted as it is written,
//! which is a way of turning the record off wearing the clothes of a retention
//! setting. So it is **unrepresentable**: the variant holds a
//! [`NonZeroU32`], and the one road a zero can still arrive by —
//! [`Keeping::for_days`], where a settings panel lands — refuses it in words.
//!
//! That is `alo-appearance`'s shape from item 8a: terracotta is not refused at
//! the door, it is a colour the type cannot hold, and the refusal in words
//! exists for the one place a value can still be typed in.
//!
//! # Nothing here reads the clock
//!
//! As in `alo-capability` and for the same reason: every question that depends
//! on time takes `now`, so what a rule removes is arithmetic a test can do
//! rather than something that has to be waited for.

use std::num::NonZeroU32;
use std::time::{Duration, SystemTime};

use alo_strings::{Counting, Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words;

/// How many seconds there are in a day, for a rule counted in whole days.
///
/// A day here is twenty-four hours. It is not a calendar day, and the
/// difference is a clock change: a record kept for thirty days keeps
/// thirty times this many seconds of it, whatever the machine's timezone did in
/// between. A rule that had to agree with a calendar would need to know which
/// calendar, which is a question this crate has no business answering.
const A_DAY: u64 = 24 * 60 * 60;

/// How long what happened on this machine is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Keeping {
    /// Nothing ages out. What a machine ships with.
    #[default]
    Forever,
    /// Anything that happened more than this many whole days ago is removed.
    ForDays(NonZeroU32),
}

impl Keeping {
    /// A record kept for this many whole days.
    ///
    /// # Errors
    ///
    /// [`KeepingError::NoDaysAtAll`] for zero, which is not a short retention
    /// but the absence of a record.
    pub fn for_days(days: u32) -> Result<Self, KeepingError> {
        NonZeroU32::new(days)
            .map(Self::ForDays)
            .ok_or(KeepingError::NoDaysAtAll)
    }

    /// How many days, where there is a number.
    #[must_use]
    pub fn days(self) -> Option<u32> {
        match self {
            Self::Forever => None,
            Self::ForDays(days) => Some(days.get()),
        }
    }

    /// The oldest moment this rule keeps, at this moment.
    ///
    /// `None` means **nothing is removed**, which covers both a record kept for
    /// good and a rule whose window reaches back past the beginning of time.
    ///
    /// The window is measured **from the epoch forwards rather than from `now`
    /// backwards**, and that is not a detail. `SystemTime::checked_sub` reaches
    /// happily into 1969 on Windows and does not on every platform, so a
    /// boundary worked out by subtracting from `now` would be a different
    /// boundary on different machines for a machine whose clock is wrong. Here
    /// a window that reaches past the epoch answers `None` on every platform,
    /// because a wrong clock must never be a way to empty a record.
    /// `docs/quirks.md` records what was measured.
    #[must_use]
    pub fn oldest_kept(self, now: SystemTime) -> Option<SystemTime> {
        let days = self.days()?;
        let window = Duration::from_secs(u64::from(days) * A_DAY);
        let since_the_epoch = now.duration_since(SystemTime::UNIX_EPOCH).ok()?;
        Some(SystemTime::UNIX_EPOCH + since_the_epoch.checked_sub(window)?)
    }

    /// Whether something that happened at this moment is still kept.
    ///
    /// Something that happened *after* `now` is kept: a moment in the future is
    /// a clock that has been put back, and the answer to a clock nobody can
    /// trust is never to remove more.
    #[must_use]
    pub fn keeps(self, at: SystemTime, now: SystemTime) -> bool {
        match self.oldest_kept(now) {
            None => true,
            Some(oldest) => at >= oldest,
        }
    }

    /// What this rule says, in the language the person reads.
    ///
    /// The number of days is put in by the counting, because how many is what
    /// picks the shape of the sentence: *one day* and *30 days* are two
    /// sentences in English, three in Polish and five in Irish.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        match self.days() {
            None => strings.say(&words::FOREVER.key(), &Filling::nothing()),
            Some(days) => strings.count(
                &words::FOR_DAYS.key(),
                &Counting::of(u64::from(days)),
                &Filling::nothing(),
            ),
        }
    }
}

/// Why that is not a length of time to keep a record for.
///
/// No `Display`, for the reason `alo-files`' `Failed` has none since item 9b: a
/// `Display` is one `to_string()` from a screen, in a settings panel whose
/// author had no reason to think about which language it is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeepingError {
    /// Zero days, which is not a retention rule.
    NoDaysAtAll,
}

impl KeepingError {
    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        match self {
            Self::NoDaysAtAll => strings.say(&words::NO_DAYS_AT_ALL.key(), &Filling::nothing()),
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
    use crate::testing::{in_english, said};

    /// A fixed moment, so that ageing out is arithmetic rather than a wait.
    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// A machine nobody has set anything on keeps everything. The default is a
    /// decision: evidence a person did not ask to lose is evidence they still
    /// have when they come to ask a question of it.
    #[test]
    fn a_machine_nobody_has_set_anything_on_keeps_everything() {
        assert_eq!(Keeping::default(), Keeping::Forever);
        assert_eq!(Keeping::default().oldest_kept(noon()), None);
        assert!(Keeping::default().keeps(SystemTime::UNIX_EPOCH, noon()));
    }

    /// **Zero days is not a short retention, it is no record**, and the type
    /// cannot hold it. This is the one road it can still arrive by, and it is
    /// refused in words a person can act on.
    #[test]
    fn a_record_kept_for_no_days_at_all_is_refused() {
        let refused = Keeping::for_days(0).unwrap_err();
        assert_eq!(refused, KeepingError::NoDaysAtAll);
        let message = said(&refused.said(&in_english()));
        assert!(message.contains("kept for good"), "{message}");
        assert!(message.contains("no evidence"), "{message}");
    }

    /// One day is a rule, and the smallest one there is.
    #[test]
    fn one_day_is_a_rule() {
        let keeping = Keeping::for_days(1).unwrap();
        assert_eq!(keeping.days(), Some(1));
        assert_eq!(
            keeping.oldest_kept(noon()),
            Some(noon() - Duration::from_secs(A_DAY))
        );
    }

    /// What a rule removes is decided by arithmetic on a moment that is passed
    /// in, so the boundary is exact and testable without waiting for it.
    #[test]
    fn what_a_rule_keeps_is_decided_at_a_moment_that_is_passed_in() {
        let thirty = Keeping::for_days(30).unwrap();
        let boundary = noon() - Duration::from_secs(30 * A_DAY);
        assert!(
            thirty.keeps(boundary, noon()),
            "the boundary itself is kept"
        );
        assert!(!thirty.keeps(boundary - Duration::from_secs(1), noon()));
        assert!(thirty.keeps(boundary + Duration::from_secs(1), noon()));
    }

    /// **A clock that has been put back is not a way to empty a record.**
    /// Something stamped after the moment pruning is running at is kept, and a
    /// machine that thinks it is still the first minute of 1970 removes nothing
    /// at all.
    ///
    /// The second half is a **portability** guarantee as much as a policy one.
    /// `SystemTime::checked_sub` walks back into 1969 on Windows, so a boundary
    /// worked out by subtracting from `now` would be a different boundary on
    /// different machines. This says the answer is the same on all of them.
    #[test]
    fn a_wrong_clock_never_removes_more() {
        let thirty = Keeping::for_days(30).unwrap();
        assert!(thirty.keeps(noon() + Duration::from_secs(A_DAY), noon()));

        let early = SystemTime::UNIX_EPOCH + Duration::from_secs(60);
        assert_eq!(thirty.oldest_kept(early), None);
        assert!(thirty.keeps(SystemTime::UNIX_EPOCH, early));
        assert!(thirty.keeps(early, early));

        // The boundary is never before the epoch, however long the rule is.
        let a_century = Keeping::for_days(365 * 100).unwrap();
        assert_eq!(a_century.oldest_kept(noon()), None);
        assert!(a_century.keeps(SystemTime::UNIX_EPOCH, noon()));
    }

    /// The rule is a setting, so it is written down and read back — and a
    /// settings file cannot bring back the zero the constructor refuses,
    /// because there is no shape for it to arrive in.
    #[test]
    fn the_rule_is_written_down_and_a_zero_cannot_be_read_back() {
        let thirty = Keeping::for_days(30).unwrap();
        let written = serde_json::to_string(&thirty).unwrap();
        assert_eq!(serde_json::from_str::<Keeping>(&written).ok(), Some(thirty));
        assert_eq!(
            serde_json::to_string(&Keeping::Forever).unwrap(),
            "\"forever\""
        );
        assert!(serde_json::from_str::<Keeping>("{\"for-days\":0}").is_err());
    }

    /// **The one sentence that counts is counted the reader's own way.** A
    /// setting that read "kept for 1 days" is the bug item 9a exists to remove,
    /// and it is a bug in every language that does not share English's two
    /// shapes.
    #[test]
    fn how_long_a_record_is_kept_is_counted_and_not_stuck_into_a_sentence() {
        let strings = in_english();
        let one = said(&Keeping::for_days(1).unwrap().said(&strings));
        let many = said(&Keeping::for_days(30).unwrap().said(&strings));
        assert!(one.contains("one day"), "{one}");
        assert!(!one.contains('1'), "{one}");
        assert!(many.contains("30 days"), "{many}");
        assert!(
            said(&Keeping::Forever.said(&strings)).contains("everything"),
            "a machine that keeps everything says so"
        );
    }

    /// A shell that never declared this crate's words shows the key and says it
    /// is a bug, rather than being handed an English sentence kept here for the
    /// purpose.
    #[test]
    fn a_rule_nobody_declared_the_words_for_says_so() {
        let strings = Strings::of(alo_strings::Vocabulary::empty());
        let said = Keeping::Forever.said(&strings);
        assert!(said.is_a_bug());
        assert_eq!(said.text(), "«keeping.forever»");
    }
}
