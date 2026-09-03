//! What every turn on this machine happens against, and what it can carry out.
//!
//! Four things a turn needs that are not the turn's own: the verbs an agent may
//! ask for, the words the person in front of the machine reads, how a path is
//! resolved, and where what happened is written down. They are made once, when
//! the daemon starts, and every turn borrows them.
//!
//! # The list an agent may ask from is the list this machine can carry out
//!
//! [`Machine::carrying_out_file_verbs`] builds the registry rather than
//! receiving one, and that is the decision this file exists for. A registry
//! handed in from outside could hold a verb nothing here can execute — the
//! application verbs are declared and portable, and what actually moves a
//! window is Wayland and D-Bus on a Linux host — and an agent asking for one
//! would be told *the machine could not*, which is a sentence about a disk
//! being full rather than about a capability this machine does not have.
//!
//! So the two lists are one list. A verb an agent can name is a verb something
//! here can do, and adding an executor and adding to the offered list are the
//! same edit. The name says which executors there are, so the day there is a
//! second one the name is wrong until somebody fixes it — which is what it is
//! for.
//!
//! # It holds no grants and no clock
//!
//! The grants are the machine's, and they are passed to each door rather than
//! held here: a grant revoked while a turn is open takes effect at the next
//! question asked of them (item 3), and a list borrowed once at the start of a
//! turn would be a list that could not change under it. Nothing here reads a
//! clock either, as everywhere else in this workspace.

use alo_capability::Verbs;
use alo_files::{Declaring, Resolving, file_verbs};
use alo_strings::Strings;

use crate::kept::Kept;

/// The verbs, the words, the resolver and the record: one machine's, shared by
/// every turn on it.
///
/// Not `Clone`, and it holds the record by exclusive borrow: two of these would
/// be two turns writing into one record with no order between them, and the
/// record is a file whose lines are read as a sequence.
pub struct Machine<'a> {
    /// What an agent may ask for, which is what this machine can carry out.
    verbs: Verbs,
    /// The words the person in front of this machine reads.
    strings: &'a Strings,
    /// Where a path really leads.
    resolving: &'a dyn Resolving,
    /// Where what happened is written down.
    kept: &'a mut dyn Kept,
}

impl<'a> Machine<'a> {
    /// A machine that can carry out the six file verbs, and offers those.
    ///
    /// The resolver is taken rather than assumed so that the decisions in this
    /// crate are testable on a machine where making a symbolic link needs a
    /// privilege — `alo_files::OnThisMachine` is the one that ships, and
    /// `alo_files::Resolving` says why there is only one.
    ///
    /// # Errors
    /// [`Declaring`], which the six as they are written cannot cause. It is a
    /// `Result` rather than an unwrap for the reason `alo-files` gives: a
    /// library that panics inside the daemon takes the daemon with it.
    pub fn carrying_out_file_verbs(
        strings: &'a Strings,
        resolving: &'a dyn Resolving,
        kept: &'a mut dyn Kept,
    ) -> Result<Self, Declaring> {
        Ok(Self {
            verbs: file_verbs()?,
            strings,
            resolving,
            kept,
        })
    }

    /// What an agent may ask for on this machine.
    ///
    /// A shell reads this to show a person what their agent can do, and it is
    /// the whole answer: there is no second list of things that are permitted
    /// but not offered.
    #[must_use]
    pub fn verbs(&self) -> &Verbs {
        &self.verbs
    }

    /// The words the person in front of this machine reads.
    #[must_use]
    pub fn strings(&self) -> &Strings {
        self.strings
    }

    /// Where a path really leads.
    pub(crate) fn resolving(&self) -> &dyn Resolving {
        self.resolving
    }

    /// Where what happened is written down.
    pub(crate) fn kept(&mut self) -> &mut dyn Kept {
        self.kept
    }
}

impl std::fmt::Debug for Machine<'_> {
    /// Written by hand because neither the resolver nor the record is a value
    /// that can be printed, and because what a reader wants here is what this
    /// machine offers rather than the addresses of two trait objects.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Machine")
            .field("verbs", &self.verbs.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use alo_capability::Verb;
    use alo_files::OnThisMachine;
    use alo_record::Record;

    /// **The offered list is the list this machine can carry out**, which is
    /// the six file verbs and nothing else. An application verb is declared,
    /// portable and unreachable here, because what moves a window is not in
    /// this repository.
    #[test]
    fn a_machine_offers_exactly_what_it_can_carry_out() {
        let strings = in_english();
        let mut record = Record::default();
        let machine =
            Machine::carrying_out_file_verbs(&strings, &OnThisMachine, &mut record).unwrap();

        let names: Vec<_> = machine.verbs().all().map(Verb::name).collect();
        assert_eq!(
            names,
            [
                "list_folder",
                "read_file",
                "find_in_folder",
                "rename_file",
                "move_file",
                "archive_folder"
            ]
        );
        assert!(
            machine.verbs().of("open_application").is_none(),
            "a verb nothing here can carry out is on the list an agent asks from"
        );
    }

    /// What a reader of a debug line wants is what this machine offers, not two
    /// trait objects it cannot print.
    #[test]
    fn what_a_machine_prints_is_what_it_offers() {
        let strings = in_english();
        let mut record = Record::default();
        let machine =
            Machine::carrying_out_file_verbs(&strings, &OnThisMachine, &mut record).unwrap();
        assert_eq!(format!("{machine:?}"), "Machine { verbs: 6, .. }");
    }
}
