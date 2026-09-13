//! What is filling a folder: a tree of sizes, each true to the byte for what
//! it counted and honest about what it could not.
//!
//! `docs/features.md` promises, at v0.5: *what is filling the disk — shown as
//! sizes you can open up and click through, not a number in Settings.* The
//! click-through is the shell's. The tree underneath it is this: a [`Node`]
//! for the folder asked about, a [`Node`] for everything inside it, and on
//! each a size that is **the sum of its children plus its own bytes**.
//!
//! # A size that is silently too small is worse than no size
//!
//! So every way a size can fall short of the truth is written on the node it
//! happens to, as a [`Counted`], and once above the tree where it is about the
//! whole count:
//!
//! - a **file with more than one name** takes up its space once, and is
//!   counted once — under the first name the walk met, with every other name
//!   saying where;
//! - a **link** is the bytes of the link itself and is never followed, because
//!   a link out of the folder is how a folder's size becomes a lie about
//!   somewhere else;
//! - a **folder the machine would not read** is a node saying so, not a zero;
//! - a **folder on another filesystem** is a node that stops there and says
//!   so, which is also how naming the root of the machine is answered: a tree
//!   that stops at each mount point;
//! - a **count that reached its bound** says so on every folder it had not
//!   finished, and once at the top, rather than reporting a partial sum as a
//!   total.
//!
//! # What this is not
//!
//! Nothing here deletes, moves or empties anything; the answer ends at the
//! answer, and what a person does next is theirs on a surface this crate
//! never draws. The whole disk is never walked unasked: a caller names a
//! folder, and the walk is `alo-files`' — borrowed, bounded, following
//! nothing — so that there is one opinion in this repository about what a
//! link is.

use std::path::{Path, PathBuf};

use alo_files::Kind;
use alo_strings::{Counting, Filling, Said, Strings};

use crate::refusing::NotMeasured;
use crate::words;

/// What is filling a folder: the folder as a tree of sizes, and what the
/// count as a whole has to say for itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holding {
    /// The folder that was asked about, as it was named.
    pub folder: PathBuf,

    /// The folder, its size, and everything inside it.
    pub tree: Node,

    /// Whether the count saw everything. When it did not, [`Self::tree`] has
    /// [`Counted::NotFinished`] on every folder it had not finished, and
    /// [`Self::not_the_whole`] has the sentence.
    pub finished: bool,

    /// The most things one count looks at: the bound it stopped at, if it
    /// stopped.
    pub most: usize,

    /// How many things were left out because their names cannot be shown.
    /// [`Self::left_unnamed`] has the sentence.
    pub unnamed: usize,
}

impl Holding {
    /// What is filling this folder, counted now.
    ///
    /// # Errors
    /// [`NotMeasured::NotCounted`] when the folder itself is not there, is
    /// not a folder, or could not be read. A folder *inside* it that cannot
    /// be read is not an error: it is a node in the answer saying so.
    #[cfg(target_os = "linux")]
    pub fn of(folder: &Path) -> Result<Self, NotMeasured> {
        use alo_files::{MOST_WALKED, Walking};
        let walked = Walking::measuring(MOST_WALKED)
            .through(folder)
            .map_err(|why| NotMeasured::NotCounted {
                at: folder.to_path_buf(),
                why,
            })?;
        Ok(crate::counting::tree_of(
            folder,
            walked,
            MOST_WALKED,
            &mut crate::looking::looked_at,
        ))
    }

    /// What is filling this folder — which this host cannot say.
    ///
    /// # Errors
    /// Always [`NotMeasured::NotOnThisHost`]. The tree's arithmetic is
    /// portable and tested everywhere, but whether a file has a second name,
    /// and whether a folder is on another filesystem, are questions only a
    /// Linux host answers — and a count that could not tell a second name
    /// from a second file would be a size that is silently wrong.
    #[cfg(not(target_os = "linux"))]
    pub fn of(folder: &Path) -> Result<Self, NotMeasured> {
        let _ = folder;
        Err(NotMeasured::NotOnThisHost)
    }

    /// What a window says above the tree when the count stopped at its
    /// bound, in the reader's language — or nothing, when it finished.
    #[must_use]
    pub fn not_the_whole(&self, strings: &Strings) -> Option<Said> {
        if self.finished {
            return None;
        }
        let most = u64::try_from(self.most).unwrap_or(u64::MAX);
        Some(strings.count(
            &words::CUT_SHORT.key(),
            &Counting::of(most),
            &Filling::of("most", most.to_string()),
        ))
    }

    /// What a window says above the tree when things were left out because
    /// their names cannot be shown, in the reader's language — or nothing,
    /// when none were.
    #[must_use]
    pub fn left_unnamed(&self, strings: &Strings) -> Option<Said> {
        if self.unnamed == 0 {
            return None;
        }
        let unnamed = u64::try_from(self.unnamed).unwrap_or(u64::MAX);
        Some(strings.count(
            &words::UNNAMED.key(),
            &Counting::of(unnamed),
            &Filling::of("unnamed", unnamed.to_string()),
        ))
    }
}

/// One thing in the tree: the folder asked about, or something inside it.
///
/// A record of facts with nothing to protect, so the fields are public.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// What it is called: its name in the folder above it, or, for the folder
    /// asked about, the path as it was named.
    pub name: String,

    /// Where it is.
    pub at: PathBuf,

    /// What it is, decided without following anything.
    pub kind: Kind,

    /// Its own bytes: a file's contents, a link's own length, and nothing for
    /// a folder. Read from the filesystem, so `stat` on [`Self::at`] shows
    /// the same number.
    pub own: u64,

    /// Its own bytes plus the sizes of everything inside it.
    pub size: u64,

    /// Whether the size is the whole truth about it, and if not, why.
    pub counted: Counted,

    /// What is inside it, in the order a person reads a folder.
    pub children: Vec<Node>,
}

impl Drop for Node {
    /// Let go of the children a level at a time rather than a child at a
    /// time: a folder can be nested deeper than a stack, and the drop the
    /// compiler would write recurses once per level.
    fn drop(&mut self) {
        let mut pending = std::mem::take(&mut self.children);
        while let Some(mut child) = pending.pop() {
            pending.append(&mut child.children);
        }
    }
}

/// Whether a node's size is the whole truth about it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Counted {
    /// It is.
    Whole,

    /// A file with more than one name, counted under another of them.
    ///
    /// Its own bytes here are none, so that the tree counts the file once;
    /// `at` is the name it was counted under.
    Elsewhere {
        /// The name the bytes are under.
        at: PathBuf,
    },

    /// The machine would not read it, so nothing inside it is counted.
    ///
    /// A folder that belongs to somebody else, usually; or a file that went
    /// away between the walk and the look at it. Its size is what could be
    /// seen, which for a folder is nothing — and this says so, which is the
    /// difference between unknown and empty.
    NotRead {
        /// What the machine said, as a sentence.
        why: String,
    },

    /// A folder on another filesystem, not entered.
    ///
    /// What is inside it takes no space on the disk being counted.
    OnAnotherFilesystem,

    /// A folder the count had not finished when it reached its bound.
    ///
    /// Its size is the size of what was seen, and smaller than the whole.
    NotFinished,
}

impl Counted {
    /// What a window says beside the size, in the reader's language — or
    /// nothing, when the size is the whole truth.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Option<Said> {
        let (word, filling) = match self {
            Self::Whole => return None,
            Self::Elsewhere { at } => (
                &words::ELSEWHERE,
                Filling::of("at", at.display().to_string()),
            ),
            Self::NotRead { .. } => (&words::NOT_READ, Filling::nothing()),
            Self::OnAnotherFilesystem => (&words::ANOTHER_FILESYSTEM, Filling::nothing()),
            Self::NotFinished => (&words::NOT_FINISHED, Filling::nothing()),
        };
        Some(strings.say(&word.key(), &filling))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **On any host but Linux, nothing is counted.** Not a tree of nothing:
    /// the refusal, in words, the way `alo-agentd` is absent rather than
    /// pretending.
    #[cfg(not(target_os = "linux"))]
    #[test]
    fn on_any_other_host_nothing_is_counted() {
        let refused = Holding::of(Path::new("Documents"));
        assert_eq!(refused, Err(NotMeasured::NotOnThisHost));
    }

    /// A size that is the whole truth has nothing to say beside it, and a
    /// count that finished with every name shown has nothing to say above
    /// it. Everything else does.
    #[test]
    #[expect(
        clippy::unwrap_used,
        reason = "in a test, a panic on an unexpected Err is the failure being reported"
    )]
    fn only_what_falls_short_of_the_truth_has_a_sentence() {
        let strings = Strings::of(crate::measuring_words().unwrap());
        assert!(Counted::Whole.said(&strings).is_none());
        for short in [
            Counted::Elsewhere {
                at: PathBuf::from("one.txt"),
            },
            Counted::NotRead {
                why: "permission denied".to_owned(),
            },
            Counted::OnAnotherFilesystem,
            Counted::NotFinished,
        ] {
            let said = short.said(&strings).unwrap();
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
        }

        let tree = Node {
            name: "Documents".to_owned(),
            at: PathBuf::from("Documents"),
            kind: Kind::Folder,
            own: 0,
            size: 0,
            counted: Counted::Whole,
            children: Vec::new(),
        };
        let whole = Holding {
            folder: PathBuf::from("Documents"),
            tree: tree.clone(),
            finished: true,
            most: 20_000,
            unnamed: 0,
        };
        assert!(whole.not_the_whole(&strings).is_none());
        assert!(whole.left_unnamed(&strings).is_none());

        let short = Holding {
            folder: PathBuf::from("Documents"),
            tree,
            finished: false,
            most: 20_000,
            unnamed: 3,
        };
        let cut = short.not_the_whole(&strings).unwrap();
        assert!(cut.text().contains("20"), "{cut}");
        let unnamed = short.left_unnamed(&strings).unwrap();
        assert!(unnamed.text().contains('3'), "{unnamed}");
    }
}
