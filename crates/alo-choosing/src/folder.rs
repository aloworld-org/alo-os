//! The person's folder as a Settings surface holds it: the folder, or the
//! reason there is none — never a bare nothing.
//!
//! [`crate::where_the_folder_is`] answers [`None`] for a login with no home
//! directory, and that is right: there is nowhere to keep anything, and a
//! folder invented under `/` or `/tmp` would be one that belongs to nobody. But
//! an `Option` is one `if let` from a surface that draws every section as the
//! release ships it, lets a person change the dock, and says nothing — and the
//! person finds the dock back where it was at the next sign-in. *Nothing leaves
//! silently* has a twin here: nothing is forgotten silently either.
//!
//! So [`the_persons_folder`] answers a [`PersonsFolder`] or a [`NoFolder`]. The
//! only road to the path a keeper is handed is [`PersonsFolder::path_of`], so a
//! surface cannot hand `alo_dock::keeping::keep` a path without having matched
//! the case where there is none; and the only thing a [`NoFolder`] can do is
//! say so, in [`NoFolder::said`] — that a change made now takes effect and will
//! not be kept past this sign-in, and why.
//!
//! # A refusal that holds no path
//!
//! [`NoFolder`] carries what was wrong with `$HOME` and nothing else. It has no
//! path in it to write to, which is how *nothing is written anywhere* is held in
//! type rather than in each surface's care: a relative `$HOME` is not followed
//! from wherever the session started, and there is no fallback folder.
//!
//! # Nothing here reads the environment either
//!
//! The two variables arrive as arguments, for `crate::place`'s reason, and the
//! rule is that module's rather than a second copy of it:
//! [`the_persons_folder`] answers a folder exactly when
//! [`crate::where_the_folder_is`] does.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use alo_strings::{Filling, Said, Strings};

use crate::place::where_the_folder_is;
use crate::words;

/// The person's folder, worked out at sign-in from what the session says.
///
/// `config_home` is `$XDG_CONFIG_HOME` and `home` is `$HOME`, each as the
/// process really has it — unset arrives as [`None`].
///
/// # Errors
/// [`NoFolder`] when neither variable names an absolute directory: a login with
/// no home directory, whose changes a Settings surface draws and cannot keep.
pub fn the_persons_folder(
    config_home: Option<&OsStr>,
    home: Option<&OsStr>,
) -> Result<PersonsFolder, NoFolder> {
    match where_the_folder_is(config_home, home) {
        Some(folder) => Ok(PersonsFolder { folder }),
        None => Err(NoFolder {
            home: match home {
                None => HomeWas::Unset,
                Some(_) => HomeWas::NotAbsolute,
            },
        }),
    }
}

/// A person's folder that exists to be kept in.
///
/// Made only by [`the_persons_folder`], so holding one means the session had
/// somewhere of the person's own to keep their settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonsFolder {
    /// `…/alo`, under `$XDG_CONFIG_HOME` or `$HOME/.config`.
    folder: PathBuf,
}

impl PersonsFolder {
    /// The folder itself.
    #[must_use]
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// The path a keeper is handed: the folder joined with the file name that
    /// keeper declares, such as `alo_dock::keeping::THE_FILE`.
    #[must_use]
    pub fn path_of(&self, file: &str) -> PathBuf {
        self.folder.join(file)
    }
}

/// Why `$HOME` could not be kept in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeWas {
    /// The session has no `$HOME` at all.
    Unset,
    /// `$HOME` is empty or relative, and following it would put a person's
    /// settings wherever the session was started from.
    NotAbsolute,
}

/// A session with no folder to keep a person's settings in.
///
/// Deliberately no `Display`, which is item 9b's rule and every refusal's in
/// this crate: the only road to words is [`said`](NoFolder::said).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoFolder {
    /// What was wrong with `$HOME`, for whoever is fixing the login. A person
    /// is told one sentence for both, because what they act on is the same.
    home: HomeWas,
}

impl NoFolder {
    /// What was wrong with `$HOME`.
    #[must_use]
    pub const fn home(&self) -> HomeWas {
        self.home
    }

    /// What a Settings surface says, in the language the person reads: a
    /// change made now takes effect and is not kept past this sign-in, because
    /// there is no home directory to keep it in.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&words::SESSION_NO_FOLDER.key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// A variable, as the environment hands one over.
    fn said(text: &str) -> &OsStr {
        OsStr::new(text)
    }

    /// **A session with a home has a folder**, the one `where_the_folder_is`
    /// names, and a keeper's path is that folder and its file.
    #[test]
    fn a_session_with_a_home_holds_the_folder() {
        let folder = the_persons_folder(None, Some(said("/home/ada"))).unwrap();
        assert_eq!(
            Some(folder.folder().to_owned()),
            where_the_folder_is(None, Some(said("/home/ada")))
        );
        assert_eq!(
            folder.path_of("dock.toml"),
            folder.folder().join("dock.toml")
        );
    }

    /// **No `$HOME` is refused**, and says so.
    #[test]
    fn no_home_is_no_folder() {
        let refused = the_persons_folder(None, None).unwrap_err();
        assert_eq!(refused.home(), HomeWas::Unset);
    }

    /// **A relative `$XDG_CONFIG_HOME` with no `$HOME` is refused** rather than
    /// followed from wherever the session started.
    #[test]
    fn a_relative_configuration_directory_and_no_home_is_no_folder() {
        let refused = the_persons_folder(Some(said("config")), None).unwrap_err();
        assert_eq!(refused.home(), HomeWas::Unset);
    }

    /// **A relative or empty `$HOME` is refused**, and it is not the same
    /// reason as none.
    #[test]
    fn a_relative_home_is_no_folder() {
        for home in ["ada", ""] {
            let refused = the_persons_folder(None, Some(said(home))).unwrap_err();
            assert_eq!(refused.home(), HomeWas::NotAbsolute, "{home:?}");
        }
    }

    /// **The sentence is the vocabulary's**, whole, untranslated in English and
    /// translated where somebody translated it.
    #[test]
    fn the_refusal_is_said_in_the_readers_language() {
        let refused = the_persons_folder(None, None).unwrap_err();
        let english = refused.said(&in_english());
        assert!(!english.is_a_bug(), "{english}");
        assert!(english.unfilled().is_empty(), "{english}");
        assert!(english.text().contains("will be kept"), "{english}");

        let german = refused.said(&translated(&[(
            words::SESSION_NO_FOLDER,
            "diese Anmeldung hat kein Home-Verzeichnis",
        )]));
        assert!(german.is_translated(), "{german}");
        assert!(german.text().contains("Home-Verzeichnis"), "{german}");
    }
}
