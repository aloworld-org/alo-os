//! The boundary one execution runs inside, and the seam a portable crate can
//! hold it through.
//!
//! ADR 0013 says what is wrong without one: `alo-record` writes down what
//! `alo-agentd` reports, so a verb with a bug in it writes a lie in the person's
//! own language with a grant id beside it. ADR 0015 is the mechanism —
//! a control group per turn, a BPF LSM on the kernel's security hooks, and an
//! entry that is removed when the turn ends. This is where a turn meets it.
//!
//! # Why a trait, and why it is shaped like [`crate::Kept`]
//!
//! A boundary is a kernel, a control group and a programme the verifier
//! accepted, and this crate is portable — it compiles on hosts that have none of
//! those. So what goes in is the same shape the record went in as: **one
//! interface, and the implementation that is real is held by the machine**, with
//! no constructor that does without one. [`crate::Machine`] takes a [`Bounding`]
//! and there is no way to make one that does not.
//!
//! **Nothing in any library here implements it except the machine's own.** A
//! type in this crate that bounded nothing would be the guarantee turned off by
//! default on every host, which ADR 0015 leaves no room for: a turn whose
//! boundary cannot be applied does not run, so there is no honest fallback to
//! write. What a *test* needs is four lines and belongs in the test that needs
//! it, which is where `alo-files`' fixtures live for the same reason.
//!
//! # What goes inside, and what must not
//!
//! The order is the security property, and it is `carrying.rs`'s to keep:
//!
//! ```text
//! resolve            outside — a thread cannot look up what it may not open yet
//! ask for the reach  outside — arithmetic about paths, and it names the bound
//! bound              the kernel is told, before the thread is in the cgroup
//! run the verb       inside  — this is the only thing in here
//! come back out      and only then
//! write the entry    outside — a thread bounded across it would be refused the record
//! ```
//!
//! [`Doing`] is what goes in, and it is deliberately the whole of it: an
//! implementation is handed one thing it can do and one way to do it. It cannot
//! be made outside this crate — `Doing::of` is `pub(crate)` — so a boundary
//! cannot manufacture an execution, only carry one out.
//!
//! **What must not go inside is anything else at all.** A panic in the work
//! prints a backtrace, and reading `/proc/self/maps` is an open like any other;
//! `alo-bounding` says the same thing from the other side, and this file is the
//! caller that honours it.

use alo_capability::{Grants, Refused};
use alo_files::{Did, Reaching, Touching};
use alo_strings::Strings;

use crate::unbounded::NoBoundary;

/// What one execution came to.
///
/// `Ok` when the machine was asked and answered — including when the disk
/// refused, which travels inside the [`Did`] as `alo-files` decided in item 6a.
/// `Err` when the grants refused a name the call would have created, which is
/// the last question the capability model asks and the only one asked inside a
/// boundary.
pub type Done = Result<Did, Refused>;

/// One execution, ready to be carried out, and the only thing that can be done
/// with it.
///
/// Made by this crate's `carrying.rs` and by nothing else. It holds what the capability
/// model has finished deciding — a call that was validated, permitted, approved
/// if it changes anything, and resolved — and the two things doing it needs: the
/// grants, for the one question left, and the words, for the refusal if that
/// question is answered no.
#[derive(Debug)]
pub struct Doing<'a> {
    /// The call, resolved and asked about again where every path really leads.
    touching: Touching,

    /// The list, asked once more inside about anything this would create.
    grants: &'a Grants,

    /// The words a refusal made in there is worded with.
    strings: &'a Strings,
}

impl<'a> Doing<'a> {
    /// One execution, ready to run.
    ///
    /// `pub(crate)`: an implementation of [`Bounding`] receives these and cannot
    /// make one, so there is no way for something holding a boundary to run
    /// something nobody authorised.
    pub(crate) fn of(touching: Touching, grants: &'a Grants, strings: &'a Strings) -> Self {
        Self {
            touching,
            grants,
            strings,
        }
    }

    /// Do it.
    ///
    /// Taken by value, so an implementation cannot run one execution twice —
    /// which is `alo_capability::Approved::redeem`'s rule arriving at the last
    /// place it could still be broken.
    pub fn done(self) -> Done {
        Did::of(self.touching, self.grants, self.strings)
    }
}

/// What puts a boundary around a turn's work on this machine.
///
/// One implementation is real and it is the machine's; see this module's
/// documentation for why there is no second one in any library here.
pub trait Bounding {
    /// Carry one execution out inside a boundary, and nothing else inside it.
    ///
    /// `reaching` is everywhere this execution has to be able to open —
    /// `alo_files::Reaching` is the resolved paths plus the folder above
    /// anything the call would create, and item 26c argues it — so an
    /// implementation turns those into whatever its kernel knows a place by
    /// rather than deciding which paths a verb needs.
    ///
    /// # Errors
    /// [`NoBoundary`] when there was none to run this inside, and ADR 0015's
    /// rule applies to every one of them: nothing ran, and the turn does not
    /// carry on. An implementation that answered `Ok` having bounded nothing
    /// would be one that turns this guarantee off silently.
    fn carrying_out(&mut self, reaching: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary>;
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{
        NothingIsBounded, a_folder_of_our_own, as_given, files, granting, in_english, noon, the_six,
    };
    use alo_capability::{Authorised, Given};
    use alo_files::{OnThisMachine, Touching};
    use std::fs;

    /// A read of a real folder, resolved and ready to be carried out.
    fn an_execution(folder: &std::path::Path, grants: &alo_capability::Grants) -> Touching {
        let call = the_six()
            .call("list_folder", &[("folder", as_given(folder))])
            .unwrap();
        let authorised = Authorised::read(&call, &files(), grants, noon()).unwrap();
        Touching::of(authorised, grants, &OnThisMachine, &in_english()).unwrap()
    }

    /// **What a boundary is handed is one thing it can do**, and doing it is
    /// what the machine would have done without one. The fixture is four lines
    /// and lives in a test, which is the whole of what this file offers.
    #[test]
    fn what_a_boundary_is_handed_carries_the_execution_out() {
        let folder = a_folder_of_our_own("bounding-doing");
        fs::write(folder.join("march.pdf"), "March").unwrap();
        let grants = granting(&[&folder]);
        let strings = in_english();

        let done = NothingIsBounded
            .carrying_out(
                &Reaching::of(&an_execution(&folder, &grants)).unwrap(),
                Doing::of(an_execution(&folder, &grants), &grants, &strings),
            )
            .unwrap();

        let did = done.unwrap();
        assert_eq!(did.answer().unwrap().listed().unwrap().things().len(), 1);
    }

    /// **The reach is made outside and handed in**, so what a boundary is told
    /// to allow is what this execution named rather than everywhere its grant
    /// reaches. Item 26b decided that; this is the seam it arrives through.
    #[test]
    fn a_boundary_is_told_what_this_execution_reaches_and_not_what_it_was_granted() {
        let folder = a_folder_of_our_own("bounding-reach");
        let elsewhere = a_folder_of_our_own("bounding-reach-elsewhere");
        fs::write(folder.join("march.pdf"), "March").unwrap();
        let grants = granting(&[&folder, &elsewhere]);

        let reaching = Reaching::of(&an_execution(&folder, &grants)).unwrap();
        assert!(reaching.holds(&folder));
        assert!(
            !reaching.holds(&elsewhere),
            "a folder this execution never named would have been inside its boundary"
        );
    }

    /// A verb that names nothing is not one of the six, so there is nothing for
    /// a boundary to be drawn around and the machine says so before anything is
    /// carried out — `alo-files`' answer, met here because this is where it is
    /// asked.
    #[test]
    fn an_execution_that_reaches_nothing_never_reaches_a_boundary() {
        let folder = a_folder_of_our_own("bounding-nothing");
        let grants = granting(&[&folder]);
        let call = the_six()
            .call(
                "find_in_folder",
                &[
                    ("folder", as_given(&folder)),
                    ("named", Given::text("march")),
                    ("most", Given::number(4)),
                ],
            )
            .unwrap();
        let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();
        let touching = Touching::of(authorised, &grants, &OnThisMachine, &in_english()).unwrap();

        // One of the six reaches at least one place, always: what would have no
        // boundary at all is a verb that is not on this machine's list.
        assert_eq!(Reaching::of(&touching).unwrap().len(), 1);
    }
}
