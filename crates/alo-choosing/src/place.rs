//! Where a person's settings are, and how that is worked out.
//!
//! [ADR 0016](../../../docs/decisions/0016-the-organisation-bounds-and-the-person-chooses.md)
//! puts the person's file under `$XDG_CONFIG_HOME/alo/` and says why: a machine
//! may have several people on it, and one person's model is not another's. It is
//! not in `/etc`, where an administrator would be reading somebody's
//! preferences on their own machine.
//!
//! # Nothing here reads the environment
//!
//! The two variables arrive as arguments, which is item 1's *nothing reads the
//! clock* applied to the other thing a process is surrounded by. A machine with
//! `$XDG_CONFIG_HOME` set to a relative path, a machine with neither variable,
//! and a service started with an empty environment are then three tests rather
//! than three things somebody has to arrange on a real login. The process that
//! really has an environment is `alo-agentd`, and it says so where it reads one.
//!
//! # The rule is the specification's, including the part that surprises people
//!
//! `$XDG_CONFIG_HOME` wins when it is set **and absolute**. The base directory
//! specification says a relative value is invalid and must be ignored, and that
//! matters more here than it reads: a relative path would put a person's
//! settings wherever the service happened to be started from, and somewhere
//! different the next time — which is the reason
//! `docs/contracts/machine-description.md` refuses a relative record path.
//!
//! An unset or empty variable falls back to `$HOME/.config`. A machine with no
//! `$HOME` either has nowhere for this file to be, and that is [`None`] rather
//! than a guess: `/` is not somebody's home directory, and a service that
//! invented one would read settings that belong to nobody.
//!
//! **Absolute is asked as [`Path::has_root`] rather than
//! [`Path::is_absolute`]**, and the difference is the host rather than the
//! path: `Path::is_absolute` answers about the machine it is compiled for, so
//! `/home/ada` is *relative* on Windows, where there is no drive letter in
//! front of it. This crate is built on both and the rule it is stating is
//! Linux's, so the question asked is the one that means the same thing
//! everywhere. `docs/quirks.md` records it, and `alo-saying` found it first.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The variable that names where a person's configuration is kept.
pub const CONFIG_HOME: &str = "XDG_CONFIG_HOME";

/// The variable that names a person's home directory.
pub const HOME: &str = "HOME";

/// The directory alo OS keeps a person's settings in.
pub const THE_FOLDER: &str = "alo";

/// The file itself, inside that directory.
///
/// Part of `docs/contracts/person-settings.md`, because a settings panel writes
/// this file and a person may edit it: where it is and what is in it are things
/// other people build against.
pub const THE_SETTINGS: &str = "settings.toml";

/// What `$HOME` is followed by when `$XDG_CONFIG_HOME` says nothing.
const DOT_CONFIG: &str = ".config";

/// Where this person's settings are, given what their session says.
///
/// `config_home` is `$XDG_CONFIG_HOME` and `home` is `$HOME`, each as the
/// process really has it — unset arrives as [`None`].
///
/// [`None`] when neither is usable, which is a login with no home directory.
/// There is nothing to read and nothing to write, and the caller says *nothing
/// has been chosen* rather than reading somewhere it made up.
#[must_use]
pub fn where_it_is(config_home: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    let under = |directory: &Path| directory.join(THE_FOLDER).join(THE_SETTINGS);
    if let Some(config) = config_home.map(Path::new)
        && config.has_root()
    {
        return Some(under(config));
    }
    match home.map(Path::new) {
        Some(home) if home.has_root() => Some(under(&home.join(DOT_CONFIG))),
        Some(_) | None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A path, as the environment hands one over.
    fn said(text: &str) -> &OsStr {
        OsStr::new(text)
    }

    /// Where the file is on an ordinary Linux login, both ways round.
    ///
    /// Written against the text rather than through `Path::join`'s own output,
    /// because a separator is the host's: this crate is built on Windows too,
    /// and what is being asserted is the shape rather than the character.
    fn as_written(at: Option<PathBuf>) -> Option<String> {
        at.map(|at| at.to_string_lossy().replace('\\', "/"))
    }

    /// **`$XDG_CONFIG_HOME` is where it looks**, which is what makes this
    /// per-person rather than per-machine.
    #[test]
    fn a_persons_settings_are_under_the_directory_their_session_names() {
        assert_eq!(
            as_written(where_it_is(
                Some(said("/home/ada/.config")),
                Some(said("/home/ada"))
            ))
            .as_deref(),
            Some("/home/ada/.config/alo/settings.toml")
        );
    }

    /// **And `$HOME/.config` when nothing named one**, which is the
    /// specification's fallback and the common case on a machine nobody has
    /// configured.
    #[test]
    fn a_session_that_names_nothing_falls_back_to_the_home_directory() {
        assert_eq!(
            as_written(where_it_is(None, Some(said("/home/ada")))).as_deref(),
            Some("/home/ada/.config/alo/settings.toml")
        );
        assert_eq!(
            as_written(where_it_is(Some(said("")), Some(said("/home/ada")))).as_deref(),
            Some("/home/ada/.config/alo/settings.toml")
        );
    }

    /// **A relative `$XDG_CONFIG_HOME` is ignored**, which the specification
    /// requires and this repository would have required anyway: it would put a
    /// person's settings wherever the service was started from, and somewhere
    /// else the next time.
    #[test]
    fn a_relative_configuration_directory_is_ignored_rather_than_followed() {
        assert_eq!(
            as_written(where_it_is(Some(said("config")), Some(said("/home/ada")))).as_deref(),
            Some("/home/ada/.config/alo/settings.toml")
        );
    }

    /// **A login with no home directory has nowhere for this file**, and the
    /// answer is nothing rather than a path under `/`. A service that invented
    /// one would read settings belonging to nobody, and on a shared machine
    /// that is somebody else's answer to *where do my questions go*.
    #[test]
    fn a_login_with_no_home_at_all_has_no_settings_file() {
        assert_eq!(where_it_is(None, None), None);
        assert_eq!(where_it_is(Some(said("config")), None), None);
        assert_eq!(where_it_is(None, Some(said("ada"))), None);
    }

    /// The two names are one string each: they are in the contract a settings
    /// panel is written against and in the argument `alo-agentd` passes.
    #[test]
    fn the_file_is_named_once() {
        assert_eq!(THE_FOLDER, "alo");
        assert_eq!(THE_SETTINGS, "settings.toml");
        assert_eq!(CONFIG_HOME, "XDG_CONFIG_HOME");
        assert_eq!(HOME, "HOME");
    }
}
