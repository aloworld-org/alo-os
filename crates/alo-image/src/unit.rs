//! A systemd unit file, as the sections and keys somebody wrote in it.
//!
//! This is the shape, and [`crate::Service`] is the meaning — two files for
//! `alo-strings`' reason since item 9: a translation as it arrives and a
//! translation that may be shown are two types, and the checking is the door
//! between them. What arrives here is text, and nothing in this file has an
//! opinion about what a key means.
//!
//! # It reads less than systemd does, and says so rather than guessing
//!
//! systemd's unit format has drop-ins, continued lines, specifiers like `%i`,
//! and templates. **None of that is read here**, and a unit using any of it is
//! [refused](crate::NotAUnit) rather than half-understood — because the whole
//! value of this crate is that a check which passes means something, and a
//! reader that silently mis-parsed a line would be a check that passes on a
//! machine it has never really read.
//!
//! What *is* read is the three things alo OS's own two units are made of: a
//! section header, a `key=value`, and a comment.
//!
//! # The two systemd rules that are honoured, because meaning depends on them
//!
//! - **The last assignment wins** for a setting that has one value, which is
//!   [`Unit::one`].
//! - **A list accumulates, and an empty assignment empties it**, which is
//!   [`Unit::listed`]. That second half is not a detail: `CapabilityBoundingSet=`
//!   with nothing after it is how a unit says *this service holds no capability
//!   at all*, and a reader that treated it as an absent key would read alo OS's
//!   strongest statement about `alo-agentd` as a line nobody wrote.

use std::collections::BTreeMap;

use crate::refusing::NotAUnit;

/// One systemd unit file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    /// Every section, and in each the keys in the order they were assigned.
    ///
    /// A `Vec` rather than a map inside, because a key may be assigned more
    /// than once and both the order and the repetition carry meaning.
    sections: BTreeMap<String, Vec<(String, String)>>,
}

impl Unit {
    /// The unit this text describes.
    ///
    /// # Errors
    ///
    /// [`NotAUnit::Continued`] for a line this reader will not join,
    /// [`NotAUnit::NoSection`] for an assignment before any section header, and
    /// [`NotAUnit::NotAnAssignment`] for a line that is neither. Every one names
    /// the line it is on, because that is what whoever is editing the unit
    /// needs.
    pub fn read(text: &str) -> Result<Self, NotAUnit> {
        let mut sections: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
        let mut section: Option<String> = None;

        for (index, raw) in text.lines().enumerate() {
            let line = raw.trim();
            let at = index + 1;

            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if line.ends_with('\\') {
                return Err(NotAUnit::Continued { at });
            }
            if let Some(name) = line.strip_prefix('[').and_then(|it| it.strip_suffix(']')) {
                section = Some(name.trim().to_owned());
                sections.entry(name.trim().to_owned()).or_default();
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(NotAUnit::NotAnAssignment {
                    at,
                    line: line.to_owned(),
                });
            };
            let Some(name) = section.as_ref() else {
                return Err(NotAUnit::NoSection {
                    at,
                    key: key.trim().to_owned(),
                });
            };
            sections
                .entry(name.clone())
                .or_default()
                .push((key.trim().to_owned(), value.trim().to_owned()));
        }
        Ok(Self { sections })
    }

    /// Whether this unit has a section by this name at all.
    #[must_use]
    pub fn has(&self, section: &str) -> bool {
        self.sections.contains_key(section)
    }

    /// Whether this key was assigned in this section, whatever it was assigned.
    ///
    /// Different from [`Unit::listed`] being empty: a unit that never mentions
    /// `AmbientCapabilities` and a unit that assigns it nothing both hold no
    /// ambient capability, and only the second one *says* so.
    #[must_use]
    pub fn says(&self, section: &str, key: &str) -> bool {
        !self.assignments(section, key).is_empty()
    }

    /// The value of a setting that has one, which is the last assignment.
    #[must_use]
    pub fn one(&self, section: &str, key: &str) -> Option<&str> {
        self.assignments(section, key).last().copied()
    }

    /// Everything a list setting names, in order.
    ///
    /// An empty assignment empties the list, which is systemd's rule and is how
    /// `CapabilityBoundingSet=` says *nothing at all*.
    #[must_use]
    pub fn listed(&self, section: &str, key: &str) -> Vec<&str> {
        let mut named: Vec<&str> = Vec::new();
        for value in self.assignments(section, key) {
            if value.is_empty() {
                named.clear();
                continue;
            }
            named.extend(value.split_whitespace());
        }
        named
    }

    /// Every value this key was assigned in this section, in order.
    ///
    /// A `Vec` rather than an iterator so that the values borrow from the unit
    /// and not from the two names they were looked up by — a caller asking about
    /// a key it built on the spot is the ordinary case.
    fn assignments<'a>(&'a self, section: &str, key: &str) -> Vec<&'a str> {
        self.sections
            .get(section)
            .into_iter()
            .flatten()
            .filter(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
            .collect()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A unit with the three shapes this reader knows in it.
    const ORDINARY: &str = "\
# a comment, and a blank line under it

[Unit]
Description=something
After=one.service
After=another.service

[Service]
Type=oneshot
ExecStart=/usr/libexec/thing
";

    /// The ordinary case: sections, keys and values come back as they were
    /// written, with the comment and the blank line gone.
    #[test]
    fn a_unit_reads_back_what_was_written_in_it() {
        let unit = Unit::read(ORDINARY).unwrap();

        assert!(unit.has("Unit"));
        assert!(unit.has("Service"));
        assert!(!unit.has("Install"));
        assert_eq!(unit.one("Unit", "Description"), Some("something"));
        assert_eq!(unit.one("Service", "Type"), Some("oneshot"));
    }

    /// **A key assigned twice accumulates**, which is what `After=` really means
    /// and what a reader keeping only the last value would get wrong.
    #[test]
    fn a_list_assigned_twice_holds_both() {
        let unit = Unit::read(ORDINARY).unwrap();
        assert_eq!(
            unit.listed("Unit", "After"),
            vec!["one.service", "another.service"]
        );
    }

    /// And one assignment naming several things is the same list.
    #[test]
    fn a_list_named_on_one_line_is_the_same_list() {
        let unit = Unit::read("[Service]\nCapabilityBoundingSet=CAP_BPF CAP_SYS_ADMIN\n").unwrap();
        assert_eq!(
            unit.listed("Service", "CapabilityBoundingSet"),
            vec!["CAP_BPF", "CAP_SYS_ADMIN"]
        );
    }

    /// **An empty assignment empties the list**, which is how a unit says that a
    /// service holds no capability at all. A reader that skipped it would read
    /// alo OS's strongest statement about `alo-agentd` as a line nobody wrote.
    #[test]
    fn an_empty_assignment_empties_the_list() {
        let unit = Unit::read(
            "[Service]\nCapabilityBoundingSet=CAP_BPF CAP_SYS_ADMIN\nCapabilityBoundingSet=\n",
        )
        .unwrap();

        assert!(unit.listed("Service", "CapabilityBoundingSet").is_empty());
        assert!(
            unit.says("Service", "CapabilityBoundingSet"),
            "and the unit still said it, which is not the same as never mentioning it"
        );
    }

    /// A setting with one value takes the last assignment, which is systemd's
    /// rule and the opposite of the one above.
    #[test]
    fn the_last_assignment_of_a_single_value_wins() {
        let unit = Unit::read("[Service]\nUser=root\nUser=alo\n").unwrap();
        assert_eq!(unit.one("Service", "User"), Some("alo"));
    }

    /// A key nobody assigned is nothing, and is not an empty string.
    #[test]
    fn a_key_nobody_assigned_is_nothing() {
        let unit = Unit::read(ORDINARY).unwrap();
        assert_eq!(unit.one("Service", "User"), None);
        assert!(!unit.says("Service", "User"));
    }

    /// **A continued line is refused rather than half-read.** This reader does
    /// not join them, and a unit that used one would otherwise be checked
    /// against a value with half of it missing.
    #[test]
    fn a_continued_line_is_refused_and_names_where_it_is() {
        let refused =
            Unit::read("[Service]\nExecStart=/usr/bin/thing \\\n  --and-more\n").unwrap_err();
        assert!(
            matches!(refused, NotAUnit::Continued { at: 2 }),
            "{refused}"
        );
    }

    /// **An assignment before any section is refused**, because there is no
    /// section for it to mean anything in.
    #[test]
    fn an_assignment_before_any_section_is_refused() {
        let refused = Unit::read("User=alo\n[Service]\n").unwrap_err();
        assert!(
            matches!(&refused, NotAUnit::NoSection { at: 1, key } if key == "User"),
            "{refused}"
        );
    }

    /// **A line that is neither a header nor an assignment is refused**, rather
    /// than skipped as though it were a comment somebody forgot to mark.
    #[test]
    fn a_line_that_is_neither_is_refused() {
        let refused = Unit::read("[Service]\nExecStart /usr/bin/thing\n").unwrap_err();
        assert!(
            matches!(&refused, NotAUnit::NotAnAssignment { at: 2, line } if line.contains("ExecStart")),
            "{refused}"
        );
    }

    /// A value with an `=` in it keeps all of it: only the first `=` divides.
    #[test]
    fn a_value_with_an_equals_in_it_keeps_it() {
        let unit = Unit::read("[Service]\nEnvironment=NAME=value\n").unwrap();
        assert_eq!(unit.one("Service", "Environment"), Some("NAME=value"));
    }

    /// An empty unit is a unit: nothing in it is a fact about it rather than a
    /// reason to refuse the file.
    #[test]
    fn an_empty_unit_reads() {
        let unit = Unit::read("").unwrap();
        assert!(!unit.has("Service"));
    }
}
