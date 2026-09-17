//! A decided sleep carried out: **the seat is locked, and only then does
//! anything reach the machine.**
//!
//! [`Going::carried_out`] is the one road from a decision to a sleeping
//! machine. It locks the seat through `alo_locking::Seat::locked` — which
//! closes the agent's overlay and keeps everything running — makes a
//! [`LockedFirst`] out of the locked seat, and hands that to
//! [`Logind::sleep`], which cannot be called without one. A resume therefore
//! never lands on an unlocked desktop, and the order is the compiler's to
//! enforce rather than a comment's.
//!
//! # A sleep that did not happen leaves the seat locked
//!
//! When the machine does not go to sleep, the seat is not unlocked again:
//! [`Slept::NotAsleep`] hands it back locked, beside what the machine said. The
//! person comes back to a lock screen whether or not the machine slept, and
//! nothing about a failed sleep is a road past signing in.

use std::time::SystemTime;

use alo_locking::Seat;
use alo_overlay::Summoning;

use crate::deciding::{Going, Why};
use crate::logind::{LockedFirst, Logind, NotSlept};

/// What came of carrying out a sleep.
#[derive(Debug)]
#[must_use = "the seat is inside, and it is locked"]
pub enum Slept<N> {
    /// The machine went to sleep, locked.
    Asleep(Asleep<N>),
    /// The machine did not go to sleep, and is locked anyway.
    NotAsleep(NotAsleep<N>),
}

/// A machine that went to sleep, and the seat it will wake to.
#[derive(Debug)]
pub struct Asleep<N> {
    /// The seat, locked.
    pub(crate) seat: Seat<N>,
    /// When it went.
    pub(crate) went_at: SystemTime,
    /// Why it went.
    pub(crate) why: Why,
}

/// A machine that was asked to sleep and did not.
#[derive(Debug)]
pub struct NotAsleep<N> {
    /// The seat, locked.
    seat: Seat<N>,
    /// What the machine said.
    why: NotSlept,
}

impl<N> Asleep<N> {
    /// The seat the machine will wake to — locked.
    #[must_use]
    pub const fn seat(&self) -> &Seat<N> {
        &self.seat
    }

    /// When the machine went to sleep.
    #[must_use]
    pub const fn went_at(&self) -> SystemTime {
        self.went_at
    }

    /// Why the machine went to sleep.
    #[must_use]
    pub const fn why(&self) -> Why {
        self.why
    }
}

impl<N> NotAsleep<N> {
    /// The seat, still locked.
    #[must_use]
    pub const fn seat(&self) -> &Seat<N> {
        &self.seat
    }

    /// What the machine said.
    #[must_use]
    pub const fn why(&self) -> &NotSlept {
        &self.why
    }

    /// The seat, still locked, for whoever carries on drawing the lock screen.
    #[must_use]
    pub fn into_seat(self) -> Seat<N> {
        self.seat
    }
}

impl Going {
    /// Lock the seat, then put the machine to sleep.
    ///
    /// `summoning` is the agent's overlay, closed by the lock if it is open.
    pub fn carried_out<N, L: Logind>(
        self,
        seat: Seat<N>,
        summoning: &mut Summoning,
        logind: &mut L,
        now: SystemTime,
    ) -> Slept<N> {
        let seat = seat.locked(summoning);
        let Some(locked_first) = locked_first(&seat) else {
            // `Seat::locked` answers a locked seat whatever it was given, so
            // this is a seat task 1 changed underneath; it is refused rather
            // than slept.
            return Slept::NotAsleep(NotAsleep {
                seat,
                why: NotSlept {
                    machine: "the seat was not locked, so sleep was not asked for".to_owned(),
                },
            });
        };
        match logind.sleep(locked_first) {
            Ok(()) => Slept::Asleep(Asleep {
                seat,
                went_at: now,
                why: self.why(),
            }),
            Err(why) => Slept::NotAsleep(NotAsleep { seat, why }),
        }
    }
}

/// Proof that this seat is locked, or nothing.
///
/// The one place a [`LockedFirst`] is made.
pub(crate) fn locked_first<N>(seat: &Seat<N>) -> Option<LockedFirst> {
    seat.is_locked().then_some(LockedFirst(()))
}

#[cfg(test)]
mod tests {
    use alo_capability::Grants;
    use alo_overlay::{Compositor, Pressed, SurfaceRefused, SurfaceRequest};

    use super::*;
    use crate::changes::Settings;
    use crate::deciding::{Decided, asked};
    use crate::holding::Holding;
    use crate::lid::Displays;
    use crate::testing::{Holds, TheMachine, anna, noon};

    /// A compositor with room for the overlay.
    struct Room;

    impl Compositor for Room {
        fn honour(&mut self, _asked: SurfaceRequest) -> Result<(), SurfaceRefused> {
            Ok(())
        }
    }

    /// A sleep decided for this reason, on a machine nothing holds awake.
    fn going(why: Why) -> Going {
        let mut holding: Holding<()> = Holding::none();
        match asked(
            why,
            &Settings::shipped(),
            Displays::OnlyItsOwn,
            &mut holding,
            &Grants::default(),
            noon(),
        ) {
            Decided::Sleeps(going) => going,
            Decided::StaysAwake(awake) => unreachable!("{awake:?}"),
        }
    }

    /// **The seat is locked before the machine is asked to sleep**, and the
    /// agent's overlay, open at the time, is closed by it.
    #[test]
    fn the_seat_is_locked_before_the_machine_sleeps() {
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let mut summoning = Summoning::closed();
        assert_eq!(summoning.press(Some(&mut Room)), Pressed::Summoned);

        let slept = going(Why::LidClosed).carried_out(
            Seat::<String>::opened(anna()),
            &mut summoning,
            &mut logind,
            noon(),
        );
        let Slept::Asleep(asleep) = slept else {
            unreachable!("the machine sleeps");
        };
        assert!(asleep.seat().is_locked());
        assert!(!summoning.is_open());
        assert_eq!(holds.slept(), 1);
        assert_eq!(asleep.why(), Why::LidClosed);
        assert_eq!(asleep.went_at(), noon());
    }

    /// **A machine that did not go to sleep is still locked**: failing to sleep
    /// is not a road past the lock screen.
    #[test]
    fn a_machine_that_did_not_sleep_stays_locked() {
        let holds = Holds::default();
        let mut logind = TheMachine::refusing(&holds);
        let slept = going(Why::YouAsked).carried_out(
            Seat::<String>::opened(anna()),
            &mut Summoning::closed(),
            &mut logind,
            noon(),
        );
        let Slept::NotAsleep(not) = slept else {
            unreachable!("the machine refused");
        };
        assert!(not.seat().is_locked());
        assert_eq!(holds.slept(), 0);
        assert!(!not.why().machine.is_empty());
        assert!(not.into_seat().is_locked());
    }

    /// **An open seat gives no proof of being locked**, and a locked one does.
    #[test]
    fn only_a_locked_seat_proves_it_was_locked_first() {
        let open = Seat::<String>::opened(anna());
        assert!(locked_first(&open).is_none());
        let locked = open.locked(&mut Summoning::closed());
        assert!(locked_first(&locked).is_some());
    }
}
