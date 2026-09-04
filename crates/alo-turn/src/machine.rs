//! What every turn on this machine happens against, and what it can carry out.
//!
//! Six things a turn needs that are not the turn's own: the verbs an agent may
//! ask for, the words the person in front of the machine reads, how a path is
//! resolved, what a verb's work runs inside, what is leaving right now, and
//! where what happened is written down. They are made once, when the daemon
//! starts, and every turn borrows them.
//!
//! # A machine cannot be made without a boundary
//!
//! [`Bounding`] is held here for the reason [`crate::Kept`] is: the promise is
//! only structural if a machine cannot exist without one. ADR 0015 says a turn
//! whose boundary cannot be applied does not run, so there is no honest
//! fallback — a machine that could carry a verb out with nothing around it
//! would be the guarantee turned off wherever somebody forgot, which is exactly
//! what ADR 0013 says is wrong with promising it in our own code.
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
//! Asking a model something is **not** one of those verbs, and the name of the
//! constructor stays right because of it: a question an agent puts to a model
//! is what an agent *is* rather than something it is granted, so it is a door
//! on [`crate::Turning`] and not a row in the registry. `alo-asking` never asks
//! the grants anything.
//!
//! # The indicator is the machine's, and the rule in force is not
//!
//! [`Indicator`] is held here for the reason the record is: **one machine has
//! one of each, and a second would be a second place to look.** Item 16 settled
//! that on the indicator itself — what alo OS does on its own goes on the same
//! list as what an agent causes, because the failure law 1 exists to prevent is
//! not *the policy was wrong* but *nobody could see it* — and a turn handed an
//! indicator of its own would put two turns on one machine on two lists. A
//! machine that could run a turn without one would make law 1's surface
//! optional equipment.
//!
//! What is **not** held here is `alo_models::SourcePolicy`. That is a rule an
//! organisation sets and can tighten while a turn is open, so it is passed to
//! the door at the moment a question is asked — exactly as the grants are, and
//! for item 3's reason: the rule that counts is the one in force now, not the
//! one that was in force when somebody pressed a key.
//!
//! # It holds no grants and no clock
//!
//! The grants are the machine's, and they are passed to each door rather than
//! held here: a grant revoked while a turn is open takes effect at the next
//! question asked of them (item 3), and a list borrowed once at the start of a
//! turn would be a list that could not change under it. Nothing here reads a
//! clock either, as everywhere else in this workspace.

use alo_capability::Verbs;
use alo_egress::Indicator;
use alo_files::{Declaring, Resolving, file_verbs};
use alo_strings::Strings;

use crate::bounding::Bounding;
use crate::kept::Kept;
use crate::shortening::{Shortened, Shortening};

/// The verbs, the words, the resolver, the boundary, the indicator and the
/// record: one machine's, shared by every turn on it.
///
/// Not `Clone`, and it holds the boundary, the indicator and the record by
/// exclusive borrow: two of these would be two turns writing into one record
/// with no order between them, and the record is a file whose lines are read as
/// a sequence.
pub struct Machine<'a> {
    /// What an agent may ask for, which is what this machine can carry out.
    verbs: Verbs,
    /// The words the person in front of this machine reads.
    strings: &'a Strings,
    /// Where a path really leads.
    resolving: &'a dyn Resolving,
    /// What a verb's work runs inside.
    ///
    /// Exclusive, because imposing one is a thing that happens to a kernel: an
    /// entry is written into a map, a thread goes into a control group and
    /// comes back out of it, and two turns doing that at once through one
    /// boundary would be two threads in one turn's cgroup.
    bounding: &'a mut dyn Bounding,
    /// What is leaving this machine right now.
    indicator: &'a mut Indicator,
    /// Where what happened is written down, and the one thing that can shorten
    /// it.
    ///
    /// The larger of the two traits, because a machine has a record and a turn
    /// only writes to one: [`Machine::kept`] hands a turn the smaller one, and
    /// [`Machine::shorten`] is the door the other half goes through.
    kept: &'a mut dyn Shortening,
}

impl<'a> Machine<'a> {
    /// A machine that can carry out the six file verbs, and offers those.
    ///
    /// The resolver is taken rather than assumed so that the decisions in this
    /// crate are testable on a machine where making a symbolic link needs a
    /// privilege — `alo_files::OnThisMachine` is the one that ships, and
    /// `alo_files::Resolving` says why there is only one.
    ///
    /// The boundary is taken for the reason the record is, and it is the
    /// stronger half of the same argument: what imposes one is a kernel, this
    /// crate is portable, and a machine that could be made without one would be
    /// a machine able to run a verb with nothing around it.
    ///
    /// The indicator is taken for a different reason again: it is the surface a
    /// person watches, so there is exactly one of it and the shell that draws
    /// it is the thing that owns it. A machine borrows it, and
    /// [`Machine::showing`] is how the shell reads it back.
    ///
    /// # Errors
    /// [`Declaring`], which the six as they are written cannot cause. It is a
    /// `Result` rather than an unwrap for the reason `alo-files` gives: a
    /// library that panics inside the daemon takes the daemon with it.
    pub fn carrying_out_file_verbs(
        strings: &'a Strings,
        resolving: &'a dyn Resolving,
        bounding: &'a mut dyn Bounding,
        indicator: &'a mut Indicator,
        kept: &'a mut dyn Shortening,
    ) -> Result<Self, Declaring> {
        Ok(Self {
            verbs: file_verbs()?,
            strings,
            resolving,
            bounding,
            indicator,
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
    ///
    /// Answers for as long as the words themselves live rather than for as
    /// long as this borrow does, so a caller can hold the words and then ask
    /// the machine for something else — which is what a turn does between
    /// wording a refusal and writing it down.
    #[must_use]
    pub fn strings(&self) -> &'a Strings {
        self.strings
    }

    /// What is leaving this machine right now.
    ///
    /// What a shell draws law 1's indicator from. Borrowed rather than lent
    /// mutably: only a turn, and alo OS on its own errands, put anything on it.
    #[must_use]
    pub fn showing(&self) -> &Indicator {
        self.indicator
    }

    /// Where a path really leads.
    ///
    /// Answers for as long as the resolver lives rather than for as long as
    /// this borrow does, as [`Machine::strings`] does: a turn resolves a call's
    /// paths and then asks the machine for the boundary to run it inside, and
    /// those are two things it holds at once.
    pub(crate) fn resolving(&self) -> &'a dyn Resolving {
        self.resolving
    }

    /// What a verb's work runs inside.
    ///
    /// `pub(crate)`, and the only caller is `carrying.rs`: a boundary is
    /// put around an execution and around nothing else, and a public door onto
    /// it would be a way to run something in a turn's cgroup that no
    /// authorisation reached.
    pub(crate) fn bounding(&mut self) -> &mut dyn Bounding {
        self.bounding
    }

    /// What is leaving, to be shown something else.
    ///
    /// `pub(crate)`: the only callers are the doors that put a question
    /// somewhere, and they hand it straight to `alo_egress::Indicator::beginning`
    /// by way of `alo-asking`. A public one would be a way to take a line off
    /// the indicator without the connection it belongs to having ended.
    pub(crate) fn indicator(&mut self) -> &mut Indicator {
        self.indicator
    }

    /// Where what happened is written down.
    ///
    /// `pub(crate)`, and the smaller of the two traits on purpose: what a turn
    /// is handed can write an entry and has no way to remove one.
    pub(crate) fn kept(&mut self) -> &mut dyn Kept {
        self.kept
    }

    /// Shorten this machine's record under the rule it is kept by.
    ///
    /// Public, because the caller is the service that holds the machine and
    /// reads the clock (`alo-agentd`, queue item 20) rather than anything
    /// inside a turn. It is a door onto the machine and not onto a turn: a
    /// [`crate::Turning`] hands nobody a `&mut Machine`, so nothing an agent can
    /// ask for arrives here, and what is removed is decided by a rule and a
    /// moment either way.
    ///
    /// # Errors
    ///
    /// [`alo_keeping::NotKept`], in every one of which nothing has been
    /// removed — see [`crate::Shortening::shorten`].
    pub fn shorten(
        &mut self,
        keeping: alo_keeping::Keeping,
        now: std::time::SystemTime,
    ) -> Result<Shortened, alo_keeping::NotKept> {
        self.kept.shorten(keeping, now)
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
    use crate::testing::{NothingIsBounded, in_english};
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
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = NothingIsBounded;
        let machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();

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
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = NothingIsBounded;
        let machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        assert_eq!(format!("{machine:?}"), "Machine { verbs: 6, .. }");
    }

    /// **A machine that has done nothing is showing nothing**, and the shell
    /// reads that off the machine rather than keeping a second copy of law 1's
    /// surface beside it.
    #[test]
    fn a_machine_that_has_caused_nothing_to_leave_is_quiet() {
        let strings = in_english();
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = NothingIsBounded;
        let machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        assert!(machine.showing().is_quiet());
        assert!(machine.showing().showing().is_empty());
    }
}
