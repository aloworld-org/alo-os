//! `network.set-proxy`: the machine's proxy becomes the one a person handed
//! over, exactly, or stays as it was.
//!
//! The door takes no text, so a person's chosen proxy reaches the broker the
//! only way anything does: as the digest of bytes the broker can find for
//! itself. Settings writes the choice to `alo_networks::proxy_file::THE_WANTED_PROXY`,
//! in a folder the broker made for the person's group, and asks for this verb
//! with the digest of what it wrote. Carrying it out, in order:
//!
//! 1. **The file is opened without following a link**, and believed only if it
//!    is a plain file, owned by the person the door is for, and no longer than a
//!    proxy setting is. Anything else — a link to a file of root's, a file the
//!    agent's login left, something enormous — is not read.
//! 2. **Its bytes digest to the identity approved**, or nothing is set: bytes
//!    changed after the approval are a different proxy.
//! 3. **It is rebuilt through `alo-proxy`'s own checks**
//!    (`alo_networks::proxy_file::rechecked`), so the machine's file only ever
//!    holds what that crate would itself have made.
//! 4. **A proxy an organisation set is not replaced by a person's.** The
//!    machine's file says whose its proxy is, and `alo_proxy::Kept` refuses.
//! 5. **The machine's file is replaced whole**: written beside itself, synced,
//!    and renamed over, `0644`, so a road out reading it never reads half of one.
//! 6. The handed-over file is removed, since it has been used.

use std::fs::{self, File, OpenOptions};
use std::io::{Read as _, Write as _};
use std::os::unix::fs::{MetadataExt as _, OpenOptionsExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};

use alo_broker::{Identity, NotCarried};
use alo_networks::proxy_file::{LONGEST_FILE, kept_on_this_machine, machines, rechecked};
use alo_proxy::{Kept, TheProxy};

/// The mode the machine's proxy file is written with: root writes, everybody
/// reads.
const THE_FILES_MODE: u32 = 0o644;

/// Setting the machine's proxy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proxy {
    /// Where a person hands a proxy over.
    wanted: PathBuf,
    /// The machine's proxy file.
    machines: PathBuf,
    /// The person who may hand one over.
    person: u32,
}

impl Proxy {
    /// Set the proxy handed over at `wanted`, by `person`, into `machines`.
    #[must_use]
    pub fn handed_over(wanted: &Path, machines: &Path, person: u32) -> Self {
        Self {
            wanted: wanted.to_owned(),
            machines: machines.to_owned(),
            person,
        }
    }

    /// Carry `network.set-proxy` out for this identity.
    ///
    /// # Errors
    /// [`NotCarried`], and the machine's proxy is as it was.
    pub fn set(&self, identity: Identity) -> Result<(), NotCarried> {
        let bytes = self.what_was_handed_over()?;
        if Identity::of_what_was_reported(&bytes) != identity {
            return Err(NotCarried(
                "the proxy handed over is not the one that was approved, so nothing was changed"
                    .to_owned(),
            ));
        }
        let wanted = rechecked(&bytes).map_err(|why| NotCarried(why.to_string()))?;
        let kept = self.now()?.changed_by_the_person(wanted).map_err(|_| {
            NotCarried(
                "the organisation that manages this machine set its proxy, so a person's is not \
                 set over it"
                    .to_owned(),
            )
        })?;
        self.written(&kept)?;
        drop(fs::remove_file(&self.wanted));
        Ok(())
    }

    /// The bytes a person handed over, believed only in the shape step 1 names.
    fn what_was_handed_over(&self) -> Result<Vec<u8>, NotCarried> {
        let refused = |why: &str| NotCarried(format!("the proxy handed over {why}"));
        let opened = rustix::fs::open(
            &self.wanted,
            rustix::fs::OFlags::RDONLY
                | rustix::fs::OFlags::NOFOLLOW
                | rustix::fs::OFlags::NONBLOCK
                | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        )
        .map_err(|why| refused(&format!("could not be opened: {why}")))?;
        let file = File::from(opened);
        let about = file
            .metadata()
            .map_err(|why| refused(&format!("could not be looked at: {why}")))?;
        if !about.file_type().is_file() {
            return Err(refused("is not a plain file"));
        }
        if about.uid() != self.person {
            return Err(refused(&format!(
                "is owned by user {}, not by the person",
                about.uid()
            )));
        }
        if about.len() > LONGEST_FILE {
            return Err(refused("is longer than a proxy setting"));
        }
        let mut bytes = Vec::new();
        file.take(LONGEST_FILE)
            .read_to_end(&mut bytes)
            .map_err(|why| refused(&format!("could not be read: {why}")))?;
        Ok(bytes)
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
