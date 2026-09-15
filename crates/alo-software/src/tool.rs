//! The rented tool, as the handful of questions and acts this crate needs of it.
//!
//! ADR 0005 rents the tool that installs and sandboxes an application, and
//! `CLAUDE.md` says a rented engine is configured and never patched. This trait
//! is the edge between the two: everything this crate decides is decided
//! before one of these is called and after it answers, and nothing about *how*
//! the tool is reached leaks past it. [`crate::rented`] is the door on a
//! machine; a test hands in its own.
//!
//! # Seven, and none of them is a command
//!
//! Each method takes values this crate has already checked — a
//! [`SourceName`] that cannot be read as an option, an `Application` whose
//! identifier has no space, separator or control character in it — and there is
//! no method that takes text to pass along. Law 2 binds the agent, and an agent
//! reaches this crate through one verb whose arguments arrive already validated;
//! the trait keeps that true one layer further down.
//!
//! # What the tool can say went wrong
//!
//! [`Failed`] is the closed list of answers this crate acts on differently.
//! Anything else is [`Failed::DidNotAnswer`] with the tool's own words kept for
//! whoever is fixing the machine — and the person is told that nothing was
//! changed, which is true, rather than what the tool said, which names it.

use alo_applications::Application;

use crate::refusing::NotDone;
use crate::source::{Configured, SourceName};

/// What the rented tool reported instead of doing what was asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failed {
    /// What the place sent could not be verified against its signature, and
    /// the tool refused it.
    SignatureNotShown,
    /// The place has no application by that identifier.
    NotOffered,
    /// The application is already installed.
    AlreadyInstalled,
    /// The application is not installed.
    NotInstalled,
    /// The tool could not be reached, or said something nothing here reads.
    DidNotAnswer {
        /// What it said, verbatim, for whoever is fixing the machine.
        said: String,
    },
}

impl Failed {
    /// What this means to a person, about this application from this place.
    pub(crate) fn about(self, source: &SourceName, application: &Application) -> NotDone {
        let source = || source.as_str().to_owned();
        let application = || application.identifier().to_owned();
        match self {
            Self::SignatureNotShown => NotDone::SignatureNotShown {
                source: source(),
                application: application(),
            },
            Self::NotOffered => NotDone::NotOffered {
                source: source(),
                application: application(),
            },
            Self::AlreadyInstalled => NotDone::AlreadyInstalled {
                application: application(),
            },
            Self::NotInstalled => NotDone::NotInstalled {
                application: application(),
            },
            Self::DidNotAnswer { said } => NotDone::DidNotAnswer { said },
        }
    }
}

/// The rented tool that installs, updates and removes sandboxed applications.
pub trait Tool {
    /// Every place the tool is set up to install from, switched off or not.
    ///
    /// # Errors
    /// [`Failed`], when the tool could not say.
    fn sources(&self) -> Result<Vec<Configured>, Failed>;

    /// The identifiers of the applications installed.
    ///
    /// # Errors
    /// [`Failed`], when the tool could not say.
    fn installed(&self) -> Result<Vec<String>, Failed>;

    /// The identifiers of the applications open right now.
    ///
    /// # Errors
    /// [`Failed`], when the tool could not say.
    fn open(&self) -> Result<Vec<String>, Failed>;

    /// Install this application from this place.
    ///
    /// # Errors
    /// [`Failed`], when it was not installed.
    fn install(&self, source: &SourceName, application: &Application) -> Result<(), Failed>;

    /// The identifiers of installed applications this place has newer
    /// versions of.
    ///
    /// # Errors
    /// [`Failed`], when the place could not be asked.
    fn updates(&self, source: &SourceName) -> Result<Vec<String>, Failed>;

    /// Update this application to the newest version its place offers.
    ///
    /// # Errors
    /// [`Failed`], when it was not updated.
    fn update(&self, application: &Application) -> Result<(), Failed>;

    /// Remove this application.
    ///
    /// # Errors
    /// [`Failed`], when it was not removed.
    fn remove(&self, application: &Application) -> Result<(), Failed>;
}
