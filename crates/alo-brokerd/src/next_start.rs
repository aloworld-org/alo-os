//! *Restart into Windows*, carried out: the firmware's next start set to the
//! entry a person approved, once — and nothing else about how the machine
//! starts, ever.
//!
//! By the time a verb reaches here the door has decided it is exactly one a
//! person approved, once, and written that down. What is left is to set the
//! right entry and no other — and the broker was never told which entry in any
//! form it could act on. It was told thirty-two bytes. So this asks the
//! firmware what it has **now**, digests what it reported for each start-up
//! entry the way the side that asked digested it, and acts on the one that
//! matches.
//!
//! | | |
//! |---|---|
//! | matched against | every start-up entry the firmware reports, by exactly the bytes it reported |
//! | acts with | `alo_starting::Firmware::start_next`, and nothing else |
//!
//! **No match is no change**, and neither are two. **An entry that does not
//! start Windows is no change**, whatever it is called: a firmware's names for
//! what it can start are whatever was typed when they were made, and on a
//! machine that has been reinstalled they are regularly wrong — so what is
//! checked is the program the entry starts.
//!
//! # The default is not touched, and there is no way here to touch it
//!
//! [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md),
//! *what stays as it was*: this is a one-start switch through the firmware's
//! next-start entry, and it leaves the menu's default alone. `alo-starting`'s
//! [`Firmware`](alo_starting::Firmware) has one method that changes anything
//! and it is that one, so *the order was not touched* is a fact about the
//! surface rather than a rule about how it is used.
//!
//! # And nothing here restarts the machine
//!
//! The restart is the person's own, afterwards, exactly as it is for the update
//! verbs: no unit on this road holds `CAP_SYS_BOOT` and this process holds no
//! capability at all.

use alo_broker::{Identity, NotCarried};
use alo_starting::{Entry, Firmware};

/// What carries *Restart into Windows* out on this machine.
#[derive(Debug)]
pub struct NextStart<F> {
    /// The firmware, asked afresh for every question and every change.
    firmware: F,
}

impl<F: Firmware> NextStart<F> {
    /// Carry it out against this firmware.
    #[must_use]
    pub const fn against(firmware: F) -> Self {
        Self { firmware }
    }

    /// The firmware, for a test to look at what it was asked.
    #[must_use]
    pub const fn firmware(&self) -> &F {
        &self.firmware
    }

    /// Set the machine to start, next time and once, the Windows whose start-up
    /// entry was approved.
    ///
    /// # Errors
    /// [`NotCarried`] when the firmware could not be asked, when no start-up
    /// entry it reports is the one that was approved, when more than one is,
    /// when the one that is does not start Windows, or when the firmware would
    /// not be told. Nothing is changed in any of them.
    pub fn windows(&self, identity: Identity) -> Result<(), NotCarried> {
        let entries = self.firmware.entries().map_err(|why| {
            NotCarried(format!(
                "this machine could not be asked what it can start, so nothing was changed: {why}"
            ))
        })?;
        let mut approved = entries
            .iter()
            .filter(|entry| Identity::of_what_was_reported(&entry.as_reported()) == identity);
        let Some(entry) = approved.next() else {
            return Err(NotCarried(
                "no start-up entry this machine has is the one that was approved, so nothing was \
                 changed"
                    .to_owned(),
            ));
        };
        if approved.next().is_some() {
            return Err(NotCarried(
                "more than one start-up entry this machine has is the one that was approved, so \
                 nothing was changed"
                    .to_owned(),
            ));
        }
        if !entry.starts_windows() {
            return Err(NotCarried(format!(
                "start-up entry {:04X} does not start Windows, so nothing was changed",
                entry.number()
            )));
        }
        self.firmware.start_next(entry.number()).map_err(|why| {
            NotCarried(format!(
                "this machine would not be told what to start next, so nothing was changed: {why}"
            ))
        })
    }
}

/// The identity of a start-up entry, as both sides of the door make one.
///
/// Here so that the side that proposes the change and the side that carries it
/// out cannot drift: one function, called by both.
#[must_use]
pub fn the_identity_of(entry: &Entry) -> Identity {
    Identity::of_what_was_reported(&entry.as_reported())
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

    use alo_starting::testing::as_a_firmware_reports_it;
    use alo_starting::{NotAnswering, NotDone, THE_WINDOWS_LOADER_IN_A_PATH};

    /// A firmware that reports what it was given and remembers what it was
    /// told.
    #[derive(Debug)]
    struct AFirmware {
        /// What it reports, or the reason it will not.
        reports: Result<Vec<Entry>, NotAnswering>,
        /// Whether it refuses to be told.
        refuses: bool,
        /// What it was told to start next.
        told: RefCell<Vec<u16>>,
    }

    impl AFirmware {
        /// A firmware reporting these entries.
        fn reporting(entries: Vec<Entry>) -> Self {
            Self {
                reports: Ok(entries),
                refuses: false,
                told: RefCell::default(),
            }
        }
    }

    impl Firmware for AFirmware {
        fn entries(&self) -> Result<Vec<Entry>, NotAnswering> {
            self.reports.clone()
        }

        fn start_next(&self, entry: u16) -> Result<(), NotDone> {
            if self.refuses {
                return Err(NotDone("this firmware is read-only".to_owned()));
            }
            self.told.borrow_mut().push(entry);
            Ok(())
        }
    }

    /// A start-up entry the firmware reports.
    fn an_entry(number: u16, named: &str, path: &str) -> Entry {
        Entry::reported(number, &as_a_firmware_reports_it(named, path)).unwrap()
    }

    /// The Windows on a machine.
    fn windows(number: u16) -> Entry {
        an_entry(number, "Windows Boot Manager", THE_WINDOWS_LOADER_IN_A_PATH)
    }

    /// alo OS on the same machine.
    fn alo_os(number: u16) -> Entry {
        an_entry(number, "alo OS", "\\EFI\\fedora\\shimx64.efi")
    }

    /// **The entry a person approved is the one the machine is set to start**,
    /// once, and it is told once.
    #[test]
    fn the_entry_approved_is_the_one_the_machine_is_told_to_start() {
        let approved = the_identity_of(&windows(2));
        let next = NextStart::against(AFirmware::reporting(vec![alo_os(1), windows(2)]));
        assert_eq!(next.windows(approved), Ok(()));
        assert_eq!(*next.firmware().told.borrow(), vec![2]);
    }

    /// **An identity no entry has changes nothing.** The entries a firmware has
    /// can change between a person approving and the machine acting, and the
    /// broker digests what is there at the moment it acts.
    #[test]
    fn an_identity_no_entry_has_changes_nothing() {
        let next = NextStart::against(AFirmware::reporting(vec![alo_os(1), windows(2)]));
        let Err(NotCarried(why)) =
            next.windows(Identity::of_what_was_reported(b"another machine's Windows"))
        else {
            panic!("an entry nobody approved was started");
        };
        assert!(why.contains("nothing was changed"), "{why}");
        assert!(next.firmware().told.borrow().is_empty());
    }

    /// **Two entries that are the same entry change nothing.** A machine that
    /// has been reinstalled keeps both, and choosing between them is a guess
    /// nobody asked for.
    ///
    /// They are in different slots, which is what makes this the real case: an
    /// identity is made over what the firmware reported and not over the number
    /// it happens to be kept under, so the same start-up entry written twice is
    /// one identity and two matches.
    #[test]
    fn more_than_one_matching_entry_changes_nothing() {
        let approved = the_identity_of(&windows(2));
        let next = NextStart::against(AFirmware::reporting(vec![
            windows(2),
            alo_os(3),
            windows(4),
        ]));
        let Err(NotCarried(why)) = next.windows(approved) else {
            panic!("one of two identical entries was chosen");
        };
        assert!(why.contains("more than one"), "{why}");
        assert!(next.firmware().told.borrow().is_empty());
    }

    /// **An entry that does not start Windows changes nothing**, however it is
    /// named. This is the refusal that makes the verb's name true: without it,
    /// *start Windows next* would start whatever was approved.
    #[test]
    fn an_entry_that_does_not_start_windows_changes_nothing() {
        let looks_like_it = an_entry(
            4,
            "Windows Boot Manager",
            "\\EFI\\somebody-else\\loader.efi",
        );
        let approved = the_identity_of(&looks_like_it);
        let next = NextStart::against(AFirmware::reporting(vec![looks_like_it]));
        let Err(NotCarried(why)) = next.windows(approved) else {
            panic!("something that is not Windows was started as Windows");
        };
        assert!(why.contains("does not start Windows"), "{why}");
        assert!(next.firmware().told.borrow().is_empty());
    }

    /// **A firmware that will not say what it can start changes nothing**, and
    /// says so rather than reading silence as *there is no Windows here*.
    #[test]
    fn a_firmware_that_will_not_answer_changes_nothing() {
        let next = NextStart::against(AFirmware {
            reports: Err(NotAnswering("nothing to read".to_owned())),
            refuses: false,
            told: RefCell::default(),
        });
        let Err(NotCarried(why)) = next.windows(the_identity_of(&windows(1))) else {
            panic!("a machine that could not be asked was told something");
        };
        assert!(why.contains("could not be asked"), "{why}");
        assert!(next.firmware().told.borrow().is_empty());
    }

    /// **A firmware that will not be told is said**, rather than answered as
    /// though the next start had been set.
    #[test]
    fn a_firmware_that_will_not_be_told_is_said() {
        let approved = the_identity_of(&windows(2));
        let next = NextStart::against(AFirmware {
            reports: Ok(vec![windows(2)]),
            refuses: true,
            told: RefCell::default(),
        });
        let Err(NotCarried(why)) = next.windows(approved) else {
            panic!("a firmware that refused was answered as though it had not");
        };
        assert!(why.contains("would not be told"), "{why}");
    }

    /// **A machine with no Windows on it changes nothing**, which is what a
    /// machine alo OS replaced is.
    #[test]
    fn a_machine_with_no_windows_changes_nothing() {
        let approved = the_identity_of(&windows(2));
        let next = NextStart::against(AFirmware::reporting(vec![alo_os(1)]));
        assert!(next.windows(approved).is_err());
        assert!(next.firmware().told.borrow().is_empty());
    }
}
