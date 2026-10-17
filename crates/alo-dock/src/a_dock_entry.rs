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
//! that law exists to refuse. Every [`ANativeAction`] variant takes no
//! arguments, and a host maps each to something it already knew how to do.
//!
//! **It had one variant until 2026-10-17 and now has two**, which is the first
//! evidence that the shape holds: the window of what a person can install
//! joined it without a field, without an argument and without a name to look
//! up. An enum that could only ever describe Settings would have been a
//! special case wearing a type's clothes; one that took a second verb
//! unchanged is the capability model working rather than being asserted.

use serde::{Deserialize, Serialize};

use crate::window::AppId;

/// One thing on a person's Dock, as the saved list holds it.
///
/// **Order is the list's, not this type's.** Where an entry sits is a fact
/// about the person's arrangement; what it *is* is this.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "Written", into = "Written")]
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
/// **No arguments, on any variant.** A native action is an enumerated verb in
/// the capability model's sense: a host matches on it and does something it
/// already knew how to do. Nothing here carries a command, a path or a name to
/// be looked up, because that is the escape hatch the second law refuses.
///
/// **And nothing here is permanent.** The owner's ruling this type exists for
/// says *native describes the implementation; it does not mean permanent*, so
/// both variants are removed, reordered and restored by exactly the same
/// operations an application entry is. A person who does not want the market on
/// their Dock unpins it like anything else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ANativeAction {
    /// Open this machine's settings.
    ///
    /// The compositor's own surface, which `alo_shortcuts::Action::Settings`
    /// already opens from a chord. The Dock asks for the same thing, so there
    /// is one way in rather than two — and a person who hides the Dock's entry
    /// still has the keyboard's.
    Settings,
    /// Open the window of what a person can install.
    ///
    /// The compositor's own surface too, so it belongs here rather than as an
    /// [`ADockEntry::Application`]: there is no application to name.
    /// `alo_software::Catalogue` is a list this machine reads and the window
    /// that shows it is one the shell draws.
    ///
    /// **This is the only way in, and that is said rather than left to be
    /// found.** [`Self::Settings`] above can afford *a person who hides the
    /// Dock's entry still has the keyboard's*; this one cannot, because no
    /// chord opens the market. Unpinning it leaves asking alo, which is a
    /// different road built separately — so a person can still get there, and
    /// the Dock is not the last door.
    Applications,
}

/// One entry as `dock.toml` writes it, which a person types as often as a panel
/// writes it.
///
/// ```toml
/// [[pins]]
/// application = "org.kde.dolphin"
///
/// [[pins]]
/// native = "settings"
///
/// [[pins]]
/// native = "applications"
/// ```
///
/// **The spelling is the variant's name, kebab-cased**, which the `serde`
/// attribute below decides rather than this documentation — so a variant added
/// without a thought spared for TOML still has a spelling, and it is the one a
/// person would have guessed.
///
/// **A table with one key rather than a tagged enum**, because serde's own
/// enum shapes are either unreadable in TOML or impossible to hand-write: an
/// externally tagged variant becomes a nested table, and an untagged one
/// cannot say which kind an unknown string was meant to be.
///
/// **Exactly one of the two**, and both halves are refused:
/// [`NotAnEntry::Neither`] for a table that names no kind, and
/// [`NotAnEntry::Both`] for one that names two. Neither can be guessed at —
/// a row with both would be an arrangement whose author believed one thing and
/// whose machine did another.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    /// An application, by the identifier its source gives it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    application: Option<String>,
    /// One of alo's own actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    native: Option<ANativeAction>,
}

/// Why a line in a person's pinned list is not an entry.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAnEntry {
    /// It named neither an application nor an action.
    #[error("a pin names neither an application nor one of alo's own actions")]
    Neither,
    /// It named both.
    #[error("a pin names an application and one of alo's own actions, and it can only be one")]
    Both,
    /// Its application is not a name this machine will take.
    ///
    /// **The one validation, not a second one.** `AppId::named` is where a
    /// name is refused, and deserialising through it is what stops a
    /// hand-written file putting something on the Dock that the rest of this
    /// crate would never have accepted.
    #[error("a pin's application is not a name: {0:?}")]
    NotAnApplication(crate::window::NotAnApp),
}

impl TryFrom<Written> for ADockEntry {
    type Error = NotAnEntry;

    fn try_from(written: Written) -> Result<Self, Self::Error> {
        match (written.application, written.native) {
            (Some(_), Some(_)) => Err(NotAnEntry::Both),
            (None, None) => Err(NotAnEntry::Neither),
            (Some(named), None) => AppId::named(&named)
                .map(Self::Application)
                .map_err(NotAnEntry::NotAnApplication),
            (None, Some(action)) => Ok(Self::Native(action)),
        }
    }
}

impl From<ADockEntry> for Written {
    fn from(entry: ADockEntry) -> Self {
        match entry {
            ADockEntry::Application(app) => Self {
                application: Some(app.name().to_owned()),
                native: None,
            },
            ADockEntry::Native(action) => Self {
                application: None,
                native: Some(action),
            },
        }
    }
}

#[cfg(test)]
#[path = "a_dock_entry_tests.rs"]
mod tests;
