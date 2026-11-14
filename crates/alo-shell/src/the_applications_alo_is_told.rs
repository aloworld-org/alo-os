//! **The applications alo is told about with a request**: what this seat was
//! last told the machine has, by identifier and the name its entry gives.
//!
//! *The owner, 2026-11-14: "Yes, send app names."* A model asked to open
//! VSCodium sent `vscodium` on a machine that had `com.vscodium.codium`, was
//! refused as *not installed*, and then installed something from Flathub. So
//! the request carries the applications installed, and the model names the
//! one meant.
//!
//! # Read at the moment the request is sent, from what the Dock holds
//!
//! Nothing new is read and nothing is watched: these are the identifiers and
//! names this seat already holds to fill the Dock and the launcher
//! (`crate::what_the_machine_offers_now`), taken at the moment the person
//! presses Enter and sent with that request only. **Names and identifiers,
//! never anything else** — no file, no window, no document.

use std::collections::BTreeMap;

use alo_instructing::TheApplicationsInstalled;

use crate::server::Server;

impl Server {
    /// The applications installed, as this seat was last told them, for the
    /// request being sent now.
    pub(crate) fn the_applications_alo_is_told(&self) -> TheApplicationsInstalled {
        the_applications_named(&self.what_this_machine_has, &self.what_each_is_called)
    }
}

/// Each identifier this seat has, with the name it was told for it.
fn the_applications_named(
    has: &[String],
    called: &BTreeMap<String, String>,
) -> TheApplicationsInstalled {
    TheApplicationsInstalled::of(has.iter().map(|identifier| {
        (
            identifier.as_str(),
            called.get(identifier).map(String::as_str),
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **What the Dock was told is what alo is told**: each application by
    /// its name, and one with no name by its identifier.
    #[test]
    fn what_the_seat_was_told_is_what_alo_is_told() {
        let has = vec![
            "com.vscodium.codium".to_owned(),
            "org.kde.dolphin".to_owned(),
        ];
        let called = BTreeMap::from([("com.vscodium.codium".to_owned(), "VSCodium".to_owned())]);
        let told = the_applications_named(&has, &called);
        assert_eq!(
            told.applications(),
            [
                ("com.vscodium.codium".to_owned(), "VSCodium".to_owned()),
                ("org.kde.dolphin".to_owned(), "org.kde.dolphin".to_owned()),
            ]
        );
    }
}
