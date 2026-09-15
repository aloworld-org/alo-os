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

    /// The file the value would replace is there and does not read, so it was
    /// not written over: it may be a person's hand edit with one mistake in it,
    /// and the next change they make in Settings must not be what throws it
    /// away. Asked of the file as it is at the moment of the write, never of
    /// what was read at sign-in. Only [`crate::put_back_as_shipped`] replaces
    /// such a file.
    OverAFileThatDidNotRead(Unread),

    /// The disk refused a step of the write: the folder, the sibling file, the
    /// sync or the rename. The file is as it was.
    Disk(std::io::ErrorKind),
}
