//! What this crate asks of the machine, two methods wide.
//!
//! The mechanism is `systemd-logind`'s, rented and configured (ADR 0011): a
//! **hold** is one of its inhibitor locks, alive for as long as the value that
//! stands for it, and a **sleep** is its `Suspend`. `crate::machine` is the
//! implementation that ships. This is a trait for the reason `alo-sessiond`'s
//! is: every decision about *whether* and *in which order* is arithmetic on
//! values, and a decision that could only be shown working on a machine with a
//! running `logind` is one the gate would never run.
//!
//! # Three kinds of hold, and no fourth
//!
//! [`Inhibit`] is closed, and each is a hold alo OS has a reason for:
//!
//! - [`Inhibit::TheLid`] — held for the whole session, so closing the lid comes
//!   to this crate's [`crate::Lid`] rule rather than to `logind`'s own
//!   configuration, which by default ignores a lid closed with a display
//!   attached and so would decide the person's question for them;
//! - [`Inhibit::UntilLocked`] — a *delay*, held for the whole session, so a
//!   sleep somebody else starts (a power key, a terminal) waits the moment it
//!   takes to lock the seat first ([`crate::UntilLocked`]);
//! - [`Inhibit::Idle`] — one for each [`crate::Keeper`], so the machine's own
//!   list of what holds it awake names the same things this crate names.
//!
//! There is no hold that blocks a sleep the person asked for. That is the
//! decision in `crate::deciding`, and this list is where it cannot be undone.
//!
//! # Sleeping takes proof that the seat was locked first
//!
//! [`Logind::sleep`] takes a [`LockedFirst`], which only `crate::going` can make
//! and only out of a seat that is locked. So *a session is locked before the
//! machine sleeps* is not an order somebody has to remember: there is no call
//! to this method that the compiler will build without it.

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// What a hold keeps from happening.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inhibit {
    /// Closing the lid is this crate's to decide. A block on the lid switch.
    TheLid,
    /// A sleep started anywhere waits until the seat is locked. A delay on
    /// sleep.
    UntilLocked,
    /// The machine does not go to sleep on its own while this is held. A block
    /// on idleness.
    Idle,
}

impl Inhibit {
    /// What `logind` calls what this holds off.
    #[must_use]
    pub const fn what(self) -> &'static str {
        match self {
            Self::TheLid => "handle-lid-switch",
            Self::UntilLocked => "sleep",
            Self::Idle => "idle",
        }
    }

    /// Whether `logind` blocks or only delays.
    #[must_use]
    pub const fn mode(self) -> &'static str {
        match self {
            Self::TheLid | Self::Idle => "block",
            Self::UntilLocked => "delay",
        }
    }

    /// The sentence this crate writes beside alo OS's own two holds, or
    /// [`None`] for [`Inhibit::Idle`], whose sentence is the keeper's.
    #[must_use]
    pub const fn word(self) -> Option<Word> {
        match self {
            Self::TheLid => Some(words::HOLDING_THE_LID),
            Self::UntilLocked => Some(words::HOLDING_UNTIL_LOCKED),
            Self::Idle => None,
        }
    }
}

/// The seat was locked before this was made.
///
/// Made in one place, `crate::going`, out of a seat that is locked; not
/// `Clone`, and with no public constructor, so it cannot be kept from one
/// sleep and spent on the next:
///
/// ```compile_fail
/// let first = alo_sleeping::LockedFirst(());
/// ```
///
/// That fails with **E0603, tuple struct constructor `LockedFirst` is
/// private**, and not on a typo.
#[derive(Debug)]
pub struct LockedFirst(pub(crate) ());

/// What can hold this machine awake and put it to sleep.
pub trait Logind {
    /// A hold. It lasts as long as this value: dropping it lets go, which is
    /// why there is no `release`.
    type Held;

    /// Take a hold of this kind, described by `why` in the person's language.
    ///
    /// # Errors
    /// [`NotHeld`] when the machine would not take it — no bus to ask, or a
    /// refusal. Nothing is held.
    fn hold(&mut self, inhibit: Inhibit, why: &Said) -> Result<Self::Held, NotHeld>;

    /// Put the machine to sleep, which takes proof the seat was locked first.
    ///
    /// # Errors
    /// [`NotSlept`] when the machine did not go to sleep.
    fn sleep(&mut self, locked_first: LockedFirst) -> Result<(), NotSlept>;
}

/// The machine would not take a hold.
///
/// Carries what the machine said, for whoever administers it; a person reads
/// [`NotHeld::said`], which names nothing rented.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotHeld {
    /// What the machine said.
    pub machine: String,
}

impl NotHeld {
    /// The string this crate declares for it.
    #[must_use]
    pub const fn word(&self) -> Word {
        words::THE_MACHINE_WOULD_NOT_HOLD
    }

    /// What a person is told.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// The machine did not go to sleep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotSlept {
    /// What the machine said.
    pub machine: String,
}

impl NotSlept {
    /// The string this crate declares for it.
    #[must_use]
    pub const fn word(&self) -> Word {
        words::DID_NOT_SLEEP
    }

    /// What a person is told: that it did not sleep, and that it is locked.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// **Only the lid and idleness are blocked; a sleep is only ever delayed.**
    /// No hold alo OS takes can stop a sleep somebody asked for.
    #[test]
    fn no_hold_blocks_a_sleep() {
        for inhibit in [Inhibit::TheLid, Inhibit::UntilLocked, Inhibit::Idle] {
            if inhibit.what() == "sleep" {
                assert_eq!(inhibit.mode(), "delay", "{inhibit:?}");
            }
        }
        assert_eq!(Inhibit::UntilLocked.mode(), "delay");
    }

    /// Both refusals are said, and neither names what was refused by.
    #[test]
    fn both_refusals_are_said_without_naming_the_machinery() {
        let strings = in_english();
        let held = NotHeld {
            machine: "org.freedesktop.DBus.Error.AccessDenied".to_owned(),
        }
        .said(&strings);
        let slept = NotSlept {
            machine: "Sleep verb not supported".to_owned(),
        }
        .said(&strings);
        for said in [held, slept] {
            assert!(!said.is_a_bug(), "{said}");
            assert!(!said.text().contains("DBus"), "{said}");
            assert!(!said.text().contains("verb"), "{said}");
        }
    }
}
