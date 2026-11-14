//! **Whether this machine's sign-in screen reads aloud, kept across starts**
//! ([ADR 0155](../../../docs/decisions/0155-the-screens-before-sign-in-speak.md) §6).
//!
//! A person who turns the reader on at the sign-in screen has turned it on for
//! that screen: it should still be on the next time the machine starts, and on
//! a screen the compositor brought back after it fell over. So the choice is
//! kept in the machine's own copy of the accessibility settings —
//! `alo_access::keeping::THE_MACHINES_FOLDER`, written by `alo-access` and by
//! nothing else (ADR 0038) — which belongs to no account.
//!
//! **That copy is also where a new account starts from**: a person who has
//! kept nothing of their own reads the machine's
//! (`alo_access::keeping::for_this_person`), which is how a reader turned on at
//! the first start is on in the account made there.
//!
//! Only the screen reader is changed here; whatever else the copy holds is
//! read and written back as it was.

use std::path::Path;

use alo_access::Setting;

/// Whether the machine's copy at `machine` has the screen reader on. A copy
/// that is not there, or does not read, is off; the second is said.
#[must_use]
pub fn was_on(machine: &Path) -> bool {
    let (turned_on, why) = alo_access::keeping::at_sign_in(machine);
    if let Some(why) = why {
        eprintln!(
            "alo-compositor: whether the sign-in screen reads aloud could not be read from {}, so \
             it starts silent: {why}",
            machine.display()
        );
    }
    turned_on.has(Setting::ScreenReader)
}

/// **Keep the screen reader `on` or off in the machine's copy at `machine`**,
/// and say on the journal where it could not be kept — the reader is still
/// turned over now; only the next start forgets.
pub fn is_now(machine: &Path, on: bool) {
    let (mut turned_on, _) = alo_access::keeping::at_sign_in(machine);
    if on {
        turned_on.turn_on(Setting::ScreenReader);
    } else {
        turned_on.turn_off(Setting::ScreenReader);
    }
    if let Err(why) = alo_access::keeping::keep_for_this_machine(machine, &turned_on) {
        eprintln!(
            "alo-compositor: the sign-in screen's reader was turned {} and that could not be kept \
             in {}, so the next start will not remember it: {why}",
            if on { "on" } else { "off" },
            machine.display()
        );
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A folder of this test's own.
    fn a_folder() -> std::path::PathBuf {
        let at = std::env::temp_dir().join(format!(
            "alo-sign-in-reader-kept-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&at).unwrap();
        at
    }

    /// **What is kept is what the next start reads**, and the other settings
    /// in the copy are left as they were.
    #[test]
    fn the_next_start_reads_what_was_kept() {
        let machine = a_folder();
        assert!(!was_on(&machine), "a machine nobody changed is silent");
        let mut before = alo_access::TurnedOn::nothing();
        before.turn_on(Setting::HighContrast);
        alo_access::keeping::keep_for_this_machine(&machine, &before).unwrap();

        is_now(&machine, true);
        assert!(was_on(&machine));
        assert!(
            alo_access::keeping::at_sign_in(&machine)
                .0
                .has(Setting::HighContrast),
            "only the reader is changed"
        );
        is_now(&machine, false);
        assert!(!was_on(&machine));
        let _ = std::fs::remove_dir_all(machine);
    }

    /// **A copy that does not read starts silent**, rather than stopping the
    /// sign-in screen — and turning the reader on writes a copy that does.
    #[test]
    fn a_copy_that_does_not_read_starts_silent() {
        let machine = a_folder();
        std::fs::write(
            machine.join(alo_access::keeping::THE_FILE),
            "not settings {{",
        )
        .unwrap();
        assert!(!was_on(&machine));
        is_now(&machine, true);
        assert!(was_on(&machine));
        let _ = std::fs::remove_dir_all(machine);
    }

    /// **A copy that cannot be written is said and is not a failure** of the
    /// screen: the reader is turned over all the same.
    #[test]
    fn a_copy_that_cannot_be_written_is_not_a_failure() {
        let machine = a_folder().join("a-file-not-a-folder");
        std::fs::write(&machine, "").unwrap();
        is_now(&machine, true);
        assert!(!was_on(&machine));
    }
}
