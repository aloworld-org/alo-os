//! **A person chooses one application for alo to use**, by name, from the
//! ones this machine has.
//!
//! [ADR 0143](../../../docs/decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
//! Q10: letting alo use an application is *in Settings … one application, by
//! name*. Until this file nothing on the machine could make that grant —
//! `alo_capability::Reach::Application` existed, the daemon honoured it, every
//! application verb asked for it, and the only road to a grant was a folder.
//! So at the default level every application verb was refused before a person
//! was ever asked, and *open VSCodium* worked only at full access.
//!
//! # The same rules as a folder, because it is the same act
//!
//! [`PickedApplication`] is sealed the way [`crate::Picked`] is: it has no
//! public constructor, and the only way to hold one is
//! [`Applications::pick`] — an application that was **in the list a person
//! was shown** and was chosen from it. So *alo is given an application nobody
//! chose* is not a branch somebody remembered to refuse; there is no value to
//! grant. The grant itself is still [`crate::Granting::of`], so how long it
//! lasts is asked the same way and checked by `alo-capability` the same way.
//!
//! # What is listed, and what is not
//!
//! - **Matched exactly**, as `alo_capability::Reach` matches: an application
//!   is its desktop file identifier, and a different case is a different
//!   application.
//! - **A terminal is not listed.** Whatever is typed into one runs, so a grant
//!   over a terminal is a grant to the whole machine by another road and
//!   `alo_capability::Grant::checked` refuses it (`GrantError::APersonsOwn`).
//!   ADR 0143 Q10 says *the terminal stays ADR 0111's own choice*, so it is
//!   left out here rather than drawn as a row that can only be refused.
//! - **Each application once**, the first reading kept — XDG's order puts a
//!   person's own copy of an entry first.
//! - **By the name a person reads**, and by its identifier where its entry gave
//!   none. A name is never guessed from an identifier.

use alo_capability::{Reach, is_a_persons_own};

use crate::picked::Chosen;
use crate::refusing::NotPicked;

/// One application a person may choose, as it is shown to them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    /// What the machine knows it by: the desktop file identifier, which is
    /// what every application verb asks a grant for.
    identifier: String,
    /// What a person reads: the name its entry gave, or the identifier.
    called: String,
}

impl Listed {
    /// What the machine knows it by.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// What a person reads.
    #[must_use]
    pub fn called(&self) -> &str {
        &self.called
    }
}

/// The applications a person is choosing among.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Applications {
    /// In the order they are shown.
    listed: Vec<Listed>,
}

/// An application a person chose.
///
/// Sealed: holding one is knowing that it was in the list a person was shown
/// and that they chose it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickedApplication {
    /// The application, as it was listed.
    listed: Listed,
}

impl PickedApplication {
    /// What the machine knows it by.
    #[must_use]
    pub fn identifier(&self) -> &str {
        self.listed.identifier()
    }

    /// The name it was listed under.
    #[must_use]
    pub fn called(&self) -> &str {
        self.listed.called()
    }

    /// What a grant made from this pick covers: **this application** and
    /// nothing else.
    #[must_use]
    pub fn reach(&self) -> Reach {
        Reach::Application(self.listed.identifier.clone())
    }
}

impl Applications {
    /// The applications a person may choose among, from what this machine
    /// has: each identifier with the name its entry gave, if it gave one.
    #[must_use]
    pub fn of(here: impl IntoIterator<Item = (String, Option<String>)>) -> Self {
        let mut listed: Vec<Listed> = Vec::new();
        for (identifier, called) in here {
            // **Not an application, rather than one tidied into being**: an
            // identifier with nothing in it, or with space around it, is not
            // one a verb could ask for, and trimming it would list a name the
            // grant would then be made over instead.
            if identifier.is_empty() || identifier.trim() != identifier {
                continue;
            }
            if is_a_persons_own(&identifier) || listed.iter().any(|it| it.identifier == identifier)
            {
                continue;
            }
            let called = called
                .filter(|called| !called.trim().is_empty())
                .unwrap_or_else(|| identifier.clone());
            listed.push(Listed { identifier, called });
        }
        // **By the name a person reads, ignoring case**, as a launcher lists
        // them; the identifier breaks a tie so the order never depends on the
        // order the entries were read in.
        listed.sort_by(|a, b| {
            a.called
                .to_lowercase()
                .cmp(&b.called.to_lowercase())
                .then_with(|| a.identifier.cmp(&b.identifier))
        });
        Self { listed }
    }

    /// The applications, in the order they are shown.
    #[must_use]
    pub fn listed(&self) -> &[Listed] {
        &self.listed
    }

    /// Whether there is nothing to choose.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.listed.is_empty()
    }

    /// The person chose the application of this identifier.
    ///
    /// # Errors
    /// [`NotPicked::NotAnApplicationShown`] when it is not in the list.
    pub fn pick(&self, identifier: &str) -> Result<Chosen, NotPicked> {
        self.listed
            .iter()
            .find(|listed| listed.identifier == identifier)
            .map(|listed| {
                Chosen::Application(PickedApplication {
                    listed: listed.clone(),
                })
            })
            .ok_or(NotPicked::NotAnApplicationShown)
    }
}

#[cfg(test)]
#[path = "applications_tests.rs"]
mod tests;
