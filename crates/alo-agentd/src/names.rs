//! What the person here called the machines this one is paired with, held while
//! the service runs — and the one `alo_corridor::Naming` the service answers
//! with.
//!
//! Every surface the local network built names the other machine through
//! `alo_corridor::Naming`: the indicator and the record entry a question down
//! the corridor leaves (`crate::corridor`), the origin a verb or a question from
//! a paired machine arrives under (`crate::hearing`, `crate::questioned`), and
//! the list a shell draws (`crate::pairing`). [`TheNames`] is what answers it on
//! a running machine, out of what `alo-remembering` read back at start and what
//! the person's door (`crate::naming_machines`) has changed since.
//!
//! # A name decides nothing
//!
//! [`TheNames`] is asked for a name and for nothing else. No proof, pairing or
//! grant reads it — `alo_nearby::Origin` keeps the identity as the principal and
//! the name beside it for reading — and nothing that finds or dials a machine
//! is handed it. What a wrong name could do is mislabel what a person reads,
//! which is why the rule a name is held to (`alo_remembering::MachineName`) is
//! about exactly that.
//!
//! # Its own lock, and always taken second
//!
//! The names are asked while the network's lock is held — a verb from a paired
//! machine is judged under it, and its origin is named there — so they cannot be
//! behind that lock: asking would wait on itself. They are behind a lock of
//! their own, and **it is always the second taken**: whoever changes a name
//! holds the network's lock first and hands the pairings in, and nothing holding
//! this lock ever reaches for the network's. One order, so no two threads can
//! each hold the lock the other wants.
//!
//! # A name goes with its pairing
//!
//! Every change is made against the pairings as they stand: a name is given
//! only to a machine a pairing with stands, the list is cut to the machines
//! still paired each time it is written, and [`TheNames::forgotten`] is what a
//! revocation — and a pairing kept afresh — calls, so a name never outlives the
//! agreement it was given under. The list in memory moves first and the disk
//! second, for `crate::network`'s reason.

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::SystemTime;

use alo_corridor::Naming;
use alo_nearby::{MachineId, Pairings};
use alo_remembering::{MachineName, MachineNames, NotRemembered};

/// Somewhere the names are written down between restarts.
///
/// One method, writing the whole list, and no way to read through it: what was
/// read is handed to [`TheNames::remembering`] by `src/main.rs`, once.
/// [`crate::keeping_names::TheNamesFile`] is the only implementation that
/// ships.
pub trait KeepingNames: Send + Sync + std::fmt::Debug {
    /// Write these names, whole.
    ///
    /// # Errors
    ///
    /// [`NotRemembered`], carried whole from `alo-remembering`.
    fn keep(&self, names: &MachineNames) -> Result<(), NotRemembered>;
}

/// Nowhere: the names are held for the session and written to nothing.
///
/// What a test that is not about the file hands in. Nothing a machine runs is
/// handed this — `crate::starting` takes the file from `main`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NothingKeepsNames;

impl KeepingNames for NothingKeepsNames {
    fn keep(&self, _names: &MachineNames) -> Result<(), NotRemembered> {
        Ok(())
    }
}

/// What the person here called the machines this one is paired with.
#[derive(Debug)]
pub struct TheNames {
    /// The names, behind their own lock.
    held: Mutex<MachineNames>,
    /// Where they are written down.
    keeping: Box<dyn KeepingNames>,
}

impl Default for TheNames {
    /// No names, written nowhere.
    fn default() -> Self {
        Self::remembering(MachineNames::none(), Box::new(NothingKeepsNames))
    }
}

impl TheNames {
    /// These names, read back off the disk, writing every change to `keeping`.
    #[must_use]
    pub fn remembering(names: MachineNames, keeping: Box<dyn KeepingNames>) -> Self {
        Self {
            held: Mutex::new(names),
            keeping,
        }
    }

    /// The names, for the length of one question.
    ///
    /// Taken anyway when poisoned, for `crate::network`'s reason: a map that is
    /// either whole or the panic was a test's.
    fn locked(&self) -> MutexGuard<'_, MachineNames> {
        self.held.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Give `machine` this name, or take its name away when `name` is nothing,
    /// against the pairings as they stand at `now`.
    ///
    /// The caller has already asked that a pairing with `machine` stands, under
    /// the network's lock, and holds that lock while this runs. The list in
    /// memory changes first — cut to the machines still paired — and is then
    /// written whole.
    ///
    /// # Errors
    ///
    /// [`NotRemembered`] when the file could not be written. The name stands
    /// for the session either way; the caller tells the person *until a
    /// restart*.
    pub fn named(
        &self,
        machine: &MachineId,
        name: Option<MachineName>,
        pairings: &Pairings,
        now: SystemTime,
    ) -> Result<(), NotRemembered> {
        let mut names = self.locked();
        match name {
            Some(name) => names.name(machine.clone(), name),
            None => {
                names.forget(machine);
            }
        }
        names.only_those_paired(pairings, now);
        self.keeping.keep(&names)
    }

    /// Take away the name of a machine whose pairing was revoked or kept
    /// afresh, and of every machine no pairing with stands at `now`.
    ///
    /// Answers `None` when nothing was taken away, so nothing is written; and
    /// otherwise whether the file could be written.
    pub fn forgotten(
        &self,
        machine: &MachineId,
        pairings: &Pairings,
        now: SystemTime,
    ) -> Option<Result<(), NotRemembered>> {
        let mut names = self.locked();
        let went = names.forget(machine);
        let others_went = names.only_those_paired(pairings, now);
        (went || others_went).then(|| self.keeping.keep(&names))
    }

    /// Every name held at the moment, for a test reading what a restart would.
    #[must_use]
    pub fn every(&self) -> MachineNames {
        self.locked().clone()
    }
}

impl Naming for TheNames {
    /// The name the person here gave this machine, if they gave one.
    fn called(&self, machine: &MachineId) -> Option<String> {
        self.locked()
            .called(machine)
            .map(|name| name.as_str().to_owned())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::sync::{Arc, Mutex};

    use alo_nearby::{MayAskIts, Pairings};

    use super::*;
    use crate::testing::{noon, paired_between, reception, the_studio};

    /// Somewhere that remembers every list it was handed.
    #[derive(Debug, Default)]
    struct Remembering {
        written: Arc<Mutex<Vec<MachineNames>>>,
    }

    impl KeepingNames for Remembering {
        fn keep(&self, names: &MachineNames) -> Result<(), NotRemembered> {
            self.written.lock().unwrap().push(names.clone());
            Ok(())
        }
    }

    /// The studio's pairings with reception, standing at noon.
    fn paired_with_reception() -> Pairings {
        let (_, on_studio) =
            paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        let mut pairings = Pairings::none();
        pairings.keep(on_studio);
        pairings
    }

    /// **A name given is what `Naming` answers, and it is written whole**;
    /// taken away, the machine is spoken of by its identity again.
    #[test]
    fn a_name_given_is_what_naming_answers_and_is_written_whole() {
        let keeping = Remembering::default();
        let written = Arc::clone(&keeping.written);
        let names = TheNames::remembering(MachineNames::none(), Box::new(keeping));
        let pairings = paired_with_reception();
        assert_eq!(names.called(&reception()), None);

        names
            .named(
                &reception(),
                Some(MachineName::checked("the reception machine").unwrap()),
                &pairings,
                noon(),
            )
            .unwrap();
        assert_eq!(
            names.called(&reception()).as_deref(),
            Some("the reception machine")
        );
        names.named(&reception(), None, &pairings, noon()).unwrap();
        assert_eq!(names.called(&reception()), None);
        let written = written.lock().unwrap();
        assert_eq!(written.len(), 2);
        assert_eq!(written.first().unwrap().len(), 1);
        assert!(written.last().unwrap().is_empty());
    }

    /// **A name goes with its pairing**: forgotten when the pairing is
    /// revoked, and nothing written when there was nothing to forget.
    #[test]
    fn a_name_is_forgotten_with_its_pairing_and_nothing_is_written_for_nothing() {
        let keeping = Remembering::default();
        let written = Arc::clone(&keeping.written);
        let names = TheNames::remembering(MachineNames::none(), Box::new(keeping));
        let mut pairings = paired_with_reception();
        assert!(names.forgotten(&reception(), &pairings, noon()).is_none());
        names
            .named(
                &reception(),
                Some(MachineName::checked("the reception machine").unwrap()),
                &pairings,
                noon(),
            )
            .unwrap();
        assert!(pairings.revoke(&reception()));
        assert!(
            names
                .forgotten(&reception(), &pairings, noon())
                .unwrap()
                .is_ok()
        );
        assert_eq!(names.called(&reception()), None);
        assert_eq!(written.lock().unwrap().len(), 2);
    }
}
