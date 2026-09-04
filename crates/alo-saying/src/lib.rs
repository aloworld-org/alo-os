//! Everything this machine can say, and the translations it says it in.
//!
//! `alo-strings` is the machinery — a key, the English beside it, a translation
//! checked against what the code really says, and the lookup that answers in
//! the reader's own language. It says of itself that **it does not read a
//! file**: where the translations live and who loads them is the shell's, and
//! the shell did not exist. Fifteen crates have declared their words onto it
//! since, and a `Strings` has still never been built anywhere but in a test.
//!
//! This crate is *who loads them*. It does two things and neither of them is a
//! decision about words:
//!
//! | | |
//! |---|---|
//! | [`everything_this_machine_can_say`] | Every crate's list, in one vocabulary |
//! | [`Loaded`] | The translations on a disk, put onto it |
//! | [`Damage`] | Everything that was meant to load and did not |
//! | [`NotSpoken`], [`LeftOut`] | A file that gave nothing, and a line left out of one that gave something |
//! | [`THE_TRANSLATIONS`] | Where a machine keeps them |
//! | [`what_a_person_would_have_to_learn`] | Every rented name in what a vocabulary says |
//!
//! ```
//! use alo_saying::{Loaded, everything_this_machine_can_say, the_translations};
//! use alo_strings::Language;
//!
//! // Everything alo OS says, plus whatever this process says on top of it.
//! let vocabulary = everything_this_machine_can_say()?;
//!
//! // Every translation the image shipped, and what did not load.
//! let loaded = Loaded::at(vocabulary, the_translations());
//! for line in loaded.damage().lines() {
//!     // Whoever is standing the machine up reads this, in a service log.
//!     assert!(!line.is_empty());
//! }
//!
//! // Whose machine it is decides which of them is preferred.
//! let mut strings = loaded.into_strings();
//! strings.prefers(&[Language::written("de")?]);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # One vocabulary for the machine, not one per process
//!
//! This is the decision the crate exists to make, and it is the one that
//! decides its shape — including the dependency on every crate in the workspace
//! that has a word in it, which is otherwise hard to justify.
//!
//! A translation is **checked against the vocabulary it is loaded into**, and a
//! key nothing declares is a mistake in the translator's file. So a process
//! that declared only the strings it says would read a translator's correct
//! line for another part of the system as an error — the shell reading the
//! daemon's refusals as wrong, the daemon reading the shortcuts panel's rows as
//! wrong — and each would be able to show its own half of one file as though it
//! were the whole. There would be no answer to *how much of Maltese is done*,
//! because there would be as many answers as there are processes.
//!
//! So the vocabulary is the machine's. The price is that this crate reaches
//! fifteen crates and inherits their dependencies, `alo-asking`'s TLS stack
//! included. That is the cheaper of the two mistakes: a shell that links a
//! little more than it uses, against a translation that means something
//! different depending on which program is reading it.
//!
//! # Nothing about a translation stops the machine
//!
//! [`Loaded::at`] has no error in it. Six things can go wrong and every one of
//! them travels in [`Damage`], because the alternative is a machine that will
//! not start and cannot say why: the sentence explaining it is in the file that
//! did not load. A machine with no translations, or with a broken one, speaks
//! English and says so — and English being visible as English is
//! `alo-strings`' own guarantee, carried by every `Said`.
//!
//! The one refusal here is [`everything_this_machine_can_say`], which fails
//! only when alo OS's own words contradict each other. That is our bug, not a
//! machine's, and `crate::collecting`'s test is where it fails.
//!
//! # A line is left out; a language is never thrown away
//!
//! `alo_strings::Vocabulary::check` refuses a whole file when anything in it
//! would come out wrong, which is right when somebody contributes one and wrong
//! when a machine loads one — a single string renamed in a release would
//! otherwise turn a person's language off on every machine at once. So the same
//! check is asked here and acted on differently: the lines that would come out
//! wrong are left out, the rest of the language is shown, and what was left out
//! is reported. [`loading`] is the argument in full.
//!
//! # What this crate is deliberately not
//!
//! **It is not a settings store.** Every translation found is loaded and
//! `alo_strings::Strings::prefers` chooses between them; which language a
//! person reads is their setting, and where that is kept is not decided yet.
//!
//! **It does not check who wrote a translation.** The sentence a person
//! approves is a string in the vocabulary (item 9g), so whoever can write these
//! files can change what somebody is agreeing to — and what answers that is the
//! image rather than a mode check. [`place`] is the whole of that argument, and
//! it is the reason a directory a person can write is a different question with
//! a different answer.
//!
//! **It says nothing itself.** There is no `words` module here and there never
//! will be: this is the crate that runs when the vocabulary has not loaded, so
//! a sentence of its own would be a key on somebody's screen. [`failing`] is
//! why its English is not the bug `CLAUDE.md` forbids.
//!
//! **It is not a capability.** Nothing here is reachable by an agent and
//! nothing here needs a grant, for the reason `alo-strings` gives: a person
//! reading their own machine in their own language is not an agent doing
//! something.
//!
//! # No rented name reaches a person
//!
//! Holding the whole list is what makes one rule checkable that was previously
//! only a habit: alo OS runs on things it did not write, and none of them is
//! something the person who bought the machine chose. *The Flatpak could not be
//! installed* asks somebody to learn what a Flatpak is before they can
//! understand why their application is not there.
//!
//! [`what_a_person_would_have_to_learn`] walks a vocabulary against
//! [`EVERYTHING_WE_RENT`] and answers with every place a rented name is said.
//! It answers with nothing today, which is the point: it costs nothing now and
//! catches the first one later. [`rented`] is what is on the list, what is
//! deliberately not, and why a note and a key are read as well as a sentence.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod arriving;
pub mod collecting;
pub mod damage;
pub mod failing;
pub mod loading;
pub mod place;
pub mod rented;

#[cfg(test)]
mod testing;

pub use arriving::THE_FORMAT;
pub use collecting::{EVERY_LIST, NotCollected, everything_this_machine_can_say};
pub use damage::Damage;
pub use failing::{LeftOut, NotSpoken};
pub use loading::Loaded;
pub use place::{THE_TRANSLATIONS, is_a_translation, the_translations};
pub use rented::{EVERYTHING_WE_RENT, Overheard, Rented, Where, what_a_person_would_have_to_learn};
