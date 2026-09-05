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
//! [`what_this_person_chose`] is the only thing that reads the environment, so
//! the fix that day is a caller's change rather than a rewrite of whatever grew
//! on top of it.
//!
//! # A login with no home has chosen nothing
//!
//! `alo_choosing::where_it_is` answers `None` when neither variable names an
//! absolute directory, and there is then no file to read. That is a person who
//! has chosen nothing — the same state as a person whose file is not there yet
//! — and not a failure: a service that refused to serve a session without a
//! home directory would be refusing reads and changes over a model nobody
//! asked it about.

use std::ffi::OsStr;

use alo_choosing::{CONFIG_HOME, HOME, NotSet, Settings, where_it_is};

/// What the person this process runs as has chosen.
///
/// The environment is read here and nowhere else in this crate.
///
/// # Errors
///
/// [`NotSet`] when the file is there and cannot be read or does not hold: a
/// malformed file is refused in the person's own words rather than treated as
/// an empty one, because *you have chosen nothing* would be false and would
/// send them to a settings panel that already says otherwise.
pub fn what_this_person_chose() -> Result<Settings, NotSet> {
    of_a_session(
        std::env::var_os(CONFIG_HOME).as_deref(),
        std::env::var_os(HOME).as_deref(),
    )
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
        assert_eq!(chosen.which(), Which::Catalogue);
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
        assert_eq!(chosen.which(), Which::Brought);
    }
}
