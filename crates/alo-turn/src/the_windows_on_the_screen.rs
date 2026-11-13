//! **What reaches the person's windows**, for the two accessibility verbs —
//! the trait, and nothing that reaches a real session.
//!
//! The same division `crate::the_door_the_compositor_serves` makes for the
//! compositor and `alo_software::Tool` for the rented installer: what opens the
//! person's accessibility tree is a bus in their session, so a crate that named
//! it could not be built or tested anywhere one is not. `alo-agentd` holds the
//! one implementation that reaches the real tree (`alo_adapters::AccessibleSession`).
//!
//! # Opened for one act, never held
//!
//! [`TheWindowsOnTheScreen::opening`] answers a tree **for one read or one
//! press**, and the executor drops it when that act ends. That is the
//! fallback's own promise (`docs/contracts/app-adapters.md`: *the connection to
//! the tree ends with the turn*) carried one step further, and it is what makes
//! *with no invocation the tree is never asked* structural rather than a habit:
//! nothing here holds a tree between acts, so there is nothing that could be
//! asked between them. A machine made at start-up holds this trait and no tree.
//!
//! # Which switch would help is the caller's to say
//!
//! [`TheWindowsOnTheScreen::working_fully`] answers whether turning on *Let alo
//! work fully inside apps like VSCodium* would give an application's controls
//! a press and its document a reading. Whether an application is drawn by
//! Chromium is `alo-launching`'s answer and whether the switch is on is the
//! person's settings file, and this crate can reach neither: both crates sit
//! above it.

use alo_adapters::{AccessibilityTree, TreeFault, WorkingFully};

/// What opens the person's accessibility tree, and says which switch would
/// help an application.
pub trait TheWindowsOnTheScreen {
    /// The person's accessibility tree, opened now, for one read or one press.
    ///
    /// # Errors
    /// [`TreeFault::NotThere`] when this session has none to open, which a
    /// person is told as *what applications show could not be read here*.
    fn opening(&mut self) -> Result<Box<dyn AccessibilityTree>, TreeFault>;

    /// Whether a switch the person can turn on would let alo press `application`'s
    /// controls and read its document — read now, so a switch turned on a
    /// moment ago counts.
    fn working_fully(&self, application: &str) -> WorkingFully;
}
