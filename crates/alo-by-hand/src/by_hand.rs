//! The way a person does the same thing without the agent.
//!
//! Two things, and both are required. **A sentence**, saying what a person
//! actually does — that is for the reader, and the floor under it is
//! [`crate::answer`]'s. **A quotation from `docs/features.md`**, naming the
//! surface that does it — that is for the check, and it is what makes this
//! document something other than a place to write reassuring sentences.
//!
//! The quotation is what ADR 0009 asks for in its own words: *every ★ agent
//! capability names the plain way to do the same thing, and if it cannot name
//! one, the plain way is missing work rather than an acceptable gap.* A plain
//! way that is not promised in the definition is not scheduled, not tiered and
//! not gated by anything — so quoting the definition is how *there is a plain
//! way* stops being an opinion.
//!
//! It is also where the release comes from. Nothing here says which release owns
//! a by-hand answer, because the line quoted already carries a tier, and a tier
//! written down twice is a tier that can disagree with itself.

use crate::answer;
use crate::wrapping::one_line;

/// How a person does the same thing without the agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByHand {
    /// The sentence, as it will be read.
    sentence: String,

    /// Every phrase it quotes out of `docs/features.md`, verbatim, before
    /// anything decides whether the definition really promises them.
    ///
    /// Kept raw so that a quotation the definition does not make is a finding
    /// with the words in it, rather than a phrase silently dropped on the way in.
    naming: Vec<String>,
}

impl ByHand {
    /// The plain way, if the document said anything.
    ///
    /// Whitespace-only is nothing said, and so is a dash — which is how a table
    /// would write *none*, and is deliberately not a way to record an empty
    /// answer as an answer.
    #[must_use]
    pub fn said(text: &str) -> Option<Self> {
        let sentence = text.trim().trim_matches('—').trim();
        (!sentence.is_empty()).then(|| Self {
            naming: backticked_in(sentence),
            sentence: sentence.to_owned(),
        })
    }

    /// The sentence, as it will be read.
    #[must_use]
    pub fn sentence(&self) -> &str {
        &self.sentence
    }

    /// Every phrase it quotes out of the definition, in the order it quotes them.
    #[must_use]
    pub fn naming(&self) -> &[String] {
        &self.naming
    }

    /// Whether it is an answer rather than a shrug.
    #[must_use]
    pub fn is_an_answer(&self) -> bool {
        answer::is_an_answer(&self.sentence)
    }
}

/// Everything written in backticks in a piece of text, in order, each read as
/// one line.
///
/// The odd-numbered pieces of a split on the backtick are what was between a
/// pair of them; an unclosed backtick therefore contributes its tail and is
/// caught downstream as a phrase the definition does not promise, which names it.
///
/// The line break markdown put inside a long quotation is taken out here rather
/// than at the comparison, so that a finding about it quotes it back on one line
/// — [`crate::wrapping`] is the whole of that argument.
fn backticked_in(text: &str) -> Vec<String> {
    text.split('`')
        .skip(1)
        .step_by(2)
        .map(one_line)
        .filter(|quoted| !quoted.is_empty())
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The shape every by-hand answer is written in, including the line break
    /// markdown puts inside a long one.
    const AN_ANSWER_AS_WRITTEN: &str = "\
a person opens the folder in the file manager and reads what is in it —
`A file manager, with trash, and archives that open`, and `File associations
— what opens what, changeable by a person` for what a document opens in.
";

    /// **The sentence and every quotation are both read**, and a quotation that
    /// wrapped onto a second line is not lost.
    #[test]
    fn an_answer_carries_its_sentence_and_what_it_quotes() {
        let answer = ByHand::said(AN_ANSWER_AS_WRITTEN).unwrap();
        assert!(answer.is_an_answer());
        assert_eq!(
            answer.naming(),
            [
                "A file manager, with trash, and archives that open".to_owned(),
                "File associations — what opens what, changeable by a person".to_owned(),
            ],
            "a quotation that wrapped onto a second line was lost"
        );
        assert!(answer.sentence().starts_with("a person opens the folder"));
    }

    /// Nothing said is nothing said, however it was written.
    #[test]
    fn an_empty_answer_is_not_an_answer() {
        assert_eq!(ByHand::said(""), None);
        assert_eq!(ByHand::said("   \n "), None);
        assert_eq!(
            ByHand::said("—"),
            None,
            "a dash was read as the way a person does something, so a verb with \
             no plain way could be answered by a punctuation mark"
        );
    }

    /// A sentence quoting nothing is still read, and still says nothing the
    /// definition promises. The finding names the verb, which needs the answer to
    /// have been parsed first.
    #[test]
    fn a_sentence_that_quotes_nothing_is_still_read() {
        let vague = ByHand::said("a person can just do it themselves somehow, obviously").unwrap();
        assert!(vague.naming().is_empty());
        assert!(
            vague.is_an_answer(),
            "the floor is about length, not quotation"
        );
    }
}
