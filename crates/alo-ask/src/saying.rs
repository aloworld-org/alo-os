//! Everything this command can say, gathered from the four crates that say it.
//!
//! This command declares **no words of its own**, and the vocabulary it renders
//! from is the argument for that: every sentence a person reads here already
//! belongs to somebody, and the only thing this file does is put the four lists
//! in one place so they can be looked up in the person's own language.
//!
//! | What a person reads | Whose string it is |
//! |---|---|
//! | *there is nothing to ask yet — write the question first* | `alo-asking` |
//! | *choose a model for this question to be answered by* | `alo-asking` |
//! | *the model runtime is not reachable* | `alo-models` |
//! | *{model} is not installed* | `alo-models` |
//! | *on this machine*, beside the answer | `alo-models` |
//! | *nothing answered on this machine* | `alo-answering` |
//! | *nothing was sent anywhere, and nothing will be unless you say so* | `alo-answering` |
//! | *this session has no folder of its own* | `alo-choosing` |
//!
//! # Four lists, not the machine's whole vocabulary
//!
//! `alo-saying` is where every crate's words are collected, and this does not
//! use it: that vocabulary is the shell's, and a command that linked it would
//! link most of this workspace to print eight sentences. The cost of asking for
//! four lists by name is that a fifth crate's word used here would look up as a
//! marked key rather than as a sentence — which is visible the first time it
//! happens, and is what `alo_strings::Strings` answers with by design instead of
//! failing.
//!
//! # A list that would not go in is not a sentence this command shows
//!
//! [`NoVocabulary`] keeps its English and its `Display`, for the reason
//! `alo_models::CatalogueError` does: its reader is whoever is fixing a word
//! list, never somebody who typed a question. It happens when two crates have
//! declared the same key, and there is no honest way to print a sentence about
//! a vocabulary that could not be built — the sentence would have had to come
//! out of it.

use alo_strings::Vocabulary;

/// Why the sentences this command reads out could not be gathered.
///
/// One variant per list, rather than one variant with a name in it, because the
/// thing to do about it is in the crate that refused — and `#[from]` on four
/// distinct types is what makes the `?` in [`what_it_can_say`] say which.
#[derive(Debug, thiserror::Error)]
pub enum NoVocabulary {
    /// `alo-asking`'s list, which is what is said when there is nothing to ask
    /// with yet.
    #[error("alo-asking's words could not be declared: {0}")]
    Asking(#[from] alo_asking::WordsError),

    /// `alo-answering`'s, which is what is said when nothing answered.
    #[error("alo-answering's words could not be declared: {0}")]
    Answering(#[from] alo_answering::WordsError),

    /// `alo-models`', which is the runtime's refusals and *on this machine*.
    #[error("alo-models' words could not be declared: {0}")]
    Models(#[from] alo_models::WordsError),

    /// `alo-choosing`'s, which is what is said about a session with nowhere of
    /// its own.
    #[error("alo-choosing's words could not be declared: {0}")]
    Choosing(#[from] alo_choosing::WordsError),
}

/// The four lists this command renders from, in one vocabulary.
///
/// # Errors
///
/// [`NoVocabulary`] if a list could not be declared into it. None of the four
/// can cause that on its own — each is tested against its own keys in its own
/// crate — so what this catches is two of them having grown the same key, which
/// is a thing only a reader of more than one list can find.
pub fn what_it_can_say() -> Result<Vocabulary, NoVocabulary> {
    let mut vocabulary = Vocabulary::empty();
    alo_asking::declare_into(&mut vocabulary)?;
    alo_answering::declare_into(&mut vocabulary)?;
    alo_models::declare_into(&mut vocabulary)?;
    alo_choosing::declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on a value that could not be built is the failure being reported"
)]
mod tests {
    use super::*;

    /// **The four lists go in together.** Each crate proves its own keys are
    /// distinct; nothing but a reader of all four can prove they are distinct
    /// *between* them, and a collision would make one crate's sentence
    /// unreachable from here rather than fail anything.
    #[test]
    fn the_four_lists_have_no_key_in_common() {
        let gathered = what_it_can_say().expect("four lists that agree on no key");
        let each = alo_asking::EVERY_WORD.len()
            + alo_answering::EVERY_WORD.len()
            + alo_models::words::EVERY_WORD.len()
            + alo_choosing::EVERY_WORD.len();
        assert_eq!(
            gathered.how_many(),
            each,
            "the vocabulary holds fewer strings than the four crates declare, so a key is shared"
        );
    }

    /// The sentences this command actually reaches for are in it — one from each
    /// list, so a crate dropped from [`what_it_can_say`] fails here rather than
    /// printing a marked key on a real machine.
    #[test]
    fn one_sentence_from_each_of_the_four_is_there() {
        let gathered = what_it_can_say().expect("four lists");
        for word in [
            alo_asking::words::NOTHING_TO_ASK,
            alo_asking::words::NO_MODEL_NAMED,
            alo_answering::words::NOTHING_WAS_SENT,
            alo_models::words::RUNTIME_UNREACHABLE,
            alo_models::words::ON_THIS_MACHINE,
            alo_choosing::SESSION_NO_FOLDER,
        ] {
            assert!(
                gathered.phrase(&word.key()).is_some(),
                "{} is not in the vocabulary this command renders from",
                word.key().as_str()
            );
        }
    }
}
