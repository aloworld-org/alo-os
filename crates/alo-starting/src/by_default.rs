//! Which system this machine starts when nobody chooses, changed — the whole
//! road from a person's approved choice to the loader's own file.
//!
//! [ADR 0066](../../../docs/decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md):
//! Settings does not write the loader's file, because a person is not root. It
//! asks the broker, and what the broker carries out is this function against a
//! [`TheLoader`]. It is here rather than in the broker's own process for the
//! reason
//! [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
//! term 3 gives: the file, the setting in it and the menu that reads it are
//! named in **one** crate, and a second place that knew their shape would be
//! the second copy the term refuses.
//!
//! # In order, and each step refusing before the next
//!
//! 1. The identity is **one of the two systems** ([`crate::the_system_named`]),
//!    or nothing happens. The two verbs on this road take arguments of the same
//!    shape, so an identity made for a start-up entry reaches here and is
//!    refused rather than acted on.
//! 2. The menu on **this** machine offers that system, or nothing happens —
//!    which is how *there is no Windows on this computer* is a refusal and not
//!    a machine set to start something it does not have.
//! 3. The file is read, and read **as an environment block**. Anything else is
//!    refused rather than replaced: a file that is not one belongs to somebody,
//!    and overwriting it would be alo OS taking a file it does not own.
//! 4. The choice is put in the block beside everything else already there, and
//!    the block is written at exactly the length it was read at. A block that
//!    will not hold another setting is a refusal, because the setting that
//!    would silently go is the base's.
//! 5. Only then the bytes are handed back to the loader's file.
//!
//! # It writes even when the answer is already that
//!
//! A change to what is already so is still carried out rather than skipped. The
//! act a person approved is *make this machine start alo OS*, and an execution
//! that quietly did nothing because the file already said so would make what
//! the record shows depend on a state nobody can see. Writing is idempotent:
//! [`crate::TheStartingChoice`] keeps one setting, whatever it is written.

use alo_broker::Identity;

use crate::chosen::TheStartingChoice;
use crate::loader::{NotRead, NotWritten, TheLoader};
use crate::offered::the_system_named;
use crate::saved::{EnvironmentBlock, NotAnEnvironmentBlock};
use crate::systems::System;

/// Why which system this machine starts by default was not changed.
///
/// Every one of them leaves the loader's file exactly as it was.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotSet {
    /// What was approved is not one of the systems this machine can start.
    #[error("what was approved is not one of the two systems, so nothing was changed")]
    NotOneOfTheSystems,

    /// This machine's menu does not offer that system.
    #[error("the menu on this machine does not offer that system, so nothing was changed")]
    NotOffered,

    /// One of the loader's files could not be read.
    #[error("{0}, so nothing was changed")]
    NotRead(#[from] NotRead),

    /// The file the choice is kept in is not one this crate wrote.
    #[error("{0}, so nothing was changed")]
    NotABlock(#[from] NotAnEnvironmentBlock),

    /// The file would not be written.
    #[error("{0}, so nothing was changed")]
    NotWritten(#[from] NotWritten),
}

/// Make this machine start, whenever nobody chooses at the menu, the system
/// whose identity was approved.
///
/// Answers with the system it now starts, for whoever has a sentence to say
/// about it.
///
/// # Errors
/// [`NotSet`], and the loader's file is exactly as it was in every one of them.
pub fn start_by_default(loader: &impl TheLoader, identity: Identity) -> Result<System, NotSet> {
    let system = the_system_named(identity).ok_or(NotSet::NotOneOfTheSystems)?;
    if !loader.offering()?.contains(&system) {
        return Err(NotSet::NotOffered);
    }
    let mut block = EnvironmentBlock::read(&loader.saved()?)?;
    TheStartingChoice::write(&mut block, system)?;
    loader.save(&block.written()?)?;
    Ok(system)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    use crate::offered::the_identity_of;
    use crate::saved::{LENGTH, SIGNATURE};

    /// A loader whose files are in memory, remembering what it was told.
    #[derive(Debug)]
    struct ALoader {
        /// What the menu offers, or the reason it could not be read.
        offers: Result<Vec<System>, NotRead>,
        /// The bytes of the file the choice is kept in, or the reason it could
        /// not be read.
        block: Result<Vec<u8>, NotRead>,
        /// Whether it refuses to be written.
        refuses: bool,
        /// Every set of bytes it was told to keep.
        written: RefCell<Vec<Vec<u8>>>,
    }

    impl ALoader {
        /// A machine with both systems on it, whose block holds `settings`.
        fn with(settings: &[(&str, &str)]) -> Self {
            let mut block = EnvironmentBlock::empty();
            for (name, value) in settings {
                block.keep(name, value).unwrap();
            }
            Self {
                offers: Ok(System::BOTH.to_vec()),
                block: Ok(block.written().unwrap()),
                refuses: false,
                written: RefCell::default(),
            }
        }

        /// What it was told to keep, read back as a block.
        fn kept(&self) -> EnvironmentBlock {
            let written = self.written.borrow();
            let last = written.last().expect("something was written");
            EnvironmentBlock::read(last).unwrap()
        }
    }

    impl TheLoader for ALoader {
        fn offering(&self) -> Result<Vec<System>, NotRead> {
            self.offers.clone()
        }

        fn saved(&self) -> Result<Vec<u8>, NotRead> {
            self.block.clone()
        }

        fn save(&self, bytes: &[u8]) -> Result<(), NotWritten> {
            if self.refuses {
                return Err(NotWritten("this file is read-only".to_owned()));
            }
            self.written.borrow_mut().push(bytes.to_owned());
            Ok(())
        }
    }

    /// **Each system a person approves is the one the machine is set to start**,
    /// in the loader's own file and written once.
    #[test]
    fn the_system_approved_is_the_one_the_machine_is_set_to_start() {
        for system in System::BOTH {
            let loader = ALoader::with(&[]);
            assert_eq!(
                start_by_default(&loader, the_identity_of(system)),
                Ok(system)
            );
            assert_eq!(loader.written.borrow().len(), 1);
            assert_eq!(TheStartingChoice::read(&loader.kept()), system);
        }
    }

    /// **Nothing else in the file is disturbed, and the file keeps its
    /// length.** The block is the base's as much as it is ours, and the loader
    /// writes it in place.
    #[test]
    fn everything_else_in_the_file_is_left_exactly_as_it_was() {
        let loader = ALoader::with(&[("boot_success", "1"), ("boot_indeterminate", "0")]);
        assert_eq!(
            start_by_default(&loader, the_identity_of(System::Windows)),
            Ok(System::Windows)
        );
        let kept = loader.kept();
        assert_eq!(kept.kept_as("boot_success"), Some("1"));
        assert_eq!(kept.kept_as("boot_indeterminate"), Some("0"));
        assert_eq!(kept.how_many(), 3);
        assert_eq!(loader.written.borrow().last().unwrap().len(), LENGTH);
    }

    /// **The same choice made twice keeps one answer**, rather than a file that
    /// grows a line every time a person changes their mind.
    #[test]
    fn the_same_choice_twice_keeps_one_answer() {
        let loader = ALoader::with(&[]);
        for system in [System::Windows, System::AloOs, System::Windows] {
            assert_eq!(
                start_by_default(&loader, the_identity_of(system)),
                Ok(system)
            );
        }
        assert_eq!(loader.kept().how_many(), 1);
        assert_eq!(TheStartingChoice::read(&loader.kept()), System::Windows);
    }

    /// **An identity that is not one of the two systems changes nothing.** The
    /// identity of a start-up entry is the case that matters: the two verbs on
    /// this road take arguments of the same shape, and one made for the other
    /// reaches here.
    #[test]
    fn an_identity_that_is_not_a_systems_changes_nothing() {
        let loader = ALoader::with(&[]);
        assert_eq!(
            start_by_default(&loader, Identity::of_what_was_reported(b"a start-up entry")),
            Err(NotSet::NotOneOfTheSystems)
        );
        assert!(loader.written.borrow().is_empty());
    }

    /// **A machine with no Windows on it is not set to start one.** That is
    /// what a machine alo OS replaced is, and the refusal is the whole reason
    /// the menu is asked before the file is touched.
    #[test]
    fn a_machine_with_no_windows_is_not_set_to_start_one() {
        let mut loader = ALoader::with(&[]);
        loader.offers = Ok(vec![System::AloOs]);
        assert_eq!(
            start_by_default(&loader, the_identity_of(System::Windows)),
            Err(NotSet::NotOffered)
        );
        assert!(loader.written.borrow().is_empty());
        // And alo OS, which such a machine does offer, is still changeable.
        assert_eq!(
            start_by_default(&loader, the_identity_of(System::AloOs)),
            Ok(System::AloOs)
        );
    }

    /// **A menu that could not be read is said, rather than read as *there is
    /// no Windows here*.** The two look the same to a machine and are entirely
    /// different to a person.
    #[test]
    fn a_menu_that_could_not_be_read_is_not_read_as_no_windows() {
        let mut loader = ALoader::with(&[]);
        loader.offers = Err(NotRead("the menu could not be read".to_owned()));
        assert_eq!(
            start_by_default(&loader, the_identity_of(System::Windows)),
            Err(NotSet::NotRead(NotRead(
                "the menu could not be read".to_owned()
            )))
        );
        assert!(loader.written.borrow().is_empty());
    }

    /// **A file that could not be read is a refusal, not a file written from
    /// nothing.** A machine whose block is missing gets no block invented for
    /// it: what is there belongs to the loader.
    #[test]
    fn a_file_that_could_not_be_read_is_not_written_from_nothing() {
        let mut loader = ALoader::with(&[]);
        loader.block = Err(NotRead("no such file".to_owned()));
        assert!(matches!(
            start_by_default(&loader, the_identity_of(System::AloOs)),
            Err(NotSet::NotRead(_))
        ));
        assert!(loader.written.borrow().is_empty());
    }

    /// **A file that is not an environment block is refused rather than
    /// replaced**, whatever else it is.
    #[test]
    fn a_file_that_is_not_an_environment_block_is_refused() {
        for bytes in [
            Vec::new(),
            b"saved_entry=alo-windows\n".to_vec(),
            vec![0xff; LENGTH],
        ] {
            let mut loader = ALoader::with(&[]);
            loader.block = Ok(bytes.clone());
            assert_eq!(
                start_by_default(&loader, the_identity_of(System::AloOs)),
                Err(NotSet::NotABlock(NotAnEnvironmentBlock::NotOne)),
                "{bytes:?} was written over"
            );
            assert!(loader.written.borrow().is_empty());
        }
    }

    /// **A block that will not hold another setting changes nothing, and
    /// nothing in it is dropped to make room.** What would go is the base's.
    #[test]
    fn a_block_that_will_not_hold_it_changes_nothing() {
        let mut full = EnvironmentBlock::empty();
        full.keep(
            "something_of_the_bases",
            &"x".repeat(LENGTH - SIGNATURE.len() - 40),
        )
        .unwrap();
        let mut loader = ALoader::with(&[]);
        loader.block = Ok(full.written().unwrap());

        let Err(NotSet::NotABlock(NotAnEnvironmentBlock::NoRoom { needed, room })) =
            start_by_default(&loader, the_identity_of(System::Windows))
        else {
            panic!("a block was grown past the file the loader writes in place");
        };
        assert!(needed > room);
        assert!(loader.written.borrow().is_empty());
    }

    /// **A file that would not be written is said**, rather than answered as
    /// though the machine's default had changed.
    #[test]
    fn a_file_that_would_not_be_written_is_said() {
        let mut loader = ALoader::with(&[]);
        loader.refuses = true;
        assert!(matches!(
            start_by_default(&loader, the_identity_of(System::Windows)),
            Err(NotSet::NotWritten(_))
        ));
        assert!(loader.written.borrow().is_empty());
    }
}
