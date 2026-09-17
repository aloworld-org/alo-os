//! Whom a drive is mounted for: a login name, and which one a user number has.
//!
//! The disk service mounts a filesystem *as a user* by that user's name, and it
//! puts the mount where that user's own session finds it. The broker is told no
//! name — a request carries thirty-two bytes and nothing else — so the name is
//! the one this machine's own account file gives the person the door is for,
//! read with [`LoginName::of_user`], and it is refused unless exactly one
//! account has that number and its name is shaped as a login name is.

/// The most bytes a login name may be, as the account tools on the base allow.
const LONGEST: usize = 32;

/// A login name: a lowercase letter or an underscore, then lowercase letters,
/// digits, underscores and dashes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoginName(String);

impl LoginName {
    /// Some text, as a login name, or nothing when it is not shaped as one.
    ///
    /// Nothing is trimmed or lowered: a name with a capital or a space in it
    /// did not come from an account file this machine wrote.
    #[must_use]
    pub fn named(text: &str) -> Option<Self> {
        let mut bytes = text.bytes();
        let first = bytes.next()?;
        let shaped = text.len() <= LONGEST
            && (first.is_ascii_lowercase() || first == b'_')
            && bytes
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-');
        shaped.then(|| Self(text.to_owned()))
    }

    /// The login name of the one account with user number `user` in an account
    /// file written as `/etc/passwd` is — or nothing when there is none, when
    /// there are two, or when its name is not a login name.
    #[must_use]
    pub fn of_user(accounts: &str, user: u32) -> Option<Self> {
        let number = user.to_string();
        let mut named = accounts
            .lines()
            .filter_map(|line| {
                let mut fields = line.split(':');
                let name = fields.next()?;
                let _password = fields.next()?;
                (fields.next()? == number).then_some(name)
            })
            .map(Self::named);
        match (named.next(), named.next()) {
            (Some(one), None) => one,
            _ => None,
        }
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The accounts the image declares, as the base writes them.
    const ACCOUNTS: &str = "root:x:0:0:Super User:/root:/bin/bash\n\
                            alo:x:1000:1000:alo OS:/home/alo:/bin/bash\n\
                            alo-agent:x:60989:60989::/:/usr/sbin/nologin\n";

    /// **The person's number names the person's login**, and the agent's names
    /// the agent's.
    #[test]
    fn a_user_number_names_its_one_account() {
        assert_eq!(LoginName::of_user(ACCOUNTS, 1000).unwrap().as_str(), "alo");
        assert_eq!(
            LoginName::of_user(ACCOUNTS, 60989).unwrap().as_str(),
            "alo-agent"
        );
    }

    /// **No account, two accounts, or a name that is not a login name is
    /// nobody**, and a number is matched whole rather than as a prefix.
    #[test]
    fn an_account_that_is_not_exactly_one_login_is_nobody() {
        assert_eq!(LoginName::of_user(ACCOUNTS, 1001), None);
        assert_eq!(LoginName::of_user(ACCOUNTS, 100), None);
        let twice = format!("{ACCOUNTS}other:x:1000:1000::/:/bin/sh\n");
        assert_eq!(LoginName::of_user(&twice, 1000), None);
        let odd = "Alo Person:x:1000:1000::/:/bin/sh\n";
        assert_eq!(LoginName::of_user(odd, 1000), None);
        assert_eq!(LoginName::of_user("", 1000), None);
    }

    /// **A login name has one shape**, and nothing a program would read as an
    /// option or a path has it.
    #[test]
    fn a_login_name_has_one_shape() {
        for good in ["alo", "_backup", "a1-b_c"] {
            assert!(LoginName::named(good).is_some(), "{good}");
        }
        for bad in [
            "",
            "Alo",
            "1alo",
            "-alo",
            "alo ",
            "a/b",
            "a:b",
            &"a".repeat(33),
        ] {
            assert_eq!(LoginName::named(bad), None, "{bad:?}");
        }
    }
}
