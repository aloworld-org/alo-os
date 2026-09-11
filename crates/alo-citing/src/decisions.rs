//! The decisions that exist, and whether each of them says what it is.
//!
//! A citation lands on a file under `docs/decisions/`. Which file is decided by
//! the number at the front of its name, which is the convention every one of the
//! twenty-five already follows.
//!
//! # A decision has to say what its status is
//!
//! This is the one thing here that is not about a pointer landing, and it is in
//! this check for the reason the task gives: **an ADR nobody recorded as accepted
//! or proposed is one every reader will read as settled.** The citation resolves,
//! the file opens, and the reader takes it for a rule — when it may be a
//! recommendation waiting on the owner, which is exactly what
//! `0025-the-default-is-what-a-machine-arrives-able-to-do.md` is.
//!
//! What a status may *say* is not judged beyond its first word being one of the
//! five an ADR can be in. Whether `accepted` was true is the owner's, not this
//! crate's.

/// Where the decisions of this repository are.
pub const THE_DECISIONS: &str = "docs/decisions";

/// How many digits a decision's number has.
const DIGITS: usize = 4;

/// How a decision records what it is.
const SAYS_WHAT_IT_IS: &str = "**Status:**";

/// The states an ADR can be in.
///
/// Not a judgement about the argument: a floor under the file, so that a reader
/// who followed a citation is told whether what they are reading was decided.
const IS_IN: [&str; 5] = [
    "accepted",
    "proposed",
    "superseded",
    "rejected",
    "withdrawn",
];

/// One decision of this repository, as its file has it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// The number at the front of its filename, four digits.
    number: String,

    /// Its filename, without the directory in front of it.
    file: String,

    /// Whether the file records what its status is.
    says_what_it_is: bool,
}

impl Decision {
    /// The number at the front of its filename.
    #[must_use]
    pub fn number(&self) -> &str {
        &self.number
    }

    /// Its filename.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Whether the file records what its status is.
    #[must_use]
    pub fn says_what_it_is(&self) -> bool {
        self.says_what_it_is
    }
}

/// Every decision among the files of `docs/decisions/`, in the order given.
///
/// A file whose name is not a decision's — a `README.md`, which that directory
/// now has — is not one of them and nothing is asked of it. A decision misnamed
/// is therefore not seen here, and shows up instead as every citation of its
/// number naming a decision nobody wrote, which is the same finding wearing the
/// name of the thing somebody has to fix.
#[must_use]
pub fn among(files: &[(&str, &str)]) -> Vec<Decision> {
    files
        .iter()
        .filter_map(|(file, text)| {
            let number: String = file
                .chars()
                .take(DIGITS)
                .take_while(char::is_ascii_digit)
                .collect();
            if number.len() != DIGITS
                || file.chars().nth(DIGITS) != Some('-')
                || !file.ends_with(".md")
            {
                return None;
            }
            Some(Decision {
                number,
                file: (*file).to_owned(),
                says_what_it_is: says_what_it_is(text),
            })
        })
        .collect()
}

/// Whether a decision's own text records what its status is.
fn says_what_it_is(text: &str) -> bool {
    text.lines()
        .filter_map(|line| line.trim_start().strip_prefix(SAYS_WHAT_IT_IS))
        .any(|said| {
            let lowered = said.to_lowercase();
            IS_IN.iter().any(|state| lowered.contains(state))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A decision's number is the one at the front of its filename, and a file
    /// named any other way is not a decision at all.
    #[test]
    fn a_decision_is_a_file_named_the_way_a_decision_is_named() {
        let found = among(&[
            ("0001-the-capability-model.md", "**Status:** accepted"),
            ("README.md", "how a decision is written"),
            ("notes.txt", "**Status:** accepted"),
        ]);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found.first().map(Decision::number), Some("0001"));
    }

    /// The five states a decision can be in are read whatever case they were
    /// written in, and whatever follows the dash.
    #[test]
    fn the_states_are_read_in_any_case() {
        for said in [
            "**Status:** accepted",
            "**Status:** **ACCEPTED, 2026-09-10** — Option B",
            "**Status:** proposed — the owner decides",
            "**Status:** superseded by a later decision",
        ] {
            assert!(
                among(&[("0001-the-capability-model.md", said)])
                    .first()
                    .is_some_and(Decision::says_what_it_is),
                "{said}"
            );
        }
    }

    /// **A decision that records no status is one every reader will take for
    /// settled** — which is the finding, not a formatting preference.
    #[test]
    fn a_decision_with_no_status_says_so() {
        for silent in [
            "# A decision — something\n\n**Date:** 2026-09-11\n",
            "# A decision\n\n**Status:** being thought about\n",
            "",
        ] {
            assert!(
                among(&[(
                    "0025-the-default-is-what-a-machine-arrives-able-to-do.md",
                    silent
                )])
                .first()
                .is_some_and(|decision| !decision.says_what_it_is()),
                "{silent:?}"
            );
        }
    }
}
