//! One local account: a name, a number, and the hash of a password.
//!
//! The account alo OS's exit gate opens with — *sign in* — and nothing more.
//! No identity provider, no tenant, no network: those are the other half of
//! phase 4 and are deliberately not represented here, so there is no field
//! for them to grow into quietly.
//!
//! # The name is a Unix login's name
//!
//! The uid ties an account to the login the image declares
//! (`usr/lib/sysusers.d/alo.conf`), so the name is held to the shape a Unix
//! login already has to have: ascii lowercase, digits, `-` and `_`, starting
//! with a letter, at most 32 long. A looser name here would be an account the
//! rest of the machine cannot have.

use crate::hashing::Hashed;
use crate::refusing::NotCreated;

/// The longest a name may be, which is the limit `useradd` already imposes.
const LONGEST_NAME: usize = 32;

/// The value every Unix call answers with when there is no user.
const NOBODY_AT_ALL: u32 = u32::MAX;

/// One account on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// What it is called.
    name: String,
    /// The login number it signs in as.
    uid: u32,
    /// The hash of its password — never the password.
    secret: Hashed,
}

impl Account {
    /// An account created now: the name and number checked, the password
    /// hashed with a fresh salt.
    ///
    /// # Errors
    ///
    /// [`NotCreated::NotAName`] for a name a Unix login cannot have,
    /// [`NotCreated::Root`] for uid 0, [`NotCreated::NobodyAtAll`] for the
    /// no-user value, [`NotCreated::NoPassword`] for an empty password, and
    /// [`NotCreated::CouldNotHash`] when the machine's randomness is
    /// unreachable.
    pub fn created(name: &str, uid: u32, password: &str) -> Result<Self, NotCreated> {
        let name = a_name(name)?;
        a_number(uid)?;
        if password.is_empty() {
            return Err(NotCreated::NoPassword);
        }
        Ok(Self {
            name,
            uid,
            secret: Hashed::of(password)?,
        })
    }

    /// An account as the store wrote it down, its hash already read.
    ///
    /// The same checks as [`Account::created`] apply to the name and the
    /// number: a store edited by hand is exactly as untrusted as a keyboard.
    ///
    /// # Errors
    ///
    /// [`NotCreated`], as [`Account::created`] — except the password's, which
    /// only a hash can fail here and `crate::hashing` has already refused.
    pub(crate) fn read(name: &str, uid: u32, secret: Hashed) -> Result<Self, NotCreated> {
        let name = a_name(name)?;
        a_number(uid)?;
        Ok(Self { name, uid, secret })
    }

    /// What it is called.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The login number it signs in as.
    #[must_use]
    pub const fn uid(&self) -> u32 {
        self.uid
    }

    /// The hash, for the store to verify against and write down.
    pub(crate) const fn secret(&self) -> &Hashed {
        &self.secret
    }
}

/// The name, believed only in the shape a Unix login already has to have.
fn a_name(written: &str) -> Result<String, NotCreated> {
    let starts_well = written
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_lowercase());
    let all_of_it = written
        .chars()
        .all(|one| one.is_ascii_lowercase() || one.is_ascii_digit() || one == '-' || one == '_');
    if !starts_well || !all_of_it || written.len() > LONGEST_NAME {
        return Err(NotCreated::NotAName);
    }
    Ok(written.to_owned())
}

/// The number, refused where it could not be a person.
const fn a_number(uid: u32) -> Result<(), NotCreated> {
    match uid {
        0 => Err(NotCreated::Root),
        NOBODY_AT_ALL => Err(NotCreated::NobodyAtAll),
        _ => Ok(()),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The ordinary case: the login the image itself declares.
    #[test]
    fn the_account_the_image_would_make_is_created() {
        let account = Account::created("alo", 1000, "correct horse battery staple").unwrap();
        assert_eq!(account.name(), "alo");
        assert_eq!(account.uid(), 1000);
        assert!(account.secret().verifies("correct horse battery staple"));
    }

    /// **Every name a Unix login cannot have is refused**, one shape at a
    /// time: empty, capitalised, leading digit, whitespace, a path separator,
    /// the `:` that delimits `/etc/passwd`, anything beyond ascii, and one
    /// character too long.
    #[test]
    fn a_name_a_unix_login_cannot_have_is_refused() {
        for wrong in [
            "",
            "Ada",
            "1ada",
            "ada lovelace",
            "ada/",
            "ada:",
            "adá",
            "-ada",
            "a_name_that_is_thirtythree_chars_",
        ] {
            assert_eq!(
                Account::created(wrong, 1000, "a password"),
                Err(NotCreated::NotAName),
                "`{wrong}` was accepted as a name"
            );
        }
    }

    /// And the shapes that look odd and are fine: digits, hyphens and
    /// underscores after a lowercase start, up to the limit.
    #[test]
    fn a_name_a_unix_login_can_have_is_accepted() {
        for fine in ["alo", "ada-l_1", "a", "a_name_that_is_thirtytwo_chars__"] {
            assert!(
                Account::created(fine, 1000, "a password").is_ok(),
                "`{fine}` was refused as a name"
            );
        }
    }

    /// **Root is refused.** The person `alo-agentd` runs as is never root
    /// (ADR 0001 §2), so an account at 0 would put every authority the
    /// capability model withholds into the daemon's own hands.
    #[test]
    fn an_account_at_uid_zero_is_refused() {
        assert_eq!(
            Account::created("alo", 0, "a password"),
            Err(NotCreated::Root)
        );
    }

    /// **The no-user value is refused**, because it is what a script that
    /// could not look a user up leaves behind, never a person.
    #[test]
    fn the_value_that_means_no_user_is_refused() {
        assert_eq!(
            Account::created("alo", u32::MAX, "a password"),
            Err(NotCreated::NobodyAtAll)
        );
    }

    /// **An empty password is refused at creation** — the one password rule
    /// this crate imposes, because anything more is product policy and
    /// anything less is an account whose door is open.
    #[test]
    fn an_account_with_no_password_is_refused() {
        assert_eq!(
            Account::created("alo", 1000, ""),
            Err(NotCreated::NoPassword)
        );
    }
}
