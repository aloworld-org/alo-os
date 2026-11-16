//! **The person's desktop asks which applications the turn wanted and was not
//! given**, for the sheet that asks whether to give alo one (the owner's
//! decision of 2026-11-16, `docs/design/when-alo-asks-for-an-application.md`).
//!
//! The daemon keeps the list on the turn; this door only asks for it. Nothing
//! here grants anything: a grant is the person's press on the sheet, carried
//! by `crate::Changing::granted` like every other.

use alo_protocol::{FromAPerson, ToAPerson};

use crate::door::TheDaemonsDoor;

/// Asking the daemon which applications the turn under way wanted.
///
/// A trait so a surface can be tested against a daemon written down in a test.
pub trait AskingWhatIsWanted {
    /// The installed applications the turn under way asked for and nothing
    /// grants, by identifier — empty when there are none — or [`None`] when no
    /// daemon answered.
    fn wanted(&self) -> Option<Vec<String>>;
}

impl AskingWhatIsWanted for TheDaemonsDoor {
    fn wanted(&self) -> Option<Vec<String>> {
        match self.asked(&FromAPerson::Wanted).ok()? {
            ToAPerson::Wanted { applications } => Some(applications),
            // Any other answer is not one this request can earn.
            _ => None,
        }
    }
}
