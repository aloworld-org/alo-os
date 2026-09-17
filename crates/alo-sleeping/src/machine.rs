//! The half that is really a machine: `systemd-logind` over the system bus.
//!
//! Two methods on one interface, and every argument is decided in
//! `crate::logind`: `Inhibit`, which answers with a descriptor a hold lives as
//! long as, and `Suspend`. Everything *about* them — whether, in which order,
//! for whom — is decided elsewhere in this crate, on any host, in the gate.
//!
//! # A hold is the descriptor, and nothing else
//!
//! `logind` keeps an inhibitor lock for as long as somebody holds the file
//! descriptor it answered `Inhibit` with. [`TheMachinesLogind::hold`] hands that
//! descriptor back as the hold itself, so the list in `crate::holding` owning a
//! value *is* the machine holding a lock, and dropping one closes the other.
//!
//! # `Suspend` is asked non-interactively
//!
//! `interactive` is `false`: this process is not a place a password prompt can
//! appear, and a sleep the person asked for should not stop to ask them for a
//! password. A machine whose policy refuses an unprompted suspend answers
//! [`NotSlept`], and the seat stays locked (`crate::going`).

use zbus::blocking::Connection;
use zbus::zvariant::OwnedFd;

use crate::logind::{Inhibit, LockedFirst, Logind, NotHeld, NotSlept};

/// The service holds and sleeps are asked of.
const LOGIND: &str = "org.freedesktop.login1";

/// The object its manager is at.
const THE_MANAGER: &str = "/org/freedesktop/login1";

/// The interface both methods are on.
const THE_INTERFACE: &str = "org.freedesktop.login1.Manager";

/// Take a hold.
const INHIBIT: &str = "Inhibit";

/// Put the machine to sleep.
const SUSPEND: &str = "Suspend";

/// Who `logind` lists as holding. The product's name, never translated.
const US: &str = "alo OS";

/// `systemd-logind`, on this machine's system bus.
#[derive(Debug)]
pub struct TheMachinesLogind {
    /// The system bus.
    bus: Connection,
}

impl TheMachinesLogind {
    /// Connect to the system bus.
    ///
    /// # Errors
    /// [`NotHeld`] when there is no system bus to connect to — nothing can be
    /// held, and nothing slept, without one.
    pub fn on_the_system_bus() -> Result<Self, NotHeld> {
        Connection::system()
            .map(|bus| Self { bus })
            .map_err(|why| NotHeld {
                machine: why.to_string(),
            })
    }
}

impl Logind for TheMachinesLogind {
    type Held = OwnedFd;

    fn hold(&mut self, inhibit: Inhibit, why: &alo_strings::Said) -> Result<OwnedFd, NotHeld> {
        let reply = self
            .bus
            .call_method(
                Some(LOGIND),
                THE_MANAGER,
                Some(THE_INTERFACE),
                INHIBIT,
                &(inhibit.what(), US, why.text(), inhibit.mode()),
            )
            .map_err(|why| NotHeld {
                machine: why.to_string(),
            })?;
        reply.body().deserialize().map_err(|why| NotHeld {
            machine: why.to_string(),
        })
    }

    fn sleep(&mut self, _locked_first: LockedFirst) -> Result<(), NotSlept> {
        self.bus
            .call_method(
                Some(LOGIND),
                THE_MANAGER,
                Some(THE_INTERFACE),
                SUSPEND,
                &(false,),
            )
            .map(|_| ())
            .map_err(|why| NotSlept {
                machine: why.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Both methods, the interface and the object are written once.** A call
    /// to put a machine to sleep assembled in two places is one where a typo is
    /// a call to something else.
    #[test]
    fn there_are_two_methods_this_crate_can_call() {
        assert_eq!(LOGIND, "org.freedesktop.login1");
        assert_eq!(THE_MANAGER, "/org/freedesktop/login1");
        assert_eq!(THE_INTERFACE, "org.freedesktop.login1.Manager");
        assert_eq!([INHIBIT, SUSPEND], ["Inhibit", "Suspend"]);
    }
}
