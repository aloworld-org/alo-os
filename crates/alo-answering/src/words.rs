//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here: a key, the sentence in the language the code is written in,
//! and the note a translator needs. `alo-strings` does the rest.
//!
//! # Every one of them is read at a bad moment
//!
//! Not one of these sentences is read by somebody who was expecting it.
//! Somebody asked a question, and instead of an answer they are looking at this
//! — which makes them the sentences most likely to be skimmed, and the ones
//! where a translation that softened anything would do real harm. Two rules
//! follow, and both are in the notes:
//!
//! **The failure lines say what did not happen, never what to try.** Suggesting
//! a provider in the sentence that reports a failure is the silent fallback
//! wearing a helpful tone: the place to say *this could be asked instead* is an
//! [offer](crate::Offer), which is a thing a person answers rather than a thing
//! they read.
//!
//! **The offers say where the question would go, in the same sentence as the
//! offer itself.** There are three of them where a lesser design would have one
//! with a clause bolted on, because *nothing leaves*, *it leaves this machine
//! and stays on your network* and *it leaves the building* are three different
//! facts about somebody's records, and ADR 0008 is emphatic that they must not
//! look alike. That is `alo-egress`' rule from item 9h — one whole sentence per
//! reason, the preposition inside it where a translator can move it — met from
//! a fourth side.
//!
//! # Nothing here counts, and nothing here quotes
//!
//! There is no [`alo_strings::Plural`], for the reason `alo-models` gives: a
//! sentence that said *2 providers* would be English's two shapes standing in
//! for Polish's three. There is no gap holding text anybody outside this
//! repository wrote either — not the question, not a model's name, not what a
//! provider said about itself. `{source}` is a clause `alo-models` words, and
//! `{status}` is a number. That is what makes every sentence here one alo OS
//! wrote rather than one it passed on.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Re-exported because this crate's own files, and the tests that read this
/// list, name it as `crate::words::Word`.
pub use alo_strings::Word;

/// What every sentence here says about the gap it shares.
///
/// Written once rather than in six notes, because a translator reading the
/// sixth would otherwise be reading the same paragraph for the sixth time.
const THE_PLACE: &str = "{source} is where the question was to be answered and arrives already in \
                         the reader's language — \"on this machine\", \"on the studio \
                         workstation, on your network\", \"by alo, in the EU\". It is a clause \
                         rather than a name, so the sentence has to read with it in place.";

// ---------------------------------------------------------------------------
// Why the question was not answered — [`crate::WentWrong`].
//
// Each one names the place, because *the model did not answer* is a different
// thing to be told depending on whether the model was on this machine or
// behind somebody's API — and the second is the one where a person is about to
// wonder whether their question went anywhere.
// ---------------------------------------------------------------------------

/// Nothing was listening, or nothing was running.
pub const NOTHING_ANSWERED: Word = Word::saying(
    "answering.wrong.nothing-answered",
    "nothing answered {source}",
)
.noting(THE_PLACE);

/// Something is there, and it did not answer in time.
pub const TOOK_TOO_LONG: Word = Word::saying(
    "answering.wrong.took-too-long",
    "nothing answered {source} within the time this machine waits",
)
.noting(THE_PLACE);

/// It answered with something that was not an answer.
pub const NOTHING_USABLE: Word = Word::saying(
    "answering.wrong.nothing-usable",
    "something answered {source}, but not with anything this machine could use",
)
.noting(THE_PLACE);

/// The model itself was not there to answer.
pub const NO_MODEL_THERE: Word = Word::saying(
    "answering.wrong.no-model-there",
    "the model this question needed was not there to answer {source}",
)
.noting(
    "The model is deliberately not named: this crate holds no text anybody outside alo OS wrote. \
     Which model a person chose is in their settings, beside this.",
);

/// The key was refused, which only a hosted provider can do.
pub const KEY_NOT_ACCEPTED: Word = Word::saying(
    "answering.wrong.key-not-accepted",
    "the key for this provider was not accepted, so nothing was answered {source}",
)
.noting(
    "Only a provider somebody added is given a key, so this sentence never names this machine or \
     one on the person's own network. Do not quote the key: this sentence must never contain a \
     credential.",
);

/// The address answered by pointing somewhere else, and nothing was carried
/// there.
pub const SENT_SOMEWHERE_ELSE: Word = Word::saying(
    "answering.wrong.sent-somewhere-else",
    "nothing was answered {source} — that address sends this machine somewhere nobody agreed to, \
     and the question was not carried there",
)
.noting(
    "The only sentence in this list where alo OS is the one that stopped something, and the last \
     clause is why it is worth reading: the question did not go anywhere. Do not soften it into \
     \"the address is wrong\" — the address may be exactly what the provider documents, and what \
     happened is that it answered by pointing elsewhere. {source} is where the question was to be \
     answered and arrives already in the reader's language.",
);

/// It answered, and what it answered was that it was in trouble.
pub const HAVING_TROUBLE: Word = Word::saying(
    "answering.wrong.having-trouble",
    "nothing was answered {source} — it answered {status}, which is a problem at that end rather \
     than yours",
)
.noting(
    "{status} is the number a web service answers with, like 503. It is not translated and it is \
     not a count of anything. The last clause is the point of the sentence: this is not something \
     the person typed wrongly, and there is nothing for them to fix.",
);

// ---------------------------------------------------------------------------
// The line that is always shown — [`crate::Failed::nothing_was_sent`].
//
// This is the whole promise in one sentence, and it is shown whether or not
// there is anywhere else to ask. ADR 0008 rejects falling back outright; a
// person who has just watched a question fail has no way of knowing that
// unless somebody says it.
// ---------------------------------------------------------------------------

/// Nothing happened instead, and nothing will without an answer from a person.
pub const NOTHING_WAS_SENT: Word = Word::saying(
    "answering.nothing-was-sent",
    "nothing was sent anywhere, and nothing will be unless you say so",
)
.noting(
    "The promise alo OS makes about a failed question: it is never quietly asked somewhere else. \
     Both halves matter — nothing has happened yet, and nothing will happen without this person. \
     A translation that shortened it to \"nothing was sent\" would drop the half that is about \
     the future.",
);

// ---------------------------------------------------------------------------
// What could be asked instead — [`crate::Offer`].
//
// The sentence a person answers. Three of them, one per kind of place, because
// where the question would go is the thing being approved and not a detail
// beside it.
// ---------------------------------------------------------------------------

/// Somewhere on this machine. Nothing leaves.
pub const ASK_HERE_INSTEAD: Word = Word::saying(
    "answering.offer.here",
    "have this question answered {source} instead, just this once — it would not leave this \
     machine",
)
.noting(
    "\"Just this once\" is exact and must survive translation: approving this is worth one \
     question and never a setting. The last clause is the good news in this one — this is the \
     offer where nothing leaves.",
);

/// A machine on the person's own network. It leaves the machine, not the
/// building.
pub const ASK_IN_THE_BUILDING_INSTEAD: Word = Word::saying(
    "answering.offer.in-the-building",
    "have this question answered {source} instead, just this once — the question would leave this \
     machine and stay on your network",
)
.noting(
    "\"Just this once\" is exact: approving this is worth one question and never a setting. The \
     second half is not reassurance to be trimmed — the question does leave this machine, and a \
     person is entitled to know that before they say yes, even though it stays in the building.",
);

/// A hosted provider. It leaves the building.
pub const ASK_OUTSIDE_INSTEAD: Word = Word::saying(
    "answering.offer.outside",
    "have this question answered {source} instead, just this once — the question would leave this \
     machine and the building",
)
.noting(
    "The largest egress alo OS ever causes, and the sentence somebody approves it with. \"Just \
     this once\" is exact. \"Leave the building\" means it goes to somebody else's computers; it \
     must not be softened into anything that sounds like a technical detail, because what is \
     leaving is the person's own work.",
);

// ---------------------------------------------------------------------------
// When there is nowhere to offer — [`crate::Elsewhere`].
//
// A machine whose organisation has closed the other doors says so in the
// policy's own words (`alo_models::NotAllowed`), which is why there is only
// one string here: the case where there was never another door.
// ---------------------------------------------------------------------------

/// This machine has no other source set up at all.
pub const NOWHERE_ELSE: Word = Word::saying(
    "answering.nowhere-else",
    "there is nowhere else set up to answer this question",
)
.noting(
    "Not a refusal: nobody forbade anything, there simply is no second place. A machine whose \
     rule forbids the other places says so in that rule's own words instead, which are \
     `alo-models`'.",
);

// ---------------------------------------------------------------------------
// An offer that was not this failure's — [`crate::NotOffered`].
// ---------------------------------------------------------------------------

/// The offer came from somewhere else, and nothing was done with it.
pub const NOT_ON_OFFER: Word = Word::saying(
    "answering.not-on-offer",
    "that was offered for a different question, so nothing was sent — ask again",
)
.noting(
    "Read by somebody who answered an offer from a dialogue that had been on screen a while. The \
     middle clause is the one that matters to them: their question did not go anywhere.",
);

/// Every string this crate can say, in the order this file declares them.
///
/// The array is what a test reads down and what [`declare_into`] walks, so a
/// word declared above and left out here is a string nothing can look up.
pub const EVERY_WORD: [Word; 13] = [
    NOTHING_ANSWERED,
    TOOK_TOO_LONG,
    NOTHING_USABLE,
    NO_MODEL_THERE,
    KEY_NOT_ACCEPTED,
    SENT_SOMEWHERE_ELSE,
    HAVING_TROUBLE,
    NOTHING_WAS_SENT,
    ASK_HERE_INSTEAD,
    ASK_IN_THE_BUILDING_INSTEAD,
    ASK_OUTSIDE_INSTEAD,
    NOWHERE_ELSE,
    NOT_ON_OFFER,
];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads — it is read by whoever is fixing the list
/// above, so it keeps its English and its `Display` for the reason
/// `alo_models::CatalogueError` does.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn answering_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// The shell has one vocabulary and every crate adds its own to it, which is
/// what the area at the front of a key is for.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_strings::Key;
    use std::collections::BTreeSet;

    /// **What we ship is held to the rule everybody else is held to.**
    /// [`Word::key`] does not check, because a key written in this file cannot
    /// arrive from anywhere; this is the test that makes that true.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                Key::named(word.named()),
                Ok(word.key()),
                "{}: {}",
                word.named(),
                Key::named(word.named()).unwrap_err()
            );
        }
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "answering", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = answering_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0);
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = answering_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every one of them carries a note**, which is true of no other list in
    /// this repository. There is a reason rather than a flourish: every
    /// sentence here is read at the moment a question failed, half of them
    /// carry a clause somebody else's crate worded, and three of them are
    /// approvals — so there is no sentence in this file a translator can
    /// safely work out from its own words.
    #[test]
    fn every_one_of_them_carries_a_note() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Nothing this crate says counts anything.** A gap named for a quantity
    /// would be English's two shapes standing in for Polish's three, and the
    /// plural rules are not written from memory.
    #[test]
    fn nothing_this_crate_says_counts_something() {
        let counting = answering_words().unwrap();
        assert_eq!(counting.counted().count(), 0);
        for word in EVERY_WORD {
            for gap in ["count", "how-many", "how_many", "number", "providers"] {
                assert!(
                    !word.says().contains(&format!("{{{gap}}}")),
                    "{}: {gap}",
                    word.named()
                );
            }
        }
    }

    /// **The three offers each say where the question goes.** An offer that
    /// named a place without saying what leaving means would be an approval
    /// somebody gave without being told the thing that matters most about it.
    #[test]
    fn every_offer_says_what_would_leave_and_says_it_is_once() {
        for (word, expected) in [
            (ASK_HERE_INSTEAD, "would not leave this machine"),
            (
                ASK_IN_THE_BUILDING_INSTEAD,
                "leave this machine and stay on your network",
            ),
            (ASK_OUTSIDE_INSTEAD, "leave this machine and the building"),
        ] {
            assert!(word.says().contains("just this once"), "{}", word.named());
            assert!(word.says().contains(expected), "{}", word.named());
            assert!(word.says().contains("{source}"), "{}", word.named());
        }
    }

    /// **A failure line never suggests where to go instead.** Somewhere else is
    /// a thing a person is asked, not a thing they are told while reading what
    /// went wrong — a sentence that recommended a provider would be the silent
    /// fallback with a polite voice.
    #[test]
    fn no_failure_line_recommends_asking_somewhere_else() {
        for word in [
            NOTHING_ANSWERED,
            TOOK_TOO_LONG,
            NOTHING_USABLE,
            NO_MODEL_THERE,
            KEY_NOT_ACCEPTED,
            SENT_SOMEWHERE_ELSE,
            HAVING_TROUBLE,
        ] {
            for suggesting in ["instead", "try ", "another"] {
                assert!(
                    !word.says().contains(suggesting),
                    "{}: {suggesting}",
                    word.named()
                );
            }
        }
    }
}
