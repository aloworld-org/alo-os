//! How a path goes back to a client, and what happens to one that cannot be
//! shown.
//!
//! This is the decision item 21b existed to make. `alo_files::Answer` carries
//! `PathBuf`s — where a file was moved to, what a search found, where an
//! archive was written — and **a path is not always text**: it is bytes on
//! Linux and ill-formed UTF-16 on Windows, and neither is required to be
//! anything a JSON string can hold.
//!
//! # Which is why `alo_files::Answer` did not gain a `Serialize`
//!
//! Three reasons, and the third is the one that decided it.
//!
//! A derived `Serialize` on a `PathBuf` **fails** on a path that is not UTF-8,
//! so the road that works for everybody's files would be a road that errors on
//! somebody's — and what that person's shell would show them is not *this file
//! has an unusual name* but whatever a daemon does with an answer it cannot
//! write down. A read that succeeded would arrive as a failure.
//!
//! It would also put the wire's shape inside the crate that touches the disk.
//! The format is a public surface (`docs/contracts/daemon-protocol.md`), and a
//! crate whose job is `std::fs` should not be a crate a protocol change has to
//! be made in. That is item 4's argument for `alo-record` being its own crate,
//! met from the other end.
//!
//! And `alo-files` has already decided what to do about text it did not write.
//! [`alo_files::Named`] refuses a name that could rewrite what an answer appears
//! to say — a control character, an escape sequence, a line break making one
//! name look like two — and the listing **counts** what it left out rather than
//! dropping it silently. A path in an answer is the same text with the same
//! problem, so it is the same rule, asked one crate further out through
//! [`alo_files::can_be_shown`] rather than written down a second time.
//!
//! # A name is shown inside a sentence; contents are shown as contents
//!
//! This rule is for **paths**, and deliberately not for what a read answered
//! with. A file that contains a tab, a line break or a terminal escape is an
//! ordinary file, and a read that refused it would be a verb that works on
//! prose and not on anything else. What keeps that safe on the wire is the
//! format rather than a check: JSON escapes a control character, so a file with
//! a line break in it still crosses as one line — and what a shell does with
//! the contents afterwards is draw them as contents, in a place where nothing
//! is being said to anybody.

use std::path::{Path, PathBuf};

/// A path as it can be shown, or nothing when it cannot.
///
/// `None` twice over: a path this machine cannot spell in Unicode, and one
/// holding something that could rewrite the answer around it.
pub(crate) fn shown(path: &Path) -> Option<String> {
    let text = path.to_str()?;
    alo_files::can_be_shown(text).then(|| text.to_owned())
}

/// Every path that can be shown, and how many could not.
///
/// The pair rather than a filtered list, because a search that quietly answered
/// with four of five files would read exactly like one that found four —
/// `alo_files::Listing`'s rule, which is why the count travels beside the list
/// everywhere in this crate.
pub(crate) fn all_shown(paths: &[PathBuf]) -> (Vec<String>, usize) {
    let mut shown_paths = Vec::with_capacity(paths.len());
    let mut could_not_be_named = 0;
    for path in paths {
        match shown(path) {
            Some(text) => shown_paths.push(text),
            None => could_not_be_named += 1,
        }
    }
    (shown_paths, could_not_be_named)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An ordinary path crosses as it is written, in whatever script it is
    /// written in.
    #[test]
    fn an_ordinary_path_crosses_as_it_is_written() {
        for path in [
            "/home/anna/Invoices/march.pdf",
            "/home/anna/Rechnungen/März 2026.pdf",
            "/home/анна/фактура.pdf",
            r"\\?\C:\Users\anna\Invoices\march.pdf",
        ] {
            assert_eq!(shown(Path::new(path)), Some(path.to_owned()), "{path}");
        }
    }

    /// **A path that could rewrite the answer around it is not shown**, and
    /// this is `alo_files::Named`'s rule reaching the wire: the name is the
    /// file's, the sentence around it is ours, and text that can move a cursor
    /// makes one of them look like the other.
    #[test]
    fn a_path_that_could_rewrite_an_answer_is_not_shown() {
        for path in [
            "/home/anna/march.pdf\nmoved: /etc/shadow",
            "/home/anna/\u{1b}[2Kmarch.pdf",
            "/home/anna/march\u{7}.pdf",
            "",
        ] {
            assert_eq!(shown(Path::new(path)), None, "{path:?}");
        }
    }

    /// **What could not be shown is counted**, because a list that silently
    /// lost one of its entries reads exactly like a complete one — and somebody
    /// would go on to conclude that a file is not there.
    #[test]
    fn what_cannot_be_shown_is_counted_rather_than_dropped() {
        let files = vec![
            PathBuf::from("/home/anna/Invoices/march.pdf"),
            PathBuf::from("/home/anna/Invoices/april\u{1b}.pdf"),
            PathBuf::from("/home/anna/Invoices/may.pdf"),
        ];
        let (shown_paths, could_not_be_named) = all_shown(&files);
        assert_eq!(shown_paths.len(), 2);
        assert_eq!(could_not_be_named, 1);
        assert!(shown_paths.iter().all(|path| path.ends_with(".pdf")));
    }

    /// Nothing to hide is nothing counted, which is the ordinary answer and the
    /// one a shell draws without a caveat.
    #[test]
    fn an_answer_with_nothing_left_out_counts_nothing() {
        let (shown_paths, could_not_be_named) =
            all_shown(&[PathBuf::from("/home/anna/Invoices/march.pdf")]);
        assert_eq!(shown_paths.len(), 1);
        assert_eq!(could_not_be_named, 0);
    }
}
