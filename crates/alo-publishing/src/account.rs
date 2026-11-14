//! **A GitHub account's name**, checked by the rule GitHub gives one — so the
//! address in the sentence a person approves is an address that can exist, and
//! nothing but letters, numbers and hyphens ever reaches the command alo runs.
//!
//! GitHub's rule, from its own sign-up form: *may only contain alphanumeric
//! characters or single hyphens, and cannot begin or end with a hyphen*, at most
//! 39 characters. GitHub treats a name without regard to case, so it is kept in
//! lower case: `Anna` and `anna` are one account and one address.

/// The longest name GitHub gives an account.
pub const THE_LONGEST: usize = 39;

/// A GitHub account's name, in lower case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account(String);

impl Account {
    /// `name`, if it is one GitHub could give an account.
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        let fits = !name.is_empty()
            && name.chars().count() <= THE_LONGEST
            && name
                .chars()
                .all(|one| one.is_ascii_alphanumeric() || one == '-')
            && !name.starts_with('-')
            && !name.ends_with('-')
            && !name.contains("--");
        fits.then(|| Self(name.to_ascii_lowercase()))
    }

    /// The name, in lower case.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }

    /// The repository GitHub Pages serves this account's site from:
    /// `<name>/<name>.github.io`.
    #[must_use]
    pub fn the_sites_repository(&self) -> String {
        format!("{0}/{0}.github.io", self.0)
    }

    /// Where anybody reads the site: `https://<name>.github.io/`.
    #[must_use]
    pub fn address(&self) -> String {
        format!("https://{}.github.io/", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Account;

    #[test]
    fn a_name_github_gives_is_an_account_in_lower_case() {
        for (asked, kept) in [
            ("anna", "anna"),
            ("Anna-Smith", "anna-smith"),
            ("a1", "a1"),
            ("x", "x"),
            (
                "abcdefghijklmnopqrstuvwxyz0123456789abc",
                "abcdefghijklmnopqrstuvwxyz0123456789abc",
            ),
        ] {
            assert_eq!(
                Account::named(asked).map(|it| it.name().to_owned()),
                Some(kept.to_owned())
            );
        }
        let anna = Account::named("Anna").map(|it| (it.the_sites_repository(), it.address()));
        assert_eq!(
            anna,
            Some((
                "anna/anna.github.io".to_owned(),
                "https://anna.github.io/".to_owned()
            ))
        );
    }

    /// **The refusal path**: nothing that could not be a GitHub name — and
    /// above all nothing a shell would read as more than a word — is an
    /// account.
    #[test]
    fn nothing_github_would_not_give_is_an_account() {
        for asked in [
            "",
            "-anna",
            "anna-",
            "an--na",
            "anna smith",
            "anna.github.io",
            "anna/site",
            "anna;reboot",
            "$(reboot)",
            "anna'",
            "änna",
            "abcdefghijklmnopqrstuvwxyz0123456789abcd",
        ] {
            assert_eq!(Account::named(asked), None, "{asked:?} became an account");
        }
    }
}
