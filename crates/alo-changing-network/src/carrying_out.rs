//! From an approval to a change: the one road, for an agent's proposal and for a
//! person's own choice.
//!
//! **An agent's proposal**, once a person approved its sentence and the turn
//! redeemed that approval ([`carry_out_approved`]):
//!
//! 1. the authority must be one of the three verbs, **from an approval**
//!    ([`crate::approved`]);
//! 2. the network manager is asked what is true now, and **what the change does
//!    to this conversation must still be what the person approved** — a machine
//!    that moved from a cable to Wi-Fi since would otherwise carry out a sentence
//!    that is no longer true ([`crate::would`]);
//! 3. the network the person approved by name is found, exactly once
//!    ([`crate::chosen`]);
//! 4. the broker's approving key is read where the broker handed it over, and a
//!    token is issued for exactly that verb, that network and that approval;
//! 5. the broker is asked, once, and its answer is what the person reads.
//!
//! **A person's own choice in Settings** ([`carry_out_by_hand`]) starts at step
//! 4, with the verb [`crate::Listed::picked`] made and the approval
//! [`alo_broker::BY_HAND`]. **The proxy** ([`set_proxy_by_hand`]) is handed over
//! where the broker looks for it, and asked for by the digest of exactly what
//! was handed over.
//!
//! Nothing here changes the network. The broker does, after it has written the
//! request down; everything before it only decides *which* verb to ask for, and
//! a refusal at any step asks for none.

use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_broker::asking::{NotAsked, ask};
use alo_broker::handing_over::the_key_handed_over;
use alo_broker::{BY_HAND, Identity, Request, SystemVerb, THE_DOOR, THE_KEY};
use alo_capability::Authorised;
use alo_networks::Networks;
use alo_networks::proxy_file::{THE_WANTED_PROXY, wanted};
use alo_proxy::TheProxy;

use crate::choosing::chosen;
use crate::cutting::{Answered, would};
use crate::refusing::NotChanged;
use crate::verbs::approved;

/// The user the broker runs as, who must own the key it hands over.
const ROOT: u32 = 0;

/// The mode a proxy is handed over with: the person's to read and write, and
/// root reads anything.
const HANDED_OVER: u32 = 0o600;

/// Where the broker is, and whom its key must come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheBroker {
    /// Its door.
    door: PathBuf,
    /// Where it hands its approving key over.
    key: PathBuf,
    /// Where a person hands a proxy over.
    wanted_proxy: PathBuf,
    /// The user that must own that key.
    from: u32,
}

impl TheBroker {
    /// The broker on this machine: its door, key and handed-over proxy where it
    /// puts them, the key believed only from root.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self {
            door: PathBuf::from(THE_DOOR),
            key: PathBuf::from(THE_KEY),
            wanted_proxy: PathBuf::from(THE_WANTED_PROXY),
            from: ROOT,
        }
    }

    /// A broker whose door, key and handed-over proxy are somewhere else,
    /// handed over by `from` — how a test reaches a broker of its own.
    #[must_use]
    pub fn at(door: &Path, key: &Path, wanted_proxy: &Path, from: u32) -> Self {
        Self {
            door: door.to_owned(),
            key: key.to_owned(),
            wanted_proxy: wanted_proxy.to_owned(),
            from,
        }
    }

    /// Ask for this verb, under this approval; a refusal to carry it out is
    /// said as `not_carried`.
    fn asked(
        &self,
        verb: SystemVerb,
        approval: u64,
        not_carried: NotChanged,
        now: SystemTime,
    ) -> Result<(), NotChanged> {
        let key = the_key_handed_over(&self.key, self.from)
            .map_err(|_| NotChanged::NothingMakesChanges)?;
        let request = Request::of(verb, key.issue(&verb, approval, now));
        match ask(&self.door, &request) {
            Ok(answer) => NotChanged::from_the_brokers(answer, not_carried),
            // Nothing was reached, so nothing was changed.
            Err(NotAsked::NoDoor(_)) => Err(NotChanged::NothingMakesChanges),
            // The request was sent and no answer came back: the change may or
            // may not have been made, so a person is sent to look rather than
            // told nothing changed.
            Err(NotAsked::NoAnswer) => Err(not_carried),
        }
    }
}

/// Carry out the change a person approved an agent's proposal for, in a
/// conversation answered so.
///
/// Takes the authority by value: one approval is one execution, and an
/// authority that was used cannot be used again.
///
/// # Errors
/// [`NotChanged`], and nothing was changed unless it is
/// [`NotChanged::CouldNotFinish`].
pub fn carry_out_approved(
    authorised: Authorised,
    networks: &impl Networks,
    answered: Answered,
    broker: &TheBroker,
    now: SystemTime,
) -> Result<SystemVerb, NotChanged> {
    let (wanted, approval) = approved(&authorised)?;
    let reported = networks
        .now()
        .map_err(|_| NotChanged::NetworkNotAnswering)?;
    if would(wanted.change(), &reported, answered) != wanted.says() {
        return Err(NotChanged::ConnectionChangedSinceApproval);
    }
    let verb = chosen(wanted.change(), &reported)?;
    broker.asked(verb, approval, NotChanged::CouldNotFinish, now)?;
    Ok(verb)
}

/// Carry out a person's own change in Settings: a network they picked, or Wi-Fi
/// turned on or off.
///
/// # Errors
/// [`NotChanged`], and nothing was changed unless it is
/// [`NotChanged::CouldNotFinish`].
pub fn carry_out_by_hand(
    verb: SystemVerb,
    broker: &TheBroker,
    now: SystemTime,
) -> Result<(), NotChanged> {
    broker.asked(verb, BY_HAND, NotChanged::CouldNotFinish, now)
}

/// Set the machine's proxy to the one a person chose in Settings.
///
/// The choice is handed over — written whole beside where the broker looks and
/// renamed into place — and the broker is asked for `network.set-proxy` with the
/// digest of exactly those bytes.
///
/// # Errors
/// [`NotChanged::ProxyNotSet`] when it could not be handed over or set, and the
/// broker's other refusals.
pub fn set_proxy_by_hand(
    proxy: &TheProxy,
    broker: &TheBroker,
    now: SystemTime,
) -> Result<(), NotChanged> {
    let bytes = wanted(proxy).map_err(|_| NotChanged::ProxyNotSet)?;
    let beside = broker.wanted_proxy.with_extension("handing-over");
    drop(fs::remove_file(&beside));
    let handed_over = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(HANDED_OVER)
        .open(&beside)
        .and_then(|mut file| {
            file.write_all(&bytes)?;
            file.sync_all()
        })
        .and_then(|()| fs::rename(&beside, &broker.wanted_proxy));
    if handed_over.is_err() {
        drop(fs::remove_file(&beside));
        return Err(NotChanged::ProxyNotSet);
    }
    let verb = SystemVerb::SetProxy(Identity::of_what_was_reported(&bytes));
    broker.asked(verb, BY_HAND, NotChanged::ProxyNotSet, now)
}
