//! Installing an application from its maker's own signed release (ADR 0158).
//!
//! The same two steps as `crate::installing`, and the indicator between them:
//!
//! 1. [`installing_a_signed_release`] asks everything that can be asked
//!    without leaving the machine — may this maker's place be used, is the
//!    application already here, is this machine one the release was built for —
//!    and answers with the errand to show. A refusal here means nothing left.
//! 2. [`install_a_signed_release`] takes the line the indicator handed back,
//!    asks again, fetches the file and its signature into a folder of its own,
//!    and refuses unless **the file hashes to the pinned SHA-256 and its
//!    signature was made by the maker's pinned key**. Only then is the rented
//!    tool handed the file. Whatever the answer, nothing fetched is kept.
//!
//! # Every way it can fail to be shown is one refusal
//!
//! A missing signature, a bad one, one made by another key, and a file that
//! does not hash to its pin are each [`NotDone::SignatureNotShown`] — the
//! sentence a person already reads when Flathub's delivery fails its check:
//! *what {source} sent for {application} could not be shown to come from
//! {source}.* The difference between them is for whoever fixes the machine,
//! and a person does the same thing about all four.
//!
//! # It does not update itself the way other applications do
//!
//! The catalogue pins one release by its checksum, so there is nothing for the
//! machine to offer as newer. A newer release reaches a machine when this
//! repository pins it.

use std::path::Path;

use alo_applications::Application;
use alo_egress::{Destination, Errand, OnItsOwn, Underway};

use crate::bound::Bound;
use crate::enabled::did_not_answer;
use crate::fetching::{Fetcher, THE_LARGEST_FILE};
use crate::installing::Installed;
use crate::maker::Maker;
use crate::refusing::NotDone;
use crate::shown::{Stopped, held_to};
use crate::signature::Verifier;
use crate::tool::Tool;

/// The only architecture the pinned bundles are built for.
pub const THE_ARCHITECTURE: &str = "x86_64";

/// One application's release, pinned: who made it, where its file is, and the
/// checksum the file must have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedRelease {
    /// The application, by the identifier its bundle installs as.
    application: Application,
    /// Who made it.
    maker: Maker,
    /// Where its bundle is. Its signature is beside it, with `.asc` added.
    bundle: String,
    /// The bundle's SHA-256, in lower-case hexadecimal.
    sha256: String,
}

/// Why a pinned release was not believed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotASignedRelease {
    /// A maker this machine does not know.
    #[error("the maker is not one this machine installs signed releases from")]
    NoSuchMaker,
    /// An address that is not one of the maker's release files.
    #[error("the address is not one of the maker's release files")]
    NotTheMakers,
    /// A checksum that is not sixty-four hexadecimal digits.
    #[error("the checksum is not a SHA-256 written in hexadecimal")]
    NotAChecksum,
}

impl SignedRelease {
    /// A release, held to what one has to be.
    ///
    /// # Errors
    /// [`NotASignedRelease`], naming the first thing that does not hold.
    pub fn pinned(
        application: Application,
        maker: &str,
        bundle: &str,
        sha256: &str,
    ) -> Result<Self, NotASignedRelease> {
        let maker = Maker::named(maker).ok_or(NotASignedRelease::NoSuchMaker)?;
        if !maker.publishes(bundle) {
            return Err(NotASignedRelease::NotTheMakers);
        }
        let is_a_checksum = sha256.len() == 64 && sha256.chars().all(|c| c.is_ascii_hexdigit());
        if !is_a_checksum {
            return Err(NotASignedRelease::NotAChecksum);
        }
        Ok(Self {
            application,
            maker,
            bundle: bundle.to_owned(),
            sha256: sha256.to_ascii_lowercase(),
        })
    }

    /// The application.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// Who made it.
    #[must_use]
    pub const fn maker(&self) -> &Maker {
        &self.maker
    }

    /// Where its bundle is.
    #[must_use]
    pub fn bundle(&self) -> &str {
        &self.bundle
    }

    /// Where its signature is.
    #[must_use]
    pub fn signature(&self) -> String {
        format!("{}.asc", self.bundle)
    }

    /// The checksum the bundle must have.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    /// The refusal a person reads about this release, whatever was wrong.
    fn not_shown(&self) -> NotDone {
        NotDone::SignatureNotShown {
            source: self.maker.name().to_owned(),
            application: self.application.identifier().to_owned(),
        }
    }
}

/// Everything asked before anything leaves, and the errand to show if it may.
///
/// # Errors
/// [`NotDone`]: the maker's place is kept out by a rule, the application is
/// already here, the tool could not say what is, or this machine is not one
/// the release was built for.
pub fn installing_a_signed_release(
    bound: &Bound,
    release: &SignedRelease,
    architecture: &str,
    tool: &(impl Tool + ?Sized),
) -> Result<OnItsOwn, NotDone> {
    let destination = may_be_used(bound, release, architecture)?;
    already_here(release.application(), tool)?;
    Ok(OnItsOwn::for_(Errand::InstallingAnApplication, destination))
}

/// Fetch, check and install it, while the errand is on the indicator.
///
/// `into` is a folder of this installation's own; the files fetched into it
/// are removed before this returns, whatever it returns.
///
/// # Errors
/// [`Stopped::NotShown`] for a line that is not this installation;
/// [`Stopped::Refused`] for everything [`installing_a_signed_release`] refuses,
/// asked again, for a file that could not be fetched, and — as
/// [`NotDone::SignatureNotShown`] — for a file whose checksum or signature does
/// not show it came from its maker.
#[expect(
    clippy::too_many_arguments,
    reason = "each is one of the rented things this road reaches through, passed \
              rather than reached for so every one of them is a test's to replace"
)]
pub fn install_a_signed_release(
    during: &Underway,
    bound: &Bound,
    release: &SignedRelease,
    architecture: &str,
    fetcher: &impl Fetcher,
    verifier: &impl Verifier,
    tool: &(impl Tool + ?Sized),
    into: &Path,
) -> Result<Installed, Stopped> {
    let destination = may_be_used(bound, release, architecture)?;
    held_to(during, Errand::InstallingAnApplication, &destination)?;
    already_here(release.application(), tool)?;
    let bundle = into.join("bundle.flatpak");
    let signature = into.join("bundle.flatpak.asc");
    let keyring = into.join("maker.gpg");
    let answer = fetched_checked_and_installed(
        release, fetcher, verifier, tool, &bundle, &signature, &keyring,
    );
    for file in [&bundle, &signature, &keyring] {
        // A file that was never written is already not kept.
        let _ = std::fs::remove_file(file);
    }
    answer?;
    Ok(Installed::sandboxed(release.application().clone()))
}

/// The part of the road that leaves files behind, so the caller can remove
/// them on every answer.
fn fetched_checked_and_installed(
    release: &SignedRelease,
    fetcher: &impl Fetcher,
    verifier: &impl Verifier,
    tool: &(impl Tool + ?Sized),
    bundle: &Path,
    signature: &Path,
    keyring: &Path,
) -> Result<(), NotDone> {
    fetcher
        .fetch(release.bundle(), bundle, THE_LARGEST_FILE)
        .map_err(did_not_answer)?;
    // **A missing signature is a refusal**, the same one as a bad signature:
    // the file is never installed on the strength of its checksum alone.
    if fetcher
        .fetch(&release.signature(), signature, 1 << 16)
        .is_err()
    {
        return Err(release.not_shown());
    }
    if !hashes_to(bundle, release.sha256()) {
        return Err(release.not_shown());
    }
    let key = release
        .maker()
        .keyring()
        .ok_or_else(|| release.not_shown())?;
    std::fs::write(keyring, key).map_err(|why| NotDone::DidNotAnswer {
        said: format!("{} could not be written: {why}", keyring.display()),
    })?;
    let signer = verifier
        .signer(bundle, signature, keyring)
        .map_err(did_not_answer)?;
    if signer.as_deref() != Some(release.maker().fingerprint()) {
        return Err(release.not_shown());
    }
    tool.install_a_file(release.application(), bundle)
        .map_err(|failed| match release.maker().source() {
            Some(source) => failed.about(&source, release.application()),
            None => release.not_shown(),
        })
}

/// Whether the file at `path` has this SHA-256, read in pieces so a large
/// bundle is never held in memory whole.
fn hashes_to(path: &Path, sha256: &str) -> bool {
    use sha2::Digest as _;
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut hasher = sha2::Sha256::new();
    let mut piece = vec![0_u8; 1 << 16];
    loop {
        match std::io::Read::read(&mut file, &mut piece) {
            Ok(0) => break,
            Ok(read) => hasher.update(piece.get(..read).unwrap_or_default()),
            Err(_) => return false,
        }
    }
    let got: String = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    got == sha256
}

/// The maker's place may be used, on a machine this release was built for, and
/// the destination the indicator shows.
fn may_be_used(
    bound: &Bound,
    release: &SignedRelease,
    architecture: &str,
) -> Result<Destination, NotDone> {
    let maker = release.maker();
    let Some(source) = maker.source() else {
        return Err(NotDone::not_enabled(maker.name()));
    };
    if let Some(set_by) = bound.keeps_out(&source) {
        return Err(NotDone::OutsideTheBound {
            source: maker.name().to_owned(),
            set_by,
        });
    }
    if architecture != THE_ARCHITECTURE {
        return Err(NotDone::NotOffered {
            source: maker.name().to_owned(),
            application: release.application().identifier().to_owned(),
        });
    }
    Destination::at(maker.host()).map_err(|_| NotDone::NowhereToReach {
        source: maker.name().to_owned(),
    })
}

/// Refused when the application is already installed.
fn already_here(application: &Application, tool: &(impl Tool + ?Sized)) -> Result<(), NotDone> {
    let installed = tool.installed().map_err(did_not_answer)?;
    if installed
        .iter()
        .any(|here| here == application.identifier())
    {
        return Err(NotDone::AlreadyInstalled {
            application: application.identifier().to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
#[path = "installing_a_signed_release_tests.rs"]
mod tests;
