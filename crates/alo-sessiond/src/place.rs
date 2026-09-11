//! Where the door is, and why it is not in the one ADR 0017 made.
//!
//! `/run/alo` is *every person's door* (ADR 0017), and beneath it a directory
//! per signed-in person. This door is open **before anybody is signed in**, it
//! belongs to no person, and putting it there would mean a name in that
//! directory that is not a person's number — which is the one thing the mode on
//! `/run/alo` was chosen to prevent somebody doing by accident.
//!
//! So it has a directory of its own, and systemd makes it: `RuntimeDirectory=`
//! in the unit, owned by this process's `User=` and `Group=`, removed when the
//! service stops. Nothing in this crate creates a directory, which is
//! `alo-agentd`'s `place.rs` rule kept — a privileged process that made
//! directories would be a privileged process with a path in its hands.

/// The directory this machine's opener keeps its door in.
///
/// Made by systemd from the unit's `RuntimeDirectory=`, never by this process.
pub const THE_DOORS_DIRECTORY: &str = "/run/alo-sessiond";

/// The door the sign-in surface knocks on.
pub const THE_DOOR: &str = "/run/alo-sessiond/sign-in.sock";

#[cfg(test)]
mod tests {
    use super::*;

    /// The door is in the directory, which is the one thing two constants can
    /// disagree about — and the unit file's `RuntimeDirectory=` is the third
    /// place it is written, held to these by `crates/alo-image`.
    #[test]
    fn the_door_is_in_the_directory_the_unit_makes() {
        assert!(THE_DOOR.starts_with(THE_DOORS_DIRECTORY));
        assert_eq!(THE_DOORS_DIRECTORY, "/run/alo-sessiond");
    }

    /// **It is not under ADR 0017's directory.** That one is per person and
    /// this door belongs to nobody, which is the whole argument in this file's
    /// header — held here so that moving it is a decision rather than an edit.
    #[test]
    fn it_is_not_in_the_directory_a_persons_door_goes_in() {
        assert!(!THE_DOOR.starts_with("/run/alo/"));
    }
}
