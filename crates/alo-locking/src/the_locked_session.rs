//! The door an unlock knocks at, which is the locked session itself.
//!
//! `alo_greeting::Greeting` authenticates and then knocks, and the door it
//! knocks at when somebody signs in is the opener, which starts a session. An
//! unlock must not start one — the session never ended — and the opener would
//! rightly refuse, because somebody is signed in already. So the unlock is the
//! same greeting, knocking at this: a door that answers *open* for the locked
//! session's own number, because it is open, and for any other number answers
//! with the opener's own refusal.
//!
//! It reaches nothing. No socket, no bus, no opener, no `logind`: the one
//! thing it knows is a number, and a knock is only ever a number
//! (`alo_sessiond::Knock`). What checks the password is the greeting's, not
//! this, which is why this door is not a second, weaker road — nothing arrives
//! here except after `alo-accounts` has said yes.

use alo_greeting::{Knocking, NotAnswered};
use alo_sessiond::{ALREADY_SIGNED_IN, Answered, Knock};

/// The locked session, as a door.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheLockedSession {
    /// The number the locked session runs as.
    uid: u32,
}

impl TheLockedSession {
    /// The door for the session running as this number.
    pub(crate) const fn running_as(uid: u32) -> Self {
        Self { uid }
    }
}

impl Knocking for TheLockedSession {
    fn knock(&self, knock: Knock) -> Result<Answered, NotAnswered> {
        if knock.person() == self.uid {
            Ok(Answered::Opened)
        } else {
            Ok(Answered::Refused(ALREADY_SIGNED_IN.key()))
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

    /// **The locked session's own number is let back in; any other number is
    /// refused** with the opener's own sentence, because to anybody else this
    /// machine is signed in already.
    #[test]
    fn only_the_locked_sessions_own_number_is_let_back_in() {
        let door = TheLockedSession::running_as(1000);
        assert_eq!(
            door.knock(Knock::on_behalf_of(1000)).unwrap(),
            Answered::Opened
        );
        assert_eq!(
            door.knock(Knock::on_behalf_of(1001)).unwrap(),
            Answered::Refused(ALREADY_SIGNED_IN.key())
        );
    }
}
