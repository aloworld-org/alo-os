//! **An application named by its name, as the machine the set is asked on
//! names it.**
//!
//! A booted machine puts a name such as *VSCodium* into a call as the identifier
//! of the one installed application that answers to it, where the grants
//! permit that one (`alo_turn::as_this_machine_names_them`). A try is held to
//! what that machine accepts, so a step is matched after the same function, on
//! the same applications the model is told ([`WhatItMayReach::applications`]),
//! with the grants the machine has: none, and full access where the set says
//! the person chose it.

use std::time::SystemTime;

use alo_applications::Installed;
use alo_applications::application::Application;
use alo_capability::{Given, Grantee, Grants, Verbs};
use alo_instructing::{TheApplicationsInstalled, WhatItMayReach};

/// The agent a try's grants are asked about.
const THE_AGENT: &str = "alo";

/// The applications and grants a call's names are matched against.
#[derive(Debug, Clone)]
pub struct AsTheMachineNames {
    /// What is installed, each by the name the model is told.
    installed: Installed,
    /// The grants: none, at full access where the person chose it.
    grants: Grants,
    /// Who asks.
    grantee: Grantee,
}

impl AsTheMachineNames {
    /// The machine a turn told `may_reach` is.
    #[must_use]
    pub fn of(may_reach: &WhatItMayReach) -> Self {
        let grantee = Grantee::named(THE_AGENT);
        let installed = Installed::holding(
            may_reach
                .applications()
                .map(TheApplicationsInstalled::applications)
                .unwrap_or_default()
                .iter()
                .filter_map(|(identifier, name)| {
                    // **An application told by its identifier has no name**:
                    // the line gives the identifier in the name's place.
                    if name == identifier {
                        Application::identified(identifier).ok()
                    } else {
                        Application::called(identifier, name).ok()
                    }
                }),
        );
        let grants = if may_reach.is_at_full_access() {
            Grants::default().at_full_access_for(&grantee)
        } else {
            Grants::default()
        };
        Self {
            installed,
            grants,
            grantee,
        }
    }

    /// `given`, for `verb`, as that machine names an application.
    #[must_use]
    pub fn naming<'g>(
        &self,
        verbs: &Verbs,
        verb: &str,
        given: &[(&'g str, Given)],
    ) -> Vec<(&'g str, Given)> {
        alo_turn::as_this_machine_names_them(
            verbs,
            &self.installed,
            &self.grants,
            &self.grantee,
            verb,
            given,
            SystemTime::now(),
        )
    }
}

#[cfg(test)]
#[path = "as_the_machine_names_tests.rs"]
mod tests;
