//! A machine that woke: the seat it wakes to, and what became of the agent's
//! work that was running when it went to sleep.
//!
//! # It wakes locked
//!
//! [`Asleep::woke`] hands back the seat the sleep locked, and nothing here
//! unlocks it. The one road back to the desktop is `alo_locking::Seat::unlocks`
//! — signing in again — exactly as it is for a lock the person made themselves.
//!
//! # A turn that was running is decided, and written down
//!
//! Work a person asked the agent for is never silently lost to a closed lid.
//! [`Woke::a_turn`] takes each turn that was under way, by value, and decides:
//!
//! - **it carries on** when it still has time left. The turn is handed back to
//!   whatever was running it, and the record says it slept through and carried
//!   on;
//! - **it is stopped** when its time ran out while the machine was asleep. The
//!   record says so with the sentence the person is shown, and the turn is
//!   ended — its grant goes with it — so nothing more is done under it.
//!
//! Both are written **before** anything is handed back, through the turn's own
//! door (`alo_turn::Turning::slept_through`). A turn whose record cannot be
//! written is ended rather than carried on, and [`AfterSleep::NotRecorded`] is
//! what the caller answers with: work that went on with no evidence of it is
//! the one outcome worse than work that stopped.
//!
//! *Its time ran out* is the rule rather than *it slept at all* because a
//! turn's length is already the person's bound on how long the agent may work
//! for them: sleeping inside it takes nothing from the person, and sleeping past
//! it would carry work into a time nobody granted.

use std::time::SystemTime;

use alo_capability::Grants;
use alo_keeping::NotKept;
use alo_locking::Seat;
use alo_strings::{Filling, Said, Strings};
use alo_turn::Turning;

use crate::deciding::Why;
use crate::going::Asleep;
use crate::words;

/// A machine that woke from a sleep.
#[derive(Debug)]
pub struct Woke<N> {
    /// The seat, still locked.
    seat: Seat<N>,
    /// When it went to sleep.
    went_at: SystemTime,
    /// When it woke.
    woke_at: SystemTime,
    /// Why it went.
    why: Why,
}

/// What became of one turn that was running when the machine went to sleep.
#[must_use = "a turn that carried on is inside, and one that stopped has a sentence for the person"]
pub enum AfterSleep<'a, 'm> {
    /// It still had time left, and carries on. Written down. Boxed, because a
    /// turn is large and the other answers are not.
    CarriedOn(Box<Turning<'a, 'm>>),
    /// Its time ran out while the machine was asleep, so it was stopped and
    /// ended. Written down, and this is what the person is shown.
    Stopped(Said),
    /// What became of it could not be written down, so it was ended. The
    /// caller answers with this rather than with anything else.
    NotRecorded(NotKept),
    /// It had already stopped before the machine slept, because something
    /// earlier could not be written down; it was ended, and nothing more could
    /// be written.
    AlreadyClosed,
}

impl<N> Asleep<N> {
    /// The machine woke at `now`. The seat is still locked.
    #[must_use]
    pub fn woke(self, now: SystemTime) -> Woke<N> {
        Woke {
            seat: self.seat,
            went_at: self.went_at,
            woke_at: now,
            why: self.why,
        }
    }
}

impl<N> Woke<N> {
    /// The seat the machine woke to — locked.
    #[must_use]
    pub const fn seat(&self) -> &Seat<N> {
        &self.seat
    }

    /// The seat, still locked, for whoever draws the lock screen and later
    /// unlocks it.
    #[must_use]
    pub fn into_seat(self) -> Seat<N> {
        self.seat
    }

    /// When the machine went to sleep.
    #[must_use]
    pub const fn went_at(&self) -> SystemTime {
        self.went_at
    }

    /// When the machine woke.
    #[must_use]
    pub const fn woke_at(&self) -> SystemTime {
        self.woke_at
    }

    /// Why the machine had gone to sleep.
    #[must_use]
    pub const fn why(&self) -> Why {
        self.why
    }

    /// Decide what becomes of a turn that was running when the machine slept,
    /// write it down, and hand it back only if it carries on.
    ///
    /// `grants` are the machine's, which a turn that ends gives its grant back
    /// to. `strings` word the sentence a stopped turn is recorded with and the
    /// person is shown.
    pub fn a_turn<'a, 'm>(
        &self,
        mut turning: Turning<'a, 'm>,
        grants: &mut Grants,
        strings: &Strings,
    ) -> AfterSleep<'a, 'm> {
        if turning.is_closed() {
            let _ended = turning.ending(grants);
            return AfterSleep::AlreadyClosed;
        }
        if turning.ends() <= self.woke_at {
            let said = strings.say(&words::STOPPED_WHILE_ASLEEP.key(), &Filling::nothing());
            let written = turning.slept_through(Some(&said), self.woke_at);
            let _ended = turning.ending(grants);
            return match written {
                Ok(()) => AfterSleep::Stopped(said),
                Err(why) => AfterSleep::NotRecorded(why),
            };
        }
        match turning.slept_through(None, self.woke_at) {
            Ok(()) => AfterSleep::CarriedOn(Box::new(turning)),
            Err(why) => {
                let _ended = turning.ending(grants);
                AfterSleep::NotRecorded(why)
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_locking::Unlocking;
    use alo_overlay::Summoning;
    use alo_record::Happened;

    use super::*;
    use crate::changes::Settings;
    use crate::deciding::{Decided, asked};
    use crate::going::Slept;
    use crate::holding::Holding;
    use crate::lid::Displays;
    use crate::testing::{
        ANNAS, Holds, TheMachine, a_turn_on_a_machine, anna, hour, in_english, noon, the_accounts,
    };

    /// Close the lid on Anna's open seat, and let the machine sleep.
    fn the_lid_closes_at(
        now: SystemTime,
        logind: &mut TheMachine,
        holding: &mut Holding<crate::testing::Hold>,
        grants: &Grants,
    ) -> Asleep<String> {
        let Decided::Sleeps(going) = asked(
            Why::LidClosed,
            &Settings::shipped(),
            Displays::OnlyItsOwn,
            holding,
            grants,
            now,
        ) else {
            unreachable!("a closed lid with nothing attached sleeps");
        };
        match going.carried_out(Seat::opened(anna()), &mut Summoning::closed(), logind, now) {
            Slept::Asleep(asleep) => asleep,
            Slept::NotAsleep(not) => unreachable!("{:?}", not.why()),
        }
    }

    /// **Suspend, then resume: the lock is in place first, and the only way
    /// back is signing in.** The walk the acceptance names — a turn running, the
    /// lid closing, the machine waking inside the turn's length — and at every
    /// step the seat is locked until the person's own password opens it.
    #[test]
    fn suspend_then_resume_lands_on_the_lock_and_the_turn_carries_on() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let mut holding = Holding::none();

        let record = a_turn_on_a_machine(|turning, grants| {
            holding
                .a_turn(&turning, noon(), &mut logind, &strings)
                .unwrap();

            let asleep = the_lid_closes_at(noon(), &mut logind, &mut holding, grants);
            assert!(asleep.seat().is_locked(), "locked before it slept");
            assert_eq!(holds.slept(), 1);

            let woke = asleep.woke(noon() + hour() / 4);
            assert!(woke.seat().is_locked(), "and woke locked");
            assert_eq!(woke.why(), Why::LidClosed);
            assert_eq!(woke.went_at(), noon());

            let AfterSleep::CarriedOn(turning) = woke.a_turn(turning, grants, &strings) else {
                unreachable!("there was time left");
            };
            assert!(!turning.is_closed());
            let _ended = (*turning).ending(grants);

            let seat = woke.into_seat();
            let seat = match seat.unlocks(the_accounts(), "anna", "not her password") {
                Unlocking::StillLocked { seat, .. } => seat,
                other => unreachable!("a wrong password unlocked: {other:?}"),
            };
            assert!(seat.is_locked());
            let Unlocking::Unlocked { seat, .. } = seat.unlocks(the_accounts(), "anna", ANNAS)
            else {
                unreachable!("her own password unlocks");
            };
            assert!(!seat.is_locked());
        });

        assert!(record.everything().any(|entry| matches!(
            entry.happened(),
            Happened::SleptThrough { stopped: None, .. }
        )));
    }

    /// **A turn whose time ran out while the machine slept is stopped with a
    /// sentence, written down, and ended** — never silently lost, never carried
    /// on into time nobody granted.
    #[test]
    fn a_turn_whose_time_ran_out_while_asleep_is_stopped_and_recorded() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let mut holding = Holding::none();

        let record = a_turn_on_a_machine(|turning, grants| {
            let asleep = the_lid_closes_at(noon(), &mut logind, &mut holding, grants);
            let woke = asleep.woke(noon() + hour() * 8);

            let AfterSleep::Stopped(said) = woke.a_turn(turning, grants, &strings) else {
                unreachable!("its hour was over by morning");
            };
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.text().contains("Ask again"), "{said}");
            assert!(woke.seat().is_locked());
        });

        let stopped: Vec<&Happened> = record
            .everything()
            .map(alo_record::Entry::happened)
            .filter(|happened| matches!(happened, Happened::SleptThrough { .. }))
            .collect();
        assert_eq!(stopped.len(), 1);
        assert!(stopped.iter().all(|happened| happened.was_stopped()));
        assert!(stopped.iter().all(|happened| {
            happened
                .why_stopped()
                .is_some_and(|why| why.as_str().contains("while this machine was asleep"))
        }));
    }
}
