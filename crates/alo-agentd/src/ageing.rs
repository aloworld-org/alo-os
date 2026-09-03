//! When what a machine no longer keeps is removed from its record.
//!
//! `alo-keeping` decides **what** a shortening removes — a rule and a moment,
//! and nothing that can name an entry — and `alo-turn` is the one door onto it.
//! This is the other question, the one queue item 20 was left holding: *when*.
//!
//! # A rule makes the timer, and the absence of one leaves the machine asleep
//!
//! `alo_keeping::Keeping::Forever` is what a machine ships with, and a machine
//! that keeps everything has nothing to do on an interval: [`Ageing::before`]
//! answers `None`, `crate::unix::ready` waits with no timeout, and the process
//! sleeps in one call until somebody says something. That is exactly what the
//! service did before this file existed, and it is still what it does on every
//! machine nobody has set a retention rule on.
//!
//! An organisation that *does* set one (ADR 0004) buys the other behaviour with
//! it. A rule counted in days is a promise about a file on a disk, and a file on
//! a disk goes on ageing whether or not anybody is talking to their agent — so
//! a service that only measured time when a message arrived could not keep a
//! promise about time. Under a rule the wait has a timeout and the machine wakes
//! to shorten; with no rule there is nothing for it to wake up for.
//!
//! # Once an hour, and the interval is alo OS's rather than an organisation's
//!
//! [`EVERY`] is an hour. The finest rule anybody can set is one whole day
//! (`alo_keeping::Keeping::for_days` counts days and refuses zero), so an hour
//! is twenty-four times finer than the smallest promise a machine can be under
//! — and a machine that is being talked to all day does not spend it reading and
//! rewriting a record to remove another few minutes.
//!
//! It is not a key in `docs/contracts/machine-description.md`, and that is the
//! division ADR 0004 draws: *how long* evidence is kept is the organisation's to
//! name, and *how often the machine tidies up* is a mechanism rather than a
//! policy. A machine cannot keep less than its rule by setting this, and it
//! cannot keep more.
//!
//! # The first one is at the start, and it falls out of the same rule
//!
//! Nothing has run yet when a service starts, so the first shortening is due
//! immediately — which is the one that matters most, because a machine that was
//! switched off for six months comes back with six months of a rule to catch up
//! on. There is no special case for it here: an [`Ageing`] that has never run is
//! due now.
//!
//! # What is recorded is the attempt, not the success
//!
//! [`Ageing::ran`] is called whether the shortening worked or was refused. A
//! record with a line nobody can read is refused every time it is asked
//! (`alo-keeping` will not rewrite a file it cannot read all of), and an
//! attempt that only counted when it succeeded would turn that into a machine
//! trying again on every round for as long as it is switched on.

use std::time::{Duration, SystemTime};

use alo_keeping::Keeping;

/// How often a machine under a retention rule shortens its record.
///
/// An hour. The module documentation is the argument for it.
pub const EVERY: Duration = Duration::from_secs(60 * 60);

/// The rule this machine's record is kept under, and when it was last acted on.
///
/// Holds no clock of its own: every question takes `now`, as everywhere else in
/// this workspace, so when a shortening is due is arithmetic a test can do
/// rather than something it has to wait for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ageing {
    /// How long the record is kept.
    keeping: Keeping,
    /// When a shortening was last attempted, if one ever was.
    last: Option<SystemTime>,
    /// Whether there is a record on a disk here at all.
    on_a_disk: bool,
}

impl Ageing {
    /// A machine keeping its record under this rule, which has shortened
    /// nothing yet.
    #[must_use]
    pub const fn under(keeping: Keeping) -> Self {
        Self {
            keeping,
            last: None,
            on_a_disk: true,
        }
    }

    /// How long the record is kept.
    #[must_use]
    pub const fn keeping(&self) -> Keeping {
        self.keeping
    }

    /// Whether a shortening should run now.
    #[must_use]
    pub fn due(&self, now: SystemTime) -> bool {
        self.before(now) == Some(Duration::ZERO)
    }

    /// How long a wait may last before the next shortening is due.
    ///
    /// `None` means there is nothing to wake up for — a machine that keeps
    /// everything, or one whose record is not on a disk — and a caller passes
    /// that straight to `crate::unix::ready` as *no timeout at all*.
    ///
    /// **The answer is never longer than [`EVERY`].** A clock put back would
    /// otherwise put the next shortening as far into the future as the clock
    /// moved, and a machine would stop keeping its rule because somebody
    /// corrected its time. Capping it costs at most one extra wake-up an hour.
    #[must_use]
    pub fn before(&self, now: SystemTime) -> Option<Duration> {
        if !self.on_a_disk || self.keeping.days().is_none() {
            return None;
        }
        let Some(last) = self.last else {
            return Some(Duration::ZERO);
        };
        // A clock at the end of representable time cannot be added to. Waiting
        // the whole interval is the answer that neither spins nor stops: the
        // next round asks again.
        let Some(due) = last.checked_add(EVERY) else {
            return Some(EVERY);
        };
        Some(due.duration_since(now).unwrap_or(Duration::ZERO).min(EVERY))
    }

    /// One shortening was attempted at this moment.
    ///
    /// Called whether it removed anything, removed nothing, or was refused —
    /// the module documentation says why the refused ones count.
    pub const fn ran(&mut self, now: SystemTime) {
        self.last = Some(now);
    }

    /// There is no record on a disk here, so there is nothing to shorten.
    ///
    /// After this the machine goes back to sleeping until somebody says
    /// something, because waking on an interval to be told the same thing again
    /// is a machine spending its day on nothing.
    pub const fn nothing_to_shorten(&mut self) {
        self.on_a_disk = false;
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A fixed moment, so that being due is arithmetic rather than a wait.
    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// Ninety days, as an organisation would set it.
    fn ninety_days() -> Keeping {
        Keeping::for_days(90).unwrap()
    }

    /// **A machine that keeps everything never wakes up to shorten anything**,
    /// which is what a machine ships with and what it does today. The wait has
    /// no timeout, exactly as it had before there was a timer at all.
    #[test]
    fn a_machine_that_keeps_everything_has_nothing_to_wake_up_for() {
        let ageing = Ageing::under(Keeping::Forever);
        assert_eq!(ageing.before(noon()), None);
        assert!(!ageing.due(noon()));
    }

    /// **The first shortening is due at once**, because a machine that was
    /// switched off for six months has six months of a rule to catch up on and
    /// nothing has run yet.
    #[test]
    fn the_first_shortening_under_a_rule_is_due_immediately() {
        let ageing = Ageing::under(ninety_days());
        assert!(ageing.due(noon()));
        assert_eq!(ageing.before(noon()), Some(Duration::ZERO));
    }

    /// After one has run, the next is an interval away and the wait is exactly
    /// what is left of it.
    #[test]
    fn the_next_one_is_an_interval_after_the_last() {
        let mut ageing = Ageing::under(ninety_days());
        ageing.ran(noon());

        assert!(!ageing.due(noon()));
        assert_eq!(ageing.before(noon()), Some(EVERY));
        assert_eq!(
            ageing.before(noon() + Duration::from_secs(600)),
            Some(EVERY - Duration::from_secs(600))
        );
        assert!(ageing.due(noon() + EVERY));
        assert!(ageing.due(noon() + EVERY + Duration::from_secs(1)));
    }

    /// **A clock put back does not stop a machine keeping its rule.** The wait
    /// is capped at the interval, so correcting a machine's time by a year
    /// costs one extra wake-up rather than a year of a retention rule going
    /// unenforced.
    #[test]
    fn a_clock_put_back_costs_a_wake_up_and_not_the_rule() {
        let mut ageing = Ageing::under(ninety_days());
        ageing.ran(noon());

        let a_year_earlier = noon() - Duration::from_secs(365 * 24 * 60 * 60);
        assert_eq!(ageing.before(a_year_earlier), Some(EVERY));
    }

    /// **A record that is not on a disk stops the timer**, rather than leaving
    /// a machine waking every hour to be told the same thing. It is what a
    /// record held only in memory answers, and it cannot be un-answered.
    #[test]
    fn a_record_that_is_not_on_a_disk_stops_the_waking_up() {
        let mut ageing = Ageing::under(ninety_days());
        assert!(ageing.due(noon()));

        ageing.ran(noon());
        ageing.nothing_to_shorten();

        assert_eq!(ageing.before(noon() + EVERY * 24), None);
        assert!(!ageing.due(noon() + EVERY * 24));
    }

    /// The rule comes back as it went in: this file decides when, and never how
    /// long.
    #[test]
    fn the_rule_is_carried_and_not_changed() {
        assert_eq!(Ageing::under(ninety_days()).keeping(), ninety_days());
        assert_eq!(Ageing::under(Keeping::Forever).keeping(), Keeping::Forever);
    }

    /// The interval is finer than the finest rule anybody can be under, by a
    /// margin — which is the whole argument for it being an hour rather than a
    /// number somebody has to think about.
    #[test]
    fn the_interval_is_finer_than_the_smallest_rule_there_is() {
        let a_day = Duration::from_secs(24 * 60 * 60);
        assert!(EVERY < a_day);
        assert_eq!(Keeping::for_days(1).unwrap().days(), Some(1));
        assert!(Keeping::for_days(0).is_err(), "a finer rule than one day");
    }
}
