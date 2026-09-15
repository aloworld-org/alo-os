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
}

impl Wanted {
    /// What a person asked for themselves, in Software in Settings.
    #[must_use]
    pub fn by_hand(application: Application, source: &str) -> Self {
        Self {
            application,
            source: source.trim().to_owned(),
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
}

/// An application that has been installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    /// The application.
    application: Application,
}

impl Installed {
    /// The application.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// What a person is told, in the language they read.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &words::INSTALLED.key(),
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
