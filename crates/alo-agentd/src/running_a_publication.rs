//! **The road a publication's command runs on: the command road**
//! ([ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)).
//!
//! The owner's decision of 2026-11-13: *"From the terminal" means the person
//! asks alo and alo runs git on its own command road — not typing into the
//! person's terminal window.* So a publication runs exactly where a command
//! alo runs does: `alo_running::run_as_the_person`, `/bin/sh -c`, as the
//! person, in their home, told nobody is there to answer
//! (`alo_running::nobody_is_there_to_answer`), for at most two minutes. The
//! line is `alo_publishing::the_line`'s, never a model's.
//!
//! **What the level decides is not decided here.** At full access a
//! publication is carried out without asking, at *ask me each time* after the
//! OK, and at the sealed box it is refused before anybody is asked
//! (`crate::publishing_at_this_level`) — so by the time anything reaches this
//! file, the person's level has already said yes.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use alo_capability::{ACommand, ItDidNotStart, WhatACommandDid};
use alo_egress::Departing;
use alo_publishing::Runs;

/// [`AsThePerson::on_this_machine`], made once for the life of the service —
/// the machine holds it for as long as it holds every other executor.
static ON_THIS_MACHINE: LazyLock<AsThePerson> = LazyLock::new(AsThePerson::on_this_machine);

/// What runs a publication on this machine, for as long as the service runs.
#[must_use]
pub fn here() -> &'static AsThePerson {
    &ON_THIS_MACHINE
}

/// Runs a publication's command as the person, from their home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsThePerson {
    /// Where the command starts.
    home: PathBuf,
}

impl AsThePerson {
    /// From `home`: the person's home on a machine.
    #[must_use]
    pub fn at_home(home: &Path) -> Self {
        Self {
            home: home.to_owned(),
        }
    }

    /// From the home this service was started with, or `/` where it has none
    /// — what a command reaches is the account's permissions either way.
    #[must_use]
    pub fn on_this_machine() -> Self {
        let home = std::env::var_os(alo_choosing::HOME)
            .map(PathBuf::from)
            .filter(|home| home.is_absolute())
            .unwrap_or_else(|| PathBuf::from("/"));
        Self::at_home(&home)
    }
}

impl Runs for AsThePerson {
    fn run(&self, line: &ACommand, _leaving: &Departing) -> Result<WhatACommandDid, ItDidNotStart> {
        alo_running::run_as_the_person(line, &self.home)
    }
}
