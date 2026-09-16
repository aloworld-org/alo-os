//! Reading what an application's windows show, for one authorised read.
//!
//! [`ReadingWindows::of`] takes an authorised `accessible.read_window` **by
//! value** and asks it everything `crate::fallback_reach` asks — before the tree
//! is touched. [`ReadingWindows::read`] consumes it and walks the application's
//! windows once (`crate::walking`), so one authority is one read, taken at that
//! moment. Nothing is kept afterwards: the answer is handed back with the
//! authority to record, and a later turn that wants to know what a window shows
//! reads it again.

use alo_applications::Installed;
use alo_capability::{Authorised, Grants, Refused};
use alo_strings::Strings;

use crate::accessibility_tree::AccessibilityTree;
use crate::adapters::Adapters;
use crate::fallback_reach::reached;
use crate::fallback_verbs::READ_WINDOW;
use crate::shown::Shown;
use crate::walking::walk;

/// An authorised read of an application's windows that has not been read yet.
///
/// Deliberately not `Clone`, like the [`Authorised`] inside it.
#[derive(Debug)]
pub struct ReadingWindows {
    /// What may run.
    authorised: Authorised,
    /// The application whose windows are read.
    application: String,
}

impl ReadingWindows {
    /// Ask everything that is asked before a window is read.
    ///
    /// # Errors
    /// [`Refused`], carrying the call: another verb's authority, the
    /// application not granted (the grants' own words) or not installed, or an
    /// application with an adapter of its own.
    pub fn of(
        authorised: Authorised,
        adapters: &Adapters,
        grants: &Grants,
        installed: &Installed,
        strings: &Strings,
    ) -> Result<Self, Refused> {
        let reached = reached(
            authorised,
            READ_WINDOW,
            adapters,
            grants,
            installed,
            strings,
        )?;
        Ok(Self {
            authorised: reached.authorised,
            application: reached.application,
        })
    }

    /// The application whose windows are read.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// Read the application's windows, once, now.
    ///
    /// # Errors
    /// [`Refused`] when nothing was read: no window on the screen, nothing on
    /// this machine to read windows through, or the application not answering.
    pub fn read(
        self,
        tree: &dyn AccessibilityTree,
        strings: &Strings,
    ) -> Result<WindowsRead, Refused> {
        match walk(tree, &self.application) {
            Ok(walked) => Ok(WindowsRead {
                authorised: self.authorised,
                shown: walked.shown,
            }),
            Err(why) => Err(Refused::worded_elsewhere(
                self.authorised.call().clone(),
                why.said(&self.application, strings),
            )),
        }
    }
}

/// What an application's windows showed, and the authority it was read under.
#[derive(Debug)]
pub struct WindowsRead {
    /// What ran.
    authorised: Authorised,
    /// What was read.
    shown: Shown,
}

impl WindowsRead {
    /// What was read.
    #[must_use]
    pub fn shown(&self) -> &Shown {
        &self.shown
    }

    /// What ran.
    #[must_use]
    pub fn authorised(&self) -> &Authorised {
        &self.authorised
    }

    /// Give back what ran, to be recorded, and what was read, to answer the
    /// turn with.
    #[must_use]
    pub fn into_parts(self) -> (Authorised, Shown) {
        (self.authorised, self.shown)
    }
}
