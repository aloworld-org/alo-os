//! The list of indexed folders as a file: one line saying what it is, then
//! one line per folder.
//!
//! `docs/contracts/file-index.md` describes it under *the list of indexed
//! folders*; this is it as code. The shape is the index file's, for the
//! index file's reasons: every line is compact JSON with no newline inside
//! it, the first line carries `format` and nothing after it does, and a
//! reader that is not alo OS needs a JSON parser and nothing else.
//!
//! # A torn file is refused whole
//!
//! The list is replaced whole by `keeping.rs`, the way an index is, so a torn
//! one is not something an interrupted write leaves behind — but a line that
//! does not parse still refuses the file, because a list read in part would
//! say a folder was never indexed while its index sat on the disk.
//!
//! # Later versions may add, and this reader lets them
//!
//! A field this version does not know is ignored on the way in, and a
//! `format` this version does not know is refused, for the reasons
//! `format.rs` gives.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Which shape the file is in.
const FORMAT: u32 = 1;

/// The first line.
#[derive(Debug, Serialize, Deserialize)]
struct Head {
    /// Which shape the file is in; what tells a head from a folder's line.
    format: u32,
}

/// One line after the first: one folder a person asked to index.
#[derive(Debug, Serialize, Deserialize)]
struct Listed {
    /// The folder, as it was named — from the root.
    folder: PathBuf,
}

/// These folders, as the text of the list's file.
///
/// # Errors
///
/// What `serde_json` said, which for these types is a folder whose path is
/// not text on this host.
pub(crate) fn written(folders: &[PathBuf]) -> Result<String, serde_json::Error> {
    let mut text = serde_json::to_string(&Head { format: FORMAT })?;
    text.push('\n');
    for folder in folders {
        text.push_str(&serde_json::to_string(&Listed {
            folder: folder.clone(),
        })?);
        text.push('\n');
    }
    Ok(text)
}

/// The folders in this text, in the order they were written.
///
/// # Errors
///
/// A sentence saying what was wrong: an empty file, a first line that is not
/// a head, a format this version does not read, or a line that is not a
/// folder's, with its line number.
pub(crate) fn read(text: &str) -> Result<Vec<PathBuf>, String> {
    let mut lines = text.lines();
    let first = lines
        .next()
        .filter(|line| !line.is_empty())
        .ok_or_else(|| "the file is empty".to_owned())?;
    let head: Head = serde_json::from_str(first)
        .map_err(|why| format!("the first line is not a list's: {why}"))?;
    if head.format != FORMAT {
        return Err(format!(
            "format {} is not one this version reads; it reads {FORMAT}",
            head.format
        ));
    }
    lines
        .enumerate()
        .filter(|(_, line)| !line.is_empty())
        .map(|(number, line)| {
            serde_json::from_str::<Listed>(line)
                .map(|listed| listed.folder)
                .map_err(|why| format!("line {} is not a folder's: {why}", number + 2))
        })
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Two folders, for these tests.
    fn two_folders() -> Vec<PathBuf> {
        vec![
            PathBuf::from("/home/ada/Documents"),
            PathBuf::from("/home/ada/Pictures"),
        ]
    }

    /// **The file is what the contract says**, line for line, and comes back
    /// whole and in order; an empty list is a head and nothing else.
    #[test]
    fn the_file_is_one_head_line_and_one_line_per_folder_and_comes_back_whole() {
        let text = written(&two_folders()).unwrap();
        assert_eq!(
            text,
            "{\"format\":1}\n{\"folder\":\"/home/ada/Documents\"}\n{\"folder\":\"/home/ada/Pictures\"}\n"
        );
        assert_eq!(read(&text).unwrap(), two_folders());

        let nothing = written(&[]).unwrap();
        assert_eq!(nothing, "{\"format\":1}\n");
        assert_eq!(read(&nothing).unwrap(), Vec::<PathBuf>::new());
    }

    /// **What is not a list is refused, and the refusal says why**: empty, a
    /// first line that is not a head, a later format, a torn line — and an
    /// index file offered as the list, which has a head and is not one.
    #[test]
    fn what_is_not_a_list_is_refused_with_the_reason() {
        assert_eq!(read("").unwrap_err(), "the file is empty");
        assert!(
            read("hello\n")
                .unwrap_err()
                .starts_with("the first line is not a list's")
        );
        assert_eq!(
            read("{\"format\":2}\n").unwrap_err(),
            "format 2 is not one this version reads; it reads 1"
        );
        let whole = written(&two_folders()).unwrap();
        let (torn, _) = whole.split_at(whole.len() - 4);
        assert!(
            read(torn)
                .unwrap_err()
                .starts_with("line 3 is not a folder's")
        );
        let an_index = "{\"format\":1,\"of\":\"/home/ada/Documents\",\"covered\":{}}\n\
                        {\"below\":\"x\",\"kind\":\"text\"}\n";
        assert!(
            read(an_index)
                .unwrap_err()
                .starts_with("line 2 is not a folder's")
        );
    }

    /// A field this version does not know is ignored, so a later version
    /// that added one still writes a file this one reads.
    #[test]
    fn a_field_from_a_later_version_is_ignored() {
        let text = "{\"format\":1,\"made\":123}\n\
                    {\"folder\":\"/home/ada/Documents\",\"since\":5}\n";
        assert_eq!(
            read(text).unwrap(),
            vec![PathBuf::from("/home/ada/Documents")]
        );
    }
}
