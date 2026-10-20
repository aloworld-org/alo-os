//! Whose settings this service reads, and where it finds them.
//!
//! `alo-choosing` knows the shape of the file and the name it goes under;
//! nothing in it knows *whose* file to open, because a library cannot know
//! which person a process is. That is this file, and it is four lines because
//! there is only one honest answer: **the environment this process was started
//! with**.
//!
//! # Why the environment is the person (ADR 0019 §3)
//!
//! `alo-agentd` runs as the person, one instance per login, started by
//! `systemd` inside their session — so `$XDG_CONFIG_HOME` and `$HOME` as this
//! process has them *are* the person's, and reading them is the daemon reading
//! its own settings rather than reaching into somebody's home directory.
//!
//! **That is a condition, not an assumption, and it is written down as one.**
//! ADR 0019 records it: the day something serves two people from one process,
//! this becomes one person reading another's choices, and it becomes so
//! silently. [`of_a_session`] therefore takes both variables as arguments and
//! [`this_sessions_own`] is the only thing on **the settings road** that reads
//! the environment, so the fix that day is a caller's change rather than a
//! rewrite of whatever grew on top of it.
//!
//! **That had stopped being true, and the claim was wider than it should ever
//! have been.** Measured 2026-10-20. `what_this_person_chose` — which this
//! paragraph named as the single reader — was called by **nothing**, while
//! `Questions::of_this_process` read the same two variables itself and is the
//! road production takes, from `starting.rs`. That duplicate is gone:
//! [`this_sessions_own`] is the one reader and both roads go through it.
//!
//! **But *nowhere else in this crate* was never true, and three different
//! spellings are why nobody noticed.** The other readers each name a variable
//! that points at the person's own directories:
//!
//! | | reads | why |
//! |---|---|---|
//! | `starting.rs` | `"HOME"`, `"XDG_DATA_HOME"`, `"XDG_DATA_DIRS"` as **literals** | the desktop-entry specification's answer for where applications live *is* the environment, and that file says so |
//! | `the_bracket_this_turn_takes.rs` | `alo_choosing::HOME`, **fully qualified** | the bracket road, whose own header claims only *nowhere else in this crate's bracket road* — which is narrower and is true |
//!
//! A bare constant, a string literal and a qualified path: no single search
//! finds all three, which is the whole reason the claim above went unchallenged.
//! So this file's claim is now about **the settings road** rather than the
//! crate, and `tests/the_person_is_read_in_named_places.rs` holds the real
//! invariant — every read of a variable naming the person's own directories is
//! on a list that says why — and it fails when a new one appears in any
//! spelling.
//!
//! # A login with no home has chosen nothing
//!
//! `alo_choosing::where_it_is` answers `None` when neither variable names an
//! absolute directory, and there is then no file to read. That is a person who
//! has chosen nothing — the same state as a person whose file is not there yet
//! — and not a failure: a service that refused to serve a session without a
//! home directory would be refusing reads and changes over a model nobody
//! asked it about.

use std::ffi::{OsStr, OsString};

use alo_choosing::{CONFIG_HOME, HOME, NotSet, Settings, where_it_is};

/// The two variables this process was started with, which are the person.
///
/// **The settings road reads the environment here and nowhere else**, and
/// `tests/the_person_is_read_in_named_places.rs` fails if that stops being so.
/// Both roads that need them — this file's [`what_this_person_chose`] and
/// `Questions::of_this_process` — call this, so the day one process serves two
/// people the change is here and the callers are already asking a function
/// rather than a global. The module header names the two other roads in this
/// crate that read the person's directories for reasons of their own.
///
/// Owned rather than borrowed because `var_os` returns an owned value and a
/// caller that wants `&OsStr` can take one; the reverse needs a temporary that
/// outlives the call, which is the shape that invites a reader to inline
/// `var_os` again and quietly make two.
#[must_use]
pub fn this_sessions_own() -> (Option<OsString>, Option<OsString>) {
    (std::env::var_os(CONFIG_HOME), std::env::var_os(HOME))
}

/// What the person this process runs as has chosen.
///
/// Reads the environment through [`this_sessions_own`] and not directly.
///
/// # Errors
///
/// [`NotSet`] when the file is there and cannot be read or does not hold: a
/// malformed file is refused in the person's own words rather than treated as
/// an empty one, because *you have chosen nothing* would be false and would
/// send them to a settings panel that already says otherwise.
pub fn what_this_person_chose() -> Result<Settings, NotSet> {
    let (config_home, home) = this_sessions_own();
    of_a_session(config_home.as_deref(), home.as_deref())
}

/// The same, for a session named rather than inherited.
///
/// # Errors
///
/// [`NotSet`], as [`what_this_person_chose`].
pub fn of_a_session(config_home: Option<&OsStr>, home: Option<&OsStr>) -> Result<Settings, NotSet> {
    match where_it_is(config_home, home) {
        Some(at) => Settings::at(&at),
        None => Ok(Settings::untouched()),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_directory_of_our_own;
    use alo_choosing::Which;

    /// A settings file with this text in it, under a configuration directory
    /// this test owns, and the directory to hand to [`of_a_session`].
    fn a_session_that_chose(what: &str, said: &str) -> std::path::PathBuf {
        let config = a_directory_of_our_own(what);
        let folder = config.join(alo_choosing::THE_FOLDER);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join(alo_choosing::THE_SETTINGS), said).unwrap();
        config
    }

    /// **What the person wrote is what the daemon reads**, off the directory
    /// their session names.
    #[test]
    fn the_choice_in_the_persons_own_file_is_what_comes_back() {
        let config = a_session_that_chose(
            "chose",
            "format = 1\n\n[answers]\ncatalogue = \"mistral-small\"\n",
        );

        let settings = of_a_session(Some(config.as_os_str()), None).unwrap();

        let chosen = settings.chosen().unwrap();
        assert_eq!(chosen.model(), "mistral-small");
        assert_eq!(chosen.on_this_machine().unwrap().which(), Which::Catalogue);
    }

    /// **A file nobody has written is a person who has not chosen**, which is
    /// the state every machine is in on its first day.
    #[test]
    fn a_session_with_no_file_in_it_has_chosen_nothing() {
        let config = a_directory_of_our_own("no-file");

        let settings = of_a_session(Some(config.as_os_str()), None).unwrap();

        assert!(settings.chosen().is_none());
    }

    /// **A login with no home at all is the same answer**, and nothing on the
    /// disk was looked at to reach it.
    #[test]
    fn a_login_with_nowhere_to_keep_settings_has_chosen_nothing() {
        let settings = of_a_session(None, None).unwrap();

        assert!(settings.chosen().is_none());
        assert!(settings.brought().weights.is_empty());
    }

    /// **A file that does not hold is refused, not read past.** The refusal is
    /// `alo-choosing`'s and it names the file, so the person is sent to the
    /// line they have to fix rather than being told they chose nothing.
    #[test]
    fn a_file_that_does_not_hold_is_refused_and_names_itself() {
        let config = a_session_that_chose("not-toml", "format = 1\n[answers\n");

        let refused = of_a_session(Some(config.as_os_str()), None).unwrap_err();

        assert_eq!(
            refused.at(),
            config
                .join(alo_choosing::THE_FOLDER)
                .join(alo_choosing::THE_SETTINGS)
        );
        assert!(
            matches!(refused, NotSet::NotUnderstood { .. }),
            "{refused:?}"
        );
    }

    /// **A relative directory is not followed**, so a session that names
    /// somewhere relative to wherever the daemon happens to have been started
    /// falls back to the home directory rather than to a path under it.
    #[test]
    fn a_relative_configuration_directory_falls_back_to_the_home_directory() {
        let home = a_directory_of_our_own("relative");
        let folder = home.join(".config").join(alo_choosing::THE_FOLDER);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join(alo_choosing::THE_SETTINGS),
            "format = 1\n\n[answers]\nbrought = \"mine\"\n\n[[brought]]\nid = \"mine\"\nbytes-on-disk = 4000000000\ndrives-verbs = \"reliably\"\n",
        )
        .unwrap();

        let settings = of_a_session(Some(OsStr::new("somewhere")), Some(home.as_os_str())).unwrap();

        let chosen = settings.chosen().unwrap();
        assert_eq!(chosen.model(), "mine");
        assert_eq!(chosen.on_this_machine().unwrap().which(), Which::Brought);
    }
}
