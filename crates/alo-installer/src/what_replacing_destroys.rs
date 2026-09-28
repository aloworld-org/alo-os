//! How much is on the Windows volume, and when the newest of a person's own
//! files there was changed.
//!
//! Installer plan task 7: before anything is written, the replacing road says
//! **from what it read rather than from assumption** how much is on the Windows
//! volume and the date of the newest file there — because *20 GB of documents
//! last touched this morning* is what makes a person stop, and a sentence about
//! a disk in the abstract does not.
//!
//! # Why a person's own files, and not every file on the volume
//!
//! Windows writes to its own volume continuously: logs, search indexes, update
//! state. So the newest file on all of `%SystemDrive%` is a system file touched
//! seconds ago, no matter what the person's own files look like. That answer is
//! true and useless — it would tell somebody whose documents were last opened in
//! 2019 that their disk changed a moment ago, which is the opposite of the thing
//! task 7 wants said.
//!
//! The measurement a person can act on is the newest of **their** files, so the
//! walk is `%SystemDrive%\Users` and the sentence says whose files were counted
//! rather than implying it read the whole disk.
//!
//! # What is read, and what is not
//!
//! Nothing here opens a file or reads any content. The walk asks each entry for
//! its last-write time, and what crosses into this crate is a count, a size in
//! bytes, and a calendar day. No name of any file a person owns is carried,
//! printed or logged.
//!
//! # The size is the volume's, and it is not a sum of the walk
//!
//! How much is used comes from the volume itself — `Size` less `SizeRemaining` —
//! and not from adding up the files the walk saw. A walk that cannot read a
//! folder would otherwise under-report how much is about to be destroyed, and
//! under-reporting *that* is the one direction this sentence must never fail in.

use serde::Deserialize;

/// The day a file was last changed, as year-month-day.
///
/// Year-month-day rather than a written-out date: it is the one form that reads
/// the same in every language this installer speaks, and the alternative is a
/// date library deciding a person's calendar for them. Windows formats it, this
/// checks the shape, and the sentence carries it as it arrived.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Day(String);

impl Day {
    /// The day a machine printed, or [`None`] when it is not a day.
    ///
    /// Ten characters, `YYYY-MM-DD`, with a month of 1 to 12 and a day of 1 to
    /// 31. It does not check that the day exists in that month: this is guarding
    /// against a machine printing something that is not a date at all, and
    /// refusing the 31st of a 30-day month would only replace a readable date
    /// with no date.
    #[must_use]
    pub fn of(printed: &str) -> Option<Self> {
        let bytes = printed.as_bytes();
        if bytes.len() != 10 {
            return None;
        }
        let digits_at = [0, 1, 2, 3, 5, 6, 8, 9];
        let dashes_at = [4, 7];
        if !digits_at
            .iter()
            .all(|at| bytes.get(*at).is_some_and(u8::is_ascii_digit))
        {
            return None;
        }
        if !dashes_at.iter().all(|at| bytes.get(*at) == Some(&b'-')) {
            return None;
        }
        let month: u8 = printed.get(5..7)?.parse().ok()?;
        let day: u8 = printed.get(8..10)?.parse().ok()?;
        ((1..=12).contains(&month) && (1..=31).contains(&day)).then(|| Self(printed.to_owned()))
    }

    /// The day, as the sentence carries it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What replacing Windows on this computer would destroy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatReplacingDestroys {
    /// How much of the Windows volume is used, in bytes.
    used: u64,
    /// How many of the person's own files the walk saw.
    files: u64,
    /// The day the newest of them was changed, or [`None`] when the walk saw
    /// none.
    newest: Option<Day>,
}

/// What the script prints.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Printed {
    /// Bytes used on the Windows volume.
    used: u64,
    /// How many files the walk saw under `\Users`.
    files: u64,
    /// The newest last-write day, or empty when the walk saw no files.
    newest: String,
}

impl WhatReplacingDestroys {
    /// What the script printed, or [`None`] when it is not the script's answer.
    ///
    /// A count of nothing and a day are refused together: a machine that says it
    /// saw no files and also names the day one of them changed is not answering,
    /// and a sentence built from half of that answer would be a guess wearing the
    /// clothes of a measurement.
    #[must_use]
    pub fn read(printed: Option<&str>) -> Option<Self> {
        let printed = serde_json::from_str::<Printed>(printed?).ok()?;
        let newest = if printed.newest.is_empty() {
            None
        } else {
            Some(Day::of(&printed.newest)?)
        };
        let agrees = (printed.files == 0) == newest.is_none();
        agrees.then_some(Self {
            used: printed.used,
            files: printed.files,
            newest,
        })
    }

    /// How much of the Windows volume is used, in bytes.
    #[must_use]
    pub const fn used(&self) -> u64 {
        self.used
    }

    /// How many of the person's own files the walk saw.
    #[must_use]
    pub const fn files(&self) -> u64 {
        self.files
    }

    /// The day the newest of them changed, or [`None`] when the walk saw none.
    #[must_use]
    pub const fn newest(&self) -> Option<&Day> {
        self.newest.as_ref()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;

    /// What a machine with files on it prints.
    const A_MACHINE_IN_USE: &str = r#"{"Used":21474836480,"Files":41233,"Newest":"2026-09-28"}"#;

    /// **A real answer is read whole.**
    #[test]
    fn a_real_answer_is_read_whole() {
        let read = WhatReplacingDestroys::read(Some(A_MACHINE_IN_USE)).unwrap();
        assert_eq!(read.used(), 20 * 1024 * 1024 * 1024);
        assert_eq!(read.files(), 41233);
        assert_eq!(read.newest().unwrap().as_str(), "2026-09-28");
    }

    /// **A machine with none of the person's files says so, and says no day.**
    #[test]
    fn no_files_of_their_own_is_an_answer_with_no_day() {
        let read =
            WhatReplacingDestroys::read(Some(r#"{"Used":1024,"Files":0,"Newest":""}"#)).unwrap();
        assert_eq!(read.files(), 0);
        assert!(read.newest().is_none());
    }

    /// **A count and a day that contradict each other are not an answer.**
    ///
    /// Both directions: files with no day, and a day with no files.
    #[test]
    fn a_count_and_a_day_that_disagree_are_no_answer() {
        for printed in [
            r#"{"Used":1024,"Files":7,"Newest":""}"#,
            r#"{"Used":1024,"Files":0,"Newest":"2026-09-28"}"#,
        ] {
            assert_eq!(
                WhatReplacingDestroys::read(Some(printed)),
                None,
                "{printed}"
            );
        }
    }

    /// **Nothing that is not the script's answer is read as one.**
    #[test]
    fn nothing_else_is_an_answer() {
        for printed in [
            None,
            Some(""),
            Some("not json"),
            Some(r#"{"Used":1024}"#),
            Some(r#"{"Used":1024,"Files":1,"Newest":"28-09-2026"}"#),
            Some(r#"{"Used":1024,"Files":1,"Newest":"2026-13-01"}"#),
            Some(r#"{"Used":1024,"Files":1,"Newest":"2026-09-32"}"#),
            Some(r#"{"Used":1024,"Files":1,"Newest":"2026-09-2"}"#),
        ] {
            assert_eq!(WhatReplacingDestroys::read(printed), None, "{printed:?}");
        }
    }

    /// **A day is a day, and nothing else is.**
    #[test]
    fn a_day_is_a_day_and_nothing_else_is() {
        assert_eq!(Day::of("2026-09-28").unwrap().as_str(), "2026-09-28");
        assert_eq!(Day::of("1999-01-01").unwrap().as_str(), "1999-01-01");
        for not_a_day in [
            "",
            "2026-09-28T10:00:00Z",
            "2026-9-28",
            "2026/09/28",
            "202X-09-28",
            "2026-00-28",
            "2026-09-00",
            "2026-09-281",
        ] {
            assert_eq!(Day::of(not_a_day), None, "{not_a_day}");
        }
    }
}
