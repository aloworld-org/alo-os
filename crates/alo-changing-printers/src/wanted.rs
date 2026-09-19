//! A change to the printers somebody wants, before anybody has asked which
//! printer that is.

use alo_broker::{Identity, SystemVerb};

/// Which of the three changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// Set a printer this machine found up.
    Add,
    /// Remove a printer that is set up.
    Remove,
    /// Make a printer that is set up the one this machine prints on.
    MakeDefault,
}

impl Change {
    /// The broker's verb for this change, to the printer with this identity.
    #[must_use]
    pub const fn to(self, identity: Identity) -> SystemVerb {
        match self {
            Self::Add => SystemVerb::AddPrinter(identity),
            Self::Remove => SystemVerb::RemovePrinter(identity),
            Self::MakeDefault => SystemVerb::SetDefaultPrinter(identity),
        }
    }

    /// Which change the broker's verb is, if it is one of the printers'.
    #[must_use]
    pub const fn of(verb: &SystemVerb) -> Option<Self> {
        match verb {
            SystemVerb::AddPrinter(_) => Some(Self::Add),
            SystemVerb::RemovePrinter(_) => Some(Self::Remove),
            SystemVerb::SetDefaultPrinter(_) => Some(Self::MakeDefault),
            SystemVerb::JoinNetwork(_)
            | SystemVerb::ForgetNetwork(_)
            | SystemVerb::SetRadio(_)
            | SystemVerb::SetProxy(_)
            | SystemVerb::ApplyStagedUpdate(_)
            | SystemVerb::RollBack(_)
            | SystemVerb::MountDrive(_)
            | SystemVerb::EjectDrive(_) => None,
        }
    }

    /// Whether the printer is one this machine found rather than one set up.
    #[must_use]
    pub const fn is_of_a_printer_found(self) -> bool {
        matches!(self, Self::Add)
    }
}

/// A change, to the printer called this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wanted {
    /// Which change.
    change: Change,
    /// The name the printer gave itself, as the person approved it.
    called: String,
}

impl Wanted {
    /// This change, to the printer called this.
    #[must_use]
    pub fn of(change: Change, called: &str) -> Self {
        Self {
            change,
            called: called.to_owned(),
        }
    }

    /// Which change.
    #[must_use]
    pub const fn change(&self) -> Change {
        self.change
    }

    /// The printer's name, as the person approved it.
    #[must_use]
    pub fn called(&self) -> &str {
        &self.called
    }
}
