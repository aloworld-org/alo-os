//! The logins the image creates, and the group they meet in.
//!
//! `systemd-sysusers` reads these lines. They are a declaration rather than a
//! `useradd` in the Containerfile because the three numbers in them are a
//! contract: `/etc/alo/agentd.toml` names all three, and
//! [ADR 0001](../../../docs/decisions/0001-the-capability-model.md) §5 makes the
//! agent a login of its own — so a machine whose accounts and whose description
//! disagree is a machine where the kernel's answer about who is on the socket
//! means something other than what the daemon thinks it means.
//!
//! # Three kinds of line, and the third is not decoration
//!
//! `g` makes a group, `u` makes a login, and `m` puts a login into a group. The
//! last one is what lets `alo-agentd` hand the socket to the agent at all:
//! changing a file's group is only allowed to a member of it, so a person who is
//! not in the agent's group is a person whose service binds a socket it cannot
//! give away.

use crate::refusing::NotDeclared;

/// One line of a `sysusers.d` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declared {
    /// A group, and its number.
    Group {
        /// What it is called.
        name: String,
        /// Its number.
        id: u32,
    },
    /// A login, its number, the group it is in if the line names one, and
    /// where it says that login lives.
    Login {
        /// What it is called.
        name: String,
        /// Its number.
        id: u32,
        /// The group it is in, when the line says `uid:gid` rather than `uid`.
        ///
        /// `None` means systemd makes a group of the same name and number,
        /// which is the ordinary shape for a person's own login.
        group: Option<u32>,
        /// The home this line declares, when it declares one.
        ///
        /// **Read since 2026-10-10, and the note above it was wrong until
        /// then.** `systemd-sysusers` writes this path into `/etc/passwd` and
        /// **does not create the directory**. Nothing in this image did either,
        /// so a person signed in to a home that was not there - and
        /// `crate::checking` now holds the image to making what this field
        /// declares.
        home: Option<String>,
    },
    /// A login put into a group it is not the owner of.
    Member {
        /// The login.
        login: String,
        /// The group it is put into.
        group: String,
    },
}

impl Declared {
    /// Where this line says its login lives, if it is a login that says.
    #[must_use]
    pub fn home(&self) -> Option<&str> {
        match self {
            Self::Login { home, .. } => home.as_deref(),
            Self::Group { .. } | Self::Member { .. } => None,
        }
    }

    /// What this line calls its login, if it is a login.
    #[must_use]
    pub fn login_name(&self) -> Option<&str> {
        match self {
            Self::Login { name, .. } => Some(name),
            Self::Group { .. } | Self::Member { .. } => None,
        }
    }

    /// The number this line gives a group by this name, if it is that line.
    #[must_use]
    pub fn group_called(&self, wanted: &str) -> Option<u32> {
        match self {
            Self::Group { name, id } if name == wanted => Some(*id),
            _ => None,
        }
    }

    /// The number this line gives a login by this name, if it is that line.
    #[must_use]
    pub fn login_called(&self, wanted: &str) -> Option<u32> {
        match self {
            Self::Login { name, id, .. } if name == wanted => Some(*id),
            _ => None,
        }
    }

    /// Whether this line puts that login into that group.
    #[must_use]
    pub fn puts(&self, wanted: &str, into: &str) -> bool {
        matches!(self, Self::Member { login, group } if login == wanted && group == into)
    }

    /// Whether this line makes a login with this number, whatever it is called.
    ///
    /// The agent's login is the one thing the machine description names that no
    /// unit file does, so it is found by its number: ADR 0001 §5 makes it a
    /// login of its own, and what has to be true is that it exists rather than
    /// what somebody called it.
    #[must_use]
    pub fn numbers_a_login(&self, wanted: u32) -> bool {
        matches!(self, Self::Login { id, .. } if *id == wanted)
    }
}

/// Every login and group a `sysusers.d` file declares.
///
/// # Errors
///
/// [`NotDeclared::TooFewFields`] for a line with no name or no number on it,
/// [`NotDeclared::NotANumber`] for an identifier that is not one, and
/// [`NotDeclared::UnknownKind`] for a type letter this reader does not know —
/// which is refused rather than skipped, because a line quietly passed over is a
/// login the image makes and this crate has never seen.
pub fn every_login(text: &str) -> Result<Vec<Declared>, NotDeclared> {
    let mut declared = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = index + 1;
        let mut fields = line.split_whitespace();
        let (Some(kind), Some(name), Some(third)) = (fields.next(), fields.next(), fields.next())
        else {
            return Err(NotDeclared::TooFewFields {
                at,
                line: line.to_owned(),
            });
        };
        // What follows the third field, with the fields themselves cut off:
        // a quoted description, then a home, then a shell.
        let rest = line
            .split_once(third)
            .map_or("", |(_, after)| after)
            .trim_start();
        declared.push(one(at, kind, name, third, rest)?);
    }
    Ok(declared)
}

/// The home a login line declares, if it declares one.
///
/// **The description is quoted and holds spaces**, so the fields cannot simply
/// be counted: `u alo 1000 "alo OS" /home/alo /bin/bash` splits into seven
/// words and the home is the sixth of them only by accident of how long the
/// description is. This takes what follows the closing quote instead, and
/// falls back to the first word when a line quotes nothing.
///
/// A `-` is systemd's own spelling for *no home declared*, and answers
/// [`None`] rather than a directory called `-`.
fn a_home_in(rest: &str) -> Option<String> {
    // **The description is one field either way**, and the fallback read it as
    // the home until a test said so: an unquoted description is a single word -
    // very often `-` - so skipping the quotes is not the same as skipping the
    // field. Both roads end past the description and before the shell.
    let after = match rest.strip_prefix('"') {
        Some(quoted) => quoted.split_once('"').map(|(_, after)| after)?,
        None => rest
            .split_once(char::is_whitespace)
            .map(|(_, after)| after)?,
    };
    let home = after.split_whitespace().next()?;
    // systemd's own spelling for *no home declared*, which is not a directory
    // called `-`.
    (home != "-").then(|| home.to_owned())
}

/// One line, once its first three fields are in hand.
///
/// **A home is read and the rest is not.** This said *everything after the
/// third field is a description, a home and a shell, and nothing alo OS
/// promises depends on any of them*, and a boot on 2026-10-10 found that the
/// home is exactly what a promise depends on. The description and the shell
/// still are not: nothing is decided by either.
fn one(
    at: usize,
    kind: &str,
    name: &str,
    third: &str,
    rest: &str,
) -> Result<Declared, NotDeclared> {
    match kind {
        "g" => Ok(Declared::Group {
            name: name.to_owned(),
            id: number(at, third)?,
        }),
        "u" => {
            let (id, group) = match third.split_once(':') {
                Some((id, group)) => (number(at, id)?, Some(number(at, group)?)),
                None => (number(at, third)?, None),
            };
            Ok(Declared::Login {
                name: name.to_owned(),
                id,
                group,
                home: a_home_in(rest),
            })
        }
        "m" => Ok(Declared::Member {
            login: name.to_owned(),
            group: third.to_owned(),
        }),
        _ => Err(NotDeclared::UnknownKind {
            at,
            kind: kind.to_owned(),
        }),
    }
}

/// An identifier, which is written as a plain number and nothing else.
fn number(at: usize, written: &str) -> Result<u32, NotDeclared> {
    written.parse().map_err(|_ignored| NotDeclared::NotANumber {
        at,
        written: written.to_owned(),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The four lines alo OS's own image ships.
    const ALL_FOUR: &str = "\
# the group the two meet in
g alo-agent 60989
u alo-agent 60989:60989 \"alo OS agent\" /var/lib/alo-agent /usr/sbin/nologin
u alo 1000 \"alo OS\" /home/alo /bin/bash
m alo alo-agent
";

    /// The ordinary case: a group, two logins and a membership, with the
    /// description, the home and the shell read past.
    #[test]
    fn the_four_lines_read_back_as_what_they_declare() {
        let declared = every_login(ALL_FOUR).unwrap();

        assert_eq!(declared.len(), 4);
        assert_eq!(
            declared.iter().find_map(|it| it.group_called("alo-agent")),
            Some(60989)
        );
        assert_eq!(
            declared.iter().find_map(|it| it.login_called("alo-agent")),
            Some(60989)
        );
        assert_eq!(
            declared.iter().find_map(|it| it.login_called("alo")),
            Some(1000)
        );
        assert!(declared.iter().any(|it| it.puts("alo", "alo-agent")));
    }

    /// **A login written `uid:gid` names its group and one written `uid` does
    /// not**, which is the difference between the agent — which is put in a
    /// group somebody else is also in — and the person, whose group is their own.
    #[test]
    fn a_login_names_its_group_only_when_the_line_does() {
        let declared = every_login(ALL_FOUR).unwrap();

        assert!(declared.contains(&Declared::Login {
            name: "alo-agent".to_owned(),
            id: 60989,
            group: Some(60989),
            home: Some("/var/lib/alo-agent".to_owned()),
        }));
        assert!(declared.contains(&Declared::Login {
            name: "alo".to_owned(),
            id: 1000,
            group: None,
            home: Some("/home/alo".to_owned()),
        }));
    }

    /// **A home is read from past the quoted description.**
    ///
    /// The description holds a space, so the fields cannot be counted: `u alo
    /// 1000 "alo OS" /home/alo /bin/bash` splits into seven words and the home
    /// is the sixth only by accident of how long the description happens to be.
    /// A reader that counted would give a person a home called `OS"`.
    #[test]
    fn a_home_is_read_from_past_the_description_however_long_it_is() {
        let declared = every_login(
            "u one 1000 \"a\" /home/one /bin/bash
             u two 1001 \"a description with several words in it\" /home/two /bin/bash
             u three 1002 - /home/three /bin/bash
             u four 1003 \"nobody lives here\" - /usr/sbin/nologin
",
        )
        .unwrap();

        let homes: Vec<Option<&str>> = declared.iter().map(Declared::home).collect();
        assert_eq!(
            homes,
            vec![
                Some("/home/one"),
                Some("/home/two"),
                Some("/home/three"),
                // systemd's own spelling for *no home*, which is not a
                // directory called `-`.
                None,
            ]
        );
    }

    /// **A type letter this reader does not know is refused**, not skipped. A
    /// line quietly passed over is a login the image makes and nothing here has
    /// ever seen.
    #[test]
    fn a_kind_this_reader_does_not_know_is_refused() {
        let refused = every_login("r - 100-200\n").unwrap_err();
        assert!(
            matches!(&refused, NotDeclared::UnknownKind { at: 1, kind } if kind == "r"),
            "{refused}"
        );
    }

    /// **An identifier that is not a number is refused.** `sysusers` will take a
    /// `-` and pick a number itself, and a login whose number was picked is a
    /// login the machine description cannot name.
    #[test]
    fn an_identifier_the_machine_picks_is_refused() {
        let refused = every_login("u alo - \"alo OS\"\n").unwrap_err();
        assert!(
            matches!(&refused, NotDeclared::NotANumber { at: 1, written } if written == "-"),
            "{refused}"
        );
    }

    /// **A line with nothing but a kind and a name is refused**, which is the
    /// mistake that really happens: a number deleted rather than changed.
    #[test]
    fn a_line_with_no_number_on_it_is_refused() {
        let refused = every_login("g alo-agent\n").unwrap_err();
        assert!(
            matches!(refused, NotDeclared::TooFewFields { at: 1, .. }),
            "{refused}"
        );
    }

    /// A name that is asked about and is not there answers nothing, which is
    /// what a check above reads as *the image never made it*.
    #[test]
    fn a_login_nobody_declared_is_nothing() {
        let declared = every_login(ALL_FOUR).unwrap();
        assert_eq!(declared.iter().find_map(|it| it.login_called("root")), None);
    }
}
