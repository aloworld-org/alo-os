//! Why a file that is there did not read.
//!
//! There is no variant for a file that is not there, because that is not a
//! refusal: it is a person who has changed nothing.

/// What was wrong with a file that is there.
///
/// **No `Display`.** Nothing here is a sentence for a person — the owning
/// crate turns this into its own refusal through [`crate::Kept::not_read`] and
/// says it in its own vocabulary. The text carried by two of the variants is
/// for whoever is fixing alo OS.
///
/// **And nothing quotes the file.** A position is a line number; a key is
/// named because it is the one thing a person needs in order to fix what they
/// typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unread {
    /// The path handed over is relative, so it would be a file wherever the
    /// process happened to start. Nothing was opened.
    NotWhereItBelongs,

    /// The disk would not hand the file over, for a reason other than it not
    /// being there — a permission, or a folder where the file should be.
    Disk(std::io::ErrorKind),

    /// The bytes are not UTF-8 text, so they are not TOML.
    NotText,

    /// The text is not TOML, and the line it stopped at where TOML says.
    NotToml {
        /// The first line, counted from one, where the text stopped being TOML.
        line: Option<usize>,
    },

    /// There is no `format` number at the top of the file, or it is not a
    /// whole number — including a file that is empty.
    NoFormat,

    /// The file says it is a format this alo OS does not read.
    AnotherFormat {
        /// The format the file said it was.
        found: i64,
    },

    /// A key at the top of the file that the shape does not have.
    UnknownKey {
        /// The key, as written.
        key: String,
    },

    /// The keys are the shape's but a value in them is not — refused by the
    /// shape's own deserialiser, whose message this is.
    NotItsShape {
        /// What the shape's deserialiser said, which is where an owning crate
        /// that answers a `serde(try_from)` with a key of its own finds it.
        said: String,
    },
}
