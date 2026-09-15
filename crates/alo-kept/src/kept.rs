//! What a crate declares to keep a file in a person's folder.
//!
//! Everything [`crate::read`] and [`crate::keep`] need to know about a file is
//! a constant or a function on this trait, so a crate cannot keep one without
//! having said its name, its format, its keys and its refusals — and cannot
//! keep one any other way.

use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::unread::Unread;
use crate::unwritten::Unwritten;

/// A shape kept as one file in a person's own folder.
///
/// Implemented by the crate that declares the shape and by nobody else, which
/// is ADR 0038's *one writer per file*.
pub trait Kept: Serialize + DeserializeOwned + PartialEq + Sized {
    /// The file's name inside the person's folder, such as `appearance.toml`.
    ///
    /// A name and not a path: where the folder is, is handed to the crate.
    const FILE: &'static str;

    /// The shape's format number, written as `format = N` at the top of the
    /// file.
    ///
    /// **Its own per file.** Appearance changing its shape says nothing about
    /// the dock's, and a file written by a release that knows another format is
    /// refused whole as [`Unread::AnotherFormat`] rather than read as whatever
    /// part of it happens to fit.
    const FORMAT: i64;

    /// Every key the shape may have at the top of its file, besides `format`.
    ///
    /// A hand-edited key that is not on this list is refused whole with the key
    /// named ([`Unread::UnknownKey`]), which is the sentence a person can act on.
    const KEYS: &'static [&'static str];

    /// What the owning crate says when its file is there and did not read.
    type NotRead;

    /// What the owning crate says when its file was not written.
    type NotWritten;

    /// The value of a person who has changed nothing.
    ///
    /// What a missing file reads as, and the one value written as a format line
    /// and nothing else.
    fn untouched() -> Self;

    /// This file, at `at`, did not read for this reason — in the owning crate's
    /// words.
    fn not_read(at: &Path, why: Unread) -> Self::NotRead;

    /// This value was not written to `at` for this reason — in the owning
    /// crate's words. The file at `at` is as it was.
    fn not_written(at: &Path, why: Unwritten) -> Self::NotWritten;
}
