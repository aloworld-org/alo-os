//! The three verbs that only look: list a folder, read a file, find something
//! by name.
//!
//! They run inside the turn and nobody is asked to approve them (ADR 0001 §5),
//! which is exactly why the bounds here matter. A read that answered with a
//! four-gigabyte file, or a listing that answered with a million names, would
//! be a way to fill a person's screen and a model's context with one call that
//! nobody was ever offered the chance to stop.
//!
//! So each of the three is bounded, and each says when it reached its bound.
//! "There is more" is a different answer from "there is no more", and a verb
//! that could not tell them apart would be a verb that reports a file as
//! missing because it stopped looking.
//!
//! # Everything here opens what was resolved
//!
//! Every path arriving here is a [`crate::Real`] — resolved, and asked about
//! twice against the grants by [`crate::Touching`]. Nothing in this file
//! resolves anything again or builds a path out of one it was given, because a
//! second resolution is a second answer to the question reach was decided on.

use std::fs::{self, File};
use std::io::Read;

use crate::answer::{Answer, Listing, Search};
use crate::failed::Failed;
use crate::named::{Kind, Named};
use crate::real::Real;
use crate::walking::{MOST_WALKED, walk};

/// The most things one listing answers with.
///
/// The same bound `find_in_folder` declares for its own answer, because they
/// are the same question about the same folder and two different limits would
/// be two different pictures of it.
const MOST_LISTED: usize = 1000;

/// The most bytes one read answers with.
///
/// A megabyte is a long document and a great deal of text. Past it, an answer
/// stops being something a person can read or a model can hold, and what the
/// person wants is an application rather than a verb.
///
/// **Public because an answer leaves this machine's process**: what a read
/// answers with crosses the daemon's socket, and the bound on that message is
/// derived from this number rather than guessed beside it. Two crates each
/// deciding how big an answer may be is two bounds that can disagree, and the
/// way they would disagree is a read that succeeded and an answer that could
/// not be sent. `docs/contracts/agent-verbs.md` states it as *a read at most a
/// megabyte*, and `crates/alo-protocol` is where it is read.
pub const MOST_READ: u64 = 1024 * 1024;

/// List what is in a folder.
///
/// # Errors
/// [`Failed`] if it is not a folder, went away, or the machine would not read
/// it.
pub(crate) fn list(folder: &Real) -> Result<Answer, Failed> {
    let at = folder.as_path();
    let what = fs::symlink_metadata(at).map_err(|why| Failed::machine(at, "listed", &why))?;
    if !what.is_dir() {
        return Err(Failed::NotAFolder {
            path: at.display().to_string(),
        });
    }

    let mut things = Vec::new();
    let mut could_not_be_named = 0;
    let mut cut_short = false;
    for entry in fs::read_dir(at).map_err(|why| Failed::machine(at, "listed", &why))? {
        if things.len() >= MOST_LISTED {
            cut_short = true;
            break;
        }
        let entry = entry.map_err(|why| Failed::machine(at, "listed", &why))?;
        match Named::of(&entry) {
            Some(named) => things.push(named),
            None => could_not_be_named += 1,
        }
    }
    things.sort();
    Ok(Answer::Listed(Listing::of(
        things,
        could_not_be_named,
        cut_short,
    )))
}

/// Read what is in a file, as text.
///
/// # Errors
/// [`Failed`] if it is not a file, is larger than one read answers with, is not
/// text, went away, or the machine would not read it.
pub(crate) fn read(file: &Real) -> Result<Answer, Failed> {
    let at = file.as_path();
    let what = fs::symlink_metadata(at).map_err(|why| Failed::machine(at, "read", &why))?;
    if what.is_dir() {
        return Err(Failed::NotAFile {
            path: at.display().to_string(),
        });
    }

    let opened = File::open(at).map_err(|why| Failed::machine(at, "read", &why))?;
    // How big it is, asked of the file that is open rather than of its name:
    // the name could be pointing at something else by now, and this handle
    // cannot be.
    let bytes = opened
        .metadata()
        .map_err(|why| Failed::machine(at, "read", &why))?
        .len();
    if bytes > MOST_READ {
        return Err(Failed::TooBig {
            path: at.display().to_string(),
            bytes,
            most: MOST_READ,
        });
    }

    // One byte past the bound, so that a file which grew between being measured
    // and being read is refused rather than answered with in part.
    let mut held = Vec::new();
    opened
        .take(MOST_READ + 1)
        .read_to_end(&mut held)
        .map_err(|why| Failed::machine(at, "read", &why))?;
    if held.len() as u64 > MOST_READ {
        return Err(Failed::TooBig {
            path: at.display().to_string(),
            bytes: held.len() as u64,
            most: MOST_READ,
        });
    }

    String::from_utf8(held)
        .map(Answer::Read)
        .map_err(|_| Failed::NotText {
            path: at.display().to_string(),
        })
}

/// Find files in a folder, and in the folders inside it, by what they are
/// called.
///
/// **The name is matched, and nothing is interpreted.** A file matches when its
/// name contains what was asked for, ignoring case. There is no wildcard, no
/// pattern and no expression — ADR 0001 §1 at the place somebody would most
/// reasonably ask for one — so a search cannot be made to mean something other
/// than a search.
///
/// # Errors
/// [`Failed`] if it is not a folder, went away, or the machine would not read
/// it.
pub(crate) fn find(folder: &Real, named: &str, most: usize) -> Result<Answer, Failed> {
    let at = folder.as_path();
    let what = fs::symlink_metadata(at).map_err(|why| Failed::machine(at, "searched", &why))?;
    if !what.is_dir() {
        return Err(Failed::NotAFolder {
            path: at.display().to_string(),
        });
    }

    let looking_for = named.to_lowercase();
    let walked = walk(at, MOST_WALKED)?;
    let mut files = Vec::new();
    let mut more = walked.cut_short;
    for step in walked.things {
        if step.kind != Kind::File {
            continue;
        }
        let name = step
            .below
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if !name.contains(&looking_for) {
            continue;
        }
        if files.len() >= most {
            more = true;
            break;
        }
        files.push(step.at);
    }
    Ok(Answer::Found(Search::of(files, more)))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_folder_of_our_own, really, said};

    /// A listing is in an order a person can read, and says what each thing is
    /// without following it.
    #[test]
    fn a_listing_is_ordered_and_says_what_each_thing_is() {
        let folder = a_folder_of_our_own("listing");
        fs::create_dir_all(folder.join("2026")).unwrap();
        fs::write(folder.join("notes.txt"), b"hello").unwrap();
        fs::write(folder.join("april.pdf"), b"an invoice").unwrap();

        let answer = list(&really(&folder)).unwrap();
        let listing = answer.listed().unwrap();
        let names: Vec<_> = listing.things().iter().map(Named::name).collect();
        assert_eq!(names, ["2026", "april.pdf", "notes.txt"]);
        assert_eq!(
            listing.things().first().map(Named::kind),
            Some(Kind::Folder)
        );
        assert_eq!(listing.things().get(1).map(Named::bytes), Some(10));
        assert!(!listing.was_cut_short());
        assert_eq!(listing.could_not_be_named(), 0);

        let _ = fs::remove_dir_all(&folder);
    }

    /// A folder that is a file is answered as what it is, rather than as an
    /// empty listing that would read as an empty folder.
    #[test]
    fn listing_something_that_is_not_a_folder_says_what_it_is() {
        let folder = a_folder_of_our_own("notafolder");
        let file = folder.join("march.pdf");
        fs::write(&file, b"an invoice").unwrap();

        let refused = list(&really(&file)).unwrap_err();
        assert!(matches!(refused, Failed::NotAFolder { .. }), "{refused:?}");

        let _ = fs::remove_dir_all(&folder);
    }

    /// Reading answers with the text in the file, exactly as it is written —
    /// what is in a document is the thing being asked for, and tidying it would
    /// answer with something nobody wrote.
    #[test]
    fn reading_answers_with_what_is_in_the_file() {
        let folder = a_folder_of_our_own("reading");
        let file = folder.join("notes.txt");
        fs::write(&file, "one\ntwo\n  three\n").unwrap();

        let answer = read(&really(&file)).unwrap();
        assert_eq!(answer, Answer::Read("one\ntwo\n  three\n".to_owned()));

        let _ = fs::remove_dir_all(&folder);
    }

    /// **A read is bounded**, and a file past the bound is refused rather than
    /// answered with in part. Half a document read as a whole one is worse than
    /// no document.
    #[test]
    fn a_file_larger_than_one_read_answers_with_is_refused() {
        let folder = a_folder_of_our_own("toobig");
        let file = folder.join("scan.txt");
        fs::write(&file, vec![b'x'; (MOST_READ + 1) as usize]).unwrap();

        let refused = read(&really(&file)).unwrap_err();
        assert!(matches!(refused, Failed::TooBig { .. }), "{refused:?}");
        assert!(said(&refused).contains("open it in an application"));

        let _ = fs::remove_dir_all(&folder);
    }

    /// A file that is not text says so, rather than answering with something
    /// that looks like text and is not.
    #[test]
    fn a_file_that_is_not_text_says_so() {
        let folder = a_folder_of_our_own("notext");
        let file = folder.join("scan.tiff");
        fs::write(&file, [0xFF_u8, 0xFE, 0x00, 0x9F]).unwrap();

        let refused = read(&really(&file)).unwrap_err();
        assert!(matches!(refused, Failed::NotText { .. }), "{refused:?}");

        let _ = fs::remove_dir_all(&folder);
    }

    /// A folder is not a file, and reading one says so instead of answering
    /// with whatever the machine makes of it.
    #[test]
    fn reading_a_folder_says_that_it_is_a_folder() {
        let folder = a_folder_of_our_own("readfolder");
        let refused = read(&really(&folder)).unwrap_err();
        assert!(matches!(refused, Failed::NotAFile { .. }), "{refused:?}");
        let _ = fs::remove_dir_all(&folder);
    }

    /// A search looks in the folders inside the folder, matches the name and
    /// nothing else, and ignores case — which is what a person means and what
    /// no wildcard is needed for.
    #[test]
    fn a_search_looks_all_the_way_down_and_matches_the_name() {
        let folder = a_folder_of_our_own("search");
        fs::create_dir_all(folder.join("2026/March")).unwrap();
        fs::write(folder.join("2026/March/March-invoice.pdf"), b"one").unwrap();
        fs::write(folder.join("2026/march-notes.txt"), b"two").unwrap();
        fs::write(folder.join("taxes.pdf"), b"three").unwrap();

        let answer = find(&really(&folder), "MARCH", 10).unwrap();
        let found = answer.found().unwrap();
        assert_eq!(found.files().len(), 2, "{:?}", found.files());
        assert!(!found.was_cut_short());
        assert!(
            found
                .files()
                .iter()
                .all(|file| file.to_string_lossy().to_lowercase().contains("march"))
        );

        let _ = fs::remove_dir_all(&folder);
    }

    /// **A search that stopped early says so.** Answering with as many as were
    /// asked for and no more, silently, would tell somebody a file is not there
    /// when what happened is that nobody looked.
    #[test]
    fn a_search_that_answered_with_as_many_as_it_was_asked_for_says_there_may_be_more() {
        let folder = a_folder_of_our_own("bounded-search");
        for which in 0..4 {
            fs::write(folder.join(format!("march-{which}.pdf")), b"an invoice").unwrap();
        }

        let two = find(&really(&folder), "march", 2).unwrap();
        let found = two.found().unwrap();
        assert_eq!(found.files().len(), 2);
        assert!(found.was_cut_short());

        let four = find(&really(&folder), "march", 4).unwrap();
        let all = four.found().unwrap();
        assert_eq!(all.files().len(), 4);
        assert!(!all.was_cut_short());

        let _ = fs::remove_dir_all(&folder);
    }

    /// A search finds files, not folders: the verb says files, and a folder
    /// answered as one would be a path a read would then refuse.
    #[test]
    fn a_search_answers_with_files_and_not_with_folders() {
        let folder = a_folder_of_our_own("onlyfiles");
        fs::create_dir_all(folder.join("march")).unwrap();
        fs::write(folder.join("march/march.pdf"), b"an invoice").unwrap();

        let answer = find(&really(&folder), "march", 10).unwrap();
        let found = answer.found().unwrap();
        assert_eq!(found.files().len(), 1);
        assert!(
            found
                .files()
                .first()
                .is_some_and(|file| file.ends_with("march.pdf"))
        );

        let _ = fs::remove_dir_all(&folder);
    }
}
