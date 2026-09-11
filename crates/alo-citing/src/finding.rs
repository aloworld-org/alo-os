//! What the check found, in sentences whoever wrote the citation acts on.
//!
//! Every finding names the citation, the words it was written in and the line it
//! is on, because it is read by whoever has that line open and one thing to
//! change. *An ADR reference does not resolve* would send them to read a
//! directory instead.

/// One thing wrong between what this repository points at and the decisions it
/// has.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Finding {
    /// A citation of a decision nobody wrote.
    ///
    /// **The one this check exists for.** An ADR reference is the one kind of
    /// pointer a reader believes without opening, because the number looks like
    /// a fact — so a number with no file behind it is not a broken link, it is a
    /// sentence that has borrowed the authority of a decision that was never
    /// made.
    #[error(
        "{file} line {line} cites `{said}`, and no decision numbered {number} exists under \
         {directory}/. If the decision is there under another number, cite that one; if it is \
         another repository's, name the repository on the same line before the number, the way \
         docs/contracts/agent-verbs.md names `alo-workplace`; and if nobody has written it, the \
         sentence is claiming a decision that was never made"
    )]
    ADecisionNobodyWrote {
        /// The number cited.
        number: String,
        /// The citation's own words.
        said: String,
        /// Where it is.
        file: String,
        /// Which line of that file.
        line: usize,
        /// Where the decisions are.
        directory: &'static str,
    },

    /// A link or a path naming a decision file that is not there.
    ///
    /// A filename is the pointer a reader is least likely to check, because it
    /// looks copied rather than typed — and it is the one that rots on its own,
    /// since a decision renamed leaves every link to it reading exactly as it
    /// did before.
    #[error(
        "{file} line {line} names `{named}`, and there is no such file under {directory}/. A \
         decision renamed leaves every link to it reading exactly as it did before, so this is \
         either a rename that was not followed through or a file nobody wrote"
    )]
    AFileNobodyWrote {
        /// The filename named.
        named: String,
        /// Where it is named.
        file: String,
        /// Which line of that file.
        line: usize,
        /// Where the decisions are.
        directory: &'static str,
    },

    /// Two files claiming one number.
    ///
    /// Which decision a citation resolves to would then depend on which file a
    /// reader opened, and both of them would look like the answer.
    #[error(
        "`{one}` and `{other}` both claim decision number {number}. Every citation of that number \
         resolves to whichever one a reader opens first, and both of them look like the answer — \
         renumber one of them, and follow the rename through the citations"
    )]
    TwoDecisionsOneNumber {
        /// The number both claim.
        number: String,
        /// One of the files.
        one: String,
        /// The other.
        other: String,
    },

    /// A decision whose own file does not say what its status is.
    ///
    /// The citation resolves and the file opens, and the reader takes it for a
    /// rule — when it may be a recommendation waiting on the owner.
    #[error(
        "`{file}` does not record what its status is. An ADR nobody recorded as accepted or \
         proposed is one every reader will read as settled, and a citation of it carries that \
         weight without anybody having given it — write a `**Status:**` line saying which of \
         accepted, proposed, superseded, rejected or withdrawn it is"
    )]
    ADecisionWithNoStatus {
        /// The decision's filename.
        file: String,
    },

    /// A neighbouring repository named as one this repository cites, which
    /// nothing cites.
    ///
    /// The list of neighbours is a standing permission for a number not to
    /// resolve here. One nobody needs is one nobody will read again, and it will
    /// still be standing on the day a citation of that repository is a typo.
    #[error(
        "`{repository}` is named as a repository whose decisions this one may cite, and nothing \
         cites one. Take it off: a standing permission for a number not to resolve is one more \
         place a citation of a decision nobody wrote can hide"
    )]
    ANeighbourNobodyCites {
        /// The repository named.
        repository: String,
    },

    /// No decisions to check anything against.
    ///
    /// Everything else here is about one citation. This is the failure that
    /// would otherwise be silent in the other direction: with no decisions read,
    /// every citation in the repository is of a decision nobody wrote, or — worse
    /// — nothing is checked at all and the check reports in the same colour it
    /// does when everything lands.
    #[error(
        "no decision was read under {directory}/, and this repository has more than twenty. Either \
         the directory could not be read or the convention moved — four digits, a hyphen, and `.md` \
         — and either way nothing was checked against anything"
    )]
    NoDecisionsToCheck {
        /// Where the decisions were looked for.
        directory: &'static str,
    },

    /// Nothing in the repository was read as citing a decision.
    ///
    /// This repository carries more than four hundred citations. None found
    /// means the files were not reached or the convention moved, and a check
    /// that has stopped looking passes in exactly the same colour as one that
    /// looked.
    #[error(
        "{files} file(s) were read and not one of them was seen to cite a decision, in a repository \
         that carries hundreds. Either those files are not the repository's Rust and Markdown or \
         the convention moved — `ADR` and four digits — and either way nothing was held to \
         anything"
    )]
    NothingCitesAnything {
        /// How many files were read.
        files: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decisions::THE_DECISIONS;

    /// The finding this crate exists for names the number, the words, the file
    /// and the line, and says both ways out — because whoever reads it has that
    /// one line open.
    #[test]
    fn a_finding_names_the_citation_and_where_it_is() {
        let nobody_wrote = "0099";
        let said = Finding::ADecisionNobodyWrote {
            number: nobody_wrote.to_owned(),
            said: format!("ADR {nobody_wrote}"),
            file: "crates/alo-files/src/lib.rs".to_owned(),
            line: 12,
            directory: THE_DECISIONS,
        }
        .to_string();
        assert!(said.contains("0099"), "{said}");
        assert!(said.contains("crates/alo-files/src/lib.rs"), "{said}");
        assert!(said.contains("line 12"), "{said}");
        assert!(said.contains(THE_DECISIONS), "{said}");
        assert!(said.contains("another repository"), "{said}");
    }

    /// Two files claiming one number names both, since which to renumber is the
    /// reader's call and neither is wrong on its own.
    #[test]
    fn two_decisions_one_number_names_both() {
        let twice = format!("{}-something-else.md", "0024");
        let said = Finding::TwoDecisionsOneNumber {
            number: "0024".to_owned(),
            one: "0024-what-a-person-signs-in-at.md".to_owned(),
            other: twice.clone(),
        }
        .to_string();
        assert!(said.contains("0024-what-a-person-signs-in-at.md"), "{said}");
        assert!(said.contains(&twice), "{said}");
    }

    /// And the two silent failures name where they looked, because there is no
    /// citation to name in either.
    #[test]
    fn the_silent_failures_name_where_they_looked() {
        let nothing = Finding::NoDecisionsToCheck {
            directory: THE_DECISIONS,
        }
        .to_string();
        assert!(nothing.contains(THE_DECISIONS), "{nothing}");

        let unread = Finding::NothingCitesAnything { files: 400 }.to_string();
        assert!(unread.contains("400"), "{unread}");
    }
}
