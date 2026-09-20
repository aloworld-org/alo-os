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
//! [`alo_broker::BY_HAND`]. **The proxy, and the password it signs in with**
//! ([`set_proxy_by_hand`]) are handed over where the broker looks for them, and
//! asked for by the digest of exactly what was handed over — one act and one
//! approval, which is ADR 0060 §1.
//!
//! Nothing here changes the network. The broker does, after it has written the
//! request down; everything before it only decides *which* verb to ask for, and
//! a refusal at any step asks for none.
//!
//! # A managed machine is refused here, in the sentence that names who set it
//!
//! ADR 0060 §5. The broker refuses a person's proxy over an organisation's and
//! always did — but it refuses in English into its own log, and everything it
//! refuses reaches a person as *the proxy was not set*. So the machine's proxy
//! file is read **before** anything is handed over, and a person on a machine
//! an organisation manages reads `alo_proxy::NotChanged::AnOrganisationSetIt` —
//! the sentence ADR 0016 already has, saying who can change it. The password
//! they typed never leaves the process they typed it into.
//!
//! This is a better sentence in front of a check that was already there, never
//! a check moved out of the privileged component.

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
use alo_networks::proxy_file::{
    THE_MACHINES_PROXY, THE_WANTED_PROXY, kept_on_this_machine, wanted,
};
use alo_networks::proxy_password::{THE_WANTED_PASSWORD, handed_over_together};
use alo_proxy::{Password, SetBy, TheProxy, handed_over};

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
    /// Where a person hands the password that proxy signs in with over.
    wanted_password: PathBuf,
    /// The machine's proxy file, read to say whose the proxy is before a
    /// person's own change is handed over at all.
    machines_proxy: PathBuf,
    /// The user that must own that key.
    from: u32,
}

impl TheBroker {
    /// The broker on this machine: its door, key and handed-over files where it
    /// puts them, the key believed only from root.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self {
            door: PathBuf::from(THE_DOOR),
            key: PathBuf::from(THE_KEY),
            wanted_proxy: PathBuf::from(THE_WANTED_PROXY),
            wanted_password: PathBuf::from(THE_WANTED_PASSWORD),
            machines_proxy: PathBuf::from(THE_MACHINES_PROXY),
            from: ROOT,
        }
    }

    /// A broker whose door, key and files are somewhere else, handed over by
    /// `from` — how a test reaches a broker of its own.
    ///
    /// The password is handed over beside the proxy, as it is on a machine, so
    /// there is one path here rather than two that could disagree.
    #[must_use]
    pub fn at(
        door: &Path,
        key: &Path,
        wanted_proxy: &Path,
        machines_proxy: &Path,
        from: u32,
    ) -> Self {
        Self {
            door: door.to_owned(),
            key: key.to_owned(),
            wanted_proxy: wanted_proxy.to_owned(),
            wanted_password: beside(wanted_proxy),
            machines_proxy: machines_proxy.to_owned(),
            from,
        }
    }

    /// Whether the proxy on this machine is the person's to change.
    ///
    /// The machine's proxy file, read as it is — `0644`, everybody's to read.
    /// A machine with no file has no proxy and it is the person's to set, which
    /// is the same answer `alo_brokerd::Proxy` gives; a file this machine did
    /// not write is not read as *no proxy*, because on a company network that
    /// would be a person quietly setting their own over an organisation's.
    ///
    /// # Errors
    /// [`NotChanged::AnOrganisationSetTheProxy`], which says who can change it.
    fn whose_it_is(&self) -> Result<(), NotChanged> {
        let bytes = match fs::read(&self.machines_proxy) {
            Ok(bytes) => bytes,
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err(NotChanged::ProxyNotSet),
        };
        let kept = kept_on_this_machine(&bytes).map_err(|_| NotChanged::ProxyNotSet)?;
        match kept.set_by() {
            SetBy::AnOrganisation => Err(NotChanged::AnOrganisationSetTheProxy),
            SetBy::ThisPerson => Ok(()),
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

/// Set the machine's proxy — and the password it signs in with — to what a
/// person chose in Settings.
///
/// The choice is handed over, and the password beside it: each written whole
/// where the broker looks and renamed into place. The broker is then asked for
/// `network.set-proxy` with the digest of exactly those bytes, both files
/// together (ADR 0060 §1), so one approval covers the proxy and the credential
/// and a machine is never left with a proxy it cannot sign in to.
///
/// `password` is [`None`] for a proxy that asks for no name, and for a person
/// changing a proxy whose password this machine already has — a password is
/// never read back out, so *replace it* is giving a new one, not editing the
/// old (ADR 0060 §6). Where one is given, the proxy must sign in under
/// [`alo_proxy::THE_PERSONS_PROXY_PASSWORD`], which
/// [`alo_proxy::WhereThePasswordIs::on_this_machine`] is; the broker refuses
/// anything else.
///
/// # Errors
/// [`NotChanged::AnOrganisationSetTheProxy`] on a machine an organisation
/// manages — **with nothing handed over at all**, so a password typed on such a
/// machine never leaves this process. [`NotChanged::ProxyNotSet`] when it could
/// not be handed over or set, and the broker's other refusals.
pub fn set_proxy_by_hand(
    proxy: &TheProxy,
    password: Option<&Password>,
    broker: &TheBroker,
    now: SystemTime,
) -> Result<(), NotChanged> {
    broker.whose_it_is()?;
    let bytes = wanted(proxy).map_err(|_| NotChanged::ProxyNotSet)?;
    let handed = match password {
        Some(password) => handed_over(password).map_err(|_| NotChanged::ProxyNotSet)?,
        None => Vec::new(),
    };
    // The password first: a proxy handed over with no password beside it is a
    // different approval from the same proxy with one, so the two files are in
    // place together before the broker is asked about either.
    hand_over(&broker.wanted_password, &handed)?;
    hand_over(&broker.wanted_proxy, &bytes)?;
    let verb = SystemVerb::SetProxy(Identity::of_what_was_reported(&handed_over_together(
        &bytes, &handed,
    )));
    broker.asked(verb, BY_HAND, NotChanged::ProxyNotSet, now)
}

/// Write these bytes where the broker looks, whole: beside, then renamed over.
///
/// Empty bytes hand nothing over and remove whatever was there, which is how a
/// proxy set without a password is asked for under exactly its own digest —
/// the sentence `docs/contracts/machine-proxy-file.md` publishes.
fn hand_over(at: &Path, bytes: &[u8]) -> Result<(), NotChanged> {
    if bytes.is_empty() {
        return match fs::remove_file(at) {
            Ok(()) => Ok(()),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(NotChanged::ProxyNotSet),
        };
    }
    let beside = at.with_extension("handing-over");
    drop(fs::remove_file(&beside));
    let handed_over = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(HANDED_OVER)
        .open(&beside)
        .and_then(|mut file| {
            file.write_all(bytes)?;
            file.sync_all()
        })
        .and_then(|()| fs::rename(&beside, at));
    if handed_over.is_err() {
        drop(fs::remove_file(&beside));
        return Err(NotChanged::ProxyNotSet);
    }
    Ok(())
}

/// The password beside a handed-over proxy: the name
/// [`alo_networks::proxy_password::THE_WANTED_PASSWORD`] has, in whatever
/// folder the proxy is handed over in.
fn beside(wanted_proxy: &Path) -> PathBuf {
    let named = Path::new(THE_WANTED_PASSWORD)
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("proxy-password"));
    wanted_proxy.with_file_name(named)
}
