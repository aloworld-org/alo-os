//! Reading and writing the bytes DNS-SD is made of.
//!
//! Nothing in this file knows what alo OS is. It is names, integers and
//! records — the alphabet the two files either side of it spell presence in,
//! kept apart because a parser that also knows what it is parsing is a parser
//! whose refusals are hard to find.
//!
//! # Reading a stranger's bytes
//!
//! Every packet here arrived from a network, so every read is bounds-checked
//! and every length is treated as a claim rather than a fact. Two of those
//! checks are worth naming:
//!
//! **A name may point backwards.** DNS keeps packets small by letting a name
//! end in a pointer to a name earlier in the same packet, and a pointer that
//! points at itself — or round a ring — reads forever. [`Packet::name`] counts
//! its steps and refuses, which is the one thing in this file that exists
//! because a packet can be hostile rather than merely truncated.
//!
//! **A length is a claim.** `rdlength` says how long a record's data is, and a
//! record claiming more than the packet holds is refused rather than read up to
//! whatever is there.

use crate::refusing::NotNearby;

/// A pointer's two high bits, which is how a name says it ends elsewhere.
const A_POINTER: u8 = 0b1100_0000;

/// The low bits of a pointer's first byte, which are part of the offset.
const NOT_THE_POINTER_BITS: u8 = 0b0011_1111;

/// How many labels or jumps one name may take before it is not a name.
///
/// A name is at most two hundred and fifty-five bytes of at least two bytes a
/// label, so a hundred and twenty-eight is past anything real and well short of
/// anything slow.
const AT_MOST_STEPS: usize = 128;

/// The longest a single label may be, from the two bits a pointer takes.
const A_LABEL_AT_MOST: usize = 63;

/// The longest a whole name may be.
const A_NAME_AT_MOST: usize = 255;

/// The record types this crate writes and reads.
pub(crate) mod kind {
    /// An address. Never written here; see `advertising.rs` for the argument.
    ///
    /// It is unused outside the test that checks no advertisement carries one,
    /// which is the whole reason the number is written down: a test looking for
    /// a record type should name it rather than spell `1`.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the one record type this crate deliberately never writes"
        )
    )]
    pub(crate) const A: u16 = 1;
    /// A service's instances, which is what a search asks for.
    pub(crate) const PTR: u16 = 12;
    /// A record's keys and values.
    pub(crate) const TXT: u16 = 16;
    /// Where an instance is: a host and a port.
    pub(crate) const SRV: u16 = 33;
}

/// The internet class, which is the only one anything here uses.
pub(crate) const IN: u16 = 1;

/// The class bits with mDNS's cache-flush bit set, which a responder sets on
/// records that are the whole truth about a name.
pub(crate) const IN_AND_THE_ONLY_ONE: u16 = 0x8001;

/// A packet being read, and how far into it the reader is.
pub(crate) struct Packet<'a> {
    /// The whole packet, because a name may point back into any of it.
    bytes: &'a [u8],
    /// How far the reader has got.
    at: usize,
}

impl<'a> Packet<'a> {
    /// A reader at the start of some bytes.
    pub(crate) const fn of(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    /// The next byte.
    fn byte(&mut self) -> Result<u8, NotNearby> {
        let byte = *self.bytes.get(self.at).ok_or(NotNearby::CutShort)?;
        self.at = self.at.saturating_add(1);
        Ok(byte)
    }

    /// The next two bytes, most significant first, as DNS writes every integer.
    pub(crate) fn sixteen(&mut self) -> Result<u16, NotNearby> {
        Ok(u16::from_be_bytes([self.byte()?, self.byte()?]))
    }

    /// The next four bytes, which is a time to live and is read and discarded.
    pub(crate) fn thirty_two(&mut self) -> Result<u32, NotNearby> {
        Ok(u32::from_be_bytes([
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
        ]))
    }

    /// The next `how_many` bytes, refused rather than clamped if the packet is
    /// shorter than it claims.
    pub(crate) fn some(&mut self, how_many: usize) -> Result<&'a [u8], NotNearby> {
        let to = self.at.checked_add(how_many).ok_or(NotNearby::CutShort)?;
        let taken = self.bytes.get(self.at..to).ok_or(NotNearby::CutShort)?;
        self.at = to;
        Ok(taken)
    }

    /// The next name, following any pointer it ends in.
    ///
    /// Labels are taken as they are written rather than lower-cased: what is
    /// done with them is the caller's, and a comparison that should ignore case
    /// should say so where it is made.
    pub(crate) fn name(&mut self) -> Result<String, NotNearby> {
        let mut name = String::new();
        let mut at = self.at;
        let mut jumped = false;
        for step in 0..=AT_MOST_STEPS {
            if step == AT_MOST_STEPS {
                return Err(NotNearby::NameNeverEnds);
            }
            let length = *self.bytes.get(at).ok_or(NotNearby::CutShort)?;
            if length & A_POINTER == A_POINTER {
                let low = *self
                    .bytes
                    .get(at.saturating_add(1))
                    .ok_or(NotNearby::CutShort)?;
                let to = usize::from(u16::from_be_bytes([length & NOT_THE_POINTER_BITS, low]));
                if !jumped {
                    self.at = at.saturating_add(2);
                    jumped = true;
                }
                at = to;
                continue;
            }
            if length == 0 {
                if !jumped {
                    self.at = at.saturating_add(1);
                }
                return Ok(name);
            }
            let length = usize::from(length);
            let from = at.saturating_add(1);
            let to = from.checked_add(length).ok_or(NotNearby::CutShort)?;
            let label = self.bytes.get(from..to).ok_or(NotNearby::CutShort)?;
            if !name.is_empty() {
                name.push('.');
            }
            name.push_str(&String::from_utf8_lossy(label));
            if name.len() > A_NAME_AT_MOST {
                return Err(NotNearby::NameNeverEnds);
            }
            at = to;
        }
        Err(NotNearby::NameNeverEnds)
    }

    /// How far the reader has got, for a caller that must step over a record's
    /// data whether or not it understood it.
    pub(crate) const fn so_far(&self) -> usize {
        self.at
    }

    /// Put the reader at a place the caller worked out, which is how a record
    /// whose data was not read is stepped over.
    ///
    /// # Errors
    ///
    /// [`NotNearby::CutShort`] if that place is past the end of the packet,
    /// which is what a record claiming to be longer than the packet looks like.
    pub(crate) fn go_to(&mut self, at: usize) -> Result<(), NotNearby> {
        if at > self.bytes.len() {
            return Err(NotNearby::CutShort);
        }
        self.at = at;
        Ok(())
    }
}

/// Write a name as DNS writes it: each label behind its length, then a zero.
///
/// No pointers are written. They save bytes in a packet that already fits, and
/// a writer that emits them is a writer whose output has to be read with the
/// whole packet in hand — which the reader here can do and every other
/// responder on the network may not.
///
/// # Errors
///
/// [`NotNearby::NotAName`] for a label over sixty-three bytes or a name over
/// two hundred and fifty-five. Not reachable from an identity this crate made,
/// and refused rather than truncated: a truncated name is a different machine's.
pub(crate) fn write_name(into: &mut Vec<u8>, name: &str) -> Result<(), NotNearby> {
    let mut written = 1_usize;
    for label in name.split('.') {
        let length = label.len();
        if length == 0 || length > A_LABEL_AT_MOST {
            return Err(NotNearby::NotAName(name.to_owned()));
        }
        written = written.saturating_add(length).saturating_add(1);
        if written > A_NAME_AT_MOST {
            return Err(NotNearby::NotAName(name.to_owned()));
        }
        let Ok(length) = u8::try_from(length) else {
            return Err(NotNearby::NotAName(name.to_owned()));
        };
        into.push(length);
        into.extend_from_slice(label.as_bytes());
    }
    into.push(0);
    Ok(())
}

/// Write a two-byte integer, most significant first.
pub(crate) fn write_sixteen(into: &mut Vec<u8>, number: u16) {
    into.extend_from_slice(&number.to_be_bytes());
}

/// Write a four-byte integer, most significant first.
pub(crate) fn write_thirty_two(into: &mut Vec<u8>, number: u32) {
    into.extend_from_slice(&number.to_be_bytes());
}

/// Write a record's data behind the length of it, which is only knowable after
/// the data is written.
///
/// # Errors
///
/// Whatever `writing` refuses, and [`NotNearby::NotAName`] if what was written
/// is longer than a record's length field can say.
pub(crate) fn write_data(
    into: &mut Vec<u8>,
    of: &str,
    writing: impl FnOnce(&mut Vec<u8>) -> Result<(), NotNearby>,
) -> Result<(), NotNearby> {
    let mut data = Vec::new();
    writing(&mut data)?;
    let Ok(length) = u16::try_from(data.len()) else {
        return Err(NotNearby::NotAName(of.to_owned()));
    };
    write_sixteen(into, length);
    into.extend_from_slice(&data);
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::{NotNearby, Packet, write_name};

    /// A name this crate writes is a name it reads back.
    #[test]
    fn a_name_written_here_is_read_back_as_it_was() {
        let mut bytes = Vec::new();
        write_name(&mut bytes, "0f1e2d3c._alo-os._tcp.local").unwrap();
        assert_eq!(
            Packet::of(&bytes).name().unwrap(),
            "0f1e2d3c._alo-os._tcp.local"
        );
    }

    /// **A name that points at itself does not read forever.** The one refusal
    /// here that exists because a packet can be hostile rather than truncated.
    #[test]
    fn a_name_pointing_at_itself_is_refused_rather_than_read_forever() {
        // Two bytes at offset zero: a pointer back to offset zero.
        let round_and_round = [0b1100_0000_u8, 0];
        assert_eq!(
            Packet::of(&round_and_round).name().unwrap_err(),
            NotNearby::NameNeverEnds
        );
    }

    /// A pointer round a ring of two names is the same refusal.
    #[test]
    fn a_ring_of_names_is_refused_too() {
        // Offset 0 points to 2; offset 2 points to 0.
        let ring = [0b1100_0000_u8, 2, 0b1100_0000, 0];
        assert_eq!(
            Packet::of(&ring).name().unwrap_err(),
            NotNearby::NameNeverEnds
        );
    }

    /// A label claiming more bytes than the packet holds is refused rather than
    /// read up to whatever is there.
    #[test]
    fn a_label_longer_than_the_packet_is_refused() {
        let claiming = [9_u8, b'a', b'b'];
        assert_eq!(
            Packet::of(&claiming).name().unwrap_err(),
            NotNearby::CutShort
        );
    }

    /// A name that never ends because the packet does is the same.
    #[test]
    fn a_name_with_no_end_is_refused() {
        let never = [1_u8, b'a', 1, b'b'];
        assert_eq!(Packet::of(&never).name().unwrap_err(), NotNearby::CutShort);
    }

    /// A label over sixty-three bytes is refused on the way out rather than
    /// truncated into a different machine's name.
    #[test]
    fn a_label_too_long_to_write_is_refused_rather_than_shortened() {
        let mut bytes = Vec::new();
        let too_long = "a".repeat(64);
        assert_eq!(
            write_name(&mut bytes, &too_long).unwrap_err(),
            NotNearby::NotAName(too_long)
        );
        assert!(
            bytes.is_empty(),
            "half a name was written before the refusal: {bytes:?}"
        );
    }

    /// An empty label — two dots together, or a trailing dot — is not a name.
    #[test]
    fn a_name_with_an_empty_label_is_refused() {
        let mut bytes = Vec::new();
        assert_eq!(
            write_name(&mut bytes, "a..b").unwrap_err(),
            NotNearby::NotAName("a..b".to_owned())
        );
    }
}
