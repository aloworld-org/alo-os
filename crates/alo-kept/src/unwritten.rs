//! Why a value was not written.
//!
//! Every one of these is refused before the rename that would have replaced
//! the file, so each means the same thing to the person: nothing changed.

use crate::unread::Unread;

/// Why [`crate::keep`] did not replace a file.
///
/// **No `Display`**, for [`Unread`]'s reason: the owning crate says it, through
/// [`crate::Kept::not_written`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unwritten {
    /// The path handed over is relative, so it would be a file wherever the
    /// process happened to start. Nothing was written.
    NotWhereItBelongs,

    /// The value has no TOML text as a file of keys: the serialiser refused
    /// it, the shape is not a table of keys, or it has a `format` key of its
    /// own that the file's would overwrite.
    NotExpressible {
        /// What went wrong, for whoever is fixing the shape.
        why: String,
    },

    /// [`crate::Kept::untouched`] would have written something besides the
    /// format line — which is a shape writing down what nobody changed.
    NotOnlyTheDifference,

    /// The text written for the value reads back as a different value.
    ReadBackAsSomethingElse,

    /// The text written for the value does not read back at all.
    ReadBackRefused(Unread),

    /// The disk refused a step of the write: the folder, the sibling file, the
    /// sync or the rename. The file is as it was.
    Disk(std::io::ErrorKind),
}
