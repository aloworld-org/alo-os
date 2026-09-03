//! Where this session keeps its runtime files, and what happens when nothing
//! says.
//!
//! `crate::place` decides the socket's name — `alo/agentd.sock` — and is handed
//! the directory it goes beneath. This is where that directory comes from, and
//! it is the one thing about this machine that is **not** in its description:
//! `$XDG_RUNTIME_DIR` is per-login, made when the person signs in and emptied
//! when they sign out, so it is a fact about the session `alo-agentd` was
//! started inside rather than a decision anybody wrote down.
//!
//! # Nothing is guessed at
//!
//! When the variable is not set, this refuses. The two things it could have done
//! instead are both worse than not starting:
//!
//! - **`/tmp`** is writable by everybody, so a directory there is one anybody
//!   can create first — and `crate::place` would then correctly refuse it as
//!   somebody else's, one layer too late to say anything useful. A socket the
//!   person's approvals travel over does not go in a shared directory.
//! - **`/run/user/<uid>`, worked out from the login**, is what the variable
//!   almost always says, and *almost always* is the problem: it is the session
//!   manager's to create, and a service that made it because it wanted a name
//!   would be standing in for a session that has not started.
//!
//! Refusing says what to do — start `alo-agentd` as a user service of the
//! signed-in person — which is `alo-models`' rule about what a refusal is for,
//! and it is the whole of ADR 0001 §2's *no ambient authority* at the level of a
//! directory: the person's session is what this service borrows its place from.
//!
//! # The value is passed in rather than read here
//!
//! [`where_it_runs`] takes what the variable said and [`from_the_environment`]
//! is the one line that goes and looks. That is not only for tidiness: in
//! edition 2024 `std::env::set_var` is `unsafe`, `CLAUDE.md` forbids `unsafe`
//! workspace-wide, and a test that cannot set the variable cannot exercise a
//! function that reads it. A decision taken over a value somebody passes in is a
//! decision with a test; one taken over the process's own environment is a
//! decision with a comment.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::refusing::NoSession;

/// The variable a session says where its runtime files go in.
///
/// Named here rather than written out at the two places that use it, because it
/// appears in a refusal a person reads and in a contract other people write
/// against.
pub const THE_VARIABLE: &str = "XDG_RUNTIME_DIR";

/// The directory this session's runtime files go in, as the environment says.
///
/// # Errors
///
/// [`NoSession`], which is [`where_it_runs`]'s.
pub fn from_the_environment() -> Result<PathBuf, NoSession> {
    where_it_runs(std::env::var_os(THE_VARIABLE).as_deref())
}

/// The directory this session's runtime files go in, given what the variable
/// said.
///
/// # Errors
///
/// [`NoSession::NotSet`] when nothing said, or when what it said was empty —
/// which is a variable that has been unset by being emptied and is not a
/// directory called nothing. [`NoSession::NotAbsolute`] otherwise, because the
/// XDG specification says the value is an absolute path and a relative one would
/// put the socket wherever the service happened to be started.
pub fn where_it_runs(said: Option<&OsStr>) -> Result<PathBuf, NoSession> {
    let Some(said) = said.filter(|said| !said.is_empty()) else {
        return Err(NoSession::NotSet);
    };
    let at = Path::new(said);
    if !at.is_absolute() {
        return Err(NoSession::NotAbsolute { at: at.to_owned() });
    }
    Ok(at.to_owned())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// What a session on an ordinary machine says.
    #[test]
    fn the_ordinary_machine_is_the_one_the_variable_names() {
        let where_it_is = where_it_runs(Some(OsStr::new("/run/user/1000"))).unwrap();
        assert_eq!(where_it_is, Path::new("/run/user/1000"));
    }

    /// **Nothing said is refused rather than guessed at**, and the refusal says
    /// what to do about it.
    #[test]
    fn nothing_said_is_refused_and_says_what_to_do() {
        let refused = where_it_runs(None).unwrap_err();
        assert_eq!(refused, NoSession::NotSet);
        assert!(refused.to_string().contains("user service"), "{refused}");
    }

    /// **A variable set to nothing is a variable that is not set.** Emptying one
    /// is how a variable is taken away in a service file, and a directory whose
    /// name is the empty string is not a directory.
    #[test]
    fn a_variable_set_to_nothing_is_not_set() {
        assert_eq!(
            where_it_runs(Some(OsStr::new(""))).unwrap_err(),
            NoSession::NotSet
        );
    }

    /// **A relative path is refused**, because where the socket went would then
    /// depend on the directory somebody happened to start the service in.
    #[test]
    fn a_relative_path_is_refused() {
        let refused = where_it_runs(Some(OsStr::new("run/user/1000"))).unwrap_err();
        assert_eq!(
            refused,
            NoSession::NotAbsolute {
                at: PathBuf::from("run/user/1000")
            }
        );
    }

    /// **Neither of the two guesses is made**, and this is the test that says
    /// so: nothing here answers with a path nobody named.
    #[test]
    fn no_directory_is_invented_when_nothing_said() {
        assert!(where_it_runs(None).is_err());
        assert!(where_it_runs(Some(OsStr::new("   "))).is_err());
    }

    /// The name of the variable is one string, because it is in a refusal a
    /// person reads and in a contract other people write against.
    #[test]
    fn the_variable_is_named_once() {
        assert_eq!(THE_VARIABLE, "XDG_RUNTIME_DIR");
        assert!(NoSession::NotSet.to_string().contains(THE_VARIABLE));
    }
}
