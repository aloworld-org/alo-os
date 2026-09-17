//! Reading `text/uri-list`, which is how every desktop drags files.
//!
//! `alo-clipboard` names the form — [`alo_clipboard::Form::Files`] is
//! `text/uri-list` and nothing else — and stops there, because a paste hands
//! the bytes to whoever asked and never looks inside them. A drop cannot stop
//! there: **which files are being dropped decides how they are delivered**, and
//! a sandboxed application is handed each of them through the documents portal
//! rather than a path it could not open. So this is the one place in either
//! crate that reads a payload, and it reads exactly one form.
//!
//! # It arrives from a client, so nothing in it is believed
//!
//! The bytes are written by the application the person dragged from. Every line
//! is checked and none is repaired: a list this cannot read is refused whole
//! ([`NotAFileList`]) rather than partly delivered, because delivering the three
//! files it understood out of four would be a drop that silently lost one.
//!
//! What is checked is what would otherwise become a path handed to a portal: it
//! is a `file:` URI, it is on **this** machine, it is absolute, it has no `..`
//! step in it, and its percent-escapes are escapes. A list longer than
//! [`THE_LONGEST_LIST`] is refused by its length before any of that.
//!
//! # A URI that is not a file is carried and not refused
//!
//! Dragging a link out of a browser puts `https://…` on a `text/uri-list`, and
//! it is a perfectly good drop with no file in it. Those lines are carried to
//! the target as they are and simply contribute no file to export — a list with
//! no `file:` URI in it is delivered [`crate::Delivery::OnTheSpot`]. What is
//! refused is a line that is not a URI at all, because then this crate cannot
//! say what it is looking at.

use std::path::{Component, PathBuf};

/// The most bytes of `text/uri-list` that are read.
///
/// A person dragging a folder's worth of files puts a few hundred bytes up;
/// sixty-four kilobytes is far past any real selection and is a bound rather
/// than a budget. What is past it is refused by name, not truncated into a
/// shorter drop.
pub const THE_LONGEST_LIST: usize = 64 * 1024;

/// What a `file:` URI may name as its host: nothing, or this machine.
const THIS_MACHINE: [&str; 2] = ["", "localhost"];

/// Every file named by a `text/uri-list`, in the order it names them.
///
/// URIs of any other scheme are not files and are skipped; the answer is empty
/// rather than an error when a list holds only those.
///
/// # Errors
/// [`NotAFileList`], naming what could not be read. Nothing is delivered on one:
/// a partly-read list would be a drop that lost a file without saying so.
pub fn files_in(bytes: &[u8]) -> Result<Vec<PathBuf>, NotAFileList> {
    if bytes.len() > THE_LONGEST_LIST {
        return Err(NotAFileList::TooLong {
            how_long: bytes.len(),
        });
    }
    let written = std::str::from_utf8(bytes).map_err(|_| NotAFileList::NotText)?;
    let mut files = Vec::new();
    for line in written.lines() {
        let line = line.trim();
        // The specification's own comment marker, and the blank line between
        // two lists that a toolkit sometimes leaves behind.
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(rest) = line.strip_prefix("file://") else {
            if line.contains("://") {
                // A link, not a file. Carried; nothing to export.
                continue;
            }
            return Err(NotAFileList::NotAUri {
                line: line.to_owned(),
            });
        };
        files.push(file_at(rest)?);
    }
    Ok(files)
}

/// The path a `file://` URI names, given everything after the two slashes.
fn file_at(rest: &str) -> Result<PathBuf, NotAFileList> {
    let Some((host, tail)) = rest.split_once('/') else {
        return Err(NotAFileList::NotAFullPath {
            path: rest.to_owned(),
        });
    };
    if !THIS_MACHINE.contains(&host) {
        return Err(NotAFileList::NotOnThisMachine {
            host: host.to_owned(),
        });
    }
    let path = PathBuf::from(decoded(&format!("/{tail}"))?);
    if !path.has_root() {
        return Err(NotAFileList::NotAFullPath {
            path: path.to_string_lossy().into_owned(),
        });
    }
    if path.components().any(|step| step == Component::ParentDir) {
        return Err(NotAFileList::CouldLeadElsewhere {
            path: path.to_string_lossy().into_owned(),
        });
    }
    Ok(path)
}

/// One URI's percent-escapes turned back into the characters they stand for.
///
/// `%20` is a space in a filename and `%C3%BC` is the `ü` in *Müller*, so a
/// list read without this would name files that are not there — which is the
/// failure that looks like a broken drop and is a decoding bug.
fn decoded(written: &str) -> Result<String, NotAFileList> {
    let mut out: Vec<u8> = Vec::with_capacity(written.len());
    let mut letters = written.chars();
    let mut buffer = [0_u8; 4];
    while let Some(letter) = letters.next() {
        if letter != '%' {
            out.extend_from_slice(letter.encode_utf8(&mut buffer).as_bytes());
            continue;
        }
        let (Some(high), Some(low)) = (letters.next(), letters.next()) else {
            return Err(NotAFileList::NotAnEscape {
                escape: "%".to_owned(),
            });
        };
        let (Some(high), Some(low)) = (high.to_digit(16), low.to_digit(16)) else {
            return Err(NotAFileList::NotAnEscape {
                escape: format!("%{high}{low}"),
            });
        };
        let Ok(byte) = u8::try_from(high * 16 + low) else {
            return Err(NotAFileList::NotAnEscape {
                escape: format!("%{high:x}{low:x}"),
            });
        };
        if byte == 0 {
            return Err(NotAFileList::NotAnEscape {
                escape: "%00".to_owned(),
            });
        }
        out.push(byte);
    }
    String::from_utf8(out).map_err(|_| NotAFileList::NotText)
}

/// Why what an application put up as a list of files is not one.
///
/// Not a refusal anybody reads in their own language, for
/// [`alo_clipboard::KindError`]'s reason: what has gone wrong is in an
/// application, and the person is not part of that conversation. What the person
/// reads is [`crate::NotHanded::NotAFileList`], one sentence, with this kept
/// beside it for whoever is fixing the application.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAFileList {
    /// More bytes than any real selection of files.
    #[error(
        "a list of {how_long} bytes is longer than the {THE_LONGEST_LIST} a drop carries: no \
         selection of files is this long, and nothing in it has been read"
    )]
    TooLong {
        /// How long the list was.
        how_long: usize,
    },
    /// Not UTF-8, which `text/uri-list` is.
    #[error("a list of files is text, and these bytes are not text at all")]
    NotText,
    /// A line that is not a URI of any scheme.
    #[error("`{line}` is not a URI, so there is no saying what file it was meant to name")]
    NotAUri {
        /// What was written on the line.
        line: String,
    },
    /// A `file:` URI naming another machine.
    #[error(
        "`{host}` is another machine, and a file on it is not a file this one can hand over — \
         a machine reaches another only under a pairing made on both (ADR 0003)"
    )]
    NotOnThisMachine {
        /// The host the URI named.
        host: String,
    },
    /// A `file:` URI with no absolute path in it.
    #[error("`{path}` is not a full path, so it does not name the same file wherever it is read")]
    NotAFullPath {
        /// What the URI named.
        path: String,
    },
    /// A path with a `..` step in it, which could name a file outside whatever
    /// was dragged.
    #[error("`{path}` has a step upwards in it, so what it names depends on where it is read from")]
    CouldLeadElsewhere {
        /// What the URI named.
        path: String,
    },
    /// A percent-escape that is not one.
    #[error("`{escape}` is not an escape: a percent sign in a URI is followed by two hex digits")]
    NotAnEscape {
        /// What was written instead.
        escape: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ordinary list: two files, in the order the application wrote them.
    #[test]
    fn a_list_of_files_is_read_in_the_order_it_was_written() {
        let list = b"file:///home/anna/march.pdf\r\nfile:///home/anna/april.pdf\r\n";
        assert_eq!(
            files_in(list),
            Ok(vec![
                PathBuf::from("/home/anna/march.pdf"),
                PathBuf::from("/home/anna/april.pdf"),
            ])
        );
    }

    /// **`Müller` and a space are test cases in a European product.** A list read
    /// without its escapes would name files that are not there.
    #[test]
    fn escapes_are_the_characters_they_stand_for() {
        let list = "file:///home/anna/M%C3%BCller%20-%20Rechnung.pdf".as_bytes();
        assert_eq!(
            files_in(list),
            Ok(vec![PathBuf::from("/home/anna/Müller - Rechnung.pdf")])
        );
    }

    /// Comments and blank lines are the specification's, and a link is a drop
    /// with no file in it rather than a refusal.
    #[test]
    fn comments_blank_lines_and_links_leave_no_file_behind() {
        let list = b"# what was dragged\n\nhttps://example.org/a\nfile:///home/anna/a.txt\n";
        assert_eq!(files_in(list), Ok(vec![PathBuf::from("/home/anna/a.txt")]));
        assert_eq!(files_in(b"https://example.org/a\n"), Ok(Vec::new()));
        assert_eq!(files_in(b""), Ok(Vec::new()));
    }

    /// Every way a list is refused, and each refuses the **whole** list: a drop
    /// that delivered three of four files would have lost one silently.
    #[test]
    fn a_list_this_cannot_read_is_refused_whole() {
        assert_eq!(
            files_in(&vec![b'a'; THE_LONGEST_LIST + 1]),
            Err(NotAFileList::TooLong {
                how_long: THE_LONGEST_LIST + 1
            })
        );
        assert_eq!(files_in(&[0xff, 0xfe]), Err(NotAFileList::NotText));
        assert_eq!(
            files_in(b"file:///home/anna/a.txt\nnot a uri\n"),
            Err(NotAFileList::NotAUri {
                line: "not a uri".to_owned()
            })
        );
        assert_eq!(
            files_in(b"file://otherbox/home/anna/a.txt"),
            Err(NotAFileList::NotOnThisMachine {
                host: "otherbox".to_owned()
            })
        );
        assert_eq!(
            files_in(b"file://home"),
            Err(NotAFileList::NotAFullPath {
                path: "home".to_owned()
            })
        );
        assert_eq!(
            files_in(b"file:///home/anna/../../etc/shadow"),
            Err(NotAFileList::CouldLeadElsewhere {
                path: "/home/anna/../../etc/shadow".to_owned()
            })
        );
        assert_eq!(
            files_in(b"file:///home/anna/a%zz.txt"),
            Err(NotAFileList::NotAnEscape {
                escape: "%zz".to_owned()
            })
        );
        assert_eq!(
            files_in(b"file:///home/anna/a%2"),
            Err(NotAFileList::NotAnEscape {
                escape: "%".to_owned()
            })
        );
        assert_eq!(
            files_in(b"file:///home/anna/a%00b"),
            Err(NotAFileList::NotAnEscape {
                escape: "%00".to_owned()
            })
        );
    }
}
