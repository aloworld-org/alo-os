//! `network.set-proxy`: the machine's proxy — and the password it signs in with
//! — become the ones a person handed over, exactly, or stay as they were.
//!
//! The door takes no text, so a person's choice reaches the broker the only way
//! anything does: as the digest of bytes the broker can find for itself.
//! Settings writes the choice to `alo_networks::proxy_file::THE_WANTED_PROXY`
//! and, where the proxy asks who this machine is, the password to
//! `alo_networks::proxy_password::THE_WANTED_PASSWORD` — both in a folder the
//! broker made for the person's group — and asks for this verb with the digest
//! of the two together. Carrying it out, in order:
//!
//! 1. **Each file is opened without following a link**, and believed only if it
//!    is a plain file, owned by the person the door is for, and no longer than
//!    its own kind of thing is (`crate::handed_over`). Anything else — a link to
//!    a file of root's, a file the agent's login left, something enormous — is
//!    not read.
//! 2. **Their bytes digest to the identity approved**, or nothing is set: bytes
//!    changed after the approval are a different proxy, and a different password
//!    is a different approval too
//!    (`alo_networks::proxy_password::handed_over_together`).
//! 3. **The proxy is rebuilt through `alo-proxy`'s own checks**
//!    (`alo_networks::proxy_file::rechecked`), so the machine's file only ever
//!    holds what that crate would itself have made.
//! 4. **A proxy an organisation set is not replaced by a person's.** The
//!    machine's file says whose its proxy is, and `alo_proxy::Kept` refuses.
//! 5. **The password is written first** (ADR 0060 §1), into the machine's own
//!    credentials, by the tool the base already has, and only under the one name
//!    a person's own machine keeps a proxy password by. A failure here sets
//!    nothing at all — which is what keeps a machine from being left with a
//!    proxy it cannot sign in to.
//! 6. **The machine's file is replaced whole**: written beside itself, synced,
//!    and renamed over, `0644`, so a road out reading it never reads half of one.
//! 7. The handed-over proxy is removed, since it has been used — and **the
//!    handed-over password is removed whichever way the act went**, refusals
//!    included, because bytes left in `/run` are a credential waiting for
//!    somebody to read them.
//!
//! # A password is set only with the proxy that asks for it
//!
//! There is no twelfth verb, and ADR 0060 §1 is the argument: two verbs cannot
//! promise a machine is never left with a proxy set that it cannot sign in to
//! without a transaction across two approvals, and an approval is never a
//! session (ADR 0001 §5). The consequence worth stating here is the useful one
//! — because the machine's proxy file is written only after the credential is,
//! **that file naming a password is the machine's statement that one is set**,
//! and a settings panel needs nothing else to read.

use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::os::unix::fs::{OpenOptionsExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};

use alo_broker::{Identity, NotCarried};
use alo_networks::proxy_file::{LONGEST_FILE, kept_on_this_machine, machines, rechecked};
use alo_networks::proxy_password::handed_over_together;
use alo_proxy::{
    Kept, LONGEST_HANDED_OVER, Scheme, TheMachinesCredentials, TheProxy, WhereThePasswordIs,
    what_was_handed_over,
};

use crate::handed_over;

/// The mode the machine's proxy file is written with: root writes, everybody
/// reads.
const THE_FILES_MODE: u32 = 0o644;

/// Setting the machine's proxy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proxy {
    /// Where a person hands a proxy over.
    wanted: PathBuf,
    /// Where a person hands its password over.
    wanted_password: PathBuf,
    /// The machine's proxy file.
    machines: PathBuf,
    /// Where a machine-wide proxy password is written.
    credentials: TheMachinesCredentials,
    /// The person who may hand one over.
    person: u32,
}

impl Proxy {
    /// Set the proxy handed over at `wanted`, with the password at
    /// `wanted_password`, by `person`, into `machines` and `credentials`.
    ///
    /// Five things rather than three, since ADR 0060 §1 made one act of what
    /// was two: the compiler then asks every caller where the credential goes
    /// rather than letting a broker be built that quietly writes none.
    #[must_use]
    pub fn handed_over(
        wanted: &Path,
        wanted_password: &Path,
        machines: &Path,
        credentials: TheMachinesCredentials,
        person: u32,
    ) -> Self {
        Self {
            wanted: wanted.to_owned(),
            wanted_password: wanted_password.to_owned(),
            machines: machines.to_owned(),
            credentials,
            person,
        }
    }

    /// Carry `network.set-proxy` out for this identity.
    ///
    /// # Errors
    /// [`NotCarried`], and the machine's proxy and its credential are as they
    /// were.
    pub fn set(&self, identity: Identity) -> Result<(), NotCarried> {
        let carried = self.carried(identity);
        // The credential the person typed, gone from `/run` whichever way this
        // went. It has either been written into the machine's own credentials
        // or refused, and in both cases keeping it here would only be a second
        // copy of a password nobody asked for.
        drop(fs::remove_file(&self.wanted_password));
        carried
    }

    /// The act itself, so that removing the handed-over password is one line
    /// above it rather than a line before every `return`.
    fn carried(&self, identity: Identity) -> Result<(), NotCarried> {
        let proxy = handed_over::bytes(&self.wanted, self.person, LONGEST_FILE, "proxy")?;
        let password = self.what_was_handed_over()?;
        if Identity::of_what_was_reported(&handed_over_together(&proxy, &password)) != identity {
            return Err(NotCarried(
                "the proxy handed over is not the one that was approved, so nothing was changed"
                    .to_owned(),
            ));
        }
        let wanted = rechecked(&proxy).map_err(|why| NotCarried(why.to_string()))?;
        let kept = self.now()?.changed_by_the_person(wanted).map_err(|_| {
            NotCarried(
                "the organisation that manages this machine set its proxy, so a person's is not \
                 set over it"
                    .to_owned(),
            )
        })?;
        self.signed_in(kept.proxy(), &password)?;
        self.written(&kept)?;
        drop(fs::remove_file(&self.wanted));
        Ok(())
    }

    /// The password bytes a person handed over, or none at all.
    ///
    /// A proxy that asks for no name is set with nothing handed over here, and
    /// that is the ordinary case: it reads no file and refuses nothing.
    fn what_was_handed_over(&self) -> Result<Vec<u8>, NotCarried> {
        if handed_over::nothing_is_there(&self.wanted_password) {
            return Ok(Vec::new());
        }
        handed_over::bytes(
            &self.wanted_password,
            self.person,
            LONGEST_HANDED_OVER,
            "password",
        )
    }

    /// The credential this proxy signs in with, written before the proxy is.
    ///
    /// Refuses, without writing anything, when a password was handed over for a
    /// proxy that asks for none, when the proxy names anywhere but the one
    /// place a person's own machine keeps a proxy password, or when the machine
    /// could not write it.
    fn signed_in(&self, proxy: &TheProxy, handed: &[u8]) -> Result<(), NotCarried> {
        if handed.is_empty() {
            return Ok(());
        }
        let password = what_was_handed_over(handed)
            .map_err(|_| NotCarried("what was handed over is not a password".to_owned()))?;
        let kept = this_machines_own(proxy).ok_or_else(|| {
            NotCarried(
                "a password was handed over for a proxy that does not sign in where this machine \
                 keeps one, so nothing was changed"
                    .to_owned(),
            )
        })?;
        self.credentials.written(kept, &password).map_err(|why| {
            NotCarried(format!(
                "the password for this machine's proxy was not written, so the proxy was not set \
                 either: {why:?}"
            ))
        })
    }

    /// The proxy the machine has now: what its file says, or — on a machine
    /// whose file has never been written — no proxy, the person's to set.
    fn now(&self) -> Result<Kept, NotCarried> {
        match fs::read(&self.machines) {
            Ok(bytes) => kept_on_this_machine(&bytes).map_err(|why| {
                NotCarried(format!(
                    "the machine's proxy file is not one this machine wrote, so it is left as it \
                     is: {why}"
                ))
            }),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
                Ok(Kept::by_this_person(TheProxy::None))
            }
            Err(why) => Err(NotCarried(format!(
                "the machine's proxy file could not be read: {why}"
            ))),
        }
    }

    /// The machine's file, replaced whole by this.
    fn written(&self, kept: &Kept) -> Result<(), NotCarried> {
        let bytes = machines(kept).map_err(|why| NotCarried(why.to_string()))?;
        let beside = self.machines.with_extension("writing");
        drop(fs::remove_file(&beside));
        let written = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(THE_FILES_MODE)
            .open(&beside)
            .and_then(|mut file| {
                file.write_all(&bytes)?;
                file.sync_all()
            })
            .and_then(|()| fs::set_permissions(&beside, fs::Permissions::from_mode(THE_FILES_MODE)))
            .and_then(|()| fs::rename(&beside, &self.machines));
        written.map_err(|why| {
            drop(fs::remove_file(&beside));
            NotCarried(format!(
                "the machine's proxy file could not be written: {why}"
            ))
        })
    }
}

/// Where this proxy keeps its password, when that is the one place a person's
/// own machine keeps one (ADR 0060 §2) — and [`None`] for anywhere else.
///
/// Every address the setting names has to agree: a proxy signing in under two
/// different names is one this machine cannot write a single credential for,
/// and writing one of the two would be a machine that signs in on one scheme
/// and not the other.
fn this_machines_own(proxy: &TheProxy) -> Option<&WhereThePasswordIs> {
    let mut found: Option<&WhereThePasswordIs> = None;
    for scheme in Scheme::EVERY {
        let Some(kept) = proxy
            .for_(scheme)
            .and_then(alo_proxy::ProxyAddress::password)
        else {
            continue;
        };
        if !kept.is_this_machines_own() {
            return None;
        }
        match found {
            Some(already) if already != kept => return None,
            _ => found = Some(kept),
        }
    }
    found
}
