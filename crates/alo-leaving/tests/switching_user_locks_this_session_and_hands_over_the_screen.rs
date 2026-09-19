//! *Switch user*, walked: the session locked first, the screen handed to the
//! sign-in, and nothing of the first person's ended.
//!
//! The order is the property, and it is the one that is wrong on a system that
//! draws the greeter and locks afterwards. What this test can see is the
//! afterwards: the seat that comes back is locked, it is the same session, and
//! the person who arrives next has to sign in through the same composition they
//! would at a cold machine.

#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_accounts::{Accounts, Session};
use alo_greeting::Standing;
use alo_leaving::Switching;
use alo_locking::{Seat, Unlocking};
use alo_overlay::{Compositor, Pressed, Summoning, SurfaceRefused, SurfaceRequest};

/// The number Anna runs as.
const ANNAS_NUMBER: u32 = 1000;

/// The number Ben runs as.
const BENS_NUMBER: u32 = 1001;

/// Anna's password, and it is right.
const ANNAS: &str = "a password nobody else knows";

/// Ben's password, on the same machine.
const BENS: &str = "a different password nobody else knows";

/// The machine's accounts: Anna and Ben.
fn the_machine() -> Accounts {
    let mut accounts = Accounts::none().unwrap();
    accounts.created("anna", ANNAS_NUMBER, ANNAS).unwrap();
    accounts.created("ben", BENS_NUMBER, BENS).unwrap();
    accounts
}

/// Anna's session, as a sign-in really opens one.
fn anna() -> Session {
    let signed_in = the_machine().signs_in("anna", ANNAS).unwrap();
    Session::opened(signed_in, ANNAS_NUMBER).unwrap()
}

/// A compositor with room for the agent's overlay.
struct Room;

impl Compositor for Room {
    fn honour(&mut self, _asked: SurfaceRequest) -> Result<(), SurfaceRefused> {
        Ok(())
    }
}

/// **Switching user locks this session and hands the screen to the sign-in** —
/// and it is still Anna's session afterwards, not a signed-out one.
#[test]
fn switching_user_locks_this_session_and_hands_the_screen_to_the_sign_in() {
    let seat = Seat::<String>::opened(anna());
    let Switching::HandedOver {
        seat: after,
        standing,
    } = alo_leaving::switching::asked(seat.clone(), &mut Summoning::closed(), &the_machine())
    else {
        panic!("an open seat was not handed over")
    };
    assert!(after.is_locked(), "the screen was handed over unlocked");
    assert_eq!(
        after.session(),
        seat.session(),
        "switching user signed somebody out"
    );
    assert_eq!(standing, Standing::SignIn);
}

/// **Anna's own session is still hers to come back to.** The seat a switch left
/// locked unlocks with Anna's password and nobody else's, which is task 1's rule
/// and not a second one this crate invented.
#[test]
fn the_first_person_comes_back_to_their_own_session() {
    let Switching::HandedOver { seat, .. } = alo_leaving::switching::asked(
        Seat::<String>::opened(anna()),
        &mut Summoning::closed(),
        &the_machine(),
    ) else {
        panic!("an open seat was not handed over")
    };

    let Unlocking::StillLocked { seat, .. } = seat.unlocks(the_machine(), "ben", BENS) else {
        panic!("ben unlocked anna's session")
    };
    let Unlocking::Unlocked { seat, .. } = seat.unlocks(the_machine(), "anna", ANNAS) else {
        panic!("anna could not get back into her own session")
    };
    assert!(!seat.is_locked());
    assert_eq!(seat.session(), &anna());
}

/// **What is handed over is a screen that asks for a name and a password**, and
/// never a *continue as somebody* — on a machine with accounts on it there is
/// only one thing a greeter can stand at, and it is worked out from the accounts
/// rather than named by this crate.
#[test]
fn what_is_handed_over_asks_for_a_name_and_a_password() {
    let Switching::HandedOver { standing, .. } = alo_leaving::switching::asked(
        Seat::<String>::opened(anna()),
        &mut Summoning::closed(),
        &the_machine(),
    ) else {
        panic!("an open seat was not handed over")
    };
    assert_eq!(standing, Standing::SignIn, "the greeter asks for a name");
    assert_eq!(standing, Standing::of(&the_machine()));
    assert_eq!(
        standing.said(&alo_strings::Strings::of(alo_strings::Vocabulary::empty())),
        None,
        "a sign-in screen says nothing of its own"
    );
}

/// **The agent's overlay is closed on the way out.** The next person at the desk
/// does not meet the last person's agent, open where they left it.
#[test]
fn the_agents_overlay_is_closed_on_the_way_out() {
    let mut summoning = Summoning::closed();
    assert_eq!(summoning.press(Some(&mut Room)), Pressed::Summoned);
    assert!(summoning.is_open());

    let switched = alo_leaving::switching::asked(
        Seat::<String>::opened(anna()),
        &mut summoning,
        &the_machine(),
    );
    assert!(!summoning.is_open(), "the agent was left up on the screen");
    assert!(switched.seat().is_locked());
}

/// **Nothing of the first person's session ends.** Switching user is a lock:
/// their applications, downloads and approved turns all keep running, which is
/// `alo_locking::Running`'s answer and is asserted here because it is the
/// clause a person notices when it is wrong.
#[test]
fn nothing_of_the_first_persons_session_ends() {
    for running in alo_locking::EVERYTHING_RUNNING {
        assert_eq!(
            running.while_locked(),
            alo_locking::WhileLocked::KeepsRunning,
            "{running:?}"
        );
    }
    let Switching::HandedOver { seat, .. } = alo_leaving::switching::asked(
        Seat::<String>::opened(anna()),
        &mut Summoning::closed(),
        &the_machine(),
    ) else {
        panic!("an open seat was not handed over")
    };
    assert_eq!(seat.session(), &anna(), "the session is the same one");
}
