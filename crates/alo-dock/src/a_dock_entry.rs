//! The two kinds of thing a person's Dock can hold.
//!
//! The owner's ruling of 2026-10-12, in their words:
//!
//! > Separate what the person sees from how it launches. The saved Dock list
//! > must distinguish an application entry from a native-action entry. Dispatch
//! > each through its appropriate handler. "Native" describes the
//! > implementation; it does not mean permanent.
//!
//! Until this type, the saved list carried application identifiers and only
//! those. There was nowhere for a native action to live, so the only way to put
//! Settings on the Dock was to invent an identifier for it — which the same
//! ruling forbids, because no such application exists and nothing would answer
//! on that bus name.
//!
//! # The last sentence is the one with teeth
//!
//! *Native describes the implementation; it does not mean permanent.* The easy
//! way to make Settings always work is to make it always present, and that
//! takes a choice away — which law 5 forbids **whatever the reason given for
//! it**. So a native entry is removed, reordered and restored by exactly the
//! same operations an application entry is, and this type is an enum rather
//! than a special case precisely so that no code path can treat one as
//! unremovable without saying so out loud.
//!
//! # Why the action is an enumerated verb
//!
//! The second law: *no code runs unless the person chose it*, and every
//! capability is an enumerated verb with typed arguments. A native entry that
//! carried a command, a path or a name to look up would be the escape hatch
//! that law exists to refuse. [`ANativeAction`] has one variant, takes no
//! arguments, and a host maps it to something it already knew how to do.

use crate::window::AppId;

/// One thing on a person's Dock, as the saved list holds it.
///
/// **Order is the list's, not this type's.** Where an entry sits is a fact
/// about the person's arrangement; what it *is* is this.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ADockEntry {
    /// An application, by the identifier its source gives it.
    ///
    /// Pressed, it reaches that application — a window of it if one is open,
    /// and otherwise a request to the bus to start it.
    Application(AppId),
    /// Something the compositor itself does.
    ///
    /// It has no identifier and no bus endpoint, because there is no
    /// application. A host carries it out directly; see [`ANativeAction`].
    Native(ANativeAction),
}

impl ADockEntry {
    /// The application this entry is, if it is one.
    ///
    /// **[`Option`] rather than a default**, and the difference is the whole
    /// point of the type: a native entry that answered with some identifier
    /// would be the invented name the ruling forbids, and every caller that
    /// draws, announces or dispatches has to decide what a native row means to
    /// it rather than being handed something that looks like an application.
    #[must_use]
    pub const fn application(&self) -> Option<&AppId> {
        match self {
            Self::Application(app) => Some(app),
            Self::Native(_) => None,
        }
    }

    /// The action this entry is, if it is a native one.
    #[must_use]
    pub const fn native(&self) -> Option<ANativeAction> {
        match self {
            Self::Application(_) => None,
            Self::Native(action) => Some(*action),
        }
    }

    // **No `is_the_same_as`.** `PartialEq` is what `Holding::pin_entry` and
    // `unpin_entry` use — `contains` and `retain` — and a named method
    // delegating to it would be a second way to ask one question, with nothing
    // in production calling it. Two application entries match on identifier and
    // two native entries on action; an application and an action never match,
    // however either is named, and the derive is what guarantees that.
}

/// What the compositor itself can be asked to do from the Dock.
///
/// **One variant, no arguments.** A native action is an enumerated verb in the
/// capability model's sense: a host matches on it and does something it already
/// knew how to do. Nothing here carries a command, a path or a name to be
/// looked up, because that is the escape hatch the second law refuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ANativeAction {
    /// Open this machine's settings.
    ///
    /// The compositor's own surface, which `alo_shortcuts::Action::Settings`
    /// already opens from a chord. The Dock asks for the same thing, so there
    /// is one way in rather than two — and a person who hides the Dock's entry
    /// still has the keyboard's.
    Settings,
}

#[cfg(test)]
#[path = "a_dock_entry_tests.rs"]
mod tests;
