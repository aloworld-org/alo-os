//! **What an installed application's package lets it reach without asking**,
//! read from the package itself.
//!
//! ADR 0143, fact 4: an application installed sandboxed is not necessarily an
//! application that asks. VSCodium's package asks Flathub for `host` — every
//! file the person can reach — and to talk to `org.freedesktop.Flatpak`, which
//! lets it start any program outside its sandbox. *Installed sandboxed* is true
//! of the mechanism and not of what that application can do, so where a person
//! looks at what is installed the machine says so (`docs/features.md`, *alo OS
//! ships VSCodium*).
//!
//! # Read off the machine, never declared beside it
//!
//! The answer comes from the package's own `metadata` file and the overrides a
//! person or the machine applied, **not from a list in this repository**. A
//! declared list could only say what an application asked for on the day
//! somebody wrote it down; an update that narrows a package, or a person who
//! narrowed it with an override, would leave the machine saying a sealed
//! package is open. And an application nobody here has heard of — VS Code
//! installed by hand — is said exactly as the shipped one is.
//!
//! # What is read, in what order
//!
//! Flatpak's own order, as its manual pages describe it (read, not measured on
//! a machine): the package's `metadata`, then the machine's overrides (`global`,
//! then the application's own), then the person's. A later file's value for a
//! name replaces an earlier one's, and a `!` before a name takes it away. The
//! person's installation is read before the machine's for the package itself,
//! because that is the one `flatpak run` finds first.
//!
//! Two things are looked for and nothing else:
//!
//! - **every file in the person's home**, which `host`, `home` and `~` give,
//!   read-only with `:ro` and read-and-change otherwise. `/home` and
//!   `/var/home` are the same directory on this image, so they count too;
//! - **starting programs outside the sandbox**, which talking or owning
//!   `org.freedesktop.Flatpak` gives (named, or under a `.*` wildcard), and so
//!   does the whole session bus (`sockets=session-bus`).
//!
//! # Where nothing is read, nothing is said
//!
//! An application with no package here — LibreOffice, which the image carries
//! in `/usr/share` — has no entry in the answer. **That is not a statement that
//! it is sealed**: it is not in a sandbox at all, and this reader has nothing to
//! say about it. A caller says nothing either way, which is the rule a sealed
//! package needs: *never said to be open*.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// How much of a person's home a package may reach without asking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TheirHome {
    /// None of it as a whole — perhaps a folder in it, which is not this.
    NotAsAWhole,
    /// Every file in it, to read.
    Read,
    /// Every file in it, to read and to change.
    ReadAndChanged,
}

/// What one package lets its application reach without asking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhatItMayReach {
    /// How much of the person's home.
    home: TheirHome,
    /// Whether it may start programs outside its sandbox.
    programs_outside: bool,
}

impl WhatItMayReach {
    /// How much of the person's home it may reach.
    #[must_use]
    pub const fn home(self) -> TheirHome {
        self.home
    }

    /// Whether it may start programs outside its sandbox — with everything
    /// the person's own account can reach.
    #[must_use]
    pub const fn starts_programs_outside(self) -> bool {
        self.programs_outside
    }

    /// Whether there is anything here a person should be told.
    #[must_use]
    pub const fn is_worth_saying(self) -> bool {
        self.programs_outside || !matches!(self.home, TheirHome::NotAsAWhole)
    }
}

/// What each installed package lets its application reach, by identifier.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WhatTheyMayReach {
    /// Only the packages that were read.
    read: BTreeMap<String, WhatItMayReach>,
}

impl WhatTheyMayReach {
    /// Nothing read: what a caller holds where it never looked.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// What this application's package lets it reach, where its package was
    /// read. [`None`] is *nothing read*, never *sealed*.
    #[must_use]
    pub fn of(&self, identifier: &str) -> Option<WhatItMayReach> {
        self.read.get(identifier).copied()
    }
}

/// Read what each of these applications' packages lets it reach.
///
/// `machines` is the machine's installation (`/var/lib/flatpak`) and
/// `persons` the person's own (`~/.local/share/flatpak`), where there is one.
/// An identifier whose package is in neither is left out of the answer.
#[must_use]
pub fn what_they_may_reach<'a>(
    machines: &Path,
    persons: Option<&Path>,
    identifiers: impl IntoIterator<Item = &'a str>,
) -> WhatTheyMayReach {
    let mut read = BTreeMap::new();
    for identifier in identifiers {
        if let Some(it) = what_it_may_reach(machines, persons, identifier) {
            read.insert(identifier.to_owned(), it);
        }
    }
    WhatTheyMayReach { read }
}

/// One application's answer, or [`None`] where its package is not here.
fn what_it_may_reach(
    machines: &Path,
    persons: Option<&Path>,
    identifier: &str,
) -> Option<WhatItMayReach> {
    // An identifier names a directory below, so one that could name another
    // place is not looked for at all.
    if identifier.is_empty()
        || identifier.starts_with('.')
        || !identifier
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return None;
    }
    let package = persons
        .into_iter()
        .chain(std::iter::once(machines))
        .find_map(|installation| {
            std::fs::read_to_string(
                installation
                    .join("app")
                    .join(identifier)
                    .join("current")
                    .join("active")
                    .join("metadata"),
            )
            .ok()
        })?;
    let mut asked = Asked::default();
    asked.take(&package);
    for installation in std::iter::once(machines).chain(persons) {
        for named in ["global", identifier] {
            if let Ok(text) = std::fs::read_to_string(installation.join("overrides").join(named)) {
                asked.take(&text);
            }
        }
    }
    Some(asked.reaches())
}

/// The names one package and its overrides asked for, as they stand.
#[derive(Debug, Default)]
struct Asked {
    /// Each filesystem name, and whether it was given read-only.
    filesystems: BTreeMap<String, bool>,
    /// Each socket name given.
    sockets: BTreeSet<String>,
    /// Each session bus name, and its policy.
    session_bus: BTreeMap<String, String>,
}

impl Asked {
    /// Take one keyfile's values over the ones already held.
    fn take(&mut self, text: &str) {
        let mut section = "";
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|it| it.strip_suffix(']')) {
                section = name;
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            match (section, key) {
                ("Context", "filesystems") => self.filesystems_given(value),
                ("Context", "sockets") => {
                    for (taken_away, name) in items(value) {
                        if taken_away {
                            self.sockets.remove(name);
                        } else {
                            self.sockets.insert(name.to_owned());
                        }
                    }
                }
                ("Session Bus Policy", name) => {
                    self.session_bus.insert(name.to_owned(), value.to_owned());
                }
                _ => {}
            }
        }
    }

    /// One `filesystems=` value, over what is held.
    fn filesystems_given(&mut self, value: &str) {
        for (taken_away, item) in items(value) {
            let (name, read_only) = match item.rsplit_once(':') {
                Some((name, "ro")) => (name, true),
                Some((name, "rw" | "create")) => (name, false),
                _ => (item, false),
            };
            let name = name.trim_end_matches('/');
            if taken_away && name == "host-reset" {
                self.filesystems.clear();
            } else if taken_away {
                self.filesystems.remove(name);
            } else {
                self.filesystems.insert(name.to_owned(), read_only);
            }
        }
    }

    /// What the names held come to.
    fn reaches(&self) -> WhatItMayReach {
        let home = ["host", "home", "~", "/home", "/var/home"]
            .into_iter()
            .filter_map(|name| self.filesystems.get(name))
            .map(|read_only| {
                if *read_only {
                    TheirHome::Read
                } else {
                    TheirHome::ReadAndChanged
                }
            })
            .max()
            .unwrap_or(TheirHome::NotAsAWhole);
        let programs_outside =
            self.sockets.contains("session-bus") || self.talks_to_flatpak_itself();
        WhatItMayReach {
            home,
            programs_outside,
        }
    }

    /// Whether it may talk to `org.freedesktop.Flatpak`: by name where it is
    /// named, else under the widest wildcard that covers it.
    fn talks_to_flatpak_itself(&self) -> bool {
        const FLATPAK: &str = "org.freedesktop.Flatpak";
        let talks = |policy: &str| matches!(policy, "talk" | "own");
        if let Some(policy) = self.session_bus.get(FLATPAK) {
            return talks(policy);
        }
        self.session_bus.iter().any(|(name, policy)| {
            name.strip_suffix(".*")
                .is_some_and(|prefix| FLATPAK.starts_with(&format!("{prefix}.")))
                && talks(policy)
        })
    }
}

/// The items of a keyfile list, each with whether a `!` took it away.
fn items(value: &str) -> impl Iterator<Item = (bool, &str)> {
    value
        .split(';')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| match item.strip_prefix('!') {
            Some(name) => (true, name),
            None => (false, item),
        })
}

#[cfg(test)]
#[path = "what_they_may_reach_tests.rs"]
mod tests;
