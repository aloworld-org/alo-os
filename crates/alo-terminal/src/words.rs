//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here: a key, the sentence in the language the code is written in,
//! and the note a translator needs. `alo-strings` does the rest.
//!
//! # These four words carry the product's central claim
//!
//! They are not labels in the ordinary sense. The gutter is the one place in
//! the terminal a program cannot write, and these are the only words it ever
//! shows — so a person reading `program` is being told something they are meant
//! to be able to rely on, and a person reading `alo` or `system` is being told
//! something a program is not able to say about itself.
//!
//! Two consequences for a translator, and both are in the notes:
//!
//! **They are one word each, in a narrow column.** The design gives the label
//! 80 of the gutter's 96 (`measures::THE_LABELS_ROOM`). A translation that
//! needs a phrase has nowhere to put it, so the note asks for the shortest
//! truthful word in the language rather than the most literal one.
//!
//! **They are lowercase because the design draws them lowercase** — `you`,
//! `alo`, `system`, `program`, read off the transcript node itself. In a
//! language where case carries meaning, the language wins: the lowercase here
//! is English typography, not a rule about the word.

use alo_strings::{Vocabulary, Word};

/// A program's own output.
pub const A_PROGRAM: Word = Word::saying("terminal.who.program", "program").noting(
    "The word in the terminal's gutter beside output a program produced. One word, in a column \
     80 units wide, so the shortest truthful word is better than the most literal one. It is \
     shown so a person can tell a program's own text apart from alo's, which is why it must not \
     be translated as something that could also describe alo — \"output\" or \"result\" would \
     both lose the whole point of the word.",
);

/// alo's own interpretation.
pub const ALO: Word = Word::saying("terminal.who.alo", "alo").noting(
    "The word in the terminal's gutter beside something alo itself wrote. \"alo\" is the \
     product's name and is not translated; it is lowercase everywhere it appears. The note \
     exists so that a translator does not translate it by mistake.",
);

/// An attributed observation, with its scope and the time it was made.
pub const THE_SYSTEM: Word = Word::saying("terminal.who.system", "system").noting(
    "The word in the terminal's gutter beside an observation something measured -- a port that \
     was listened on, a permission that was read -- rather than something alo worked out. One \
     word, in a column 80 units wide. It means the machine itself observed this, so a word \
     meaning \"the operating system\" is right and a word meaning \"automatic\" is not.",
);

/// What the person submitted.
pub const THE_PERSON: Word = Word::saying("terminal.who.you", "you").noting(
    "The word in the terminal's gutter beside what the person themselves typed or submitted. \
     Addressed to the reader, so the second person: it says \"you\" to the person reading it, \
     not \"the user\". Where a language distinguishes formal and familiar address, alo uses the \
     one it uses elsewhere for the person who owns the machine. One word, in a column 80 units \
     wide.",
);

/// A command parked on a read from a terminal.
pub const WAITING_FOR_THE_KEYBOARD: Word =
    Word::saying("terminal.waiting.keyboard", "waiting for the keyboard").noting(
        "Said about a command that looks stuck. It is parked reading from the terminal, so it is \
         waiting for the person to type something. Not an error and not a fault: the sentence \
         exists so that somebody does not stop a command that was only waiting for them.",
    );

/// A command parked on a read from a socket.
pub const WAITING_FOR_THE_NETWORK: Word =
    Word::saying("terminal.waiting.network", "waiting for the network").noting(
        "Said about a command that looks stuck. It is parked reading from a socket, so it is \
         waiting for something on a network or on another machine. The phrase means whatever it \
         is talking to, not this machine's own connection being down.",
    );

/// A command parked on another process, or on a lock one holds.
pub const WAITING_FOR_ANOTHER_PROCESS: Word = Word::saying(
    "terminal.waiting.another-process",
    "waiting for another program, or for a lock",
)
.noting(
    "Said about a command that looks stuck. It is waiting for a different program to finish, to \
     write something, or to let go of a lock on a file. One sentence covers all three because \
     the evidence cannot separate them, and inventing a distinction would be a guess.",
);

/// A command that is not waiting at all.
pub const STILL_COMPUTING: Word = Word::saying("terminal.waiting.computing", "still computing")
    .noting(
        "Said about a command that looks stuck and is not: the machine has it on a processor and \
         it is working. This is a measurement rather than a shrug -- the kernel says the command \
         is running -- and it is the sentence that stops somebody killing something nearly done.",
    );

/// Nothing found settles it.
pub const NOBODY_CAN_TELL: Word = Word::saying(
    "terminal.waiting.not-known",
    "waiting for something alo cannot identify",
)
.noting(
    "Said about a command that looks stuck when the evidence does not settle why. It must not be \
     translated as an error or as a failure to look: alo did look, and what it found does not \
     answer the question. A translation that reads like \"unknown error\" would say something \
     this does not.",
);

/// A command the answer said destroys something.
pub const IT_DESTROYS_SOMETHING: Word =
    Word::saying("terminal.explained.destructive", "it destroys something").noting(
        "A mark beside an explanation of a command a person has not run yet. It says the \
         explanation claimed the command destroys something — deletes, overwrites or wipes. It \
         is what the answer said and not what this machine checked, so a translation must not \
         read like a verdict the system reached: \"warning: dangerous command\" would claim far \
         more than this says.",
    );

/// A command the answer said needs rights beyond the person's own.
pub const IT_NEEDS_MORE_THAN_YOUR_RIGHTS: Word = Word::saying(
    "terminal.explained.elevated",
    "it needs more than your own rights",
)
.noting(
    "A mark beside an explanation of a command a person has not run yet. It says the explanation \
     claimed the command needs rights beyond the person's own — what other systems call \
     administrator or root. Address the person as the owner of the machine rather than as \
     somebody lacking permission: they have the rights, and the point is that this command \
     reaches for them.",
);

/// A command the answer said sends something off the machine.
pub const SOMETHING_LEAVES_THIS_MACHINE: Word = Word::saying(
    "terminal.explained.leaves",
    "something leaves this machine",
)
.noting(
    "A mark beside an explanation of a command a person has not run yet. It says the explanation \
     claimed something goes off this machine — uploaded, sent, or fetched from somewhere that \
     then knows it was asked. On a system sold on sovereignty this is the mark people look for, \
     so the translation should be plain and concrete rather than technical: \"network access\" \
     is weaker than the thing being said.",
);

/// The sentence every explanation carries, whatever it says.
pub const AN_EXPLANATION_AND_NOT_A_GUARANTEE: Word = Word::saying(
    "terminal.explained.not-a-guarantee",
    "an explanation, not a guarantee",
)
.noting(
    "Shown with every explanation of a command, always, and never on its own. It warns that the \
     explanation may be wrong about what the command will do, because a script, an alias or \
     something the command downloads can change what happens. Keep it short — it sits under the \
     explanation every single time, so a long sentence becomes noise a person stops reading, \
     which would defeat the only thing it is for.",
);

/// One string whose wording depends on a number.
///
/// Three of this crate's strings count something, and a count is not a
/// substitution: *1 test passed* and *2 tests passed* differ in English by one
/// letter and in several EU languages by more than that, so the number governs
/// the sentence rather than filling a hole in it. `alo_strings::Plural` is the
/// mechanism and this is the shape seventeen other crates already declare it
/// in.
pub struct Counted {
    /// What names it.
    named: &'static str,
    /// The gap the number goes in.
    number: &'static str,
    /// What it says about one thing.
    one: &'static str,
    /// What it says about any other number of things.
    other: &'static str,
    /// What a translator needs to know.
    note: &'static str,
}

impl Counted {
    /// What names it.
    ///
    /// `unchecked` because a key written in this file is checked **by a test**
    /// rather than at every call — `no_plural_shares_a_key` and
    /// `every_counted_key_is_a_key` in this module's tests — which is the
    /// shape `alo-finding` already uses for the same reason: a `Result` here
    /// would be an error every caller must handle about a mistake only this
    /// file can make.
    #[must_use]
    pub fn key(&self) -> alo_strings::Key {
        alo_strings::Key::unchecked(self.named)
    }

    /// What names it, as written.
    #[must_use]
    pub const fn named(&self) -> &'static str {
        self.named
    }

    /// The gap the number goes in.
    #[must_use]
    pub const fn number(&self) -> &'static str {
        self.number
    }

    /// What it says about one thing.
    #[must_use]
    pub const fn says_of_one(&self) -> &'static str {
        self.one
    }

    /// What it says about any other number of things.
    #[must_use]
    pub const fn says_of_others(&self) -> &'static str {
        self.other
    }

    /// What a translator needs to know.
    #[must_use]
    pub const fn note(&self) -> &'static str {
        self.note
    }
}

/// How many tests the output said passed.
pub const SO_MANY_PASSED: Counted = Counted {
    named: "terminal.result.passed",
    number: "passed",
    one: "{passed} passed",
    other: "{passed} passed",
    note: "Part of the summary above a test result a program printed, beside how many failed and \
           how many were ignored. {passed} is the number. **The two English forms are the same, \
           and that is deliberate**: the design draws \"Tests · 23 passed · 1 failed\" and English \
           does not inflect \"passed\", so adding a noun would put a word on the screen the design \
           does not have. A language where the number governs the sentence should differ here, \
           which is the whole reason this is a counted string rather than a plain one. It reports \
           what the output said and not what this machine checked - the sentence under it says so \
           - so a translation must not read like a verdict.",
};

/// How many tests the output said failed.
pub const SO_MANY_FAILED: Counted = Counted {
    named: "terminal.result.failed",
    number: "failed",
    one: "{failed} failed",
    other: "{failed} failed",
    note: "Part of the summary above a test result a program printed. {failed} is the number, and \
           it is often nought - \"0 failed\" must read naturally, because that is the case a person \
           sees most. The two English forms are the same for the reason given on the passed count: \
           the design draws no noun here. It reports what the output said rather than what this \
           machine found.",
};

/// How many tests the output said were ignored.
pub const SO_MANY_IGNORED: Counted = Counted {
    named: "terminal.result.ignored",
    number: "ignored",
    one: "{ignored} ignored",
    other: "{ignored} ignored",
    note: "Part of the summary above a test result a program printed, and shown only when the \
           number is not nought. An ignored test is one that was deliberately not run - skipped \
           rather than failed - and the distinction matters, so a word meaning \"failed\" or \
           \"missing\" would say something this does not. The two English forms are the same for \
           the reason given on the passed count.",
};

/// The label above a recognised test result.
pub const TESTS: Word = Word::saying("terminal.result.tests", "Tests").noting(
    "The one-word label above the summary of a test result a program printed, as the design \
     draws it. It names what the numbers beside it are about. One word if the language has one.",
);

/// What printed a recognised test result.
pub const FROM_A_CARGO_TEST_RUN: Word =
    Word::saying("terminal.result.from-cargo-test", "cargo test output").noting(
        "Names where a recognised test result was read from. \"cargo test\" is the name of a \
         program and is never translated; the word \"output\" around it is. It appears inside the \
         sentence that says the result was not independently verified.",
    );

/// What printed a recognised benchmark comparison.
pub const FROM_A_HYPERFINE_RUN: Word =
    Word::saying("terminal.benchmark.from-hyperfine", "hyperfine output").noting(
        "Names where a recognised benchmark was read from. \"hyperfine\" is the name of a program \
         somebody ran and is never translated; the word \"output\" around it is. It appears inside \
         the sentence that says the result was not independently verified, exactly as the \
         cargo-test one does.",
    );

/// The summary above a recognised benchmark comparison.
pub const LOWER_IN_THIS_COMPARISON: Word = Word::saying(
    "terminal.benchmark.lower",
    "{what} \u{b7} {percent}% lower in this comparison",
)
.noting(
    "{what} is the name the benchmarked program gave its fastest run and is never translated — \
     it is a stranger's label. {percent} is a whole number without a sign. The design's own \
     wording is \"Build time \u{b7} 25% lower in this comparison\"; \"in this comparison\" is \
     load-bearing and must survive translation, because it says the figure is about these two \
     runs and not a claim about the programs in general.",
);

/// The note under a benchmark's bars, saying how to read them.
pub const SAME_LINEAR_SCALE: Word = Word::saying(
    "terminal.benchmark.scale",
    "Same linear scale \u{b7} Lower is faster",
)
.noting(
    "Sits under the bars and says how to read them: both bars use one scale, so their lengths \
     may be compared directly, and a shorter bar is a faster run. The design's frame carries a \
     third clause, \"Inspect values when focused\", which is left out until focusing a value does \
     something — see this crate's benchmark module for why.",
);

/// What printed a recognised file diff.
pub const FROM_A_GIT_DIFF: Word = Word::saying("terminal.diff.from-git-diff", "git diff output")
    .noting(
        "Names where a recognised diff was read from. \"git diff\" is the name of a command \
         somebody ran and is never translated; the word \"output\" around it is.",
    );

/// That looking at a recognised diff applies nothing.
pub const NO_CHANGES_APPLIED_BY_THIS_VIEW: Word = Word::saying(
    "terminal.diff.nothing-applied",
    "Source: {what} \u{b7} No changes applied by this view",
)
.noting(
    "The provenance under a recognised file diff. {what} is where it was read from and arrives \
     already in the reader's language. **The second clause is the point of the sentence**: a \
     diff is the one recognised result whose subject a person might fear alo could act on, and \
     this says it does not. The design's own wording. Do not soften it into \"read only\" — \
     what it promises is about this view and not about the file.",
);

/// That a recognised result is the program's claim and nobody checked it.
pub const NOT_INDEPENDENTLY_VERIFIED: Word = Word::saying(
    "terminal.result.not-verified",
    "Source: {what} · Not independently verified",
)
.noting(
    "Shown directly under the summary of every test result a program printed, always, and never \
     on its own. {what} names where the numbers came from. It warns that alo OS read these \
     numbers out of a stranger's output and did not run the tests or check the arithmetic - the \
     design's own words for the rule are \"formatted output is not system evidence\". Keep it \
     short: it is under every such summary.",
);

/// Every string this crate can say, in the order this file declares them.
///
/// The array is what a test reads down and what [`declare_into`] walks, so a
/// word declared above and left out here is a string nothing can look up.
pub const EVERY_WORD: [Word; 21] = [
    A_PROGRAM,
    ALO,
    THE_SYSTEM,
    THE_PERSON,
    WAITING_FOR_THE_KEYBOARD,
    WAITING_FOR_THE_NETWORK,
    WAITING_FOR_ANOTHER_PROCESS,
    STILL_COMPUTING,
    NOBODY_CAN_TELL,
    IT_DESTROYS_SOMETHING,
    IT_NEEDS_MORE_THAN_YOUR_RIGHTS,
    SOMETHING_LEAVES_THIS_MACHINE,
    AN_EXPLANATION_AND_NOT_A_GUARANTEE,
    TESTS,
    FROM_A_CARGO_TEST_RUN,
    FROM_A_HYPERFINE_RUN,
    FROM_A_GIT_DIFF,
    NO_CHANGES_APPLIED_BY_THIS_VIEW,
    LOWER_IN_THIS_COMPARISON,
    SAME_LINEAR_SCALE,
    NOT_INDEPENDENTLY_VERIFIED,
];

/// How many lines a recognised diff took away.
pub const SO_MANY_REMOVALS: Counted = Counted {
    named: "terminal.diff.removals",
    number: "removals",
    one: "{removals} removal",
    other: "{removals} removals",
    note: "Part of the summary above a recognised file diff, beside how many lines were added. \
           {removals} is the number. The design draws \"src/auth.ts \u{b7} 1 removal, 1 addition\".",
};

/// How many lines a recognised diff put there.
pub const SO_MANY_ADDITIONS: Counted = Counted {
    named: "terminal.diff.additions",
    number: "additions",
    one: "{additions} addition",
    other: "{additions} additions",
    note: "Part of the summary above a recognised file diff, beside how many lines were removed. \
           {additions} is the number.",
};

/// Everything this crate can say about a number of things.
///
/// The array [`declare_into`] walks, as [`EVERY_WORD`] is for the rest: a
/// plural declared above and left out here is a string nothing can look up.
pub const EVERY_COUNTED: [Counted; 5] = [
    SO_MANY_PASSED,
    SO_MANY_FAILED,
    SO_MANY_IGNORED,
    SO_MANY_REMOVALS,
    SO_MANY_ADDITIONS,
];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads — it is read by whoever is fixing the list
/// above, so it keeps its English and its `Display`.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A countable string that is not one.
    #[error(transparent)]
    Counted(#[from] alo_strings::PluralError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn terminal_words() -> Result<Vocabulary, WordsError> {
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
    for counted in EVERY_COUNTED {
        vocabulary.counts(
            alo_strings::Plural::counting(
                counted.key(),
                counted.number,
                counted.one,
                counted.other,
            )?
            .noting(counted.note)?,
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "words_tests.rs"]
mod tests;
