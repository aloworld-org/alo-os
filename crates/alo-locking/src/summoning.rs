//! The agent's key, pressed at a locked machine.
//!
//! **The agent overlay cannot be summoned while locked.** The agent acts for
//! the signed-in person, under their grants, with their context offered at the
//! moment they invoke it (ADR 0001). Somebody at a locked machine is not known
//! to be that person, so a key that brought the agent up there would be an
//! agent taking instructions — and offering the last turn's answer — to
//! whoever happens to be standing at the desk.
//!
//! So the key is answered here before `alo_overlay::Summoning` is reached at
//! all. While locked, the compositor is asked **nothing**: not asked and then
//! refused, not asked for something smaller — nothing, which the test below
//! counts. The answer is [`NotWhileLocked`], whose sentence is the one the
//! lock screen already shows.

use alo_overlay::{Compositor, Pressed, Summoning};

use crate::refusing::NotWhileLocked;
use crate::seat::Seat;

impl<N> Seat<N> {
    /// The key that opens the agent was pressed.
    ///
    /// On an open seat this is `alo_overlay::Summoning::press`, unchanged.
    ///
    /// # Errors
    /// [`NotWhileLocked`] on a locked seat, with the overlay's summoning and
    /// the compositor untouched.
    pub fn press_the_agents_key(
        &self,
        summoning: &mut Summoning,
        compositor: Option<&mut dyn Compositor>,
    ) -> Result<Pressed, NotWhileLocked> {
        match self {
            Self::Locked(_) => Err(NotWhileLocked),
            Self::Open(_) => Ok(summoning.press(compositor)),
        }
    }
}

#[cfg(test)]
mod tests {
    use alo_overlay::{SurfaceRefused, SurfaceRequest};

    use super::*;
    use crate::testing::{ANNAS, anna, the_machine};
    use crate::unlocking::Unlocking;

    /// A compositor that counts how often it was asked for the overlay.
    #[derive(Default)]
    struct Counting {
        /// How many requests arrived.
        asked: usize,
    }

    impl Compositor for Counting {
        fn honour(&mut self, _asked: SurfaceRequest) -> Result<(), SurfaceRefused> {
            self.asked = self.asked.saturating_add(1);
            Ok(())
        }
    }

    /// **The agent overlay cannot be summoned while locked**: the key is
    /// refused with the lock screen's own sentence, and the compositor is
    /// asked nothing however often it is pressed.
    #[test]
    fn the_agents_key_asks_nothing_while_locked() {
        let mut summoning = Summoning::closed();
        let mut compositor = Counting::default();
        let seat = Seat::<String>::opened(anna()).locked(&mut summoning);

        for _ in 0..3 {
            assert_eq!(
                seat.press_the_agents_key(&mut summoning, Some(&mut compositor)),
                Err(NotWhileLocked)
            );
        }
        assert_eq!(compositor.asked, 0);
        assert!(!summoning.is_open());
    }

    /// **Unlocked, the same key summons the agent once**, which is what makes
    /// the refusal a rule about the lock rather than a broken key.
    #[test]
    fn the_agents_key_works_again_once_the_person_is_back() {
        let mut summoning = Summoning::closed();
        let mut compositor = Counting::default();
        let seat = Seat::<String>::opened(anna()).locked(&mut summoning);

        let Unlocking::Unlocked { seat, .. } = seat.unlocks(the_machine(), "anna", ANNAS) else {
            unreachable!("the right password did not unlock")
        };
        assert_eq!(
            seat.press_the_agents_key(&mut summoning, Some(&mut compositor)),
            Ok(Pressed::Summoned)
        );
        assert_eq!(compositor.asked, 1);
    }
}
