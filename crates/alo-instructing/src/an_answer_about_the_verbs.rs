//! **An answer that talks about the verbs is not an answer for the person.**
//!
//! A model is told *these are the only verbs there are*, and when none of them
//! fits it says so — in the words it was given. Walked on a booted machine on
//! 2026-11-08 (image `02bd280e`, provider OpenAI): *make a folder called demo in
//! my Documents* came back to the person as *"Folders cannot be created
//! directly with the given verbs. Please create the folder manually in your
//! Documents."* The person never saw a verb. That sentence is this text leaking
//! through a model, and it reached them as alo's reply.
//!
//! [ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md):
//! *"There is no verb for that" is never an answer a person sees.*
//!
//! # What this decides, and how narrowly
//!
//! [`speaks_of_the_verbs`] answers one question about an answer the model meant
//! for the person: does it talk about this text's own machinery — the word
//! *verb*, or the name of a verb on this machine's list, like `create_folder` —
//! **where the person's request did not**. The second half is what keeps it
//! honest: somebody asking *what is a verb?*, or asking about `move_file` by
//! name, is answered in their own terms, and the answer is theirs.
//!
//! It is a test of words, not of meaning, and it is the narrowest one that
//! catches what was walked. The word *verb* means nothing to a person who did
//! not bring it, so an answer that uses it unasked is talking to the
//! instructions rather than to them; and a verb's name is an identifier this
//! text taught the model. Nothing else is looked for.
//!
//! # What becomes of such an answer
//!
//! It is not shown. The loop (`alo-agent`) tells the model why, in
//! [`NOT_FOR_THE_PERSON`], and the turn goes on — the same road a line that
//! tried to be a request and failed already takes. A model that keeps doing it
//! runs out of rounds, and the person is told the turn got nowhere, which is
//! true, rather than what the model said about verbs, which they cannot use.
//!
//! **No grade is about [`NOT_FOR_THE_PERSON`]**, for `WHAT_TO_SEND_NEXT`'s
//! reason: it is shown only once a turn has taken a step, and every grade is
//! earned on a first round.

use alo_capability::Verbs;

/// What a model is shown when an answer it meant for the person talked about
/// the verbs.
///
/// English, like every line a model reads (this crate's header says why), and
/// written as an instruction rather than a complaint, because the model has
/// rounds left to use.
pub const NOT_FOR_THE_PERSON: &str = "\
not shown to the person: that answer talks about verbs, which the person never sees. \
Do what the request asks with a read or a propose line, or a run line where no verb fits, or \
answer in the person's own terms without mentioning verbs.";

/// The word this text uses for what a model may ask for, in the forms a model
/// writes it.
const THE_WORD: [&str; 2] = ["verb", "verbs"];

/// **Whether `answer` talks about the verbs where `request` did not.**
///
/// True when the answer uses the word *verb* or *verbs* on its own, in any case,
/// or names a verb `verbs` declares — and the request contains none of what
/// was found. Whole words only: *verbose* and *proverbs* are not about verbs.
#[must_use]
pub fn speaks_of_the_verbs(answer: &str, request: &str, verbs: &Verbs) -> bool {
    let said = words_in(answer);
    let asked = words_in(request);
    let unasked =
        |word: &str| said.iter().any(|it| it == word) && !asked.iter().any(|it| it == word);
    THE_WORD.iter().any(|word| unasked(word)) || verbs.all().any(|verb| unasked(verb.name()))
}

/// Every whole word in `text`, lower-cased, where a word is a run of letters,
/// digits and underscores — so a verb's name, `create_folder`, is one word.
fn words_in(text: &str) -> Vec<String> {
    text.split(|it: char| !(it.is_alphanumeric() || it == '_'))
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::the_verbs;

    /// **What was walked is caught.** The answer the person read on
    /// 2026-11-08, word for word.
    #[test]
    fn the_answer_the_walk_found_is_not_for_the_person() {
        assert!(speaks_of_the_verbs(
            "Folders cannot be created directly with the given verbs. Please create the folder \
             manually in your Documents.",
            "make a folder called demo in my Documents",
            &the_verbs(),
        ));
    }

    /// A verb named by its identifier is the same leak in another form.
    #[test]
    fn a_verbs_name_the_person_never_said_is_not_for_them() {
        let verbs = the_verbs();
        let name = verbs.all().next().map(|verb| verb.name().to_owned());
        let name = name.unwrap_or_default();
        assert!(!name.is_empty(), "the fixture declares no verb");
        assert!(speaks_of_the_verbs(
            &format!("I can only use {name} here."),
            "tidy my invoices",
            &verbs,
        ));
    }

    /// **The refusal half: an ordinary answer is the person's.** Nothing about
    /// verbs, and nothing is taken from them.
    #[test]
    fn an_answer_that_does_not_mention_them_reaches_the_person() {
        assert!(!speaks_of_the_verbs(
            "The capital of France is Paris.",
            "what is the capital of France",
            &the_verbs(),
        ));
    }

    /// **A person who asked about verbs is answered about verbs.** The word
    /// came from them, so the answer is in their own terms.
    #[test]
    fn a_person_who_asked_about_verbs_is_answered() {
        assert!(!speaks_of_the_verbs(
            "A verb is a word for an action, like run or write.",
            "what is a verb?",
            &the_verbs(),
        ));
        assert!(!speaks_of_the_verbs(
            "Verbs in German go second in a main clause.",
            "where do verbs go in German?",
            &the_verbs(),
        ));
    }

    /// Whole words only: a word that contains *verb* is not about verbs.
    #[test]
    fn a_word_that_only_contains_it_is_not_it() {
        assert!(!speaks_of_the_verbs(
            "That proverb is verbose and reverberates.",
            "tell me a proverb",
            &the_verbs(),
        ));
    }

    /// Case does not hide it.
    #[test]
    fn case_does_not_hide_it() {
        assert!(speaks_of_the_verbs(
            "No VERB can do that.",
            "rename my photos",
            &the_verbs(),
        ));
    }
}
