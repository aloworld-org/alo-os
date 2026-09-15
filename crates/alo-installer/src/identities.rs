//! The four things a program is ever pointed at, each only ever made valid.
//!
//! Every change this installer makes to a machine names a disk by Windows'
//! number for it, a partition by its number on that disk, a drive by its
//! letter, or a start-up entry by the identifier Windows gave it. Those are the
//! only values of the machine's that ever reach a program's arguments, so each
//! is a type that cannot hold anything else: a number is a number, a letter is
//! one letter from A to Z, and an entry is exactly a braced GUID. There is no
//! way to build a script or an argument out of text Windows printed without
//! passing through one of these.

use std::fmt;

/// A disk, as Windows numbers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiskNumber(pub u32);

/// A partition, as Windows numbers it on its disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartitionNumber(pub u32);

/// A drive letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Letter(char);

/// A start-up entry, as Windows identifies it: `{xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry(String);

impl Letter {
    /// The letter, if it is one drive letter.
    #[must_use]
    pub fn of(written: &str) -> Option<Self> {
        let mut chars = written.trim().chars();
        let first = chars.next()?.to_ascii_uppercase();
        let rest = chars.as_str();
        (first.is_ascii_uppercase() && (rest.is_empty() || rest == ":")).then_some(Self(first))
    }

    /// The letter alone.
    #[must_use]
    pub fn letter(self) -> char {
        self.0
    }

    /// The drive, as a person is shown it: `C:`.
    #[must_use]
    pub fn drive(self) -> String {
        format!("{}:", self.0)
    }

    /// The root of the drive, as a path: `C:\`.
    #[must_use]
    pub fn root(self) -> String {
        format!("{}:\\", self.0)
    }
}

impl Entry {
    /// The first braced GUID in what a program printed, if there is one.
    ///
    /// The tool that makes an entry says so in the language Windows is set to,
    /// so the sentence around the identifier is never read — only the one
    /// shape an identifier has.
    #[must_use]
    pub fn first_in(printed: &str) -> Option<Self> {
        printed
            .match_indices('{')
            .filter_map(|(at, _)| printed.get(at..at + 38))
            .find_map(Self::of)
    }

    /// Exactly one braced GUID.
    #[must_use]
    pub fn of(written: &str) -> Option<Self> {
        let inner = written.strip_prefix('{')?.strip_suffix('}')?;
        let groups: Vec<&str> = inner.split('-').collect();
        let shaped = groups.len() == 5
            && groups.iter().zip([8, 4, 4, 4, 12]).all(|(group, length)| {
                group.len() == length && group.bytes().all(|b| b.is_ascii_hexdigit())
            });
        shaped.then(|| Self(written.to_ascii_lowercase()))
    }

    /// The identifier, braces and all.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DiskNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for PartitionNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A drive letter is one letter, and nothing that merely starts with one.
    #[test]
    fn a_letter_is_one_letter() {
        assert_eq!(Letter::of("C").unwrap().drive(), "C:");
        assert_eq!(Letter::of("d:").unwrap().root(), "D:\\");
        for not in ["", "CC", "1", "C:\\", "C;", "C :", "é", "C'; Remove-Item"] {
            assert_eq!(Letter::of(not), None, "{not:?}");
        }
    }

    /// **An entry is exactly a braced GUID**, found in a sentence in any
    /// language and never anything around it.
    #[test]
    fn an_entry_is_exactly_a_braced_guid() {
        let printed =
            "La entrada se copió correctamente en {6B1D5C2A-0F3E-11EF-9A1B-00155D012345}.";
        assert_eq!(
            Entry::first_in(printed).unwrap().as_str(),
            "{6b1d5c2a-0f3e-11ef-9a1b-00155d012345}"
        );
        assert_eq!(Entry::first_in("{bootmgr} {not-a-guid}"), None);
        for not in [
            "{6b1d5c2a-0f3e-11ef-9a1b-00155d01234}",
            "6b1d5c2a-0f3e-11ef-9a1b-00155d012345",
            "{6b1d5c2a-0f3e-11ef-9a1b-00155d01234g}",
            "{6b1d5c2a0f3e-11ef-9a1b-00155d0123455}",
            "{bootmgr}",
        ] {
            assert_eq!(Entry::of(not), None, "{not}");
        }
    }
}
