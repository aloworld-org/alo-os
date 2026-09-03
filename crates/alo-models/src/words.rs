//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here: a key, the sentence in the language the code is written in,
//! and the note a translator needs. `alo-strings` does the rest.
//!
//! It is the fifth list of its kind — `alo-files` in item 9b, `alo-shortcuts`
//! in 9c, `alo-appearance` in 9d, `alo-capability` in 9e — and it is
//! deliberately the same shape as those four. What is different is what the
//! reader is about to do with what it says.
//!
//! # Two of these are read while somebody decides whether to paste a contract
//!
//! [`crate::InferenceSource`]'s four strings and the policy's three are not
//! refusals of something already typed. They are what a person reads *before*
//! they ask a question — *by Mistral, in the EU*, or *by someone, which has not
//! said where it runs* — and ADR 0008 puts them where the answer appears rather
//! than in a settings page somebody would have to go looking for. A translation
//! that softened the second one would take away the only thing on the screen
//! saying the question is about to leave the building, so its note says so.
//!
//! # And three of them are not refusals at all
//!
//! [`crate::Cost`]'s two lines and [`LICENCE_IS_YOURS`] are read by somebody
//! who has just pointed alo OS at weights of their own, and nothing was
//! stopped. `docs/features.md` promises the machine *warns and then gets out of
//! the way*, so a translation that made *larger than the memory this machine
//! has* sound like a door closing would take away the promise rather than the
//! politeness. Their notes say so, and `weights.rs` is where the code makes it
//! structurally true.
//!
//! The rest say what to do. That was true of this crate's English before it
//! moved here — `provider.rs`'s *give the provider a name — it is what you will
//! see when it answers* is the queue's own reference for the rule — and the
//! notes say so wherever a translation could lose it.
//!
//! # Nothing here counts anything out loud
//!
//! There is no [`alo_strings::Plural`] in this list, and that is item 10's
//! decision kept rather than an omission. A sentence that said *2 models* would
//! be English's two shapes standing in for Polish's three, so the numbers this
//! crate knows — how many names were left out, how many bytes a download needs,
//! how much disk is free — are **accessors beside the sentence** and not gaps
//! inside it. Whoever draws the panel counts them with
//! [`alo_strings::Strings::count`], in the reader's own language, at the moment
//! they are shown.
//!
//! # What is not here, and why that is not an oversight
//!
//! [`crate::CatalogueError`] keeps its English and its `Display`. It refuses
//! **the catalogue this repository ships**, parsed out of a file that was
//! signed with the release, so its reader is whoever is fixing that file rather
//! than whoever is using the machine. It is `alo-capability`'s `VerbError` and
//! `alo-shortcuts`' `DefaultsError` one crate on.
//!
//! [`crate::Model`], [`crate::Licence`] and the names inside them are **not
//! strings this crate says**. A publisher writes them, they arrive in the
//! catalogue as data, and translating a licence's name would be inventing one.
//! They are held to the rule a filename is held to in `alo-files`: shown as
//! they were written, never reworded.
//!
//! [`crate::CommercialUse`], [`crate::catalogue::OnCpu`] and
//! [`crate::Driving`] have no words here either, and that is the same
//! decision rather than three oversights. They are enums a catalogue panel
//! would label; nothing in this repository draws that panel yet, so there is no
//! English to move, and whoever builds it declares those words rather than
//! having them invented here. What *is* here is the sentence a person is shown
//! when a machine has no model to give the agent — because that is a refusal
//! this crate makes, and a refusal is worded by whoever makes it (item 9e).

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Lifted into `alo-strings` by item 9d. Re-exported here because this crate's
/// own files, and the tests that read this list, name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// Where an answer came from — [`crate::InferenceSource`].
//
// Read at the moment an answer appears, and read again inside every refusal
// the policy makes, which is why they are strings rather than something a
// caller assembles: a refusal and the place named inside it are in one
// language.
// ---------------------------------------------------------------------------

/// The weights are here and nothing left.
pub const ON_THIS_MACHINE: Word = Word::saying("models.source.this-machine", "on this machine");

/// A machine somebody paired with, on their own network (ADR 0003).
pub const ON_A_PAIRED_MACHINE: Word = Word::saying(
    "models.source.paired-machine",
    "on {machine}, on your network",
)
.noting(
    "{machine} is the name a person gave the other machine when they paired with it, and is never \
     translated. \"Your network\" is the person's own — the question left this machine and stayed \
     in the building.",
);

/// A provider that has said where it runs.
pub const BY_A_PROVIDER: Word = Word::saying("models.source.hosted", "by {provider}, in {region}")
    .noting(
        "{provider} is the name a person gave the provider when they added it. {region} is where \
         that provider says it runs, in the provider's own words — \"the EU\", \"Switzerland\". \
         Neither is translated.",
    );

/// A provider that has not.
pub const BY_A_PROVIDER_SOMEWHERE: Word = Word::saying(
    "models.source.hosted-unstated",
    "by {provider}, which has not said where it runs",
)
.noting(
    "This is read by somebody deciding whether to put a contract into a question. It has to sound \
     like what it is — nobody knows where this goes — rather than like a detail that has not been \
     filled in yet. {provider} is never translated.",
);

// ---------------------------------------------------------------------------
// What this machine's policy will not do — [`crate::NotAllowed`].
//
// An organisation named the rule (ADR 0004) and this is it in words. A policy
// nobody can understand is a policy people work around, so each one says what
// the machine is set to and what was asked for, rather than that something was
// not permitted.
// ---------------------------------------------------------------------------

/// A question that would leave the building.
pub const OUTSIDE_THE_BUILDING: Word = Word::saying(
    "models.policy.outside-the-building",
    "this machine is set to keep questions in the building, and {source} would send this one \
     outside it",
)
.noting(
    "{source} is where the answer would have come from and arrives already in the reader's \
     language. \"In the building\" means on this machine or on one somebody here paired with.",
);

/// A question that would leave the region an organisation named.
pub const OUTSIDE_THE_REGION: Word = Word::saying(
    "models.policy.outside-the-region",
    "this machine is set to use inference in {region} only, and {source} does not meet that",
)
.noting(
    "{region} is the region the organisation named, in their own words, and is never translated. \
     {source} arrives already in the reader's language. \"Inference\" is where a model answers.",
);

/// A question that would leave this machine at all.
pub const NOT_ON_THIS_MACHINE: Word = Word::saying(
    "models.policy.not-this-machine",
    "this machine is set to answer only on itself, and {source} is somewhere else",
)
.noting("{source} arrives already in the reader's language.");

// ---------------------------------------------------------------------------
// No model on this machine can be given the agent — [`crate::NoAgentHere`].
//
// Read at setup, by somebody who has just been told the thing they came for is
// not available on the machine in front of them. Each says which of the three
// reasons it is, and each is shown with [`THE_OTHER_PLACES`] under it — there
// is no road to one of these sentences that does not carry that one with it,
// because a refusal that named no alternative is how a silent substitution
// starts looking reasonable (ADR 0008).
// ---------------------------------------------------------------------------

/// Nothing in the catalogue is a candidate on this machine at all.
pub const NOTHING_TO_CHOOSE_FROM: Word = Word::saying(
    "models.agent.nothing-to-choose-from",
    "no model in this catalogue both runs on this machine and can be used without reading a \
     licence first",
)
.noting(
    "Two conditions in one sentence because both are true at once and the next step is the same \
     either way: either nothing fits this machine's memory, or everything that fits carries \
     licence conditions somebody has to read before an organisation relies on it. \"Runs on this \
     machine\" means with no graphics card, in the memory it has.",
);

/// Models run here, and the measurement has not been made on any of them.
pub const NONE_MEASURED: Word = Word::saying(
    "models.agent.none-measured",
    "no model that runs on this machine has been measured driving the verbs, so alo OS will not \
     give one the agent",
)
.noting(
    "This is not a verdict on those models — nobody has run the measurement on them, and saying \
     so plainly is the point of the line. \"Driving the verbs\" is a model producing the typed \
     instruction an agent turn asks it for, several times over; a model that cannot do that is \
     safe and useless. Do not soften it into \"might not work\".",
);

/// Models run here, they have been measured, and none is dependable enough.
pub const NONE_CLEARS_THE_BAR: Word = Word::saying(
    "models.agent.none-clears-the-bar",
    "the models that run on this machine do not produce a workable instruction often enough to be \
     given the agent",
)
.noting(
    "Said after a measurement, where the previous line is said when there has not been one, and a \
     translation that made the two sound alike would hide which of them happened. \"Instruction\" \
     is the typed call an agent turn asks a model for. How many run and how many were measured are \
     numbers shown beside this line rather than inside it.",
);

/// The two places that are still open, named and not chosen between.
pub const THE_OTHER_PLACES: Word = Word::saying(
    "models.agent.elsewhere",
    "you can use a model on a machine you have paired with on your network, or a provider you \
     add — whichever you prefer, and alo OS will not choose for you",
)
.noting(
    "A second line, shown under whichever of the three refusals was made and never on its own. It \
     names both alternatives and picks neither, on purpose: quietly substituting one is exactly \
     what ADR 0008 forbids. Do not drop either half, and do not reorder them into a \
     recommendation.",
);

// ---------------------------------------------------------------------------
// A provider somebody was adding — [`crate::ProviderError`].
//
// Read by a person looking at the settings panel they have just typed
// something into, so every one of them says what to do about it.
// ---------------------------------------------------------------------------

/// A provider with no name.
pub const PROVIDER_UNNAMED: Word = Word::saying(
    "models.provider.unnamed",
    "give the provider a name — it is what you will see when it answers",
)
.noting(
    "The second half is the reason the name matters: every answer says where it came from, and \
     this is the word that appears there.",
);

/// Something that is not an address.
pub const NOT_AN_ADDRESS: Word = Word::saying(
    "models.provider.not-an-address",
    "that does not look like an address: it should start with https://",
)
.noting("https:// is not translated.");

/// An address a key would travel to in clear.
pub const INSECURE_ENDPOINT: Word = Word::saying(
    "models.provider.insecure-endpoint",
    "this address is not https, so your key and your questions would travel unencrypted — use \
     https, or a service on this machine",
)
.noting(
    "\"Your key\" is the credential the provider gave them. https is not translated. The refusal \
     is firm on purpose: \"it is only our internal network\" is how an unencrypted key gets \
     shipped (ADR 0003).",
);

/// A name another provider already has.
pub const PROVIDER_ALREADY_ADDED: Word = Word::saying(
    "models.provider.already-added",
    "you already have a provider called {name}",
)
.noting(
    "{name} is what the person called the other provider, as they typed it, and is never \
             translated.",
);

// ---------------------------------------------------------------------------
// Weights somebody brought themselves — [`crate::WeightsError`],
// [`crate::Cost`], and the line about whose licence these are.
//
// Read at the moment somebody points alo OS at a model of their own. Two of
// them are refusals of something just typed and say what to do; the other three
// are not refusals at all, and that is the thing a translation here must not
// lose. `docs/features.md` promises the machine *warns and then gets out of the
// way*, so the sentence about a model too large for this machine has to read as
// a fact somebody was told rather than as a door being closed.
// ---------------------------------------------------------------------------

/// Weights with no name for the runtime to answer to.
pub const WEIGHTS_UNNAMED: Word = Word::saying(
    "models.brought.unnamed",
    "say which weights: the name the model runtime on this machine knows them by",
)
.noting(
    "Said to somebody pointing alo OS at a model they already have. The name is the one the \
     runtime answers to rather than one they invent, because it is what alo OS will ask that \
     runtime for.",
);

/// Weights already on this machine's list.
pub const WEIGHTS_ALREADY_BROUGHT: Word = Word::saying(
    "models.brought.already-brought",
    "you already have weights called {name}",
)
.noting(
    "{name} is the name the runtime knows those weights by, as it reports it, and is never \
     translated.",
);

/// They fit in the memory this machine has.
pub const WEIGHTS_FIT: Word = Word::saying(
    "models.brought.fits",
    "these weights fit in the memory this machine has",
)
.noting(
    "Shown once, where somebody adds a model of their own. What they need and what the machine \
     has are numbers shown beside this line rather than inside it.",
);

/// They are larger than the memory this machine has.
pub const WEIGHTS_LARGER_THAN_MEMORY: Word = Word::saying(
    "models.brought.larger-than-memory",
    "these weights are larger than the memory this machine has — alo OS will still run them, and \
     this machine will be slow",
)
.noting(
    "This is a warning and never a refusal, and the second half is not a softener: the model runs. \
     A translation that made it sound like alo OS had declined would take away the thing this \
     product promises about hardware somebody owns. Shown once, where they add the model. The two \
     sizes are numbers shown beside this line rather than inside it.",
);

/// Whose terms these weights come with.
pub const LICENCE_IS_YOURS: Word = Word::saying(
    "models.brought.licence-is-yours",
    "these weights are yours, and so are their terms — alo OS states the licence of what it offers \
     and has not read the licence of a model it did not offer you",
)
.noting(
    "A second line, shown under whichever of the two lines above was said and never on its own. It \
     is not a warning that something is wrong and not a judgement about the model: it says who is \
     answerable for the terms. Do not shorten it into a disclaimer.",
);

// ---------------------------------------------------------------------------
// A key as it was just typed — [`crate::SecretError`].
//
// Neither of them repeats what was typed, and neither may start: a key in a
// sentence is a key in a log.
// ---------------------------------------------------------------------------

/// Nothing typed where a key was expected.
pub const KEY_BLANK: Word = Word::saying(
    "models.key.blank",
    "paste the key this provider gave you, or leave it out if it does not need one",
)
.noting(
    "A provider that needs no key is given none at all, which is a different thing from being \
     given an empty one — so the sentence has to offer both.",
);

/// A key with something in it that cannot be sent.
pub const KEY_NOT_SENDABLE: Word = Word::saying(
    "models.key.not-sendable",
    "that key has something in it that cannot be sent — copy it again, without the line around it",
)
.noting(
    "It is almost always a line break copied along with the key out of a web page. Do not quote \
     the key: this sentence must never contain a credential.",
);

// ---------------------------------------------------------------------------
// A provider that answered — [`crate::Tried`].
//
// One line, and up to two more beside it. They are separate strings rather
// than one sentence with clauses appended, which is `alo-shortcuts`' rule from
// item 9c: the separator between them is not punctuation a program can pick,
// so the panel draws them as lines and the translator writes each one whole.
// ---------------------------------------------------------------------------

/// It answered, the key was accepted, and it offers something.
pub const THAT_WORKED: Word = Word::saying(
    "models.tried.worked",
    "that worked, and this provider says what it offers",
);

/// It answered, the key was accepted, and it offers nothing.
pub const THAT_WORKED_WITH_NOTHING: Word = Word::saying(
    "models.tried.worked-with-nothing",
    "that worked, and this provider offers no models to choose from",
)
.noting(
    "The test succeeded. Somebody who read only \"that worked\" would go looking for a list that \
     is not going to appear, which is what the second half prevents.",
);

/// More names came back than anybody reads in a dialogue.
pub const THE_LIST_WAS_CUT: Word = Word::saying(
    "models.tried.list-was-cut",
    "the list is longer than this and was cut",
)
.noting(
    "A line of its own beside the answer, not a clause inside it. A cut list that did not say so \
     reads exactly like a complete one, and somebody would conclude from it that a model is not \
     offered.",
);

/// Some of them could not be shown.
pub const SOME_NAMES_LEFT_OUT: Word = Word::saying(
    "models.tried.some-names-left-out",
    "some names could not be shown and were left out",
)
.noting(
    "A line of its own beside the answer. The provider wrote those names and some of them carried \
     something that cannot be put in a list — a line break, a control character — so they were \
     counted rather than shown.",
);

// ---------------------------------------------------------------------------
// A provider that was not tested, or was and did not work —
// [`crate::NotTried`].
//
// None of them repeats the provider's own words. An error surface that quoted
// whatever a remote service said would be a way for somebody else's text to
// arrive on a person's screen wearing ours.
// ---------------------------------------------------------------------------

/// Nothing answered at all.
pub const PROVIDER_UNREACHABLE: Word = Word::saying(
    "models.not-tried.unreachable",
    "nothing answered at that address — check the address, and that this machine is online",
);

/// The address sent this machine somewhere else.
pub const PROVIDER_REDIRECTED: Word = Word::saying(
    "models.not-tried.redirected",
    "that address sends this machine somewhere else, and a key is not carried to an address \
     nobody agreed to — use the address the provider documents",
)
.noting(
    "The middle clause is the reason rather than an apology: the address this machine's policy \
     was asked about is the address it reaches, so a redirect is refused instead of followed.",
);

/// It will not answer without a key, and none was given.
pub const PROVIDER_NEEDS_A_KEY: Word = Word::saying(
    "models.not-tried.needs-a-key",
    "this provider will not answer without a key — add the one it gave you",
)
.noting(
    "This one and the next are the two that get confused with each other, and the difference \
     matters: nobody typed a key here. Telling this person their key was rejected sends them to \
     check a key that does not exist.",
);

/// A key was given and was not accepted.
pub const KEY_NOT_ACCEPTED: Word = Word::saying(
    "models.not-tried.key-not-accepted",
    "that key was not accepted — check it is the whole key, and that it is this provider's",
)
.noting(
    "The whole reason testing a provider exists: found while somebody is still looking at the \
     field they typed it into. \"The whole key\" means none of it was left behind when it was \
     copied.",
);

/// Something answered, and it was not a provider.
pub const NOT_A_PROVIDER: Word = Word::saying(
    "models.not-tried.not-understood",
    "that address answered, but not like a provider this system can use — check it is the address \
     of the API rather than of the website",
)
.noting(
    "Saying \"bad key\" here would send somebody to check a key that is fine. \"API\" is the \
     address a provider documents for programs, as against the one people visit.",
);

/// It answered, and said it was having trouble.
pub const PROVIDER_NOT_WELL: Word = Word::saying(
    "models.not-tried.not-well",
    "the provider answered {status}, which is a problem at their end — try again in a moment",
)
.noting(
    "{status} is the number a web service answers with, like 503. It is not translated and it is \
     not a count of anything. The sentence exists to say plainly that this is not something the \
     person typed wrongly.",
);

// ---------------------------------------------------------------------------
// The model runtime — [`crate::RuntimeError`].
//
// Read while somebody is downloading gigabytes or waiting for a model to
// answer, which is the moment they are least willing to guess. None of them
// names an internal state, and none of them carries the runtime's own words.
// ---------------------------------------------------------------------------

/// The runtime is not answering.
pub const RUNTIME_UNREACHABLE: Word = Word::saying(
    "models.runtime.unreachable",
    "the model runtime is not reachable",
)
.noting(
    "The runtime is the part of alo OS that holds the models and answers questions with \
             them. It is on this machine.",
);

/// The runtime is there and has not answered a question yet.
pub const RUNTIME_TOOK_TOO_LONG: Word = Word::saying(
    "models.runtime.took-too-long",
    "the model on this machine did not answer in the time alo OS waits — a smaller model, or a \
     shorter question, answers sooner",
)
.noting(
    "Read after a long wait, and it is the one runtime line that is not about something being \
     wrong: the model is running and is slow. On a machine with no graphics card that is ordinary, \
     so a translation that made it sound like a crash would send somebody looking for a fault that \
     is not there.",
);

/// A model alo OS does not offer.
pub const MODEL_NOT_OFFERED: Word = Word::saying(
    "models.runtime.not-offered",
    "{model} is not a model this system offers",
)
.noting(
    "{model} is the name that was asked for and is never translated. This is a refusal rather \
     than a failure: alo OS offers a curated list, every entry with its licence stated.",
);

/// A model that is not on this machine.
pub const MODEL_NOT_INSTALLED: Word =
    Word::saying("models.runtime.not-installed", "{model} is not installed").noting(
        "\"Installed\" means the weights are on this machine's disk, taking up room — as \
             against loaded, which means it is in memory and answering quickly.",
    );

/// Not enough disk for the download.
pub const NOT_ENOUGH_DISK: Word = Word::saying(
    "models.runtime.not-enough-disk",
    "there is not enough room on this disk for that download",
)
.noting(
    "How much it needed and how much is free are numbers, shown beside this line rather than \
     inside it — see this file's note about counting.",
);

/// The runtime answered with something unusable.
pub const RUNTIME_UNUSABLE: Word = Word::saying(
    "models.runtime.unusable",
    "the model runtime gave an answer that could not be used",
);

/// A download that stopped before it finished.
pub const DOWNLOAD_INCOMPLETE: Word = Word::saying(
    "models.runtime.download-incomplete",
    "the download stopped before it finished, and nothing was installed",
)
.noting(
    "The second half is what the person actually wants to know: no disk was spent and there is no \
     half-model to clean up.",
);

/// Every string this crate can say, in the order this file declares them.
///
/// The array is what a test reads down and what [`declare_into`] walks, so a
/// word declared above and left out here is a string nothing can look up.
pub const EVERY_WORD: [Word; 39] = [
    ON_THIS_MACHINE,
    ON_A_PAIRED_MACHINE,
    BY_A_PROVIDER,
    BY_A_PROVIDER_SOMEWHERE,
    OUTSIDE_THE_BUILDING,
    OUTSIDE_THE_REGION,
    NOT_ON_THIS_MACHINE,
    NOTHING_TO_CHOOSE_FROM,
    NONE_MEASURED,
    NONE_CLEARS_THE_BAR,
    THE_OTHER_PLACES,
    PROVIDER_UNNAMED,
    NOT_AN_ADDRESS,
    INSECURE_ENDPOINT,
    PROVIDER_ALREADY_ADDED,
    WEIGHTS_UNNAMED,
    WEIGHTS_ALREADY_BROUGHT,
    WEIGHTS_FIT,
    WEIGHTS_LARGER_THAN_MEMORY,
    LICENCE_IS_YOURS,
    KEY_BLANK,
    KEY_NOT_SENDABLE,
    THAT_WORKED,
    THAT_WORKED_WITH_NOTHING,
    THE_LIST_WAS_CUT,
    SOME_NAMES_LEFT_OUT,
    PROVIDER_UNREACHABLE,
    PROVIDER_REDIRECTED,
    PROVIDER_NEEDS_A_KEY,
    KEY_NOT_ACCEPTED,
    NOT_A_PROVIDER,
    PROVIDER_NOT_WELL,
    RUNTIME_UNREACHABLE,
    RUNTIME_TOOK_TOO_LONG,
    MODEL_NOT_OFFERED,
    MODEL_NOT_INSTALLED,
    NOT_ENOUGH_DISK,
    RUNTIME_UNUSABLE,
    DOWNLOAD_INCOMPLETE,
];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads — it keeps its English and its `Display` for
/// the reason [`crate::CatalogueError`] does. It exists because
/// [`declare_into`] can genuinely fail against a vocabulary that already holds
/// one of these keys.
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
pub fn model_words() -> Result<Vocabulary, WordsError> {
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
    /// arrive from anywhere; this is the test that makes that true, and it is
    /// the same shape as `alo-shortcuts` putting every shipped binding back
    /// through `Chord::checked`.
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
            assert_eq!(word.key().area(), "models", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it — which is the whole of what this file has to get right.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = model_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0);
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = model_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// The sentences carrying something off the machine — a provider's name, a
    /// region somebody typed, a status code, a model id — say so, because a
    /// translator with no product in front of them cannot tell a gap that is a
    /// word from a gap that is a value.
    #[test]
    fn the_ones_a_translator_cannot_work_out_carry_a_note() {
        for word in [
            ON_A_PAIRED_MACHINE,
            BY_A_PROVIDER,
            BY_A_PROVIDER_SOMEWHERE,
            OUTSIDE_THE_BUILDING,
            OUTSIDE_THE_REGION,
            NOT_ON_THIS_MACHINE,
            PROVIDER_ALREADY_ADDED,
            WEIGHTS_ALREADY_BROUGHT,
            PROVIDER_NOT_WELL,
            MODEL_NOT_OFFERED,
        ] {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **No sentence here counts anything.** Item 10 settled it in this crate
    /// before `alo-strings` existed, and the plural rules are not written from
    /// memory: a gap named for a quantity would be English's two shapes
    /// standing in for Polish's three.
    #[test]
    fn nothing_this_crate_says_counts_something() {
        let counting = model_words().unwrap();
        assert_eq!(counting.counted().count(), 0);
        for word in EVERY_WORD {
            for gap in ["count", "how-many", "how_many", "number", "models"] {
                assert!(
                    !word.says().contains(&format!("{{{gap}}}")),
                    "{}: {gap}",
                    word.named()
                );
            }
        }
    }
}
