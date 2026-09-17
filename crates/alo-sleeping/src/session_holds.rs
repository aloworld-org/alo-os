//! The two holds alo OS keeps for as long as a person is signed in.
//!
//! - [`TheLidIsOurs`] — closing the lid comes to [`crate::Lid`], and is not
//!   decided by the machine's own configuration first.
//! - [`UntilLocked`] — a sleep started anywhere else waits the moment it takes
//!   to lock the seat. A sleep this crate starts is locked first by
//!   `Going::carried_out`; this is the same promise for a sleep it did not
//!   start: a power key, a terminal, the machine's own low-battery action.
//!
//! Both are taken when a session opens, and both let go when their value is
//! dropped. [`UntilLocked::before_sleeping`] is the one place the second lets
//! go early, and it locks the seat before it does.

use alo_locking::Seat;
use alo_overlay::Summoning;
use alo_strings::{Filling, Strings};

use crate::logind::{Inhibit, Logind, NotHeld};
use crate::words;

/// Closing the lid is this crate's to decide, for as long as this is held.
#[derive(Debug)]
pub struct TheLidIsOurs<H> {
    /// The machine's hold on the lid.
    _held: H,
}

impl<H> TheLidIsOurs<H> {
    /// Take the lid.
    ///
    /// # Errors
    /// [`NotHeld`] when the machine would not hand it over — and then closing
    /// the lid is the machine's own configuration's, which a surface must say.
    pub fn taken<L: Logind<Held = H>>(logind: &mut L, strings: &Strings) -> Result<Self, NotHeld> {
        let why = strings.say(&words::HOLDING_THE_LID.key(), &Filling::nothing());
        logind
            .hold(Inhibit::TheLid, &why)
            .map(|held| Self { _held: held })
    }
}

/// A sleep started anywhere waits for the seat to lock, for as long as this is
/// held.
#[derive(Debug)]
pub struct UntilLocked<H> {
    /// The machine's delay on sleep.
    held: H,
}

impl<H> UntilLocked<H> {
    /// Take the delay.
    ///
    /// Taken again after every resume: [`UntilLocked::before_sleeping`] spends
    /// it.
    ///
    /// # Errors
    /// [`NotHeld`] when the machine would not take it.
    pub fn taken<L: Logind<Held = H>>(logind: &mut L, strings: &Strings) -> Result<Self, NotHeld> {
        let why = strings.say(&words::HOLDING_UNTIL_LOCKED.key(), &Filling::nothing());
        logind
            .hold(Inhibit::UntilLocked, &why)
            .map(|held| Self { held })
    }

    /// The machine said it is about to sleep: lock the seat, **then** let the
    /// sleep go ahead.
    ///
    /// Hands back the locked seat. The delay is let go of at the end of this
    /// call and not before — it is the last thing dropped here, after the seat
    /// is locked.
    #[must_use]
    pub fn before_sleeping<N>(self, seat: Seat<N>, summoning: &mut Summoning) -> Seat<N> {
        let locked = seat.locked(summoning);
        drop(self.held);
        locked
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{Holds, TheMachine, anna, in_english};

    /// **A sleep somebody else started finds the seat locked before it is let
    /// go**, and the delay is gone afterwards.
    #[test]
    fn a_sleep_started_elsewhere_waits_for_the_lock() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let lid = TheLidIsOurs::taken(&mut logind, &strings).unwrap();
        let until = UntilLocked::taken(&mut logind, &strings).unwrap();
        assert_eq!(holds.alive(), 2);
        assert_eq!(
            holds.taken(),
            [Inhibit::TheLid, Inhibit::UntilLocked],
            "the lid is blocked and the sleep only delayed"
        );

        let seat = until.before_sleeping(
            alo_locking::Seat::<String>::opened(anna()),
            &mut Summoning::closed(),
        );
        assert!(seat.is_locked());
        assert_eq!(holds.alive(), 1, "the delay let go, the lid did not");
        drop(lid);
        assert_eq!(holds.alive(), 0);
    }

    /// **Neither hold is pretended** when the machine will not take it.
    #[test]
    fn a_refused_hold_is_not_pretended() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::refusing(&holds);
        assert!(TheLidIsOurs::taken(&mut logind, &strings).is_err());
        assert!(UntilLocked::taken(&mut logind, &strings).is_err());
        assert_eq!(holds.alive(), 0);
    }
}
