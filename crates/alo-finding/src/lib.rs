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
//! | [`Index`], [`Index::of`] | One folder, indexed now |
//! | [`Index::again`] | The same folder, indexed again, reading only what changed |
//! | [`Index::answer`], [`Query`], [`Answer`] | An answer from the index alone, beside what was not searched |
//! | [`NotSearched`] | What the query could not be held against |
//! | [`NotAsked`] | A query that is not one, refused before anything is searched |
//! | [`Index::kept_at`], [`Index::read_from`], [`Index::where_kept`] | The file the index lives in |
//! | [`Entry`], [`Kind`], [`Contents`], [`Moment`] | One thing under the folder, and what is known about it |
//! | [`Covered`] | What the walk could not reach, so a search can say what it did not look at |
//! | [`NotIndexed`] | The seven ways there is no index at all |
//! | [`words`] | Every sentence this crate can say, in the reader's language |
//!
//! ```no_run
//! use std::path::Path;
//!
//! use alo_finding::{Index, Kind, Query};
//!
//! let index = Index::of(Path::new("/home/ada/Documents"))?;
//! let invoices = index.answer(&Query::of_kind(Kind::Pdf).and_named("invoice"))?;
//! let about_the_summer = index.answer(&Query::saying("contract summer"))?;
//! assert!(about_the_summer.not_searched.no_reader.contains(&invoices.found[0]));
//! let at = Index::where_kept(
//!     std::env::var_os("XDG_DATA_HOME").as_deref(),
//!     std::env::var_os("HOME").as_deref(),
//!     &index.of,
//! )?;
//! index.kept_at(&at)?;
//! let back = Index::read_from(&at, &index.of)?;
//! assert_eq!(back.entries, index.entries);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod answer;
pub mod asking;
pub mod covered;
pub mod entry;
pub mod index;
pub mod kind;
pub mod query;
pub mod refusing;
pub mod words;

mod format;
mod indexing;
mod keeping;
mod place;
mod reading;
mod searching;
mod wording;

pub use answer::{Answer, NotSearched};
pub use asking::{A_NAME, A_SENTENCE, NotAsked};
pub use covered::{Covered, Unread};
pub use entry::{Contents, Entry, Moment};
pub use index::Index;
pub use kind::{Kind, SNIFFED};
pub use place::{DATA_HOME, HOME, THE_FOLDER, THE_INDEXES};
pub use query::Query;
pub use refusing::NotIndexed;
pub use words::{
    Counted, EVERY_COUNTED, EVERY_WORD, Word, WordsError, declare_into, finding_words,
};
