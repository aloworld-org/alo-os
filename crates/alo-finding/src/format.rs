//! The index as a file: one line saying what it is, then one line per entry.
//!
//! `docs/contracts/file-index.md` is the public description; this is it as
//! code. The shape is the record's, for the record's reasons: every line is
//! compact JSON with no newline inside it, the first line carries `format`
//! and nothing after it does, and a reader that is not alo OS needs a JSON
//! parser and nothing else.
//!
//! # A torn file is refused whole
//!
//! The file is replaced whole by `keeping.rs`, so a torn one is not something
//! an interrupted write leaves behind — but a disk can still hand back half a
//! file, and a half-read index would answer *nothing matched* about the half
//! that was lost. So a line that does not parse refuses the file, and the
//! caller indexes again.
//!
//! # Later versions may add, and this reader lets them
//!
//! A field this version does not know is ignored on the way in, so an index
//! written by a version that added one is still read by this one. A `format`
//! this version does not know is refused, because a changed meaning of an
//! existing field is exactly what a new number is for.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::covered::Covered;
use crate::entry::{Entry, Moment};
use crate::index::Index;

/// Which shape the file is in.
const FORMAT: u32 = 1;

/// The first line.
#[derive(Debug, Serialize, Deserialize)]
struct Head {
    /// Which shape the file is in; what tells a head from an entry.
    format: u32,
    /// The folder this is the index of.
    of: PathBuf,
    /// The moment the index was made, as the caller said it. Added after
    /// the first indexes were written, so it is left out rather than written
    /// as nothing, and a head without it still reads — which is what
    /// `docs/contracts/file-index.md` says a later field does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    made: Option<Moment>,
    /// What the walk under it could not reach.
    covered: Covered,
}

/// This index, as the text of its file.
///
/// # Errors
///
/// What `serde_json` said, which for these types is a folder whose path is
/// not text on this host.
pub(crate) fn written(index: &Index) -> Result<String, serde_json::Error> {
    let head = Head {
        format: FORMAT,
        of: index.of.clone(),
        made: index.made,
        covered: index.covered.clone(),
    };
    let mut text = serde_json::to_string(&head)?;
    text.push('\n');
    for entry in &index.entries {
        text.push_str(&serde_json::to_string(entry)?);
        text.push('\n');
    }
    Ok(text)
}

/// The index in this text.
///
/// # Errors
///
/// A sentence saying what was wrong: an empty file, a first line that is not
/// a head, a format this version does not read, or an entry that does not
/// parse, with its line number.
pub(crate) fn read(text: &str) -> Result<Index, String> {
    let mut lines = text.lines();
    let first = lines
        .next()
        .filter(|line| !line.is_empty())
        .ok_or_else(|| "the file is empty".to_owned())?;
    let head: Head = serde_json::from_str(first)
        .map_err(|why| format!("the first line is not an index's: {why}"))?;
    if head.format != FORMAT {
        return Err(format!(
            "format {} is not one this version reads; it reads {FORMAT}",
            head.format
        ));
    }
    let entries = lines
        .enumerate()
        .filter(|(_, line)| !line.is_empty())
        .map(|(number, line)| {
            serde_json::from_str::<Entry>(line)
                .map_err(|why| format!("line {} is not an entry: {why}", number + 2))
        })
        .collect::<Result<Vec<Entry>, String>>()?;
    Ok(Index {
        of: head.of,
        made: head.made,
        covered: head.covered,
        entries,
        opened: 0,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::entry::{Contents, Moment};
    use crate::kind::Kind;

    /// An index for these tests.
    fn an_index() -> Index {
        Index {
            of: PathBuf::from("/home/ada/Documents"),
            made: Some(Moment {
                secs: 1_760_000_000,
                nanos: 0,
            }),
            covered: Covered {
                whole: true,
                most: 20_000,
                unread: Vec::new(),
                elsewhere: vec!["Elsewhere".to_owned()],
                not_entered: Vec::new(),
                unnamed: 1,
            },
            entries: vec![
                Entry {
                    below: "2026".to_owned(),
                    kind: Kind::Folder,
                    bytes: 0,
                    modified: Moment { secs: 5, nanos: 0 },
                    contents: Contents::NotAFile,
                },
                Entry {
                    below: "2026/march.pdf".to_owned(),
                    kind: Kind::Text,
                    bytes: 10,
                    modified: Moment { secs: 7, nanos: 1 },
                    contents: Contents::Read {
                        words: vec!["an".to_owned(), "invoice".to_owned()],
                    },
                },
            ],
            opened: 2,
        }
    }

    /// **The file is what the contract says**, line for line, and comes back
    /// whole — with `opened` not in it, because how many files were read is
    /// about the indexing and not the index.
    #[test]
    fn the_file_is_one_head_line_and_one_line_per_entry_and_comes_back_whole() {
        let text = written(&an_index()).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines.first().copied().unwrap(),
            r#"{"format":1,"of":"/home/ada/Documents","made":{"secs":1760000000,"nanos":0},"covered":{"whole":true,"most":20000,"unread":[],"elsewhere":["Elsewhere"],"not_entered":[],"unnamed":1}}"#
        );
        assert_eq!(
            lines.get(2).copied().unwrap(),
            r#"{"below":"2026/march.pdf","kind":"text","bytes":10,"modified":{"secs":7,"nanos":1},"contents":{"were":"read","words":["an","invoice"]}}"#
        );
        assert!(text.ends_with('\n'));

        let back = read(&text).unwrap();
        assert_eq!(back.of, an_index().of);
        assert_eq!(
            back.made,
            an_index().made,
            "the moment comes back as it was said"
        );
        assert_eq!(back.covered, an_index().covered);
        assert_eq!(back.entries, an_index().entries);
        assert_eq!(back.opened, 0);
    }

    /// **What is not an index is refused, and the refusal says why**: empty,
    /// a first line that is not a head, a later format, a torn entry.
    #[test]
    fn what_is_not_an_index_is_refused_with_the_reason() {
        assert_eq!(read("").unwrap_err(), "the file is empty");
        assert!(
            read("hello\n")
                .unwrap_err()
                .starts_with("the first line is not an index's")
        );
        assert!(
            read(r#"{"below":"x","kind":"text","bytes":0,"modified":{"secs":0,"nanos":0},"contents":{"were":"not-text"}}"#)
                .unwrap_err()
                .starts_with("the first line is not an index's"),
            "an entry is not a head"
        );
        let later = written(&an_index())
            .unwrap()
            .replacen("\"format\":1", "\"format\":2", 1);
        assert_eq!(
            read(&later).unwrap_err(),
            "format 2 is not one this version reads; it reads 1"
        );
        let whole = written(&an_index()).unwrap();
        let (torn, _) = whole.split_at(whole.len() - 20);
        assert!(
            read(torn)
                .unwrap_err()
                .starts_with("line 3 is not an entry")
        );
    }

    /// A field this version does not know is ignored, so a later version
    /// that added one still writes a file this one reads.
    #[test]
    fn a_field_from_a_later_version_is_ignored() {
        let text = written(&an_index())
            .unwrap()
            .replacen("\"format\":1,", "\"format\":1,\"checked\":123,", 1)
            .replacen(
                "\"kind\":\"text\",",
                "\"kind\":\"text\",\"colour\":\"blue\",",
                1,
            );
        let back = read(&text).unwrap();
        assert_eq!(back.entries, an_index().entries);
    }

    /// **An index written before the moment was kept still reads**, with no
    /// moment rather than an invented one: `made` was added to the head after
    /// the first indexes were written, and the contract says a reader takes a
    /// head without a later field. And an index with no moment is written
    /// with the field left out, not as `null`, so the file is the one an
    /// earlier version wrote.
    #[test]
    fn a_head_without_the_moment_still_reads_and_is_written_without_it() {
        let earlier = r#"{"format":1,"of":"/home/ada/Documents","covered":{"whole":true,"most":20000,"unread":[],"elsewhere":[],"not_entered":[],"unnamed":0}}"#;
        let back = read(earlier).unwrap();
        assert_eq!(back.made, None);
        assert_eq!(back.of, PathBuf::from("/home/ada/Documents"));

        let text = written(&back).unwrap();
        assert_eq!(text.lines().next().unwrap(), earlier);
        assert!(!text.contains("made"));
    }
}
