//! What came back, and where it came from.
//!
//! # An answer cannot exist without its provenance
//!
//! `docs/features.md` promises at v0.01: **★ Where the answer came from is said
//! where the answer appears** — *"on this machine", "on the studio
//! workstation", "by a provider you added" — beside the answer, not buried in a
//! setting.* `ROADMAP.md` calls it the promise on its list most easily lost,
//! because it is a sentence that must appear every single time and nothing was
//! forcing it to.
//!
//! This is what forces it. `Answer::new` is `pub(crate)` and takes the source,
//! there is no constructor that does not, and [`Answer::came_from`] renders it
//! in the reader's own language. So a shell holding an answer is holding the
//! sentence about where it came from, and showing one without the other is a
//! thing somebody has to *decide* to do rather than a thing they can forget.
//!
//! # It is held the way the question is
//!
//! No `Serialize` and a [`Debug`](fmt::Debug) written by hand, for
//! [`crate::question`]'s reason: an answer is made out of somebody's question
//! and whatever they had open, and ADR 0001 §7 keeps neither. What the record
//! keeps about this moment is that something left the machine — who, where to,
//! why and when — which is `alo-record`'s and is written from the departure
//! this crate hands back.
//!
//! # And what came back is never put into a sentence of ours
//!
//! [`Answer::text`] is the model's words, shown as the model's words.
//! `alo-answering` refuses to hold text anybody else wrote precisely so that no
//! sentence alo OS says can turn out to be one a provider composed, and the
//! same rule applies here from the other side: this crate has two strings of
//! its own, neither has a gap, and neither can be handed an answer.

use std::fmt;

use alo_models::InferenceSource;
use alo_strings::{Said, Strings};

/// An answer, and the place it came from.
pub struct Answer {
    /// What came back, as the model wrote it.
    text: String,
    /// Where it came from (ADR 0008), carried rather than reconstructed.
    source: InferenceSource,
    /// The model it was put to, as it was named when it was asked.
    model: String,
}

impl Answer {
    /// Made by [`crate::Asking`] and by nothing else, at the moment an answer
    /// arrived from a place that was decided about and shown.
    ///
    /// The source is the one the attempt was permitted against, not one read
    /// off the reply: a provider that named itself in its own answer would be
    /// choosing what a person is told about where their question went.
    pub(crate) fn new(text: String, source: InferenceSource, model: String) -> Self {
        Self {
            text,
            source,
            model,
        }
    }

    /// What the model answered, in its own words.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Where it came from.
    #[must_use]
    pub fn source(&self) -> &InferenceSource {
        &self.source
    }

    /// Which model answered, as it was named when it was asked.
    ///
    /// What was asked for rather than what the reply echoed back. A provider
    /// echoes a name it wrote, and this is the one place a person checks that
    /// their question went to the model they chose — so it is the name they
    /// chose, and a provider cannot answer that question on its own behalf.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Where this came from, in the language the person reads.
    ///
    /// Shown beside the answer, which is the whole of the promise. A [`Said`]
    /// rather than a `String` so that whoever puts it inside a longer line —
    /// *answered by alo, in the EU at 14:02* — has a line that is only as
    /// translated as this clause is.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// `alo_models::model_words` answers with the key, marked.
    #[must_use]
    pub fn came_from(&self, strings: &Strings) -> Said {
        self.source.said(strings)
    }
}

/// Says where the answer came from, and nothing about what it says.
///
/// Written by hand for [`crate::question`]'s reason: an answer is made out of
/// somebody's question and whatever they had open, and a derived `Debug` would
/// put it in every log line that formats a structure holding one.
impl fmt::Debug for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Answer")
            .field("source", &self.source)
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, mistral_source, translated};
    use alo_models::Region;

    fn answered() -> Answer {
        Answer::new(
            "The tenant may not sublet without written consent.".to_owned(),
            mistral_source(),
            "mistral-small-latest".to_owned(),
        )
    }

    /// **The promise as a test.** There is no way to hold an answer and not be
    /// able to say where it came from, because the two are one value.
    #[test]
    fn an_answer_always_knows_where_it_came_from() {
        let answer = answered();
        assert_eq!(answer.source(), &mistral_source());
        assert_eq!(
            answer.came_from(&in_english()).text(),
            "by Mistral, in the EU"
        );
    }

    /// **And it says so in the language the person reads**, with the provider's
    /// name and the region it stated coming through as they were written.
    #[test]
    fn where_it_came_from_is_said_in_the_readers_own_language() {
        let strings = translated(&[]);
        let said = answered().came_from(&strings);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().starts_with("von Mistral"), "{said}");
        assert!(said.text().contains("the EU"), "{said}");
    }

    /// A provider that has not said where it runs says so beside its own
    /// answer, which is where somebody deciding whether to ask it again will
    /// read it.
    #[test]
    fn an_undeclared_provider_says_so_beside_its_own_answer() {
        let answer = Answer::new(
            "…".to_owned(),
            InferenceSource::Hosted {
                provider: "someone".to_owned(),
                region: Region::Unknown,
            },
            "a-model".to_owned(),
        );
        assert!(
            answer
                .came_from(&in_english())
                .text()
                .contains("has not said where it runs")
        );
    }

    /// An answer is made out of somebody's question, so no rendering of it
    /// shows what it says.
    #[test]
    fn an_answer_never_appears_in_anything_that_renders_it() {
        let debugged = format!("{:?}", answered());
        assert!(!debugged.contains("sublet"), "{debugged}");
        assert!(debugged.contains("Mistral"), "{debugged}");
        assert!(debugged.contains("mistral-small-latest"), "{debugged}");
    }

    /// The model named is the one that was asked for. A provider's own echo of
    /// a name is text it wrote, and this is where a person checks that their
    /// question went where they chose.
    #[test]
    fn the_model_named_is_the_one_that_was_asked_for() {
        assert_eq!(answered().model(), "mistral-small-latest");
        assert_eq!(
            answered().text(),
            "The tenant may not sublet without written consent."
        );
    }
}
