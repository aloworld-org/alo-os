//! The way in: a file read whole, or refused whole.
//!
//! Two clauses of the rule live here. **No file is a person who has changed
//! nothing**, so [`read`] answers [`Kept::untouched`] for it rather than an
//! error. **A file that is there and wrong is refused whole**, so nothing here
//! answers with the keys that did read beside the one that did not.
//!
//! The checks run in the order a person would need them answered. A file that
//! is not TOML says so first; then its `format`, because a file from a release
//! that knows a later format would otherwise be refused for a key that is
//! perfectly good in that format; then any key the shape does not have, named;
//! and only then the values, which the shape's own deserialiser judges.

use std::path::Path;

use crate::kept::Kept;
use crate::unread::Unread;
use crate::writing::THE_FORMAT_KEY;

/// The file at `at`, read.
///
/// # Errors
///
/// [`Kept::not_read`], in the owning crate's words, for a file that is there
/// and did not read — including a relative `at`, which is refused before
/// anything is opened. A file that is not there is not an error: it is
/// [`Kept::untouched`].
pub fn read<K: Kept>(at: &Path) -> Result<K, K::NotRead> {
    as_it_is(at).map_err(|why| K::not_read(at, why))
}

/// The file at `at` as it is on the disk this moment, or why it did not read.
///
/// The one reading of a file there is: [`read`] says its refusal in the owning
/// crate's words, and `crate::replacing` asks it again at the moment of a write.
pub(crate) fn as_it_is<K: Kept>(at: &Path) -> Result<K, Unread> {
    if !at.has_root() {
        return Err(Unread::NotWhereItBelongs);
    }
    let bytes = match std::fs::read(at) {
        Ok(bytes) => bytes,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(K::untouched()),
        Err(why) => return Err(Unread::Disk(why.kind())),
    };
    let text = std::str::from_utf8(&bytes).map_err(|_| Unread::NotText)?;
    from_text(text)
}

/// This text, read as the file at `at` would be.
///
/// For whoever already holds the text — a portal, one day — and for the tests
/// that ask each refusal without a disk. `at` is carried only so that the
/// refusal names the file.
///
/// # Errors
///
/// [`Kept::not_read`] for text that is not this shape, whole.
pub fn read_text<K: Kept>(text: &str, at: &Path) -> Result<K, K::NotRead> {
    from_text(text).map_err(|why| K::not_read(at, why))
}

/// This text as the shape, or why not.
pub(crate) fn from_text<K: Kept>(text: &str) -> Result<K, Unread> {
    let mut table = toml::from_str::<toml::Table>(text).map_err(|why| Unread::NotToml {
        line: why.span().and_then(|span| line_of(text, span.start)),
    })?;
    match table.remove(THE_FORMAT_KEY) {
        Some(toml::Value::Integer(found)) if found == K::FORMAT => {}
        Some(toml::Value::Integer(found)) => return Err(Unread::AnotherFormat { found }),
        Some(_) | None => return Err(Unread::NoFormat),
    }
    if let Some(key) = table.keys().find(|key| !K::KEYS.contains(&key.as_str())) {
        return Err(Unread::UnknownKey { key: key.clone() });
    }
    toml::Value::Table(table)
        .try_into::<K>()
        .map_err(|why| Unread::NotItsShape {
            said: why.message().to_owned(),
        })
}

/// The line, counted from one, that this byte of the text is on.
fn line_of(text: &str, byte: usize) -> Option<usize> {
    let before = text.as_bytes().get(..byte)?;
    Some(before.iter().filter(|b| **b == b'\n').count() + 1)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{Edge, Example, a_folder_of_our_own};

    /// Asked of text alone.
    fn refused(text: &str) -> Unread {
        from_text::<Example>(text).unwrap_err()
    }

    /// **A file with every key the shape has reads as the shape.**
    #[test]
    fn a_file_in_the_shape_reads_as_it() {
        let read: Example = from_text(
            "format = 1\nbackground = \"navy\"\nedge = \"left\"\n\n[displays]\nHDMI-1 = \"sand\"\n",
        )
        .unwrap();
        assert_eq!(read.background.as_deref(), Some("navy"));
        assert_eq!(read.edge, Some(Edge::Left));
        assert_eq!(
            read.displays.get("HDMI-1").map(String::as_str),
            Some("sand")
        );
    }

    /// **A format line and nothing else is a person who has changed nothing.**
    #[test]
    fn a_format_and_nothing_else_is_untouched() {
        assert_eq!(
            from_text::<Example>("format = 1\n").unwrap(),
            Example::untouched()
        );
    }

    /// **Text that is not TOML says where it stopped being TOML**, and nothing
    /// more — no quotation of what somebody typed.
    #[test]
    fn text_that_is_not_toml_names_the_line() {
        assert_eq!(
            refused("format = 1\nbackground = \"navy\"\nedge = \n"),
            Unread::NotToml { line: Some(3) }
        );
    }

    /// **A file with no format is refused**, including one that is empty,
    /// because an empty file is not what a whole write ever leaves.
    #[test]
    fn a_file_with_no_format_is_refused_even_when_empty() {
        assert_eq!(refused(""), Unread::NoFormat);
        assert_eq!(refused("background = \"navy\"\n"), Unread::NoFormat);
        assert_eq!(refused("format = \"1\"\n"), Unread::NoFormat);
    }

    /// **Another format is refused whole**, before its keys are judged, so a
    /// newer release's file is not reported as a typo.
    #[test]
    fn another_format_is_refused_before_its_keys_are_judged() {
        assert_eq!(
            refused("format = 2\nwallpaper = \"navy\"\n"),
            Unread::AnotherFormat { found: 2 }
        );
    }

    /// **A key the shape does not have is named**, and the good key beside it
    /// is not honoured.
    #[test]
    fn a_key_that_is_not_on_the_list_is_named_and_nothing_is_honoured() {
        assert_eq!(
            refused("format = 1\nbackground = \"navy\"\nwallpaper = \"sand\"\n"),
            Unread::UnknownKey {
                key: "wallpaper".to_owned()
            }
        );
    }

    /// **A value the shape refuses refuses the whole file.**
    #[test]
    fn a_value_the_shape_refuses_refuses_the_file() {
        assert!(matches!(
            refused("format = 1\nbackground = \"navy\"\nedge = \"middle\"\n"),
            Unread::NotItsShape { .. }
        ));
    }

    /// **A relative path is refused rather than read**, because it would be a
    /// file wherever the process happened to start.
    #[test]
    fn a_relative_path_is_refused_rather_than_read() {
        let (_, why) = read::<Example>(Path::new("alo/example.toml")).unwrap_err();
        assert_eq!(why, Unread::NotWhereItBelongs);
    }

    /// **Bytes that are not text are refused as that.**
    #[test]
    fn bytes_that_are_not_text_are_refused() {
        let folder = a_folder_of_our_own("reading-not-text");
        let at = folder.join("example.toml");
        std::fs::write(&at, [b'f', 0xff, 0xfe]).unwrap();
        let (named, why) = read::<Example>(&at).unwrap_err();
        assert_eq!(why, Unread::NotText);
        assert_eq!(named, at);
    }

    /// **A folder where the file should be is there and wrong**, rather than a
    /// person who has changed nothing.
    #[test]
    fn a_folder_where_the_file_should_be_is_refused() {
        let folder = a_folder_of_our_own("reading-a-folder");
        let at = folder.join("example.toml");
        std::fs::create_dir(&at).unwrap();
        assert!(matches!(
            read::<Example>(&at).unwrap_err().1,
            Unread::Disk(_)
        ));
    }

    /// Lines are counted from one, and a position past the text has none.
    #[test]
    fn lines_are_counted_from_one() {
        assert_eq!(line_of("a\nb\nc", 0), Some(1));
        assert_eq!(line_of("a\nb\nc", 4), Some(3));
        assert_eq!(line_of("a", 9), None);
    }
}
