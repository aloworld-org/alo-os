//! Where this repository points at a decision by number, read out of the text.
//!
//! This is the half of the check no list can do. A list of citations kept beside
//! the decisions would only ever say that one list agrees with another; what has
//! to be answered is *which numbers this repository actually sends a reader to*,
//! and the only place that is written down is the sentences themselves — a
//! crate's rustdoc, a finding's own words, a promise's evidence.
//!
//! # The convention this reads, which is therefore a rule
//!
//! **A decision is cited as `ADR` and four digits** — `ADR 0001`, and `ADR-0015`
//! and `ADRs 0023` are read too, because both are written here. Four digits is
//! the form every one of the four hundred citations in this repository already
//! uses, `docs/decisions/README.md` now says so where whoever writes the next one
//! reads it, and this file is what makes it load-bearing rather than tidy.
//!
//! A list continues, so `alo-workplace`'s own `ADRs 0023, 0047, 0057, 0058` is
//! four citations and not one — a number that only a comma introduces is the
//! easiest place for a pointer to go unchecked. A continuation has to be
//! introduced by a comma or by `and`, and its four digits have to begin with `0`
//! and not run into a hyphen, so the year in `ADR 0018, 2026-09-04` is not read
//! as a decision nobody wrote.
//!
//! # A neighbouring repository's decisions
//!
//! `alo-workplace` has ADRs of its own and this repository cites four of them.
//! Nothing under `docs/decisions/` here answers for those, and refusing them
//! would be this check demanding that a true sentence be deleted.
//!
//! So a citation is read as **this** repository's unless the line names the
//! repository it points into, before the number. `alo-workplace`'s ADR 0047 is
//! elsewhere's, written just like that; the same number with no repository in
//! front of it on its line is a decision nobody wrote, and is refused. The unit
//! is the line, which is what makes the rule checkable and also what makes it a
//! demand on writing: keep the repository's name with the number it qualifies. A
//! reader needs exactly the same thing, and cannot scroll up for it either.

use crate::citation::Citation;

/// How a decision is cited.
const AS: &str = "ADR";

/// How many digits a decision's number has.
const DIGITS: usize = 4;

/// Every reference to a decision by number in one file, in the order they are
/// written.
///
/// `neighbours` are the repositories other than this one whose decisions this
/// text may point into; a citation qualified by one of them carries its name and
/// is nothing this repository's `docs/decisions/` answers for.
#[must_use]
pub fn cited_in(file: &str, text: &str, neighbours: &[&str]) -> Vec<Citation> {
    let mut cited = Vec::new();
    for (index, line) in text.lines().enumerate() {
        gather(file, line, index + 1, neighbours, &mut cited);
    }
    cited
}

/// Every citation on one line, appended in the order they are written.
fn gather(file: &str, line: &str, at: usize, neighbours: &[&str], cited: &mut Vec<Citation>) {
    for (start, _) in line.match_indices(AS) {
        if line
            .get(..start)
            .and_then(|before| before.chars().next_back())
            .is_some_and(|previous| previous.is_alphanumeric())
        {
            continue;
        }
        let Some(after) = line.get(start.saturating_add(AS.len())..) else {
            continue;
        };
        let plural = after.strip_prefix('s').unwrap_or(after);
        let Some(spelled) = plural
            .strip_prefix(' ')
            .or_else(|| plural.strip_prefix('-'))
        else {
            continue;
        };
        let Some(number) = four_digits(spelled) else {
            continue;
        };

        let from = line
            .get(..start)
            .and_then(|before| neighbours.iter().find(|named| before.contains(**named)))
            .map(|named| (*named).to_owned());
        let said = line
            .get(start..)
            .and_then(|from_here| from_here.get(..from_here.len().min(said_width(plural, after))))
            .unwrap_or(AS);
        cited.push(Citation::new(
            number.clone(),
            said.to_owned(),
            file.to_owned(),
            at,
            from.clone(),
        ));

        let mut rest = spelled.get(DIGITS..).unwrap_or_default();
        while let Some((continued, after_it)) = continuation(rest) {
            cited.push(Citation::new(
                continued.clone(),
                continued,
                file.to_owned(),
                at,
                from.clone(),
            ));
            rest = after_it;
        }
    }
}

/// How many bytes of a line one citation's own words take up: `ADR`, the `s` or
/// the separator, and the digits.
fn said_width(plural: &str, after: &str) -> usize {
    AS.len()
        .saturating_add(after.len().saturating_sub(plural.len()))
        .saturating_add(1)
        .saturating_add(DIGITS)
}

/// The four digits at the front of `spelled`, where they are four and are not
/// the front of a longer number.
fn four_digits(spelled: &str) -> Option<String> {
    let number: String = spelled
        .chars()
        .take(DIGITS)
        .take_while(char::is_ascii_digit)
        .collect();
    if number.len() != DIGITS {
        return None;
    }
    match spelled.chars().nth(DIGITS) {
        Some(next) if next.is_ascii_digit() => None,
        _ => Some(number),
    }
}

/// The next number of a list, where the text continues into one, and what is
/// left of the line after it.
///
/// A comma or an `and` has to introduce it, so two numbers that merely stand
/// near each other are not read as a list; and the digits have to begin with `0`
/// and not run into a hyphen, so a date is not read as a decision.
fn continuation(rest: &str) -> Option<(String, &str)> {
    let (introduced, listed) = match rest.strip_prefix(',') {
        Some(after_comma) => (true, after_comma),
        None => (false, rest),
    };
    let spaced = listed.strip_prefix(' ')?;
    let (introduced, numbered) = match spaced.strip_prefix("and ") {
        Some(after_and) => (true, after_and),
        None => (introduced, spaced),
    };
    if !introduced || !numbered.starts_with('0') {
        return None;
    }
    let number = four_digits(numbered)?;
    let after = numbered.get(DIGITS..).unwrap_or_default();
    if after.starts_with('-') {
        return None;
    }
    Some((number, after))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A number belonging to `alo-workplace` and to no decision here.
    ///
    /// Never written beside the word that makes it a citation — the check reads
    /// this file too, and a number with no decision behind it, spelled out, would
    /// be a pointer this repository really carries.
    const ELSEWHERES: &str = "0047";

    /// The numbers of every citation in a text, for a test that is about which
    /// numbers were seen rather than about where they were.
    fn numbers(text: &str, neighbours: &[&str]) -> Vec<String> {
        cited_in("a-file.md", text, neighbours)
            .iter()
            .map(|citation| citation.number().to_owned())
            .collect()
    }

    /// The three spellings this repository really uses, each read as one
    /// citation of the number it names.
    #[test]
    fn the_spellings_this_repository_uses_are_read() {
        assert_eq!(
            numbers("ADR 0001 §3 says a grant is a folder somebody picked.", &[]),
            ["0001"]
        );
        assert_eq!(numbers("The ADR-0015 reporting tension.", &[]), ["0015"]);
        assert_eq!(numbers("carried forward (its ADRs 0023)", &[]), ["0023"]);
    }

    /// A list is as many citations as it has numbers. The one that is only
    /// introduced by a comma is the easiest place for a pointer to go unchecked.
    #[test]
    fn a_list_is_as_many_citations_as_it_has_numbers() {
        assert_eq!(
            numbers("`alo-workplace` (its ADRs 0023, 0047, 0057 and 0058)", &[]),
            ["0023", "0047", "0057", "0058"]
        );
    }

    /// And a date after a citation is not one of them, nor is a number that
    /// merely stands near one.
    #[test]
    fn a_date_after_a_citation_is_not_a_decision() {
        assert!(numbers("ADR 0018, 2026-09-04 rewrote the history.", &[]).len() == 1);
        assert_eq!(numbers("ADR 0018 0047 stands alone.", &[]), ["0018"]);
        assert_eq!(
            numbers("ADR 00123 is not four digits.", &[]),
            [] as [&str; 0]
        );
        assert_eq!(numbers("ADR 12 is not four digits.", &[]), [] as [&str; 0]);
    }

    /// A word that merely ends in those three letters is not a citation.
    #[test]
    fn a_word_ending_in_those_letters_is_not_a_citation() {
        assert_eq!(numbers("The SADR 0001 protocol.", &[]), [] as [&str; 0]);
    }

    /// **A citation qualified by a neighbouring repository is that repository's**
    /// — and the same number, unqualified, is this one's.
    #[test]
    fn a_neighbours_decision_carries_the_neighbours_name() {
        let qualified = cited_in(
            "docs/contracts/agent-verbs.md",
            "the workspace (`alo-workplace` ADR 0047), because making a question wait",
            &["alo-workplace"],
        );
        assert_eq!(
            qualified.first().and_then(Citation::elsewhere),
            Some("alo-workplace"),
            "{qualified:?}"
        );

        let bare = cited_in(
            "a-file.md",
            &format!("Unchanged from ADR {ELSEWHERES}."),
            &["alo-workplace"],
        );
        assert_eq!(
            bare.first().and_then(Citation::elsewhere),
            None,
            "a bare citation was excused by a repository nobody named"
        );
    }

    /// The name has to come **before** the number on the line it qualifies. A
    /// reader cannot scroll up for it either, and neither can this.
    #[test]
    fn the_name_qualifies_only_what_follows_it() {
        let after = cited_in(
            "a-file.md",
            &format!("ADR {ELSEWHERES} is what `alo-workplace` decided."),
            &["alo-workplace"],
        );
        assert_eq!(
            after.first().and_then(Citation::elsewhere),
            None,
            "{after:?}"
        );
    }

    /// The words a citation was written in are quoted back, so whoever has the
    /// line open recognises it.
    #[test]
    fn a_citation_quotes_the_words_it_was_written_in() {
        let cited = cited_in("a-file.md", "see ADR-0015 and ADRs 0023 below", &[]);
        let said: Vec<&str> = cited.iter().map(Citation::said).collect();
        assert_eq!(said, ["ADR-0015", "ADRs 0023"]);
    }

    /// Lines are counted from one, because that is how an editor counts them.
    #[test]
    fn lines_are_counted_from_one() {
        let cited = cited_in("a-file.md", "nothing here\nADR 0009 here", &[]);
        assert_eq!(cited.first().map(Citation::line), Some(2));
    }
}
