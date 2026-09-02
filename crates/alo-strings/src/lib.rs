//! Every string a person reads, named rather than written in place.
//!
//! `CLAUDE.md` says hardcoded English is a bug, and *"English plus the big
//! five" is the same bug wearing a business case: it serves some people and not
//! others, and the ones it skips are those with the least software in their own
//! language already.* This crate is the machinery that makes translating
//! something that can arrive rather than something the shell has to be
//! rewritten for — the first target being all 24 official EU languages, and any
//! language somebody contributes after that.
//!
//! There are no translations here yet. What is here is what stops English being
//! written into the shell while the shell is being written.
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`key`] | What names one string |
//! | [`template`] | A sentence with named gaps in it |
//! | [`filling`] | What goes into the gaps |
//! | [`phrase`] | One string the code can say: its key, its English, its note to a translator |
//! | [`word`] | The same thing as a crate writes it: three literals in a `const` |
//! | [`plural`] | One string that counts something, and the shapes it takes |
//! | [`form`] | The six shapes a counted sentence takes |
//! | [`cldr`] | Which form a language uses for which number, read from CLDR |
//! | [`vocabulary`] | Everything the code can say, and where a translation is checked against it |
//! | [`translation`] | One language's strings as they arrive, and what can be wrong with them |
//! | [`speaking`] | A translation that has been checked: the only thing the lookup accepts |
//! | [`language`] | Which language, and which way it is read |
//! | [`union`] | The Union's 24, each named in itself |
//! | [`said`] | One answer, and where it came from |
//! | [`strings`] | The lookup, and the chain it walks |
//!
//! ```
//! use alo_strings::{Filling, Key, Language, Phrase, Strings, Translation, Vocabulary};
//!
//! // The code declares what it can say, in the language the code is written in.
//! let too_big = Key::named("files.too-big")?;
//! let mut vocabulary = Vocabulary::empty();
//! vocabulary.says(Phrase::says(
//!     too_big.clone(),
//!     "{path} holds {bytes} bytes and a verb reads at most {most}",
//! )?)?;
//!
//! // A translation arrives, and is checked against what the code actually says
//! // before a word of it can reach anybody.
//! let german = vocabulary
//!     .check(Translation::into_language(Language::written("de")?).says(
//!         too_big.clone(),
//!         "{path} ist {bytes} Bytes groß, und ein Verb liest höchstens {most}",
//!     ))
//!     .map_err(|wrongs| wrongs.to_string())?;
//!
//! let mut strings = Strings::of(vocabulary);
//! strings.speaks(german)?;
//! strings.prefers(&[Language::written("de")?]);
//!
//! let said = strings.say(
//!     &too_big,
//!     &Filling::of("path", "/home/ada/notes")
//!         .and("bytes", "4 000 000")
//!         .and("most", "1 000 000"),
//! );
//! assert_eq!(
//!     said.text(),
//!     "/home/ada/notes ist 4 000 000 Bytes groß, und ein Verb liest höchstens 1 000 000"
//! );
//! assert!(said.is_translated());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # The three decisions
//!
//! **English is a source, not a default.** The English sentence lives beside
//! the key in the code, the way `alo-shortcuts` keeps its default bindings and
//! `alo-appearance` keeps its shipped wallpaper: only the difference is stored,
//! and a translation is the difference. That is not the hardcoded English
//! `CLAUDE.md` forbids, and the distinction is carried by a type rather than by
//! a convention — every answer is a [`Said`], and a [`Said`] says whether
//! somebody translated it. *Shown English because nothing was translated* is
//! therefore a state that can be seen, listed and marked, rather than one that
//! is discovered by a person in Latvia after the release.
//!
//! **A translation is checked against what the code actually says.** A
//! translator writes in a language nobody on this team reads, and the moment to
//! find a mistake in their file is when it is loaded rather than when somebody's
//! disk is full and the sentence saying so comes out with a hole where the file
//! name should be. So a gap the source has and the translation dropped is
//! refused, a gap the translation invented is refused, and the refusals are
//! addressed to a translator rather than to a programmer. Missing strings are
//! *not* refused: a language arrives a few hundred strings at a time, and a
//! check that insisted on a complete file would mean nobody ever saw the first
//! half of anybody's work.
//!
//! **A language is named in its own language.** A picker offering *Greek* is a
//! picker the people it exists for cannot read. The 24 are in [`union`] with
//! their own names, and a language somebody contributes shows its tag until
//! somebody adds its name beside it.
//!
//! # What this crate is deliberately not
//!
//! **It does not read a file.** [`Translation`] is serde and nothing here opens
//! anything, which is the same line `alo-shortcuts` and `alo-appearance` draw:
//! where the translations live and who loads them is the shell's, and the shell
//! does not exist yet.
//!
//! **It does not format numbers, dates or sizes.** How a number is written
//! belongs to the region rather than to the language — a person reading Swedish
//! in Finland writes a time the Finnish way — and `alo-appearance` settled that
//! first. A [`Filling`] takes text the caller has already made.
//!
//! **It is not a capability.** Nothing here is reachable by an agent and
//! nothing here needs a grant: a person reading their own machine in their own
//! language is not an agent doing something.
//!
//! # A sentence that counts something
//!
//! *1 byte* and *2 bytes* are one sentence in English with two shapes. Polish
//! has three, Irish five, and Latvian has one for nothing at all — so a
//! sentence with a number in it is declared as a [`Plural`] rather than a
//! [`Phrase`], asked for with [`Strings::count`] rather than [`Strings::say`],
//! and answered with the form **the reader's own language** uses for **that**
//! number.
//!
//! ```
//! use alo_strings::{Counting, Filling, Key, Language, Plural, Strings, Translation, Vocabulary};
//!
//! let key = Key::named("files.found")?;
//! let mut vocabulary = Vocabulary::empty();
//! vocabulary.counts(Plural::counting(
//!     key.clone(),
//!     "how_many",
//!     "1 file",
//!     "{how_many} files",
//! )?)?;
//!
//! // Polish counts in three for a whole number, and none of them is `other`.
//! let polish = vocabulary
//!     .check(
//!         Translation::into_language(Language::written("pl")?)
//!             .says(key.for_form(alo_strings::Form::One), "1 plik")
//!             .says(key.for_form(alo_strings::Form::Few), "{how_many} pliki")
//!             .says(key.for_form(alo_strings::Form::Many), "{how_many} plików"),
//!     )
//!     .map_err(|wrongs| wrongs.to_string())?;
//!
//! let mut strings = Strings::of(vocabulary);
//! strings.speaks(polish)?;
//! strings.prefers(&[Language::written("pl")?]);
//!
//! assert_eq!(strings.count(&key, &Counting::of(1), &Filling::nothing()).text(), "1 plik");
//! assert_eq!(strings.count(&key, &Counting::of(3), &Filling::nothing()).text(), "3 pliki");
//! assert_eq!(strings.count(&key, &Counting::of(7), &Filling::nothing()).text(), "7 plików");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! **The rules are read, not remembered.** [`cldr`] is the cardinal plural
//! table from `unicode-org/cldr`, quoted arm by arm, and it covers whole
//! numbers because alo OS counts things and a thing is a whole number. A
//! language whose rules nobody has read is not guessed at: a countable string
//! translated into one is refused, addressed to whoever is contributing it.
//!
//! # What this crate does not answer yet
//!
//! **Counting something that is not a whole number.** *1.5 hours* takes a
//! different form from *1 hour* in several languages, and nothing here can
//! express it. That is a decision to reopen with the CLDR operands in front of
//! it, not a form to pick as though the number had been whole; [`cldr`] says
//! what it would cost.
//!
//! **The strings of the crate that has not moved.** `alo-files` (item 9b),
//! `alo-shortcuts` (9c), `alo-appearance` (9d), `alo-capability` (9e) and
//! `alo-models` (9f) have all moved and declare their own words, each in a
//! `words` module built out of [`Word`]. `alo-egress` has not: its
//! destinations, its indicator line and its policy refusal are still English in
//! the source, and moving them is item 9h.
//!
//! **The sentence a person approves.** `alo_capability::Call` renders its
//! sentence when the call is made and keeps the string, so what a shell shows
//! and what the approval and the record keep are two renderings of one string
//! rather than one value worded twice. Item 9e decided they should be one —
//! the argument it made about refusals, applied to the sentence — and item 9g
//! is where a `Call` starts carrying a key and a filling instead.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod cldr;
pub mod filling;
pub mod form;
pub mod key;
pub mod language;
pub mod phrase;
pub mod plural;
pub mod said;
pub mod speaking;
pub mod strings;
pub mod template;
pub mod translation;
pub mod union;
pub mod vocabulary;
pub mod word;

pub use cldr::Counting;
pub use filling::Filling;
pub use form::{EVERY_FORM, Form};
pub use key::{Key, KeyError};
pub use language::{Direction, Language, LanguageError};
pub use phrase::{Phrase, PhraseError};
pub use plural::{Plural, PluralError};
pub use said::{CameFrom, Said};
pub use speaking::Speaking;
pub use strings::{Showing, Strings, StringsError};
pub use template::{Filled, Template, TemplateError};
pub use translation::{Amiss, Translation, Wrong, Wrongs};
pub use union::{OFFICIAL, Official, THE_SOURCE};
pub use vocabulary::{Vocabulary, VocabularyError};
pub use word::{Word, WordError};
