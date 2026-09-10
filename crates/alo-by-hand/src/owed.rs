//! A verb whose plain way is owed, and the release that owns the answer.
//!
//! Most of this document should quote the definition. This is the other form,
//! and it exists because the honest state of one verb today is that **nothing in
//! `docs/features.md` promises the plain way at all** — `archive_folder` makes an
//! archive, the definition promises archives that *open*, and stretching the one
//! to cover the other would be this check lying in the first change that used it.
//!
//! So the second form is *owed*, and it costs something to write: the release
//! that owns the answer has to be named, and it has to be a release the
//! definition actually makes promises for. That is what stops *owed* being a way
//! round the check. It is a debt with a date on it rather than a shrug, and
//! whoever reads `docs/by-hand.md` can see exactly how many there are.
//!
//! ADR 0009 is what makes the debt real: a capability an agent has and a person
//! does not is a capability that disappears when somebody's card is declined.

use crate::answer;

/// How an owed answer names the release that owns it.
const IN_BRACKETS: (char, char) = ('[', ']');

/// A plain way that is owed, and the release that owes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owed {
    /// The release as the document names it, if it named one at all.
    ///
    /// Kept as text: whether it is a release alo OS ships is a question about
    /// `docs/features.md`, and this file has never read that document.
    release: Option<String>,

    /// Why there is no plain way yet, in words somebody can act on.
    sentence: String,
}

impl Owed {
    /// What is owed, if the document said anything.
    ///
    /// The release comes first and in brackets — `[v0.5] because …` — because a
    /// reader scanning the document for what is missing is scanning for the
    /// release. Anything else is read as a sentence naming no release, which is
    /// a finding rather than a parse failure: the verb it is about still has to
    /// be named.
    #[must_use]
    pub fn said(text: &str) -> Option<Self> {
        let said = text.trim();
        if said.is_empty() {
            return None;
        }
        let (release, sentence) = match said.strip_prefix(IN_BRACKETS.0) {
            Some(after) => match after.split_once(IN_BRACKETS.1) {
                Some((named, rest)) => (Some(named.trim().to_owned()), rest),
                None => (None, said),
            },
            None => (None, said),
        };
        let sentence = sentence.trim().trim_start_matches('—').trim();
        (!sentence.is_empty()).then(|| Self {
            release,
            sentence: sentence.to_owned(),
        })
    }

    /// The release the document says owns the answer, if it named one.
    #[must_use]
    pub fn release(&self) -> Option<&str> {
        self.release.as_deref()
    }

    /// Why there is no plain way yet, as it will be read.
    #[must_use]
    pub fn sentence(&self) -> &str {
        &self.sentence
    }

    /// Whether it is an answer rather than a shrug.
    #[must_use]
    pub fn is_an_answer(&self) -> bool {
        answer::is_an_answer(&self.sentence)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The shape an owed answer is written in: the release, then why.
    #[test]
    fn an_owed_answer_names_the_release_and_says_why() {
        let owed = Owed::said(
            "[v0.5] — the file manager promises archives that open and not \
             archives a person makes.",
        )
        .unwrap();
        assert_eq!(owed.release(), Some("v0.5"));
        assert!(owed.sentence().starts_with("the file manager promises"));
        assert!(owed.is_an_answer());
    }

    /// An answer that names no release is read rather than refused. It is a debt
    /// with nobody on the hook for it, and the finding has to be able to name the
    /// verb.
    #[test]
    fn an_answer_with_no_release_is_still_read() {
        let vague =
            Owed::said("nothing anywhere promises this and nobody has scheduled it").unwrap();
        assert_eq!(vague.release(), None);
        assert!(vague.is_an_answer());

        let unclosed =
            Owed::said("[v0.5 nothing anywhere promises this, and the bracket is open").unwrap();
        assert_eq!(
            unclosed.release(),
            None,
            "an unclosed bracket was read as a release, so the release named \
             would be the rest of the sentence"
        );
    }

    /// Nothing said is nothing said, and a release with nothing after it says
    /// nothing: a debt has to say why, or the next reader learns only that
    /// somebody knew.
    #[test]
    fn a_release_on_its_own_is_not_an_answer() {
        assert_eq!(Owed::said(""), None);
        assert_eq!(Owed::said("  \n "), None);
        assert_eq!(
            Owed::said("[v0.5]"),
            None,
            "a release with no sentence after it was accepted as what is owed"
        );
        assert!(
            !Owed::said("[v0.5] not yet.").unwrap().is_an_answer(),
            "`not yet` was accepted as why a person cannot do something by hand"
        );
    }
}
