//! Where a picture goes: a file in a folder the person chose, or the
//! clipboard.
//!
//! The plan's acceptance has a clause that reads like an afterthought and is
//! not one — **never both unasked**. A screenshot tool that quietly did both is
//! a tool that leaves a copy of everything a person photographed in a folder
//! they were not thinking about, including the ones they took to paste into a
//! message and never meant to keep.
//!
//! So *both* is a value somebody constructs on purpose
//! ([`WhereItGoes::a_file_and_the_clipboard`]) and nothing else can produce it:
//! there is no `Default` here, no `From`, and no field anybody can set after
//! the fact. A caller that wants one road asks for one road, and the other road
//! is not travelled — [`crate::Screenshot::take`] is handed the machine's one
//! clipboard whatever the destination, and a test watches it come back
//! untouched.
//!
//! # The folder is one somebody chose, and this crate has none of its own
//!
//! A [`crate::Folder`] arrives from whatever asked for the picture. There is no
//! default folder anywhere in this crate, nothing read from an environment, and
//! nowhere a path can be assembled — so a picture written to a file is written
//! where somebody said, or it is not written. `folder.rs` says what that claim
//! is and what it deliberately is not.
//!
//! # There is no third road
//!
//! Not a network folder, not a *share* step, not a cloud anything. The plan's
//! constraint is that nothing is uploaded, shared or sent anywhere by taking a
//! screenshot, and the shape of this type is where that is true: there are two
//! destinations, both on this machine, and nothing in this crate can reach a
//! network — `tests/nothing_leaves_when_a_picture_is_taken.rs` reads the
//! manifests and says so.

use std::path::Path;

use crate::folder::Folder;

/// Where a picture of the screen is to go.
///
/// There are three destinations and one of them is *both*, which is the whole
/// of *never both unasked*: nothing widens one destination into two after the
/// fact, and nothing produces a destination out of nothing.
///
/// ```compile_fail
/// # fn main() {
/// let folder = alo_capturing::Folder::chosen(std::path::Path::new("/home/anna/Pictures"))
///     .expect("a folder");
/// let mut going = alo_capturing::WhereItGoes::a_file(&folder);
/// // There is no way to turn one destination into two after the fact.
/// going.and_also_the_clipboard();
/// # }
/// ```
///
/// ```compile_fail
/// # fn main() {
/// // And nothing produces a destination out of nothing.
/// let going: alo_capturing::WhereItGoes = Default::default();
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhereItGoes {
    /// Into a file, in a folder somebody picked.
    AFile(Folder),
    /// Onto the clipboard, and nowhere else.
    TheClipboard,
    /// Both, because somebody asked for both.
    AFileAndTheClipboard(Folder),
}

impl WhereItGoes {
    /// Into a file in this folder, and not onto the clipboard.
    #[must_use]
    pub fn a_file(folder: &Folder) -> Self {
        Self::AFile(folder.clone())
    }

    /// Onto the clipboard, and into no file.
    #[must_use]
    pub const fn the_clipboard() -> Self {
        Self::TheClipboard
    }

    /// Both — which is only ever this, asked for.
    #[must_use]
    pub fn a_file_and_the_clipboard(folder: &Folder) -> Self {
        Self::AFileAndTheClipboard(folder.clone())
    }

    /// The folder a file is to be written in, where one is.
    #[must_use]
    pub fn folder(&self) -> Option<&Path> {
        match self {
            Self::AFile(folder) | Self::AFileAndTheClipboard(folder) => Some(folder.at()),
            Self::TheClipboard => None,
        }
    }

    /// Whether a file is to be written.
    #[must_use]
    pub const fn writes_a_file(&self) -> bool {
        matches!(self, Self::AFile(_) | Self::AFileAndTheClipboard(_))
    }

    /// Whether the clipboard is to be given the picture.
    #[must_use]
    pub const fn reaches_the_clipboard(&self) -> bool {
        matches!(self, Self::TheClipboard | Self::AFileAndTheClipboard(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::a_folder_somebody_chose;

    /// **A file goes to a file and nowhere near the clipboard.**
    #[test]
    fn a_file_is_a_file_and_not_the_clipboard() {
        let folder = a_folder_somebody_chose("pictures");
        let where_it_goes = WhereItGoes::a_file(&folder);
        assert!(where_it_goes.writes_a_file());
        assert!(!where_it_goes.reaches_the_clipboard());
        assert_eq!(where_it_goes.folder(), Some(folder.at()));
    }

    /// **The clipboard goes to the clipboard and writes nothing**, so a picture
    /// taken to be pasted leaves nothing behind in a folder.
    #[test]
    fn the_clipboard_is_the_clipboard_and_writes_no_file() {
        let where_it_goes = WhereItGoes::the_clipboard();
        assert!(!where_it_goes.writes_a_file());
        assert!(where_it_goes.reaches_the_clipboard());
        assert_eq!(where_it_goes.folder(), None);
    }

    /// **Both is one value, and only somebody asking produces it.** There is no
    /// `Default` here and no way to widen one destination into two after the
    /// fact, which is the whole of *never both unasked*.
    #[test]
    fn both_is_a_thing_somebody_asked_for() {
        let folder = a_folder_somebody_chose("pictures");
        let both = WhereItGoes::a_file_and_the_clipboard(&folder);
        assert!(both.writes_a_file());
        assert!(both.reaches_the_clipboard());

        // And neither of the single destinations is it.
        assert_ne!(both, WhereItGoes::a_file(&folder));
        assert_ne!(both, WhereItGoes::the_clipboard());
    }

    /// **Exactly one of the three does both**, so *never both unasked* is
    /// arithmetic over the type rather than a rule somebody has to remember.
    /// The compile-fail examples on [`WhereItGoes`] are the other half.
    #[test]
    fn exactly_one_of_the_three_does_both() {
        let folder = a_folder_somebody_chose("pictures");
        let every = [
            WhereItGoes::a_file(&folder),
            WhereItGoes::the_clipboard(),
            WhereItGoes::a_file_and_the_clipboard(&folder),
        ];
        assert_eq!(
            every
                .iter()
                .filter(|going| going.writes_a_file() && going.reaches_the_clipboard())
                .count(),
            1
        );
        assert_eq!(
            every.iter().filter(|going| going.writes_a_file()).count(),
            2
        );
        assert_eq!(
            every
                .iter()
                .filter(|going| going.reaches_the_clipboard())
                .count(),
            2
        );
    }
}
