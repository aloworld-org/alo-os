//! *This computer starts this system when nobody chooses*, carried out: the
//! loader's own saved default set to the system a person approved, and nothing
//! else about how the machine starts, ever.
//!
//! By the time a verb reaches here the door has decided it is exactly one a
//! person approved, once, and written that down. What is left is to change the
//! one place the answer is kept
//! ([ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
//! term 3), which is under `/boot` and therefore root's — which is the whole
//! reason this is a verb rather than something Settings does
//! ([ADR 0066](../../../docs/decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md)).
//!
//! | | |
//! |---|---|
//! | matched against | the systems this machine's own menu offers |
//! | acts with | `alo_starting::start_by_default`, and nothing else |
//!
//! # Nothing here knows what is in that file
//!
//! Every decision about the loader's files — what they are, how they are read,
//! what may be written into them and at what length — is
//! `alo_starting`'s, and this hands the verb's argument to it. A broker that
//! knew the shape of the file would be the second place that knows it, which is
//! exactly what ADR 0062's third term refuses, and this process is meant to be
//! auditable in an afternoon rather than to hold a file format.
//!
//! # The next start is not touched, and there is no way here to touch it
//!
//! This sets the default. `crate::NextStart` sets the next start. They are two
//! carriers over two verbs because they are two acts, and neither has a method
//! that could do the other's.
//!
//! # And nothing here restarts the machine
//!
//! The restart is the person's own, whenever they next switch the machine off:
//! no unit on this road holds `CAP_SYS_BOOT` and this process holds no
//! capability at all.

use alo_broker::{Identity, NotCarried};
use alo_starting::{TheLoader, start_by_default};

/// What carries *start this system when nobody chooses* out on this machine.
#[derive(Debug)]
pub struct ByDefault<L> {
    /// The loader's files, read afresh for every question and every change.
    loader: L,
}

impl<L: TheLoader> ByDefault<L> {
    /// Carry it out against these files.
    #[must_use]
    pub const fn against(loader: L) -> Self {
        Self { loader }
    }

    /// The loader's files, for a test to look at what they hold.
    #[must_use]
    pub const fn loader(&self) -> &L {
        &self.loader
    }

    /// Make this machine start, whenever nobody chooses, the system that was
    /// approved.
    ///
    /// # Errors
    /// [`NotCarried`] when what was approved is not one of the two systems,
    /// when this machine's menu does not offer it, when the loader's files
    /// could not be read, when the file the answer is kept in is not one this
    /// machine wrote, when it will not hold the answer, or when it would not be
    /// written. Nothing is changed in any of them.
    pub fn set(&self, identity: Identity) -> Result<(), NotCarried> {
        start_by_default(&self.loader, identity)
            .map(|_| ())
            .map_err(|why| NotCarried(why.to_string()))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    use alo_starting::{
        EnvironmentBlock, NotRead, NotWritten, System, TheStartingChoice, the_identity_of,
    };

    /// A loader whose files are in memory, remembering what it was told.
    #[derive(Debug)]
    struct ALoader {
        /// What the menu offers.
        offers: Vec<System>,
        /// The bytes of the file the choice is kept in.
        block: Vec<u8>,
        /// Every set of bytes it was told to keep.
        written: RefCell<Vec<Vec<u8>>>,
    }

    impl ALoader {
        /// A machine offering these systems, whose block is empty.
        fn offering(offers: Vec<System>) -> Self {
            Self {
                offers,
                block: EnvironmentBlock::empty().written().unwrap(),
                written: RefCell::default(),
            }
        }

        /// Which system it was last told to start.
        fn now_starts(&self) -> Option<System> {
            let written = self.written.borrow();
            let last = written.last()?;
            Some(TheStartingChoice::read(
                &EnvironmentBlock::read(last).unwrap(),
            ))
        }
    }

    impl TheLoader for ALoader {
        fn offering(&self) -> Result<Vec<System>, NotRead> {
            Ok(self.offers.clone())
        }

        fn saved(&self) -> Result<Vec<u8>, NotRead> {
            Ok(self.block.clone())
        }

        fn save(&self, bytes: &[u8]) -> Result<(), NotWritten> {
            self.written.borrow_mut().push(bytes.to_owned());
            Ok(())
        }
    }

    /// **The system a person approved is the one the machine is set to start**,
    /// and the file is written once.
    #[test]
    fn the_system_approved_is_the_one_the_machine_is_set_to_start() {
        for system in System::BOTH {
            let by_default = ByDefault::against(ALoader::offering(System::BOTH.to_vec()));
            assert_eq!(by_default.set(the_identity_of(system)), Ok(()));
            assert_eq!(by_default.loader().written.borrow().len(), 1);
            assert_eq!(by_default.loader().now_starts(), Some(system));
        }
    }

    /// **An identity that is not one of the two systems changes nothing**, and
    /// the identity of a start-up entry is the one that matters: the two verbs
    /// on this road take arguments of the same shape.
    #[test]
    fn an_identity_that_is_not_a_systems_changes_nothing() {
        let by_default = ByDefault::against(ALoader::offering(System::BOTH.to_vec()));
        let Err(NotCarried(why)) =
            by_default.set(Identity::of_what_was_reported(b"a start-up entry"))
        else {
            panic!("a verb naming no system changed which system this machine starts");
        };
        assert!(why.contains("nothing was changed"), "{why}");
        assert!(by_default.loader().written.borrow().is_empty());
    }

    /// **A machine with no Windows on it is not set to start one**, which is
    /// what a machine alo OS replaced is.
    #[test]
    fn a_machine_with_no_windows_is_not_set_to_start_one() {
        let by_default = ByDefault::against(ALoader::offering(vec![System::AloOs]));
        let Err(NotCarried(why)) = by_default.set(the_identity_of(System::Windows)) else {
            panic!("a machine with no Windows on it was set to start Windows");
        };
        assert!(why.contains("nothing was changed"), "{why}");
        assert!(by_default.loader().written.borrow().is_empty());
    }

    /// **A file that is not one this machine wrote is refused rather than
    /// replaced**, whatever else it is.
    #[test]
    fn a_file_that_is_not_this_machines_is_not_written_over() {
        let mut loader = ALoader::offering(System::BOTH.to_vec());
        loader.block = b"something else entirely".to_vec();
        let by_default = ByDefault::against(loader);
        assert!(by_default.set(the_identity_of(System::AloOs)).is_err());
        assert!(by_default.loader().written.borrow().is_empty());
    }
}
