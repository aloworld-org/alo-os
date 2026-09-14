//! Search your own files: an index that lives on the machine.
//!
//! `docs/features.md` promises, at v0.5: *search your own files, without
//! asking anything — by name, kind, date and contents, in the file manager,
//! indexed on the machine.* And the sentence the promise turns on: *a machine
//! whose only search is a conversation is a machine somebody locked out of
//! their own documents.* This crate is the index — the thing the file manager
//! and the agent will both ask — written first so that the agent's *"where is
//! that file?"* becomes a nicer road to it rather than the road.
//!
//! # What an index holds, and where it lives
//!
//! An [`Index`] is one folder a person named, walked once, with one [`Entry`]
//! for everything under it: where it is below the folder, its [`Kind`] read
//! from its own bytes rather than its extension, its size, when it was last
//! written as the filesystem says, and its [`Contents`] — the words in it, for
//! a kind that is text, or the reason there are none. A [`Query`] over any of
//! those four is answered by [`Index::answer`] from the index alone: the disk
//! is never walked to answer, which is why an index of a folder that has
//! since been unplugged still answers.
//!
//! # An answer says what it did not look at
//!
//! An [`Answer`] is what matched **beside** what was not searched — a
//! [`NotSearched`] listing the folders the machine would not read, the
//! folders on another disk, the files that could not be opened, the kinds
//! with no reader, and the folder outside which nothing was looked at — so
//! that an empty answer is *nothing matched* and never *nothing was looked
//! at*. A query that is not a query — empty, or longer than a sentence — is
//! refused with a [`NotAsked`] before anything is searched, rather than
//! answered with everything. The answer carries how long it took, and
//! `tests/a_search_answers_in_time_and_says_what_it_did_not_read.rs` builds
//! an index of ten thousand files and times it.
//!
//! The index is **on this machine**, in a file the person owns under their
//! own directory — [`Index::where_kept`] says where, and
//! `docs/contracts/file-index.md` says what is in it — written whole by
//! [`Index::kept_at`] and read back by [`Index::read_from`]. Indexing again
//! is incremental: [`Index::again`] walks the folder and reads only a file
//! whose size or time has changed, and [`Index::opened`] says how many it
//! read, so that a test can count.
//!
//! # An index says when it was made, and an answer says how old it is
//!
//! [`Index::of`] and [`Index::again`] take the moment the index is made
//! **from the caller**, as a [`std::time::SystemTime`], and write it into the
//! file's first line as [`Index::made`]; nothing in this crate reads a clock,
//! and the shipped-source test holds the one stopwatch here to timing a
//! search. [`Answer::made`] carries that moment back out beside the results,
//! so a window can say *as of Tuesday* rather than answering about last
//! Tuesday's folder as if it were now. An index file written before the
//! moment was kept still reads, with no moment rather than an invented one.
//!
//! # Which folders are indexed
//!
//! [`Indexed`] is the list of the folders a person asked to have indexed,
//! kept beside the indexes in the same directory and written whole the same
//! way. [`Indexed::index_of`] hands back a folder's index read from the disk,
//! or refuses in words saying the folder was never indexed — and never walks
//! the folder to find out. [`Indexed::keep`] writes an index and puts its
//! folder on the list; [`Indexed::forget`] takes a folder off the list and
//! removes its index file with it; [`Indexed::again`] brings a folder's
//! index up to date by its name — read, indexed again reading only what
//! changed, kept — in one call, at a moment the caller names, refusing a
//! folder never indexed rather than indexing it for the first time and
//! keeping the index of a folder that is gone rather than forgetting it.
//! Nothing here watches a folder: no `inotify`, no thread, no timer — when
//! an index is brought up to date is the caller's decision. The list is not
//! a grant: a folder being indexed says nothing about whether an agent may
//! search it, and the list holds folders and nothing the record does.
//!
//! # One search over every indexed folder
//!
//! A person's search box is not a folder's. [`Indexed::answer`] puts one
//! [`Query`] to every folder on the list in one call and hands back an
//! [`Everywhere`]: one [`OfFolder`] per folder, **in the list's order**, each
//! carrying the folder it is of and either a [`Held`] — what matched, what
//! was not searched and the moment its index was made, held apart from the
//! index it came from — or the [`NotIndexed`] that stood where the answer
//! would be, because its index file could not be read or is not an index.
//! A folder that would not answer is a named refusal **beside** the others,
//! never a gap, so that *nothing matched* is never said about a folder
//! nobody looked at. The query is checked once before any index file is
//! opened, an empty list answers with no folders and no refusal, and nothing
//! ranks across folders or within one. It is not a verb: an agent's
//! `search_files` still names one granted folder.
//!
//! # An agent asks the same index, under a grant
//!
//! `search_files` is the verb — declared in [`verbs`] in the shape
//! `alo-files` declares its six, a read over a granted folder, so it runs
//! inside the turn with no approval to name and is refused outside its grant
//! like anything else. [`Searched`] is the door from an
//! [`alo_files::Touching`] — the call permitted, and the folder made real —
//! to [`Index::answer`], and it hands the authorisation back so the record
//! can be written. A person in the file manager takes none of that road:
//! [`Index::answer`] takes an index and a query, and there is no argument
//! through which an agent and a person could be told apart.
//!
//! # What is deliberately not here
//!
//! **Contents are never sent anywhere.** Nothing in this crate opens a socket,
//! and `tests/nothing_here_opens_a_socket_or_asks_anybody.rs` reads the shipped
//! source to say so. **Nothing here is a conversation**: no model is asked what
//! a file is about, and *contents* means the words in the file, decided by
//! `wording.rs`. **The index is not the record** and holds nothing the record
//! does: no agent, no grant, no approval, nothing about who asked — because
//! a person searching their own files is not an agent and is not asking
//! anybody. **The walk is `alo-files`'**, with its measuring policy, so a link
//! is an entry that is a link and is never followed, an unreadable folder is
//! noted in [`Covered`] rather than failing the index, and another filesystem
//! under the folder is noted and not entered.
//!
//! | | |
//! |---|---|
//! | [`Index`], [`Index::of`] | One folder, indexed at a moment the caller names |
//! | [`Index::again`] | The same folder, indexed again, reading only what changed |
//! | [`Index::made`], [`Answer::made`] | When the index was made, and so how old an answer is |
//! | [`Index::answer`], [`Query`], [`Answer`] | An answer from the index alone, beside what was not searched |
//! | [`NotSearched`] | What the query could not be held against |
//! | [`NotAsked`] | A query that is not one, refused before anything is searched |
//! | [`Index::kept_at`], [`Index::read_from`], [`Index::where_kept`] | The file the index lives in |
//! | [`Indexed`], [`Indexed::read_from`], [`Indexed::index_of`] | Which folders are indexed, and the index for one by its name |
//! | [`Indexed::keep`], [`Indexed::forget`] | A folder put on the list with its index, or taken off it with its index removed |
//! | [`Indexed::again`] | A folder's index brought up to date by its name, in one call |
//! | [`Indexed::answer`], [`Everywhere`], [`OfFolder`] | One query over every indexed folder: one answer or one named refusal per folder, in the list's order |
//! | [`Held`], [`Unsearched`] | An answer held apart from the index it came from |
//! | [`Entry`], [`Kind`], [`Contents`], [`Moment`] | One thing under the folder, and what is known about it |
//! | [`Covered`] | What the walk could not reach, so a search can say what it did not look at |
//! | [`NotIndexed`] | The twelve ways there is no index at all |
//! | [`finding_verbs`], [`verbs::declare_into`] | The search verb, declared for an agent's list |
//! | [`Searched`], [`NotAnswered`] | A permitted search put to the index, and why it might not answer |
//! | [`words`] | Every sentence this crate can say, in the reader's language |
//!
//! ```no_run
//! use std::path::Path;
//! use std::time::SystemTime;
//!
//! use alo_finding::{Index, Indexed, Kind, Query};
//!
//! // The caller's clock, not the crate's: when an index is made is the
//! // caller's decision, and the moment is written into the index.
//! let index = Index::of(Path::new("/home/ada/Documents"), SystemTime::now())?;
//! let invoices = index.answer(&Query::of_kind(Kind::Pdf).and_named("invoice"))?;
//! let about_the_summer = index.answer(&Query::saying("contract summer"))?;
//! assert!(about_the_summer.not_searched.no_reader.contains(&invoices.found[0]));
//! assert_eq!(about_the_summer.made, index.made, "an answer says how old it is");
//! let mut indexed = Indexed::read_from(
//!     std::env::var_os("XDG_DATA_HOME").as_deref(),
//!     std::env::var_os("HOME").as_deref(),
//! )?;
//! indexed.keep(&index)?;
//! let back = indexed.index_of(&index.of)?;
//! assert_eq!(back.entries, index.entries);
//! assert!(indexed.index_of(Path::new("/home/ada/Pictures")).is_err());
//! // Later — when the caller decides — brought up to date by its name.
//! let fresh = indexed.again(&index.of, SystemTime::now())?;
//! assert!(fresh.opened <= index.opened, "only what changed was read");
//! // One search over every folder the person asked to have indexed.
//! let everywhere = indexed.answer(&Query::named("contract"))?;
//! for of in &everywhere.answers {
//!     match &of.answered {
//!         Ok(held) => println!("{}: {} found, as of {:?}", of.folder.display(), held.found.len(), held.made),
//!         Err(why) => println!("{}: {why}", of.folder.display()),
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod answer;
pub mod asking;
pub mod covered;
pub mod entry;
pub mod everywhere;
pub mod held;
pub mod index;
pub mod indexed;
pub mod kind;
pub mod query;
pub mod refusing;
pub mod searched;
pub mod unanswered;
pub mod verbs;
pub mod words;

mod format;
mod indexing;
mod keeping;
mod listed;
mod place;
mod reading;
mod searching;
mod wording;

pub use answer::{Answer, NotSearched};
pub use asking::{A_NAME, A_SENTENCE, NotAsked};
pub use covered::{Covered, Unread};
pub use entry::{Contents, Entry, Moment};
pub use everywhere::{Everywhere, OfFolder};
pub use held::{Held, Unsearched};
pub use index::Index;
pub use indexed::Indexed;
pub use kind::{Kind, SNIFFED};
pub use place::{DATA_HOME, HOME, THE_FOLDER, THE_INDEXES};
pub use query::Query;
pub use refusing::NotIndexed;
pub use searched::Searched;
pub use unanswered::NotAnswered;
pub use verbs::{Declaring, finding_verbs};
pub use words::{
    Counted, EVERY_COUNTED, EVERY_WORD, Word, WordsError, declare_into, finding_words,
};
