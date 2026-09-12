//! What the build itself checks about the logins the image makes.
//!
//! `crate::logins` reads what `sysusers.d` **declares**; this reads what
//! `image/Containerfile` **asserts** after `systemd-sysusers` has run. They are
//! two different statements about the same five numbers, and the gap between
//! them is not theoretical: on the pinned base, `systemd-sysusers` does not fail
//! on a number somebody else has. It says one line in a build log and creates
//! the login with a different number — or puts it in the group that already
//! holds the one it was asked for, which is how alo OS's agent was quietly put
//! into `systemd-resolve`'s group the first time this image was built
//! (`docs/quirks.md`).
//!
//! So a declaration is a **request**, and the `test` lines at the foot of the
//! recipe are the only place it becomes a fact. A login added to `sysusers.d`
//! with no assertion beside it is a login whose number the machine may not have,
//! on a base that allocates system logins downward and can take any of them in
//! an update. Nothing but a reader of both files can see that, because both
//! files are individually correct.
//!
//! # Read leniently, judged strictly
//!
//! [`Asserted::read`] never refuses, for `crate::runtime`'s reason: a recipe
//! that asserts nothing reads as one that asserts nothing, and
//! `crate::checking` is what turns each absence into a [`Wrong`](crate::Wrong)
//! naming the decision it breaks.

/// What a login's number is asserted with, in a build step.
const A_LOGINS_NUMBER: &str = "$(id -u ";

/// What a group's number is asserted with.
const A_GROUPS_NUMBER: &str = "$(getent group ";

/// What a membership is asserted with: the groups a login is in, listed by name.
const THE_GROUPS_A_LOGIN_IS_IN: &str = "id -nG ";

/// What that listing is matched against, one whole line at a time.
const MATCHED_WHOLE: &str = "grep -qx ";

/// What the recipe's assertions say, read off its text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Asserted {
    /// Each login the build checks the number of, and the number it insists on.
    logins: Vec<(String, u32)>,
    /// Each group the build checks the number of, and the number it insists on.
    groups: Vec<(String, u32)>,
    /// Each membership the build checks: the login, and the group it must be in.
    memberships: Vec<(String, String)>,
}

impl Asserted {
    /// What this Containerfile asserts, read off its text.
    ///
    /// Comments are read past rather than searched, because the shape being
    /// looked for — a name and a number — is exactly what the prose above these
    /// lines explains at length, and a checker satisfied by a paragraph about an
    /// assertion is a checker satisfied by nothing.
    #[must_use]
    pub fn read(containerfile: &str) -> Self {
        let mut asserted = Self::default();

        for line in containerfile.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            if let Some(what) = numbered(line, A_LOGINS_NUMBER) {
                asserted.logins.push(what);
            }
            if let Some(what) = numbered(line, A_GROUPS_NUMBER) {
                asserted.groups.push(what);
            }
            if let Some(what) = membership(line) {
                asserted.memberships.push(what);
            }
        }

        asserted
    }

    /// The number the build insists this login has, where it insists on one.
    #[must_use]
    pub fn login_called(&self, name: &str) -> Option<u32> {
        found(&self.logins, name)
    }

    /// The number the build insists this group has, where it insists on one.
    #[must_use]
    pub fn group_called(&self, name: &str) -> Option<u32> {
        found(&self.groups, name)
    }

    /// Whether the build insists this login is in that group.
    #[must_use]
    pub fn puts(&self, login: &str, into: &str) -> bool {
        self.memberships
            .iter()
            .any(|(who, group)| who == login && group == into)
    }
}

/// The number a pair was written down with, where one was.
fn found(pairs: &[(String, u32)], name: &str) -> Option<u32> {
    pairs
        .iter()
        .find(|(called, _number)| called == name)
        .map(|(_called, number)| *number)
}

/// The name and number this line asserts, where it asserts one of this kind.
///
/// The name is whatever the command was asked about and the number is what the
/// comparison insists on, so `test "$(id -u alo)" = 1000` is `alo` and `1000`.
/// A line that names somebody and compares against nothing is not an assertion
/// and answers nothing.
fn numbered(line: &str, asked: &str) -> Option<(String, u32)> {
    let (_before, after) = line.split_once(asked)?;
    let name = after
        .split([')', '|', ' '])
        .next()
        .filter(|name| !name.is_empty())?;
    let (_before, compared) = line.rsplit_once('=')?;
    let number = compared
        .split_whitespace()
        .next()?
        .trim_matches('"')
        .parse()
        .ok()?;
    Some((name.to_owned(), number))
}

/// The login and group this line asserts a membership between, where it does.
fn membership(line: &str) -> Option<(String, String)> {
    let (_before, after) = line.split_once(THE_GROUPS_A_LOGIN_IS_IN)?;
    let login = after
        .split([' ', '|'])
        .next()
        .filter(|login| !login.is_empty())?;
    let (_before, after) = line.split_once(MATCHED_WHOLE)?;
    let group = after
        .split_whitespace()
        .next()
        .filter(|group| !group.is_empty())?;
    Some((login.to_owned(), group.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The foot of alo OS's own recipe, as the shape this reader is for.
    const THE_FOOT: &str = "\
RUN systemd-sysusers \\
 && test \"$(id -u alo)\" = 1000 \\
 && test \"$(id -u alo-agent)\" = 60989 \\
 && test \"$(getent group alo-agent | cut -d: -f3)\" = 60989 \\
 && id -nG alo | tr ' ' '\\n' | grep -qx alo-agent
";

    /// **A name and a number read back as the assertion they are**, for each of
    /// the three shapes the recipe uses.
    #[test]
    fn the_three_shapes_read_back_as_what_the_build_insists_on() {
        let asserted = Asserted::read(THE_FOOT);

        assert_eq!(asserted.login_called("alo"), Some(1000));
        assert_eq!(asserted.login_called("alo-agent"), Some(60989));
        assert_eq!(asserted.group_called("alo-agent"), Some(60989));
        assert!(asserted.puts("alo", "alo-agent"));
    }

    /// A name nothing asserts answers nothing, which is what the check above
    /// reads as *the build never insisted on that number*.
    #[test]
    fn a_name_the_build_never_asks_about_is_nothing() {
        let asserted = Asserted::read(THE_FOOT);

        assert_eq!(asserted.login_called("alo-model"), None);
        assert_eq!(asserted.group_called("alo"), None);
        assert!(!asserted.puts("alo-agent", "alo"));
    }

    /// **Prose about an assertion is not an assertion.** Every number in this
    /// recipe has a paragraph above it explaining why it is that number, and a
    /// reader that searched comments would find the explanation and report the
    /// build as checking something it does not run.
    #[test]
    fn a_comment_asserts_nothing() {
        let asserted = Asserted::read("# test \"$(id -u alo)\" = 1000, which is asserted below\n");

        assert_eq!(asserted.login_called("alo"), None);
    }

    /// **A line that names a login and compares against nothing is not an
    /// assertion.** `id -u alo` on its own is a build step that prints a number
    /// and carries on whatever it is.
    #[test]
    fn a_line_that_compares_against_nothing_asserts_nothing() {
        let asserted = Asserted::read("RUN echo \"$(id -u alo)\"\n");

        assert_eq!(asserted.login_called("alo"), None);
    }

    /// A number that is not one is not an assertion either — `= $SOMETHING` is
    /// a comparison against whatever a build argument happened to hold.
    #[test]
    fn a_comparison_against_something_that_is_not_a_number_asserts_nothing() {
        let asserted = Asserted::read("RUN test \"$(id -u alo)\" = \"${THE_PERSON}\"\n");

        assert_eq!(asserted.login_called("alo"), None);
    }
}
