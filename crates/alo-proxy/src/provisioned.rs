//! The machine's own passwords, as a unit was given them.
//!
//! [ADR 0059](../../../docs/decisions/0059-where-a-machine-wide-proxy-password-is-kept.md)
//! decided where a **machine-wide** proxy password lives, and this file is that
//! decision as something a road can ask. It is the second half of
//! [`crate::WhereThePasswordIs`], exactly as `alo-secrets` is the second half of
//! `alo_models::SecretRef`: that type is a name, and this is what the name
//! refers to.
//!
//! # Why not the keyring ADR 0022 chose
//!
//! Because of *when*, and because of *whose*. A provider's key is the person's
//! and is wanted inside a turn, with them signed in; a machine's proxy password
//! is the organisation's and is wanted at `multi-user.target`, by a unit asking
//! whether there is an update, before anybody has signed in at all. ADR 0059's
//! second section is the whole argument; it is not repeated here.
//!
//! What is worth repeating is the consequence: **nothing in this file opens a
//! bus, needs a session or names a person.** A credential the machine was given
//! is read from a directory that was there before the login screen was.
//!
//! # The unit is named, never worked out
//!
//! [`TheMachinesPasswords::given_to`] takes a unit's name and nothing else, and
//! builds `/run/credentials/<unit>/<entry>` from it. `alo_secrets::TheBus::of`
//! takes a uid and nothing else for the same reason, and states it better than
//! this file could: a function that takes a name cannot be pointed somewhere by
//! a variable.
//!
//! So `$CREDENTIALS_DIRECTORY` is **not read here and there is no code path
//! that could**. It is the variable systemd exports for this, and reading it
//! would be one environment variable deciding where alo OS looks for a
//! credential — the thing ADR 0022 refused `DBUS_SESSION_BUS_ADDRESS` for. Each
//! road's crate writes its own unit's name as a constant instead, which is also
//! the list of unit files that have to carry the line ADR 0059 names.
//!
//! # Nothing found is believed
//!
//! systemd's documented behaviour is that these files are a regular file each,
//! `0400`, owned by the unit's own user, on a mount that goes away when the
//! unit stops. **This file does not take that on trust**, because a credential
//! read out of something that is not that is a credential this machine cannot
//! say anything about: it checks that what it found is a regular file, that
//! nobody but its owner may read it, and that what it holds is a password
//! rather than a paragraph. ADR 0022 recorded another library's documented
//! behaviour as though it were measured and had to withdraw it; this file is
//! written so that nothing has to be withdrawn.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::password::{NotAPassword, Password, WhereThePasswordIs};
use crate::signing_in::{NotSignedIn, WhereThePasswordsAre};

/// Where a unit's own credentials are on a machine running systemd.
///
/// One directory per unit, named for the unit — `/run/credentials/alo-agentd.service`
/// — which is why [`TheMachinesPasswords::given_to`] takes the unit and joins
/// it here rather than being handed a whole path by a caller.
pub const WHERE_THEY_ARE: &str = "/run/credentials";

/// The most bytes a proxy password may be kept as.
///
/// Long enough for any password a company proxy asks for, short enough that
/// something which is not a password — a certificate somebody put under the
/// wrong name, a log — is refused as one rather than sent to a proxy.
pub const LONGEST_PASSWORD: u64 = 4096;

/// The passwords this machine was given, as one unit sees them.
///
/// Holds a directory and nothing else. Making one reaches nothing: a road that
/// goes straight out, and a proxy that asks for no name, never cause a single
/// read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheMachinesPasswords {
    /// The directory this unit's credentials are in.
    directory: PathBuf,
}

impl TheMachinesPasswords {
    /// The passwords the machine gave this unit.
    ///
    /// `unit` is the unit's own name as systemd writes it, suffix and all —
    /// `alo-agentd.service`. A crate writes its own as a constant beside the
    /// road it takes; there is no parameter anywhere that could name another
    /// machine's directory, and no variable is read.
    #[must_use]
    pub fn given_to(unit: &str) -> Self {
        Self {
            directory: Path::new(WHERE_THEY_ARE).join(unit),
        }
    }

    /// The same reader, somewhere else — a test's.
    ///
    /// Public because the roads this serves are in four other crates, and a
    /// test that cannot say *the machine was given this* can only show that the
    /// reading compiles. `crate::TheRentedEvaluator::at` is the same door for
    /// the same reason. It points a **reader** somewhere, never a writer and
    /// never a program: the worst a wrong path can do here is refuse a road.
    #[must_use]
    pub fn at(directory: &Path) -> Self {
        Self {
            directory: directory.to_path_buf(),
        }
    }

    /// The directory this reads, for a caller that has to say where it looked.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

impl WhereThePasswordsAre for TheMachinesPasswords {
    fn password(&self, kept: &WhereThePasswordIs) -> Result<Password, NotSignedIn> {
        let entry = one_thing_to_look_up(kept.as_str()).ok_or(NotSignedIn::NotALookableName)?;
        let at = self.directory.join(entry);

        let about = match fs::symlink_metadata(&at) {
            Ok(about) => about,
            Err(why) if why.kind() == ErrorKind::NotFound => {
                // Told apart so that *this unit was never given any* and *it was
                // given some and not this one* are different things for whoever
                // is fixing the machine, even though the person reads one
                // sentence for both.
                return Err(if self.directory.is_dir() {
                    NotSignedIn::NotKept
                } else {
                    NotSignedIn::NothingKeepsIt
                });
            }
            Err(why) => return Err(NotSignedIn::NotRead(why.kind())),
        };
        if !about.is_file() {
            return Err(NotSignedIn::NotKept);
        }
        if !only_its_owners(&about) {
            return Err(NotSignedIn::ReadableByAnybody);
        }
        if about.len() > LONGEST_PASSWORD {
            return Err(NotSignedIn::LongerThanAPassword);
        }

        let held = fs::read(&at).map_err(|why| NotSignedIn::NotRead(why.kind()))?;
        let held = String::from_utf8(held)
            .map_err(|_| NotSignedIn::NotAPassword(NotAPassword::NotSendable))?;
        Password::typed(&held).map_err(NotSignedIn::NotAPassword)
    }
}

/// The name as one thing to look up in a directory, or [`None`] where it is not
/// one.
///
/// `crate::WhereThePasswordIs` holds a name to one line, which is what a
/// settings panel needs; a directory needs more than that. A name with a
/// separator in it would name something beside the unit's own credentials, and
/// `..` would name the directory above them. Neither is refused by trimming it
/// into shape: a name that is not the name somebody wrote is a password looked
/// up somewhere nobody chose.
pub(crate) fn one_thing_to_look_up(name: &str) -> Option<&str> {
    if name.is_empty() || name == "." || name == ".." {
        return None;
    }
    if name.contains(['/', '\\', ':', '\0']) {
        return None;
    }
    Some(name)
}

/// Whether nobody but the file's owner may read it.
#[cfg(unix)]
pub(crate) fn only_its_owners(about: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt as _;

    about.permissions().mode() & 0o077 == 0
}

/// On a host that does not say it in a mode, this check has nothing to ask.
///
/// Said rather than left out: alo OS is a Linux system and this half exists so
/// that the crate builds where its tests are written, not because the question
/// has a second answer.
#[cfg(not(unix))]
#[expect(
    clippy::missing_const_for_fn,
    reason = "the same signature as the unix half, which reads permissions"
)]
pub(crate) fn only_its_owners(_about: &fs::Metadata) -> bool {
    true
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_directory_of_its_own;

    /// The name a company writes for its proxy's password.
    fn named() -> WhereThePasswordIs {
        WhereThePasswordIs::named("the company proxy").unwrap()
    }

    /// A password kept under that name, as a machine would have been given one.
    fn given(directory: &Path, name: &str, password: &str) {
        fs::create_dir_all(directory).expect("a directory");
        let at = directory.join(name);
        fs::write(&at, password).expect("a password");
        kept_the_way_a_machine_keeps_one(&at);
    }

    /// The permissions systemd gives a credential, so that a test reads what a
    /// machine would.
    #[cfg(unix)]
    fn kept_the_way_a_machine_keeps_one(at: &Path) {
        use std::os::unix::fs::PermissionsExt as _;

        fs::set_permissions(at, fs::Permissions::from_mode(0o400)).expect("a credential's mode");
    }

    /// Nothing to do where a mode is not what says it.
    #[cfg(not(unix))]
    fn kept_the_way_a_machine_keeps_one(_at: &Path) {}

    /// **The unit's name is what the directory is built from**, and nothing
    /// else reaches the decision.
    #[test]
    fn the_directory_is_built_from_the_units_name_and_nothing_else() {
        let passwords = TheMachinesPasswords::given_to("alo-agentd.service");
        assert_eq!(
            passwords.directory(),
            Path::new(WHERE_THEY_ARE).join("alo-agentd.service")
        );
    }

    /// **The password the machine was given is what comes back.**
    #[test]
    fn the_password_the_machine_was_given_is_what_comes_back() {
        let directory = a_directory_of_its_own("given-a-password");
        given(&directory, "the company proxy", "hunter2");

        let password = TheMachinesPasswords::at(&directory)
            .password(&named())
            .expect("the machine was given it");
        assert_eq!(format!("{password:?}"), "Password(…)");
    }

    /// A password written with the newline an editor leaves on the end is the
    /// password, which is `crate::Password::typed`'s rule and not a second one
    /// here.
    #[test]
    fn a_password_written_with_a_newline_on_the_end_is_the_password() {
        let directory = a_directory_of_its_own("a-newline-on-the-end");
        given(&directory, "the company proxy", "hunter2\n");
        assert!(
            TheMachinesPasswords::at(&directory)
                .password(&named())
                .is_ok()
        );
    }

    /// **A unit that was given no credentials at all is told apart** from one
    /// that was given some and not this.
    #[test]
    fn a_unit_given_nothing_and_a_unit_given_something_else_are_told_apart() {
        let nowhere = a_directory_of_its_own("given-nothing").join("never-made");
        assert_eq!(
            TheMachinesPasswords::at(&nowhere)
                .password(&named())
                .unwrap_err(),
            NotSignedIn::NothingKeepsIt
        );

        let directory = a_directory_of_its_own("given-something-else");
        given(&directory, "another-machines-proxy", "hunter2");
        assert_eq!(
            TheMachinesPasswords::at(&directory)
                .password(&named())
                .unwrap_err(),
            NotSignedIn::NotKept
        );
    }

    /// **A name that is not one thing to look up is refused**, and nothing is
    /// read — which is what keeps a lookup inside the unit's own directory.
    #[test]
    fn a_name_that_would_reach_outside_the_directory_is_refused() {
        let directory = a_directory_of_its_own("reaching-outside");
        given(&directory, "the company proxy", "hunter2");
        for name in [
            "../the company proxy",
            "..",
            ".",
            "a/b",
            "a\\b",
            "sneaky:name",
        ] {
            let kept = WhereThePasswordIs::named(name).unwrap();
            assert_eq!(
                TheMachinesPasswords::at(&directory)
                    .password(&kept)
                    .unwrap_err(),
                NotSignedIn::NotALookableName,
                "{name}"
            );
        }
    }

    /// **A password anybody on the machine could read is refused**, rather than
    /// sent to a proxy as though nothing were wrong.
    #[cfg(unix)]
    #[test]
    fn a_password_anybody_could_read_is_refused() {
        use std::os::unix::fs::PermissionsExt as _;

        let directory = a_directory_of_its_own("readable-by-anybody");
        given(&directory, "the company proxy", "hunter2");
        fs::set_permissions(
            directory.join("the company proxy"),
            fs::Permissions::from_mode(0o444),
        )
        .expect("a mode");

        assert_eq!(
            TheMachinesPasswords::at(&directory)
                .password(&named())
                .unwrap_err(),
            NotSignedIn::ReadableByAnybody
        );
    }

    /// **What is kept and is not a password is refused as one**, whether it is
    /// blank, unsendable or longer than a password.
    #[test]
    fn what_is_kept_and_is_not_a_password_is_refused() {
        let directory = a_directory_of_its_own("not-a-password");

        given(&directory, "the company proxy", "   \n");
        assert_eq!(
            TheMachinesPasswords::at(&directory)
                .password(&named())
                .unwrap_err(),
            NotSignedIn::NotAPassword(NotAPassword::Blank)
        );

        given(&directory, "the company proxy", "hun\u{1b}ter2");
        assert_eq!(
            TheMachinesPasswords::at(&directory)
                .password(&named())
                .unwrap_err(),
            NotSignedIn::NotAPassword(NotAPassword::NotSendable)
        );

        let too_much = "x".repeat(usize::try_from(LONGEST_PASSWORD).unwrap() + 1);
        given(&directory, "the company proxy", &too_much);
        assert_eq!(
            TheMachinesPasswords::at(&directory)
                .password(&named())
                .unwrap_err(),
            NotSignedIn::LongerThanAPassword
        );
    }

    /// **Something that is not a file is not a password**, so a directory left
    /// under the name is refused rather than read.
    #[test]
    fn something_under_the_name_that_is_not_a_file_is_refused() {
        let directory = a_directory_of_its_own("not-a-file");
        fs::create_dir_all(directory.join("the company proxy")).expect("a directory");
        assert_eq!(
            TheMachinesPasswords::at(&directory)
                .password(&named())
                .unwrap_err(),
            NotSignedIn::NotKept
        );
    }

    /// **The reader says nothing about what it read**, formatted or otherwise.
    #[test]
    fn the_reader_formatted_says_nothing_about_what_it_read() {
        let directory = a_directory_of_its_own("formatted");
        given(&directory, "the company proxy", "hunter2");
        let passwords = TheMachinesPasswords::at(&directory);
        let _ = passwords
            .password(&named())
            .expect("the machine was given it");
        assert!(!format!("{passwords:?}").contains("hunter2"));
    }
}
