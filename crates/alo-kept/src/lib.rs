//! How a file in a person's own folder is kept.
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
//! gives each of a person's settings to the crate that declares its shape, one
//! file each, beside `settings.toml` in `$XDG_CONFIG_HOME/alo/`. Four small
//! files written by four crates would go wrong in the same four ways, and a
//! rule invented separately in each is four chances to get it wrong — so the
//! rule is decided once, here, and carried in type rather than in habit. A
//! crate keeps its file by implementing [`Kept`], and has no other way to.
//!
//! # The four clauses
//!
//! 1. **Only the difference is written, under a format of its own.** A shape
//!    names its file and its [`Kept::FORMAT`]; what goes into the file is the
//!    shape's own value, which for every crate ADR 0038 names is its
//!    `Changes`. What the person has not changed is not in the file, and
//!    [`Kept::untouched`] is written as the format line and nothing else —
//!    [`Unwritten::NotOnlyTheDifference`] refuses a shape that would write
//!    more.
//! 2. **No file means the person has changed nothing.** [`read`] answers
//!    [`Kept::untouched`] for a file that is not there, and it is never an
//!    error: an untouched machine has no file, and nothing ever made one.
//! 3. **A file that is there and wrong is refused whole**, in the owning
//!    crate's own words — [`Kept::not_read`] turns what was wrong into the
//!    owning crate's refusal — and nothing in it is honoured. There is no
//!    function here that answers with part of a file, because half a file is
//!    the machine choosing the other half.
//! 4. **A write is whole or not at all, and read back before it counts.**
//!    [`keep`] turns the value into text and refuses unless that text reads
//!    back as the same value; writes it to a sibling file, syncs it, reads the
//!    sibling back off the disk and asks the same question again; and only then
//!    renames it over the old file. Every refusal happens before the rename, so
//!    a write that fails leaves the file as it was.
//!
//! # What is not here
//!
//! **Where the folder is.** `alo_choosing::where_the_folder_is` works that out
//! without reading an environment, and whoever starts a session hands each
//! crate the path it keeps at. Nothing in this crate reads an environment
//! variable, and it depends on no crate that keeps a file.
//!
//! **Anything a person reads.** [`Unread`] and [`Unwritten`] have no
//! `Display`: they are handed to the owning crate, which says them in its own
//! words through its own vocabulary.
//!
//! **A watcher.** A change made in Settings is drawn at once because Settings
//! and the compositor are one process; a file edited by hand is read at the
//! next sign-in. A background reader of the person's folder is a mechanism
//! nobody asked for.
//!
//! **`settings.toml` itself.** `alo_choosing::Choosing` keeps that file by the
//! same rule and predates this crate. It reads more than one format and a
//! daemon reads it too, so its reader is its own; the rule it follows is the
//! one written here.

mod disk;
mod kept;
mod reading;
#[cfg(test)]
mod testing;
mod unread;
mod unwritten;
mod writing;

pub use kept::Kept;
pub use reading::{read, read_text};
pub use unread::Unread;
pub use unwritten::Unwritten;
pub use writing::{THE_FORMAT_KEY, keep, text_of};
