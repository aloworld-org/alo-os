//! From an approval to a change: the one road, for an agent's proposal and for
//! a person's own choice.
//!
//! **An agent's proposal**, once a person approved its sentence and the turn
//! redeemed that approval ([`carry_out_approved`]):
//!
//! 1. the authority must be one of the three verbs, **from an approval** —
//!    `alo_capability::Authorised` for a change cannot exist without one, and
//!    this asks again ([`crate::approved`]);
//! 2. the printer the person approved by name is found among this machine's
//!    printers, exactly once ([`crate::chosen`]);
//! 3. the broker's approving key is read where the broker handed it over, and a
//!    token is issued for exactly that verb, that printer and that approval;
//! 4. the broker is asked, once, and its answer is what the person reads.
//!
//! **A person's own choice in Settings** ([`carry_out_by_hand`]) starts at step
//! 3, with the verb [`crate::by_hand::picked`] made and the approval
//! [`alo_broker::BY_HAND`].
//!
//! Nothing here changes a printer. The broker does, after it has written the
//! request down; everything before it only decides *which* verb to ask for, and
//! a refusal at any step asks for none.

use std::path::PathBuf;
use std::time::SystemTime;

use alo_broker::asking::{NotAsked, ask};
use alo_broker::handing_over::the_key_handed_over;
use alo_broker::{BY_HAND, Request, SystemVerb, THE_DOOR, THE_KEY};
use alo_capability::Authorised;
use alo_printing::Called;

use crate::choosing::{ThisMachinesPrinters, chosen};
use crate::refusing::NotChanged;
use crate::verbs::approved;

/// The user the broker runs as, who must own the key it hands over.
const ROOT: u32 = 0;

/// Where the broker is, and whom its key must come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheBroker {
    /// Its door.
    door: PathBuf,
    /// Where it hands its approving key over.
    key: PathBuf,
    /// The user that must own that key.
    from: u32,
}

impl TheBroker {
    /// The broker on this machine: its door and key where it puts them, the key
    /// believed only from root.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self {
            door: PathBuf::from(THE_DOOR),
            key: PathBuf::from(THE_KEY),
            from: ROOT,
        }
    }

    /// A broker whose door and key are somewhere else, handed over by `from` —
    /// how a test reaches a broker of its own.
    #[must_use]
    pub fn at(door: impl Into<PathBuf>, key: impl Into<PathBuf>, from: u32) -> Self {
        Self {
            door: door.into(),
            key: key.into(),
            from,
        }
    }

    /// Ask for this verb, under this approval, about the printer called this.
    fn asked(
        &self,
        verb: SystemVerb,
        approval: u64,
        called: &Called,
        now: SystemTime,
    ) -> Result<(), NotChanged> {
        let key = the_key_handed_over(&self.key, self.from)
            .map_err(|_| NotChanged::NothingMakesChanges)?;
        let request = Request::of(verb, key.issue(&verb, approval, now));
        match ask(&self.door, &request) {
            Ok(answer) => NotChanged::from_the_brokers(answer, called),
            // Nothing was reached, so nothing was changed.
            Err(NotAsked::NoDoor(_)) => Err(NotChanged::NothingMakesChanges),
            // The request was sent and no answer came back: the change may or
            // may not have been made, so a person is sent to look rather than
            // told nothing changed.
            Err(NotAsked::NoAnswer) => Err(NotChanged::CouldNotFinish(called.clone())),
        }
    }
}

/// Carry out the change a person approved an agent's proposal for.
///
/// Takes the authority by value: one approval is one execution, and an
/// authority that was used cannot be used again.
///
/// # Errors
/// [`NotChanged`], and no printer was changed unless it is
/// [`NotChanged::CouldNotFinish`].
pub fn carry_out_approved(
    authorised: Authorised,
    printers: &impl ThisMachinesPrinters,
    broker: &TheBroker,
    now: SystemTime,
) -> Result<SystemVerb, NotChanged> {
    let wanted = approved(&authorised).map_err(|_| NotChanged::NotAPrinterChange)?;
    let Some(approval) = authorised.from_approval() else {
        return Err(NotChanged::NotAPrinterChange);
    };
    let verb = chosen(&wanted, printers)?;
    let called = Called::Named(wanted.called().to_owned());
    broker.asked(verb, approval.as_u64(), &called, now)?;
    Ok(verb)
}

/// Carry out a person's own change in Settings, to the printer called this.
///
/// # Errors
/// [`NotChanged`], and no printer was changed unless it is
/// [`NotChanged::CouldNotFinish`].
pub fn carry_out_by_hand(
    verb: SystemVerb,
    called: &Called,
    broker: &TheBroker,
    now: SystemTime,
) -> Result<(), NotChanged> {
    broker.asked(verb, BY_HAND, called, now)
}
