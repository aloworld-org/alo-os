//! **What each application a fresh machine ships calls itself**, pinned from
//! the package it is installed from.
//!
//! A booted machine reads every installed application's desktop entry, and
//! a model is told *VSCodium (com.vscodium.codium)*; the machine then accepts
//! *VSCodium* as that application (`alo_applications::Installed::answering_to`).
//! A check that holds the model to what the machine accepts
//! (`alo-every-try`) has no machine to read, and the names are not ours to
//! write: they are each packager's `Name=`. So they are pinned here, in
//! `the-shipped-names.txt`, which is **generated** from the packages
//! themselves (`src/bin/alo-shipped-names.rs`) and never typed — rule 3 of
//! ADR 0156, one source for names.
//!
//! The name is the entry's **untranslated** `Name`, read by the machine's own
//! reader (`alo_installed::read`), which leaves every `Name[xx]` aside.

use alo_applications::Installed;
use alo_applications::application::{Application, Called};

use crate::shipped::Shipped;

/// The pinned names, built into this crate.
pub const THE_NAMES: &str = include_str!("../the-shipped-names.txt");

/// The comment every [`THE_NAMES`] begins with: what the file is, and that it
/// is generated.
pub const THE_HEADER: &str = concat!(
    "# The name each application a fresh machine ships gives itself: the\n",
    "# untranslated Name= of the desktop entry its package exports, one line each,\n",
    "# its identifier, a tab, and that name.\n",
    "#\n",
    "# GENERATED. Do not edit by hand. These names are not ours and cannot be read\n",
    "# off this repository: alo-shipping installs each application at first boot,\n",
    "# so they exist only in the package. `.github/workflows/shipped-names.yml`\n",
    "# installs every application in shipped.toml and runs\n",
    "#   alo-shipped-names <the folders their entries are exported to>\n",
    "# which prints what this file should hold and fails when it holds anything\n",
    "# else. It never writes this file: a person copies what it printed.\n",
    "#\n",
);

/// Why the names could not be read or made.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotNamed {
    /// A line that is not an identifier, a tab and a name.
    #[error("not an identifier, a tab and a name: {line:?}")]
    NotALine {
        /// The line.
        line: String,
    },
    /// An identifier named twice.
    #[error("{identifier} is named twice")]
    Twice {
        /// The identifier.
        identifier: String,
    },
    /// An identifier no fresh machine ships.
    #[error("{identifier} is not shipped (shipped.toml)")]
    NotShipped {
        /// The identifier.
        identifier: String,
    },
    /// The shipped list itself would not read.
    #[error("shipped.toml would not read: {why}")]
    TheListWouldNotRead {
        /// Why.
        why: String,
    },
    /// A shipped application with no entry, or an entry that gives no name,
    /// where its entries were read.
    #[error("{identifier} has no desktop entry where the entries were read")]
    NoEntry {
        /// The identifier.
        identifier: String,
    },
}

/// Each shipped application's name, in `shipped.toml`'s order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShippedNames {
    /// Identifier and name.
    named: Vec<(String, String)>,
}

impl ShippedNames {
    /// The names pinned in this crate.
    ///
    /// # Errors
    /// [`NotNamed`], which a test makes impossible for the file that ships.
    pub fn pinned() -> Result<Self, NotNamed> {
        let shipped = Shipped::decided().map_err(|why| NotNamed::TheListWouldNotRead {
            why: format!("{why:?}"),
        })?;
        Self::read(THE_NAMES, &shipped)
    }

    /// Names from `text`, every identifier one `shipped` holds. A line that is
    /// empty or begins with `#` is a comment.
    ///
    /// # Errors
    /// [`NotNamed`], naming the first line that does not hold.
    pub fn read(text: &str, shipped: &Shipped) -> Result<Self, NotNamed> {
        let mut named: Vec<(String, String)> = Vec::new();
        for line in text.lines() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let not_a_line = || NotNamed::NotALine {
                line: line.to_owned(),
            };
            let (identifier, name) = line.split_once('\t').ok_or_else(not_a_line)?;
            let (identifier, name) = (identifier.trim(), name.trim());
            if identifier.is_empty() || name.is_empty() {
                return Err(not_a_line());
            }
            if !shipped
                .every()
                .iter()
                .any(|pinned| pinned.application().identifier() == identifier)
            {
                return Err(NotNamed::NotShipped {
                    identifier: identifier.to_owned(),
                });
            }
            if named.iter().any(|(already, _)| already == identifier) {
                return Err(NotNamed::Twice {
                    identifier: identifier.to_owned(),
                });
            }
            named.push((identifier.to_owned(), name.to_owned()));
        }
        Ok(Self { named })
    }

    /// The name `identifier` gives itself, where it is pinned.
    #[must_use]
    pub fn name_of(&self, identifier: &str) -> Option<&str> {
        self.named
            .iter()
            .find(|(named, _)| named == identifier)
            .map(|(_, name)| name.as_str())
    }

    /// Every identifier and name, in order.
    #[must_use]
    pub fn every(&self) -> &[(String, String)] {
        &self.named
    }
}

/// **What [`THE_NAMES`] should hold**, read off `installed` — the entries the
/// packages exported, read by `alo_installed::read` — for every application
/// `shipped` holds: the file's own header, then one line each.
///
/// # Errors
/// [`NotNamed::NoEntry`], for the first shipped application `installed` has
/// no entry for.
pub fn as_their_entries_name_them(
    shipped: &Shipped,
    installed: &Installed,
) -> Result<String, NotNamed> {
    let mut made = THE_HEADER.to_owned();
    for pinned in shipped.every() {
        let identifier = pinned.application().identifier();
        // **The packager's own words only**: an entry read off a disk always
        // gives those, and an application with none has no name to pin.
        let name = installed
            .knows(identifier)
            .and_then(Application::name)
            .and_then(|called| match called {
                Called::ByWhoeverPackagedIt(name) => Some(name.as_str()),
                Called::InOurOwnWords(_) => None,
            })
            .ok_or_else(|| NotNamed::NoEntry {
                identifier: identifier.to_owned(),
            })?;
        made.push_str(&format!("{identifier}\t{name}\n"));
    }
    Ok(made)
}

#[cfg(test)]
#[path = "shipped_names_tests.rs"]
mod tests;
