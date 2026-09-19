//! What a file that begins with an ISO base media header says it is — read
//! from the brands it names, and never from the header alone.
//!
//! The four bytes `ftyp` at offset four are the start of a family, not of a
//! format. The same header begins the container MP4 names, QuickTime's, the
//! one 3GP uses, the one a photograph from a telephone is saved in and the one
//! a raw picture from a camera is saved in. What tells them apart is the
//! **brands** the header goes on to name: a major brand, then a list of every
//! other brand the file claims to be compatible with.
//!
//! # Why the list is an allowance rather than a refusal
//!
//! This module names the brands that mean **the container MP4 names**, and a
//! file whose brands are none of them is not claimed to be anything. The
//! opposite shape — treat every `ftyp` file as a film unless its brand is one
//! of a list of exceptions — reads the same until somebody is sent a format
//! nobody here has heard of, and then it says *this is an MP4 video* about a
//! file that is not one. That is what this machine did until 2026-09-19: a
//! photograph from a telephone was reported as a film, which is a worse answer
//! than *this machine does not recognise what this file is*, because a person
//! acts on it.
//!
//! An allowance is wrong in the direction a person can work with. A film in a
//! brand nobody listed here reads as *not recognised* — honest about this
//! machine, and one line in [`THE_MP4_FAMILY`] away from being named.
//!
//! # It reads the header and no further
//!
//! The whole brand list is inside the first box of the file, which is inside
//! the [`crate::THE_HEAD`] bytes already read. Nothing here asks for another
//! byte, and a header that says it is longer than what was read is honoured
//! only as far as it was read — a file cut off inside its own first box has
//! named the brands it named.

/// Every brand that means the container MP4 names — including the names
/// QuickTime and 3GP write, which are the same container under other titles
/// and hold the films people are actually sent.
///
/// A brand is four bytes and is padded with spaces, which is why several end
/// in one. They are compared exactly, including case: `M4A ` is a sound
/// recording and `m4a ` is not a brand anybody writes.
pub(crate) const THE_MP4_FAMILY: [&[u8; 4]; 27] = [
    // The ISO base media brands themselves, which almost every file in this
    // family lists among its compatible brands even when its major brand is
    // something more particular.
    b"isom", b"iso2", b"iso4", b"iso5", b"iso6", b"iso8", b"iso9",
    // What MP4 itself is written as.
    b"mp41", b"mp42", b"mp71", b"avc1", b"dash", b"cmfc", b"mmp4",
    // What Apple writes: a film, a film with sound alone, and QuickTime's own.
    b"M4V ", b"M4VH", b"M4VP", b"M4A ", b"M4B ", b"M4P ", b"qt  ",
    // What a telephone recording a film writes.
    b"3gp4", b"3gp5", b"3gp6", b"3gp7", b"3g2a", b"3g2b",
];

/// What a file beginning with this header is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Iso {
    /// A film in the container MP4 names.
    AFilm,
    /// Sound with no picture, in that same container.
    ASoundRecording,
    /// An ISO base media file whose brands name none of the family above, so
    /// nothing here says what it is.
    NotOneOfOurs,
}

/// Whether a file's first bytes are an ISO base media header.
pub(crate) fn is_the_header(head: &[u8]) -> bool {
    head.get(4..8) == Some(&b"ftyp"[..])
}

/// What the brands in this header say the file is.
///
/// `head` begins at the start of the file and has already been checked by
/// [`is_the_header`].
pub(crate) fn look(head: &[u8]) -> Iso {
    let major = head.get(8..12).unwrap_or_default();
    if !brands(head).any(|brand| THE_MP4_FAMILY.iter().any(|named| named.as_slice() == brand)) {
        return Iso::NotOneOfOurs;
    }
    // Sound with no picture is saved in the same container under its own major
    // brand, and that is the one distinction worth drawing: a person is shown a
    // sound recording rather than a film with nothing to see. The major brand
    // alone decides it, because a film may perfectly well list `M4A ` among the
    // formats it is compatible with.
    if major == b"M4A " || major == b"M4B " || major == b"M4P " {
        Iso::ASoundRecording
    } else {
        Iso::AFilm
    }
}

/// Every brand the header names: the major brand, then the compatible brands
/// after the minor version, as far as the header was read.
///
/// The box's own length says where the list ends. A length that runs past what
/// was read is taken as what was read, and a length that stops inside the fixed
/// part leaves nothing to iterate — both are files that are not going to be
/// named here, and neither is a reason to read further.
fn brands(head: &[u8]) -> impl Iterator<Item = &[u8]> {
    /// Where the compatible brands begin: length, `ftyp`, major brand, minor
    /// version.
    const AFTER_THE_FIXED_PART: usize = 16;

    let said = u32::from_be_bytes([
        *head.first().unwrap_or(&0),
        *head.get(1).unwrap_or(&0),
        *head.get(2).unwrap_or(&0),
        *head.get(3).unwrap_or(&0),
    ]);
    let ends = usize::try_from(said)
        .unwrap_or(usize::MAX)
        .min(head.len())
        .max(AFTER_THE_FIXED_PART.min(head.len()));
    let compatible = head.get(AFTER_THE_FIXED_PART..ends).unwrap_or_default();
    let (whole, _) = compatible.as_chunks::<4>();
    head.get(8..12)
        .into_iter()
        .chain(whole.iter().map(<[u8; 4]>::as_slice))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A header naming this major brand and these compatible ones.
    fn a_header(major: &[u8; 4], compatible: &[&[u8; 4]]) -> Vec<u8> {
        let length = 16 + compatible.len() * 4;
        let mut head = Vec::new();
        head.extend_from_slice(&u32::try_from(length).unwrap_or(0).to_be_bytes());
        head.extend_from_slice(b"ftyp");
        head.extend_from_slice(major);
        head.extend_from_slice(&[0, 0, 0, 0]);
        for brand in compatible {
            head.extend_from_slice(*brand);
        }
        head
    }

    /// **A film in the family is a film**, whether the brand that says so is
    /// the major one or one of the compatible ones.
    #[test]
    fn a_film_in_the_family_is_a_film() {
        assert_eq!(look(&a_header(b"mp42", &[b"isom", b"avc1"])), Iso::AFilm);
        assert_eq!(look(&a_header(b"isom", &[])), Iso::AFilm);
        assert_eq!(look(&a_header(b"qt  ", &[])), Iso::AFilm);
        assert_eq!(look(&a_header(b"3gp4", &[b"isom"])), Iso::AFilm);
        // A brand nobody here lists, on a file that also says it is one we do.
        assert_eq!(look(&a_header(b"XAVC", &[b"isom", b"mp42"])), Iso::AFilm);
    }

    /// **Sound with no picture is told apart by the major brand**, and a film
    /// that merely claims to be compatible with it is still a film.
    #[test]
    fn sound_with_no_picture_is_told_apart_by_the_major_brand() {
        assert_eq!(look(&a_header(b"M4A ", &[b"isom"])), Iso::ASoundRecording);
        assert_eq!(look(&a_header(b"M4B ", &[])), Iso::ASoundRecording);
        assert_eq!(look(&a_header(b"mp42", &[b"M4A "])), Iso::AFilm);
    }

    /// **A file whose brands are none of ours is not claimed to be one of
    /// ours.** The refusal this module exists for: a photograph from a
    /// telephone, a picture in the format a web page sends, and a brand nobody
    /// has heard of are all *not one of ours* rather than *a film*.
    #[test]
    fn brands_that_are_none_of_ours_are_not_claimed() {
        for major in [b"heic", b"heix", b"mif1", b"avif", b"crx ", b"jp2 "] {
            assert_eq!(look(&a_header(major, &[])), Iso::NotOneOfOurs, "{major:?}");
        }
        assert_eq!(
            look(&a_header(b"heic", &[b"mif1", b"miaf", b"MiHB"])),
            Iso::NotOneOfOurs
        );
    }

    /// **A header that does not hold together names only what it named.** A
    /// length longer than the file, a length inside the fixed part, and a
    /// header cut off mid-brand each read what is there and claim nothing more.
    #[test]
    fn a_header_that_does_not_hold_together_claims_nothing_more() {
        let mut said_too_long = a_header(b"heic", &[b"mif1"]);
        said_too_long.splice(..4, [0xff, 0xff, 0xff, 0xff]);
        assert_eq!(look(&said_too_long), Iso::NotOneOfOurs);

        let mut said_too_long = a_header(b"mp42", &[]);
        said_too_long.splice(..4, [0xff, 0xff, 0xff, 0xff]);
        assert_eq!(look(&said_too_long), Iso::AFilm);

        // A length that stops inside the fixed part: the major brand is still
        // the major brand.
        let mut said_too_short = a_header(b"isom", &[b"mp42"]);
        said_too_short.splice(..4, [0, 0, 0, 8]);
        assert_eq!(look(&said_too_short), Iso::AFilm);

        // Cut off part way through a compatible brand.
        let mut cut = a_header(b"heic", &[b"isom"]);
        cut.truncate(18);
        assert_eq!(look(&cut), Iso::NotOneOfOurs);

        // Nothing after `ftyp` at all.
        assert!(is_the_header(b"\0\0\0\x18ftyp"));
        assert_eq!(look(b"\0\0\0\x18ftyp"), Iso::NotOneOfOurs);
    }

    /// **Every brand is four bytes and each is listed once**, because a table
    /// compared exactly is a table a typing mistake makes silently smaller.
    #[test]
    fn every_brand_is_four_bytes_and_listed_once() {
        let mut named: Vec<&[u8; 4]> = THE_MP4_FAMILY.to_vec();
        named.sort_unstable();
        named.dedup();
        assert_eq!(named.len(), THE_MP4_FAMILY.len());
    }
}
