//! Installing an application: asked first, shown while it happens, and granted
//! nothing.
//!
//! Two steps, and the indicator between them.
//!
//! 1. [`installing`] asks everything that can be asked without leaving the
//!    machine — may this place be used, is the application already here — and
//!    answers with the `alo_egress::OnItsOwn` to put on the indicator. A
//!    refusal here means nothing left.
//! 2. [`install`] takes the `alo_egress::Underway` the indicator handed back,
//!    holds it to *installing, from this place* ([`crate::shown`]), asks the
//!    place again — a rule can change between one step and the next, and the
//!    second answer is the one that counts — and only then reaches the rented
//!    tool.
//!
//! # It arrives with no grants, and there is no way for it not to
//!
//! Neither step takes the machine's grants, `alo_capability::Agent` or anything
//! that could make one. An installed application is reachable by nothing — not
//! a folder, not the camera, not an agent's verb — until a person allows it
//! something through a portal or grants an agent the application, both of which
//! are acts somewhere else. `tests/installing_updating_and_removing.rs` reads
//! the one list before and after to hold that as a fact rather than a shape.
//!
//! # The rented tool checks the signature, and a failure is refused in words
//!
//! A place set up without signature checking is refused in step 1, before
//! anything is fetched. A place that checks, and whose delivery fails the
//! check, is refused by the tool, which keeps nothing it fetched; that answer
//! comes back as [`NotDone::SignatureNotShown`], naming the place and the
//! application.

use alo_applications::Application;
use alo_egress::{Errand, OnItsOwn, Underway};
use alo_strings::{Filling, Said, Strings};

use crate::enabled::{Enabled, did_not_answer};
use crate::refusing::NotDone;
use crate::shown::{Stopped, held_to};
use crate::tool::Tool;
use crate::whether_it_is_sandboxed::Sandboxing;
use crate::words;

/// An application a person — or a person approving an agent's proposal — asked
/// to install, and the place it is to come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wanted {
    /// The application.
    application: Application,
    /// The place, as it was named. Checked against what is enabled at each
    /// step rather than once, because what is enabled can change.
    source: String,
    /// Whether a person chose to do without the sandbox on this installation.
    sandboxing: Sandboxing,
}

impl Wanted {
    /// What a person asked for themselves, in Software in Settings — **sandboxed**.
    ///
    /// This signature is unchanged and now carries
    /// [`Sandboxing::Sandboxed`]. An unsandboxed installation cannot be expressed
    /// here: it has a constructor of its own that a caller has to name, which is how
    /// *never the default* is held. See `crate::whether_it_is_sandboxed`.
    #[must_use]
    pub fn by_hand(application: Application, source: &str) -> Self {
        Self {
            application,
            source: source.trim().to_owned(),
            sandboxing: Sandboxing::Sandboxed,
        }
    }

    /// What a person asked for themselves, **deliberately without a sandbox**.
    ///
    /// Named rather than defaulted, and named rather than a flag on
    /// [`Self::by_hand`], so that the only way to reach this state is to write this
    /// sentence. A caller who forgets gets the sandboxed install, which is the safe
    /// outcome — the inversion `crate::whether_it_is_sandboxed` explains.
    ///
    /// *Forbiddable by policy on a managed machine* is **not** checked here and has no
    /// road in this crate: see task 15's owed clause. Nothing in this constructor may be
    /// read as that check having happened.
    #[must_use]
    pub fn unsandboxed_by_hand(application: Application, source: &str) -> Self {
        Self {
            application,
            source: source.trim().to_owned(),
            sandboxing: Sandboxing::Unsandboxed,
        }
    }

    /// The application wanted.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// The place it is to come from, as it was named.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Whether this installation is sandboxed.
    #[must_use]
    pub const fn sandboxing(&self) -> Sandboxing {
        self.sandboxing
    }
}

/// An application that has been installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    /// The application.
    application: Application,
    /// How it was installed, kept so a surface is handed the fact rather than
    /// deriving it. See `crate::whether_it_is_sandboxed`.
    sandboxing: Sandboxing,
}

impl Installed {
    /// The application.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// Whether it was installed sandboxed.
    #[must_use]
    pub const fn sandboxing(&self) -> Sandboxing {
        self.sandboxing
    }

    /// What a person is told, in the language they read.
    ///
    /// **An unsandboxed installation is told a different sentence, not an extra one.**
    /// The ordinary sentence promises *it has been given nothing: it asks when it needs a
    /// file, the camera or anything else* — and that promise is false without a sandbox.
    /// Appending a marking to it would leave a person holding two sentences, one of which
    /// is a lie.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let word = if self.sandboxing.is_worth_marking() {
            words::INSTALLED_UNSANDBOXED.key()
        } else {
            words::INSTALLED.key()
        };
        strings.say(
            &word,
            &Filling::of(words::APPLICATION, self.application.identifier()),
        )
    }
}

/// Everything asked before anything leaves, and the errand to show if it may.
///
/// # Errors
/// [`NotDone`] — the place may not be used ([`Enabled::usable`]), the
/// application is already installed, or the tool could not say what is.
pub fn installing(
    enabled: &Enabled,
    wanted: &Wanted,
    tool: &impl Tool,
) -> Result<OnItsOwn, NotDone> {
    let (_, destination) = enabled.usable(&wanted.source)?;
    already_here(wanted, tool)?;
    Ok(OnItsOwn::for_(
        Errand::InstallingAnApplication,
        destination.clone(),
    ))
}

/// Install it, while the errand is on the indicator.
///
/// # Errors
/// [`Stopped::NotShown`] for a line that is not this installation;
/// [`Stopped::Refused`] for everything [`installing`] refuses, asked again,
/// and for what the rented tool refused.
pub fn install(
    during: &Underway,
    enabled: &Enabled,
    wanted: Wanted,
    tool: &impl Tool,
) -> Result<Installed, Stopped> {
    let (source, destination) = enabled.usable(&wanted.source)?;
    held_to(during, Errand::InstallingAnApplication, destination)?;
    already_here(&wanted, tool)?;
    tool.install(source.name(), &wanted.application)
        .map_err(|failed| failed.about(source.name(), &wanted.application))?;
    Ok(Installed {
        application: wanted.application,
        sandboxing: wanted.sandboxing,
    })
}

/// Refused when the application is already installed.
fn already_here(wanted: &Wanted, tool: &impl Tool) -> Result<(), NotDone> {
    let installed = tool.installed().map_err(did_not_answer)?;
    if installed
        .iter()
        .any(|here| here == wanted.application.identifier())
    {
        return Err(NotDone::AlreadyInstalled {
            application: wanted.application.identifier().to_owned(),
        });
    }
    Ok(())
}
