//! The names a person gave the machines this one is paired with, as a list and
//! as they are written down.
//!
//! ```toml
//! format = 1
//!
//! [[machine]]
//! identity = "0f1e2d3c4b5a69788796a5b4c3d2e1f0"
//! called = "the reception machine"
//! ```
//!
//! # Beside the pairings, never in them
//!
//! `alo_nearby::keeping` writes exactly the row two people made, and a name is
//! not something two people made: one person gave it, on one machine, and the
//! other machine never hears it. A pairings row with a name in it would be a
//! pairing on terms nobody confirmed — `docs/contracts/pairings-file.md` says
//! there is no field for one — so the names are a list and a file of their own,
//! and nothing a pairing proves reads them.
//!
//! # A name outlives nothing its pairing does not
//!
//! Both [`read`] and [`MachineNames::only_those_paired`] take the pairings and
//! the moment, and keep a name only for a machine a pairing with stands at that
//! moment. So a name whose pairing was revoked, or ran out while the machine
//! was switched off, is gone when the list is read — the property the pairings
//! file has for its own rows, held here by the same means rather than by
//! whoever remembers to tidy.
//!
//! # Every name is checked again on the way in
//!
//! Each row is handed to [`MachineName::checked`], the rule the person's door
//! holds a name to, and a row that fails it refuses the **whole** file, for the
//! grants file's reason: a list that silently lost a name is a list that lies
//! about what a person reads.

use std::collections::BTreeMap;
use std::time::SystemTime;

use alo_nearby::{MachineId, Pairings};
use serde::{Deserialize, Serialize};

use crate::named::{MachineName, NotAName};

/// Which shape of names file this alo OS writes and reads.
pub const THE_NAMES_FORMAT: u32 = 1;

/// Why the names a machine kept were not read, or could not be written.
///
/// English, with a `Display`, for this crate's reason; and every arm means
/// **no names were read**.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotNames {
    /// The text is not the shape the names file takes.
    #[error("the names a machine keeps are not the shape they take: {why}")]
    NotTheShape {
        /// What would not parse, in the parser's words.
        why: String,
    },
    /// Names written for an alo OS this is not.
    #[error(
        "these names say format {format}, and this alo OS reads only format {}",
        THE_NAMES_FORMAT
    )]
    AnotherFormat {
        /// The number the file says.
        format: u32,
    },
    /// A row naming something that is not a machine's identity.
    #[error("the name in row {row} is for `{said}`, which is not a machine's identity")]
    NotAMachine {
        /// Which row, counting from one.
        row: usize,
        /// What it said.
        said: String,
    },
    /// A row whose name is not one a person may give.
    #[error("the name in row {row} is not a name a person may give a machine: {why}")]
    NotAName {
        /// Which row, counting from one.
        row: usize,
        /// Which part of the rule it fails.
        why: NotAName,
    },
    /// Two rows about one machine.
    #[error("the names file names `{identity}` twice, and a machine has one name")]
    TwoNamesForOneMachine {
        /// The machine named twice.
        identity: String,
    },
}

/// What the person here called each machine this one is paired with.
///
/// One name for each machine at most, keyed by identity. Nothing on it is
/// asked by a proof, a pairing or a grant.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MachineNames {
    /// By identity.
    called: BTreeMap<MachineId, MachineName>,
}

impl MachineNames {
    /// No names at all: every machine is spoken of by its identity.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            called: BTreeMap::new(),
        }
    }

    /// What the person called this machine, if they called it anything.
    #[must_use]
    pub fn called(&self, machine: &MachineId) -> Option<&MachineName> {
        self.called.get(machine)
    }

    /// Give this machine this name, replacing any it had.
    pub fn name(&mut self, machine: MachineId, name: MachineName) {
        self.called.insert(machine, name);
    }

    /// Take this machine's name away, answering whether it had one.
    pub fn forget(&mut self, machine: &MachineId) -> bool {
        self.called.remove(machine).is_some()
    }

    /// Every machine and its name, in the order of their identities.
    pub fn every(&self) -> impl Iterator<Item = (&MachineId, &MachineName)> {
        self.called.iter()
    }

    /// How many machines have a name.
    #[must_use]
    pub fn len(&self) -> usize {
        self.called.len()
    }

    /// Whether no machine has a name.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.called.is_empty()
    }

    /// Keep only the names of machines a pairing with stands at `now`,
    /// answering whether any went.
    pub fn only_those_paired(&mut self, pairings: &Pairings, now: SystemTime) -> bool {
        let before = self.called.len();
        self.called
            .retain(|machine, _| pairings.paired_with(machine, now));
        self.called.len() != before
    }
}

/// The file's shape, as serde sees it.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Kept {
    /// Which shape this file is in.
    format: u32,
    /// The names, one table each.
    #[serde(rename = "machine", default)]
    machines: Vec<KeptName>,
}

/// One machine's table: its identity and what the person called it.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct KeptName {
    /// The machine, by its identity.
    identity: String,
    /// What the person called it.
    called: String,
}

/// The names this text holds, believed only whole, keeping only those of
/// machines a pairing with stands at `now`.
///
/// # Errors
///
/// [`NotNames::NotTheShape`] for text that is not this file,
/// [`NotNames::AnotherFormat`] before any row is looked at, and for a row: not
/// an identity, not a name the rule allows, or a machine named twice — each
/// naming the row.
pub fn read(text: &str, pairings: &Pairings, now: SystemTime) -> Result<MachineNames, NotNames> {
    let kept: Kept = toml::from_str(text).map_err(|why| NotNames::NotTheShape {
        why: why.to_string(),
    })?;
    if kept.format != THE_NAMES_FORMAT {
        return Err(NotNames::AnotherFormat {
            format: kept.format,
        });
    }
    let mut names = MachineNames::none();
    for (at, one) in kept.machines.iter().enumerate() {
        let row = at.saturating_add(1);
        let machine = MachineId::read(&one.identity).map_err(|_| NotNames::NotAMachine {
            row,
            said: one.identity.clone(),
        })?;
        let name =
            MachineName::checked(&one.called).map_err(|why| NotNames::NotAName { row, why })?;
        if names.called(&machine).is_some() {
            return Err(NotNames::TwoNamesForOneMachine {
                identity: machine.as_str().to_owned(),
            });
        }
        names.name(machine, name);
    }
    names.only_those_paired(pairings, now);
    Ok(names)
}

/// These names as the file that keeps them.
///
/// Written as they stand: whoever holds the list decides which pairings they
/// belong to, with [`MachineNames::only_those_paired`], before handing it here.
///
/// # Errors
///
/// [`NotNames::NotTheShape`] if the value would not serialise, which strings
/// cannot cause.
pub fn written(names: &MachineNames) -> Result<String, NotNames> {
    let kept = Kept {
        format: THE_NAMES_FORMAT,
        machines: names
            .every()
            .map(|(machine, name)| KeptName {
                identity: machine.as_str().to_owned(),
                called: name.as_str().to_owned(),
            })
            .collect(),
    };
    toml::to_string(&kept).map_err(|why| NotNames::NotTheShape {
        why: why.to_string(),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::testing::{noon, the_studio_paired_with_reception};

    /// The machine the fixture pairs the studio with.
    fn reception() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// Reception, called what the studio's person calls it.
    fn named_reception() -> MachineNames {
        let mut names = MachineNames::none();
        names.name(
            reception(),
            MachineName::checked("the reception machine").unwrap(),
        );
        names
    }

    /// **What was written reads back**, name for name, while the pairing
    /// stands.
    #[test]
    fn names_written_read_back_while_the_pairing_stands() {
        let (pairings, _) = the_studio_paired_with_reception();
        let text = written(&named_reception()).unwrap();
        assert!(
            text.contains("called = \"the reception machine\""),
            "{text}"
        );
        assert_eq!(read(&text, &pairings, noon()).unwrap(), named_reception());
    }

    /// **A name outlives nothing its pairing does not**: read against no
    /// pairing, or after the pairing ended, the name is not on the list.
    #[test]
    fn a_name_whose_pairing_is_gone_is_not_read_back() {
        let (pairings, _) = the_studio_paired_with_reception();
        let text = written(&named_reception()).unwrap();
        assert!(read(&text, &Pairings::none(), noon()).unwrap().is_empty());
        let two_days_later = noon() + Duration::from_secs(2 * 86_400);
        assert!(read(&text, &pairings, two_days_later).unwrap().is_empty());

        let mut held = named_reception();
        assert!(held.only_those_paired(&Pairings::none(), noon()));
        assert!(held.is_empty());
    }

    /// **A file hand-edited into a row the rule would not allow is refused
    /// whole**: not an identity, a name that reads as one, a line break, a
    /// machine named twice, a field nobody declared, and another format.
    #[test]
    fn a_hand_edited_names_file_is_refused_whole() {
        let (pairings, _) = the_studio_paired_with_reception();
        let id = reception();
        let id = id.as_str();
        for (text, refused) in [
            (
                "format = 1\n[[machine]]\nidentity = \"disan-laptop\"\ncalled = \"x\"\n".to_owned(),
                "not a machine",
            ),
            (
                format!("format = 1\n[[machine]]\nidentity = \"{id}\"\ncalled = \"{id}\"\n"),
                "identity",
            ),
            (
                format!("format = 1\n[[machine]]\nidentity = \"{id}\"\ncalled = \"a\\nb\"\n"),
                "control",
            ),
            (
                format!(
                    "format = 1\n[[machine]]\nidentity = \"{id}\"\ncalled = \"a\"\n\
                     [[machine]]\nidentity = \"{id}\"\ncalled = \"b\"\n"
                ),
                "twice",
            ),
            (
                format!(
                    "format = 1\n[[machine]]\nidentity = \"{id}\"\ncalled = \"a\"\ntrusted = true\n"
                ),
                "shape",
            ),
            ("format = 2\n".to_owned(), "format 2"),
        ] {
            let why = read(&text, &pairings, noon()).unwrap_err();
            assert!(why.to_string().contains(refused), "{why}: {text}");
        }
    }

    /// A name replaced is replaced, and one forgotten says whether there was
    /// one.
    #[test]
    fn a_name_is_replaced_and_forgotten() {
        let mut names = named_reception();
        names.name(reception(), MachineName::checked("the front desk").unwrap());
        assert_eq!(names.len(), 1);
        assert_eq!(
            names.called(&reception()).unwrap().as_str(),
            "the front desk"
        );
        assert!(names.forget(&reception()));
        assert!(!names.forget(&reception()));
    }
}
