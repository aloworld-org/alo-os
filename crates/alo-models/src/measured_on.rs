//! Which machine a grade was earned on, and when.
//!
//! [`crate::Driving`] is a measurement, and ADR 0007 says it is ours rather than
//! a publisher's. What it did not say is **where** it was made, and for five
//! grades and then seven that lived in a comment above the entries — *Ollama
//! 0.33.3, four CPU cores* — which a reader of one entry had no reason to find
//! and a later edit had no reason to keep. A grade with no machine beside it is a
//! claim, and the catalogue carried seven of them.
//!
//! So an entry that states a grade states this block, and the loader refuses
//! either half alone: a grade with no machine, and a machine beside a grade
//! nobody ran.
//!
//! # A grade travels; the machine is still written down
//!
//! Whether these weights emit a valid verb call nine times in ten is a property
//! of the weights and holds wherever they run — that is why a grade made on a
//! Mac may stand in a catalogue shipped for a certified workstation. The machine
//! is written down anyway, for the reason a laboratory writes down its
//! instrument: a reader who disagrees with a grade needs to know what to run it
//! on to find out, and a runtime that later turns out to have mangled a prompt
//! makes every grade it served suspect at once. Neither question can be asked of
//! a grade that does not say.
//!
//! # Why each field is refused rather than encouraged
//!
//! - **The machine**, because *it ran* is not a machine. It must name the memory
//!   it had, in gigabytes, because memory is what decided whether the run could
//!   happen at all: every 7B entry stayed unmeasured for a fortnight on a box of
//!   5.8 GiB, and a grade that does not say how much room it was earned in cannot
//!   be compared with the one that could not be.
//! - **The date**, as a calendar date, because a model's upload can be re-pointed
//!   and a runtime upgraded between two measurements that would otherwise read as
//!   one.
//! - **The runtime**, because the prompt reaches the weights through it: a chat
//!   template is the runtime's and not the model's, and two versions of one
//!   runtime can put the same question to the same file differently.

use serde::{Deserialize, Serialize};

/// **Where and when a grade was earned.**
///
/// Present on exactly the entries whose `drives_verbs` is a grade, and
/// [`crate::Catalogue::parse`] refuses it anywhere else. Every field is
/// required, for [`crate::Requantised`]'s reason: the case where a field is
/// missing is exactly the case a reader needed it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasuredOn {
    /// The machine, as a person would recognise it: the processor where it is
    /// known, and **the memory it had**, in `GB` or `GiB`.
    pub machine: String,

    /// The day the run finished, as `YYYY-MM-DD`.
    pub date: String,

    /// The runtime that served the weights, with its version — `Ollama 0.34.0`.
    pub runtime: String,

    /// **How many attempts drove the verbs**, beside [`of`](Self::of) — so a
    /// grade can be re-derived, and a grade from one round landing near a line
    /// can be told apart from one out of two. The catalogue requires both; a
    /// person's settings file may carry neither, because that contract grew
    /// them after it was written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drove: Option<u32>,

    /// **How many attempts were made** — ten a round.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub of: Option<u32>,
}

impl MeasuredOn {
    /// What is wrong with this statement, in words a curator can act on, or
    /// [`None`] when it holds together.
    pub(crate) fn what_is_wrong_with_it(&self) -> Option<&'static str> {
        if self.machine.trim().is_empty() {
            return Some(
                "a grade measured on no machine in particular: name the machine the run was made \
                 on, or the grade is a claim",
            );
        }
        if !names_its_memory(&self.machine) {
            return Some(
                "a machine that does not say how much memory it had: state it in GB or GiB, \
                 because memory is what decided whether the run could be made at all",
            );
        }
        if !is_a_calendar_date(&self.date) {
            return Some(
                "a measurement with no day a reader can check: write the date the run finished as \
                 YYYY-MM-DD",
            );
        }
        if !names_a_version(&self.runtime) {
            return Some(
                "a runtime with no version: the prompt reaches the weights through it, so name \
                 the runtime and the release that served the run",
            );
        }
        match (self.drove, self.of) {
            (None, None) => None,
            (Some(drove), Some(of)) if of > 0 && drove <= of => None,
            (Some(_), Some(_)) => Some(
                "counts that cannot be a run: more attempts drove than were made, or none were \
                 made at all",
            ),
            (Some(_), None) | (None, Some(_)) => Some(
                "half a count: say how many attempts drove the verbs and how many were made, or \
                 neither",
            ),
        }
    }
}

/// Whether a description carries a memory figure — a number followed by `GB`
/// or `GiB`.
fn names_its_memory(machine: &str) -> bool {
    let words: Vec<&str> = machine
        .split(|letter: char| letter.is_whitespace() || letter == ',' || letter == ';')
        .filter(|word| !word.is_empty())
        .collect();
    let is_a_figure = |word: &str| {
        word.chars().any(|letter| letter.is_ascii_digit())
            && word
                .chars()
                .all(|letter| letter.is_ascii_digit() || letter == '.')
    };
    words.windows(2).any(|pair| match pair {
        [figure, unit] => {
            is_a_figure(figure) && matches!(unit.trim_end_matches(['.', ')']), "GB" | "GiB")
        }
        _ => false,
    })
}

/// Whether this is a real day written as `YYYY-MM-DD`.
fn is_a_calendar_date(written: &str) -> bool {
    let parts: Vec<&str> = written.split('-').collect();
    let [year, month, day] = parts.as_slice() else {
        return false;
    };
    if year.len() != 4 || month.len() != 2 || day.len() != 2 {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        year.parse::<u32>(),
        month.parse::<u32>(),
        day.parse::<u32>(),
    ) else {
        return false;
    };
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days_in_month).contains(&day)
}

/// Whether a runtime is named with a version: a name, then something with a
/// digit and a dot in it.
fn names_a_version(runtime: &str) -> bool {
    let mut words = runtime.split_whitespace();
    let named = words.next().is_some_and(|name| {
        name.chars()
            .next()
            .is_some_and(|letter| letter.is_alphabetic())
    });
    named
        && words.any(|word| {
            word.contains('.')
                && word
                    .trim_start_matches('v')
                    .chars()
                    .all(|letter| letter.is_ascii_digit() || letter == '.')
                && word.chars().any(|letter| letter.is_ascii_digit())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A statement that holds together, so the refusals below are about the
    /// fault each names rather than about a fixture that was never sound.
    fn sound() -> MeasuredOn {
        MeasuredOn {
            machine: "Apple M3, 8 GB unified memory".to_owned(),
            date: "2026-09-13".to_owned(),
            runtime: "Ollama 0.34.0".to_owned(),
            drove: Some(8),
            of: Some(20),
        }
    }

    /// **Counts are both or neither, and could have been a run.**
    #[test]
    fn counts_that_cannot_be_a_run_are_refused() {
        let neither = MeasuredOn {
            drove: None,
            of: None,
            ..sound()
        };
        assert_eq!(neither.what_is_wrong_with_it(), None);
        for (drove, of, saying) in [
            (Some(3), None, "half a count"),
            (None, Some(10), "half a count"),
            (Some(11), Some(10), "cannot be a run"),
            (Some(0), Some(0), "cannot be a run"),
        ] {
            let wrong = MeasuredOn {
                drove,
                of,
                ..sound()
            };
            assert!(
                wrong
                    .what_is_wrong_with_it()
                    .is_some_and(|why| why.contains(saying)),
                "{wrong:?}"
            );
        }
    }

    #[test]
    fn a_statement_that_says_all_three_things_holds() {
        assert_eq!(sound().what_is_wrong_with_it(), None);
        let guest = MeasuredOn {
            machine: "a WSL2 Ubuntu guest, four cores and 5.8 GiB, no graphics card".to_owned(),
            ..sound()
        };
        assert_eq!(guest.what_is_wrong_with_it(), None);
    }

    /// **Every way a grade can be given a machine without being given one.**
    #[test]
    fn every_way_a_machine_can_be_named_without_being_stated_is_refused() {
        for (machine, date, runtime, saying) in [
            (
                "  ",
                "2026-09-13",
                "Ollama 0.34.0",
                "no machine in particular",
            ),
            ("Apple M3", "2026-09-13", "Ollama 0.34.0", "how much memory"),
            (
                "Apple M3, lots of GB",
                "2026-09-13",
                "Ollama 0.34.0",
                "how much memory",
            ),
            (
                "Apple M3 8GB",
                "2026-09-13",
                "Ollama 0.34.0",
                "how much memory",
            ),
            ("Apple M3, 8 GB", "13/09/2026", "Ollama 0.34.0", "no day"),
            ("Apple M3, 8 GB", "2026-09-31", "Ollama 0.34.0", "no day"),
            ("Apple M3, 8 GB", "2026-9-13", "Ollama 0.34.0", "no day"),
            ("Apple M3, 8 GB", "", "Ollama 0.34.0", "no day"),
            ("Apple M3, 8 GB", "2026-09-13", "Ollama", "no version"),
            ("Apple M3, 8 GB", "2026-09-13", "", "no version"),
            ("Apple M3, 8 GB", "2026-09-13", "0.34.0", "no version"),
        ] {
            let wrong = MeasuredOn {
                machine: machine.to_owned(),
                date: date.to_owned(),
                runtime: runtime.to_owned(),
                ..sound()
            };
            assert!(
                wrong
                    .what_is_wrong_with_it()
                    .is_some_and(|why| why.contains(saying)),
                "{wrong:?} was accepted, or refused for something other than `{saying}`"
            );
        }
    }

    /// The calendar is the calendar, including the day that exists one year in
    /// four.
    #[test]
    fn a_date_is_a_day_that_happened() {
        assert!(is_a_calendar_date("2028-02-29"));
        assert!(!is_a_calendar_date("2026-02-29"));
        assert!(!is_a_calendar_date("2100-02-29"));
        assert!(is_a_calendar_date("2000-02-29"));
        assert!(!is_a_calendar_date("2026-13-01"));
        assert!(!is_a_calendar_date("2026-00-10"));
        assert!(!is_a_calendar_date("2026-01-00"));
        assert!(!is_a_calendar_date("2026-01-01-01"));
    }
}
