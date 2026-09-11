//! Whose file an entry names when its publisher never published one.
//!
//! [ADR 0026](../../../docs/decisions/0026-whose-requantisation-this-catalogue-vouches-for.md)
//! is the whole of why this file exists, and rule 6 of `data/catalogue.toml` is
//! it as the sentence a curator reads.
//!
//! # The question it answers
//!
//! Rule 4 made [`crate::Model::quantisation`] point at a file. Two entries could
//! point at nothing: `eurollm-9b-instruct` and `teuken-7b-instruct` name
//! publishers who ship safetensors and no GGUF at all, so every four-bit file of
//! either model is somebody else's requantisation. They are also the two
//! European entries, which the catalogue carries *because nobody else lists
//! them* — so a catalogue that never names a third party's artefact is one whose
//! eligibility rule quietly tracks whether a publisher has a distribution team.
//!
//! The decision is that such a file **may** be named, and that naming one costs
//! three statements: whose it is, which file exactly, and what a reader needs to
//! know about it. What this catalogue borrows is a *file*, stated as somebody
//! else's and pinned so it cannot change underneath us. It never borrows a
//! measurement — ADR 0007 is untouched, and [`crate::Driving`] is still a grade
//! `alo-driving` earned on a machine we ran it on.
//!
//! # Why each of the three is refused rather than encouraged
//!
//! - **The requantiser**, because a name read out of a URL is a name nobody
//!   wrote down. It is also refused when it equals the publisher: that is the
//!   publisher's own artefact and this block is a claim about it that is not
//!   true.
//! - **The pin**, because a tag can be re-pointed at a different file after we
//!   measured the first one. With a `sha256` beside it, *the file we graded* and
//!   *the file a machine fetches* are the same file or the fetch fails; without
//!   one, they differ silently, which is the failure this whole decision is
//!   about.
//! - **The note**, because rule 1 already refuses a licence that says
//!   "conditions apply" without saying which, and a derivative somebody else
//!   built and uploaded is the same shape of claim one field over. Requiring a
//!   sentence is also what stops an entry being completed by a curator who has
//!   not looked at the file: there is nothing true to write in it if they have
//!   not.

use serde::Deserialize;

/// How many characters a `sha256` is written in.
const A_DIGEST_IS: usize = 64;

/// **Whose artefact this entry names, when it is not the publisher's.**
///
/// Absent on every entry whose `artefact` its own publisher publishes, which is
/// most of them — and absent on an entry that names no artefact at all, where
/// there is nothing for it to be about.
///
/// Every field is required. A half-written statement about somebody else's file
/// fails to load rather than loading with a blank, for the reason
/// [`crate::Licence::note`] is required whenever commercial use carries
/// conditions: the case where a field is missing is exactly the case a reader
/// needed it.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requantised {
    /// Who made this artefact — a person or an organisation, as they publish
    /// under.
    ///
    /// Never the publisher of the weights: an entry that says so is describing
    /// a first-party artefact as a stranger's, and the loader refuses it.
    pub by: String,

    /// **The artefact's own `sha256`**, as the repository it is fetched from
    /// publishes it: sixty-four lowercase hexadecimal characters.
    ///
    /// One spelling rather than two, so that two entries naming one file cannot
    /// disagree about its digest, and lowercase because that is how the file
    /// lists a curator copies it from write it.
    pub sha256: String,

    /// What a reader needs to know about this file beyond who made it: how it
    /// was made where the uploader says so, and any terms the upload carries
    /// beyond the model's own.
    pub note: String,
}

impl Requantised {
    /// What is wrong with this statement, in words a curator can act on, or
    /// [`None`] when it holds together.
    ///
    /// `publisher` and `names_an_artefact` are the two things about the entry
    /// around it that this claim has to agree with, which is why the check lives
    /// here and the entry's other rules live in [`crate::catalogue`].
    pub(crate) fn what_is_wrong_with_it(
        &self,
        publisher: &str,
        names_an_artefact: bool,
    ) -> Option<&'static str> {
        if !names_an_artefact {
            return Some(
                "a requantiser named beside no artefact: rule 6 says whose file an entry means \
                 and rule 4 says which file, and this entry names nobody's",
            );
        }
        if self.by.trim().is_empty() {
            return Some(
                "an artefact somebody else made and no name for who: choosing whose \
                 requantisation this catalogue vouches for is a decision that carries a name",
            );
        }
        if self.by.trim().eq_ignore_ascii_case(publisher.trim()) {
            return Some(
                "a requantisation attributed to the model's own publisher, which is a first-party \
                 artefact: drop the block rather than describing it as a stranger's",
            );
        }
        if !is_a_digest(&self.sha256) {
            return Some(
                "a pin nothing can check: state the artefact's own sha256 as its repository \
                 publishes it, in sixty-four lowercase hexadecimal characters, or a tag re-pointed \
                 after we measured it changes the file underneath the grade",
            );
        }
        if self.note.trim().is_empty() {
            return Some(
                "a third party's artefact with nothing said about it: say how it was made where \
                 the uploader says so, and what its upload carries beyond the model's own terms",
            );
        }
        None
    }
}

/// Whether this is a `sha256` as a repository writes one.
fn is_a_digest(written: &str) -> bool {
    written.len() == A_DIGEST_IS
        && written
            .chars()
            .all(|letter| letter.is_ascii_digit() || ('a'..='f').contains(&letter))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A statement that holds together, so the refusals below are about the
    /// fault each names rather than about a fixture that was never sound.
    fn sound() -> Requantised {
        Requantised {
            by: "somebody-else".to_owned(),
            sha256: "a".repeat(A_DIGEST_IS),
            note: "built from the publisher's own release with the pinned converter".to_owned(),
        }
    }

    #[test]
    fn a_statement_that_says_all_three_things_holds() {
        assert_eq!(sound().what_is_wrong_with_it("A Publisher", true), None);
    }

    /// **Each of the three, missing**, and the two agreements with the entry
    /// around it. This is the half a green catalogue cannot show anybody.
    #[test]
    fn every_way_a_borrowed_file_can_be_named_without_being_stated_is_refused() {
        let blank_name = Requantised {
            by: "   ".to_owned(),
            ..sound()
        };
        let blank_note = Requantised {
            note: " ".to_owned(),
            ..sound()
        };
        for (wrong, publisher, names_an_artefact, saying) in [
            (sound(), "A Publisher", false, "names nobody's"),
            (blank_name, "A Publisher", true, "no name for who"),
            (sound(), "somebody-ELSE", true, "own publisher"),
            (blank_note, "A Publisher", true, "nothing said about it"),
        ] {
            assert!(
                wrong
                    .what_is_wrong_with_it(publisher, names_an_artefact)
                    .is_some_and(|why| why.contains(saying)),
                "{wrong:?} was accepted, or refused for something other than `{saying}`"
            );
        }
    }

    /// **A pin nothing can check is not a pin.** Every shape a digest can be
    /// written wrongly in, including the one that looks right — a hash of the
    /// wrong length, which is what a truncated copy-paste leaves.
    #[test]
    fn a_digest_that_is_not_one_is_refused() {
        for written in [
            "",
            "not-a-digest",
            &"a".repeat(A_DIGEST_IS - 1),
            &"a".repeat(A_DIGEST_IS + 1),
            &"A".repeat(A_DIGEST_IS),
            &format!("{}g", "a".repeat(A_DIGEST_IS - 1)),
            &format!(" {}", "a".repeat(A_DIGEST_IS - 1)),
        ] {
            assert!(!is_a_digest(written), "`{written}` was read as a digest");
            let wrong = Requantised {
                sha256: written.to_owned(),
                ..sound()
            };
            assert!(
                wrong
                    .what_is_wrong_with_it("A Publisher", true)
                    .is_some_and(|why| why.contains("pin nothing can check")),
                "`{written}` was accepted as a pin"
            );
        }
        assert!(is_a_digest(&format!(
            "{}{}",
            "0123456789abcdef".repeat(3),
            "0123456789abcdef"
        )));
    }
}
