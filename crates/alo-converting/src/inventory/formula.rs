//! What a spreadsheet formula calls for, when what it calls for will not be the
//! same tomorrow.
//!
//! Two kinds of formula cannot survive being turned into a page: one whose value
//! is **the moment it was calculated**, and one whose value is **chance**. A PDF
//! copy shows the number that was there when the engine read it, and a person
//! who opens the copy next week is looking at last week's answer presented as
//! this week's. ADR 0039 §4 says a copy names what it could not carry, and these
//! are two of the things it could not carry.
//!
//! # Why this is its own file
//!
//! Excel and an OpenDocument spreadsheet call these functions by **the same
//! names** — `NOW`, `TODAY`, `RAND` — because both took them from the same
//! lineage, and an OpenDocument writes the call as `of:=NOW()` where Excel
//! writes `NOW()`. Two readers with two opinions about what counts as a call to
//! `NOW` would be two answers to one question, and the one that drifted would be
//! whichever was written second.

use crate::carried::Field;

/// The functions whose value is the moment the sheet is calculated.
const THE_CURRENT_MOMENT: [&str; 2] = ["NOW", "TODAY"];

/// The functions whose value is a random number.
const A_RANDOM_NUMBER: [&str; 3] = ["RAND", "RANDBETWEEN", "RANDARRAY"];

/// What a formula is, when it calls for the moment or for chance.
///
/// A function counts where its name stands on its own before a bracket, so
/// `NOW()` is the moment and `KNOWN()` and a sheet called `NOW` are not. An
/// OpenDocument's namespace prefix and its leading `=` are before the name and
/// are not letters, so they are handled by the same rule rather than by a case
/// of their own.
pub(crate) fn field_of(formula: &str) -> Option<Field> {
    let formula = formula.to_ascii_uppercase();
    let calls = |function: &str| {
        formula.match_indices(function).any(|(at, _)| {
            let before = formula.get(..at).and_then(|before| before.chars().last());
            let after = formula.get(at + function.len()..);
            before.is_none_or(|letter| !(letter.is_ascii_alphanumeric() || letter == '_'))
                && after.is_some_and(|after| after.trim_start().starts_with('('))
        })
    };
    if THE_CURRENT_MOMENT.iter().any(|function| calls(function)) {
        return Some(Field::TheCurrentMoment);
    }
    A_RANDOM_NUMBER
        .iter()
        .any(|function| calls(function))
        .then_some(Field::ARandomNumber)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The moment and chance are found by function**, and what only looks
    /// like one is not.
    #[test]
    fn the_moment_and_chance_are_found_by_function() {
        assert_eq!(field_of("NOW()"), Some(Field::TheCurrentMoment));
        assert_eq!(field_of("A1+today ()"), Some(Field::TheCurrentMoment));
        assert_eq!(field_of("_xlfn.RANDARRAY(3)"), Some(Field::ARandomNumber));
        assert_eq!(field_of("RANDBETWEEN(1,6)"), Some(Field::ARandomNumber));
        assert_eq!(field_of("A2*2"), None);
        assert_eq!(field_of("KNOWN(A1)"), None);
        assert_eq!(field_of("NOW!A1"), None);
        assert_eq!(field_of("SUM(NOWHERE)"), None);
    }

    /// **An OpenDocument writes the same call with a prefix and an `=`**, and
    /// it is the same call.
    #[test]
    fn an_opendocument_writes_the_same_call_differently() {
        assert_eq!(field_of("of:=NOW()"), Some(Field::TheCurrentMoment));
        assert_eq!(
            field_of("of:=[.A1]+RAND()"),
            Some(Field::ARandomNumber),
            "a formula whose call is not the first thing in it"
        );
        assert_eq!(field_of("of:=SUM([.A1:.A9])"), None);
    }
}
