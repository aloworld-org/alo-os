//! A verb whose plain way is owed, and why.
//!
//! Most of this document should quote the definition. This is the other form,
//! and it exists because the honest state of one verb today is that **nothing in
//! `docs/features.md` promises the plain way at all** — `archive_folder` makes an
//! archive, the definition promises archives that *open*, and stretching the one
//! to cover the other would be this check lying in the first change that used it.
//!
//! ADR 0009 is what makes the debt real: a capability an agent has and a person
//! does not is a capability that disappears when somebody's card is declined.
//!
//! # The debt has lost its date, and that is a weakening
//!
//! Until 2026-10-12 an owed answer had to name the release that owed it —
//! `[v0.5] because …` — and the release had to be one the definition actually
//! made promises for. That was what stopped *owed* being a way round the check:
//! a debt with a date on it rather than a shrug.
//!
//! The owner removed the versions ([ADR
//! 0105](../../../docs/decisions/0105-there-are-no-versions-until-there-is-a-launch.md)),
//! so there is no release to name and the requirement is gone with it. **What
//! remains is the weaker half**: a debt must still say *why*, in words somebody
//! can act on, which [`Owed::is_an_answer`] holds it to.
//!
//! This is recorded rather than quietly absorbed because it is a check that got
//! looser, not one that moved. If an ordering comes back — the launch list is
//! the candidate — a debt should be made to name its place in it again.

use crate::answer;

/// A plain way that is owed, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owed {
    /// Why there is no plain way yet, in words somebody can act on.
    sentence: String,
}

impl Owed {
    /// What is owed, if the document said anything.
    ///
    /// **No brackets are stripped any more.** A `[v0.5]` left in the document
    /// would now be read as part of the sentence and show up in the finding,
    /// which is deliberate: a stale release in the text should be visible rather
    /// than silently swallowed by a parser that no longer has a use for it.
    #[must_use]
    pub fn said(text: &str) -> Option<Self> {
        let sentence = text.trim().trim_start_matches('\u{2014}').trim();
        (!sentence.is_empty()).then(|| Self {
            sentence: sentence.to_owned(),
        })
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

    /// **An owed answer says why, and that is now the whole of the shape.**
    #[test]
    fn an_owed_answer_says_why() {
        let owed = Owed::said(
            "\u{2014} the file manager promises archives that open and not \
             archives a person makes.",
        )
        .unwrap();
        assert!(owed.sentence().starts_with("the file manager promises"));
        assert!(owed.is_an_answer());
    }

    /// Nothing said is nothing said: a debt has to say why, or the next reader
    /// learns only that somebody knew.
    #[test]
    fn nothing_said_is_not_an_answer() {
        assert_eq!(Owed::said(""), None);
        assert_eq!(Owed::said("  \n "), None);
        assert_eq!(Owed::said("\u{2014}"), None, "a dash is not a reason");
        assert!(
            !Owed::said("not yet.").unwrap().is_an_answer(),
            "`not yet` was accepted as why a person cannot do something by hand"
        );
    }

    /// **A release left in the document is read as part of the sentence**, so it
    /// shows up where a reader will see it rather than being stripped by a
    /// parser that no longer uses it.
    #[test]
    fn a_stale_release_is_visible_rather_than_swallowed() {
        let stale = Owed::said("[v0.5] nothing promises making an archive").unwrap();

        assert!(
            stale.sentence().starts_with("[v0.5]"),
            "it was stripped: {}",
            stale.sentence()
        );
    }
}
