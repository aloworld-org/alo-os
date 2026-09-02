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
//! # What this crate does not answer yet
//!
//! **Plural forms**, which is item 9a in `docs/autonomy/QUEUE.md`. *1 byte* and
//! *2 bytes* are one sentence in English and two in Polish, three in Irish, and
//! Latvian has a form for zero. Getting that wrong for 24 languages from memory
//! is exactly the kind of thing this repository refuses to do quickly, so it is
//! a second item with the CLDR rules in front of it rather than a guess shipped
//! as a promise. Nothing here has to move for it: a plural phrase becomes one
//! key per form, and the vocabulary is already the place that would know which
//! forms a language needs.
//!
//! **The strings themselves.** `alo-files`, `alo-shortcuts` and
//! `alo-appearance` still hold their English in their own error types and
//! labels. Moving each of them onto this crate is its own item — 9b, 9c and 9d
//! — because a half-moved crate reads exactly like a finished one.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod filling;
pub mod key;
pub mod language;
pub mod phrase;
pub mod said;
pub mod speaking;
pub mod strings;
pub mod template;
pub mod translation;
pub mod union;
pub mod vocabulary;

pub use filling::Filling;
pub use key::{Key, KeyError};
pub use language::{Direction, Language, LanguageError};
pub use phrase::{Phrase, PhraseError};
pub use said::{CameFrom, Said};
pub use speaking::Speaking;
pub use strings::{Showing, Strings, StringsError};
pub use template::{Filled, Template, TemplateError};
pub use translation::{Amiss, Translation, Wrong, Wrongs};
pub use union::{OFFICIAL, Official, THE_SOURCE};
pub use vocabulary::{Vocabulary, VocabularyError};
