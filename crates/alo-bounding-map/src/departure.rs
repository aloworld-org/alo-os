//! Where one turn may connect to, and how many of those there are.
//!
//! [`crate::Place`] is somewhere on a disk; a [`Departure`] is somewhere on a
//! network. They sit in the same entry of the same map because they are the
//! same question asked twice — *what may this turn reach* — and because a third
//! map would change what `alo-bounding`'s
//! `the_program_has_nowhere_to_write_what_it_sees` asserts. That test names the
//! two maps exactly, on the argument that *a program that had somewhere to
//! write would have to have somewhere, and this is the list of everywhere it
//! has*. Growing a value the daemon already writes keeps it true as written.
//!
//! # One departure is one destination, and that is the whole point
//!
//! A person shown *asking alo, in Frankfurt* has been shown one place. If the
//! permission that follows were *this turn may now open sockets*, the sentence
//! they read would have authorised every address on the internet, and the
//! indicator would have become a thing that happens near a connection rather
//! than a thing that describes it.
//!
//! So a departure is an **address and a port**. A turn reaching an address
//! nobody showed is refused even while another departure of its own is open.
//!
//! # Both address families, in one shape
//!
//! An address is 128 bits because that is what the wider of the two is. An IPv4
//! address is the 32 bits it is, in the low end, with the family beside it —
//! kept rather than inferred, because `::1.2.3.4` and `1.2.3.4` are different
//! destinations and a shape that could not tell them apart would let one stand
//! for the other.

/// One of the two ways an address is written.
///
/// The numbers are the kernel's own `AF_INET` and `AF_INET6`, because the
/// programme reads them out of a `sockaddr` and comparing against anything else
/// would mean converting on the hot path for no reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// `AF_INET`, and the address is the low 32 bits.
    Four,

    /// `AF_INET6`, and the address is all 128.
    Six,
}

impl Family {
    /// `AF_INET` as this machine's kernel numbers it.
    pub const INET: u16 = 2;

    /// `AF_INET6` as this machine's kernel numbers it.
    pub const INET6: u16 = 10;

    /// Which family a number names, or [`None`] for one this does not enforce.
    ///
    /// **[`None`] is not *allow*.** A family the programme cannot read the
    /// address of is one it cannot check a destination against, and
    /// `alo-bounding-kernel` refuses those — a Unix socket, a netlink socket and
    /// anything else are not destinations this bound can describe, so a turn
    /// does not reach them.
    #[must_use]
    pub const fn of(number: u16) -> Option<Self> {
        match number {
            Self::INET => Some(Self::Four),
            Self::INET6 => Some(Self::Six),
            _ => None,
        }
    }

    /// The number the kernel writes.
    #[must_use]
    pub const fn number(self) -> u16 {
        match self {
            Self::Four => Self::INET,
            Self::Six => Self::INET6,
        }
    }
}

/// Somewhere one turn was shown it was about to connect to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Departure {
    /// The address, in host order, with an IPv4 one in the low 32 bits.
    address: u128,

    /// The port, in host order.
    port: u16,

    /// Which family the address is, as the kernel's own number.
    ///
    /// Kept raw rather than as a [`Family`], for one reason worth the
    /// awkwardness: **a slot nothing is looking at is then all zeroes**. Zero is
    /// not a family any kernel names, so an empty slot reads back as an empty
    /// slot, and an entry the daemon has not filled is not accidentally a
    /// destination. A value that had to carry a real family to mean *nowhere*
    /// would make *nowhere* a place somebody could arrange to be shown.
    family: u16,
}

impl Departure {
    /// A destination somebody was shown.
    #[must_use]
    pub const fn of(family: Family, address: u128, port: u16) -> Self {
        Self {
            address,
            port,
            family: family.number(),
        }
    }

    /// The address.
    #[must_use]
    pub const fn address(&self) -> u128 {
        self.address
    }

    /// The port.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Which family the address is, or [`None`] for a number no kernel names —
    /// which is what an empty slot and an unreadable one both are.
    #[must_use]
    pub const fn family(&self) -> Option<Family> {
        Family::of(self.family)
    }

    /// The value as the map holds it: the address in two words, then the family
    /// and the port packed into a third.
    ///
    /// The order is the only thing keeping two separately compiled programs
    /// talking about the same destination, so it is decided here and nowhere
    /// else.
    #[must_use]
    pub const fn words(&self) -> [u64; WORDS_EACH] {
        [
            (self.address >> 64) as u64,
            self.address as u64,
            ((self.family as u64) << 16) | self.port as u64,
        ]
    }

    /// A destination read back out of the map.
    ///
    /// Cannot fail, and does not try to. A family number the map does not name
    /// is kept as it was found: every destination a caller can *construct* has
    /// one of the two real families, so a nonsense one matches nothing, which
    /// is the direction a value read out of shared memory has to fail in.
    #[must_use]
    pub const fn of_words(words: [u64; WORDS_EACH]) -> Self {
        let [high, low, both] = words;
        Self {
            address: ((high as u128) << 64) | low as u128,
            port: both as u16,
            family: (both >> 16) as u16,
        }
    }
}

/// How many words of the map one destination is.
pub const WORDS_EACH: usize = 3;

/// The most destinations one turn can be shown.
///
/// A turn asks a question, is shown where the answer is coming from, and may be
/// offered somewhere else when that place fails — `alo-answering`'s *never a
/// silent fallback* is one question and at most one offer. Two is that, and a
/// call needing more is refused rather than bounded to the first two, which is
/// `alo-bounding`'s rule for places and is this one for the same reason: a bound
/// is not a thing to give somebody most of.
pub const DESTINATIONS: usize = 2;

/// How many words all of a turn's destinations are: the count, then each.
pub const WORDS: usize = 1 + DESTINATIONS * WORDS_EACH;

/// A destination kept in a slot nothing is looking at.
///
/// Not a sentinel: nothing compares against it, because the count decides how
/// far the check looks. It is here so that two bounds holding the same
/// destinations are the same value whichever door they came through.
const NOWHERE: Departure = Departure {
    address: 0,
    port: 0,
    family: 0,
};

/// Everywhere one turn may connect to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Departures {
    /// The destinations, with everything from `how_many` onwards [`NOWHERE`].
    held: [Departure; DESTINATIONS],

    /// How many of them a person was shown.
    how_many: usize,
}

impl Departures {
    /// A turn shown nothing, which may connect to nothing.
    ///
    /// **This is the ordinary case and it is default-deny.** A turn is bound
    /// before it does anything, and until the person is shown a departure there
    /// is no destination in its entry — so every connection it makes is
    /// refused. Nothing has to be written for that to be true.
    #[must_use]
    pub const fn none() -> Self {
        Self::of_words([0; WORDS])
    }

    /// The destinations somebody was shown, in the order they were shown.
    ///
    /// [`None`] when there are more than [`DESTINATIONS`]. An empty list is
    /// [`Self::none`] and is allowed, because a turn that has been shown nothing
    /// is the state every turn begins in.
    ///
    /// ```
    /// use alo_bounding_map::{Departure, Departures, Family};
    ///
    /// let frankfurt = Departure::of(Family::Four, 0x0102_0304, 443);
    /// let somewhere_else = Departure::of(Family::Four, 0x0506_0708, 443);
    /// let shown = Departures::of(&[frankfurt]).expect("one is not too many");
    ///
    /// assert!(shown.holds(frankfurt));
    /// assert!(!shown.holds(somewhere_else));
    /// // The same address on another port is another destination.
    /// assert!(!shown.holds(Departure::of(Family::Four, 0x0102_0304, 80)));
    /// // And a turn shown nothing may connect to nothing.
    /// assert!(!Departures::none().holds(frankfurt));
    /// ```
    #[must_use]
    pub fn of(shown: &[Departure]) -> Option<Self> {
        if shown.len() > DESTINATIONS {
            return None;
        }
        let mut held = [NOWHERE; DESTINATIONS];
        for (slot, departure) in held.iter_mut().zip(shown) {
            *slot = *departure;
        }
        Some(Self {
            held,
            how_many: shown.len(),
        })
    }

    /// Whether this is a destination the person was shown.
    #[must_use]
    pub fn holds(&self, where_to: Departure) -> bool {
        self.each().any(|shown| shown == where_to)
    }

    /// The destinations, and nothing beyond the count.
    pub fn each(&self) -> impl Iterator<Item = Departure> + '_ {
        self.held.iter().copied().take(self.how_many)
    }

    /// How many destinations this turn was shown.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.how_many
    }

    /// Whether this turn was shown none, which is where every turn starts.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.how_many == 0
    }

    /// The value as the map holds it: the count, then three words each.
    #[must_use]
    pub const fn words(&self) -> [u64; WORDS] {
        let [first, second] = self.held;
        let [one_high, one_low, one_both] = first.words();
        let [two_high, two_low, two_both] = second.words();
        [
            self.how_many as u64,
            one_high,
            one_low,
            one_both,
            two_high,
            two_low,
            two_both,
        ]
    }

    /// The destinations read back out of the map.
    ///
    /// Cannot fail, and clamps rather than refusing: a count larger than
    /// [`DESTINATIONS`] is read as [`DESTINATIONS`], and a count of nothing
    /// stays nothing — a turn that may connect nowhere. Both directions fail
    /// closed.
    #[must_use]
    pub const fn of_words(words: [u64; WORDS]) -> Self {
        let [
            how_many,
            one_high,
            one_low,
            one_both,
            two_high,
            two_low,
            two_both,
        ] = words;
        let how_many = if how_many < DESTINATIONS as u64 {
            how_many as usize
        } else {
            DESTINATIONS
        };
        let held = [
            take([one_high, one_low, one_both], how_many > 0),
            take([two_high, two_low, two_both], how_many > 1),
        ];
        Self { held, how_many }
    }
}

/// One slot, if the count says a person was shown it.
const fn take(words: [u64; WORDS_EACH], counted: bool) -> Departure {
    if counted {
        Departure::of_words(words)
    } else {
        Departure::of_words([0; WORDS_EACH])
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The map is two programs sharing memory, so a round trip through it has to
    /// be exactly the identity — and in the order written here, because the
    /// kernel half reads the same words by position.
    #[test]
    fn a_destination_survives_the_map_unchanged() {
        let shown = Departures::of(&[
            Departure::of(Family::Four, 0x0102_0304, 443),
            Departure::of(Family::Six, 0x2001_0db8 << 96, 8080),
        ])
        .expect("two is not too many");
        assert_eq!(Departures::of_words(shown.words()), shown);
        assert_eq!(shown.len(), 2);
    }

    /// **A turn shown nothing may connect to nothing**, and that is the state
    /// every turn is in before anybody is shown anything.
    #[test]
    fn a_turn_shown_nothing_reaches_nowhere() {
        let none = Departures::none();
        assert!(none.is_empty());
        assert!(!none.holds(Departure::of(Family::Four, 0x0102_0304, 443)));
        assert_eq!(Departures::of_words(none.words()), none);
    }

    /// **One departure is one destination.** The address that was shown is
    /// permitted and nothing else is — not another address, not another port,
    /// and not the same numbers read as the other family.
    #[test]
    fn being_shown_one_destination_permits_exactly_that_one() {
        let frankfurt = Departure::of(Family::Four, 0x0102_0304, 443);
        let shown = Departures::of(&[frankfurt]).expect("one is not too many");

        assert!(shown.holds(frankfurt));
        assert!(!shown.holds(Departure::of(Family::Four, 0x0102_0305, 443)));
        assert!(!shown.holds(Departure::of(Family::Four, 0x0102_0304, 80)));
        assert!(!shown.holds(Departure::of(Family::Six, 0x0102_0304, 443)));
    }

    /// More than there is room for is refused rather than cut down, because a
    /// bound is not a thing to give somebody most of.
    #[test]
    fn more_destinations_than_an_entry_holds_is_refused() {
        let one = Departure::of(Family::Four, 1, 443);
        assert!(Departures::of(&[one; DESTINATIONS]).is_some());
        assert!(Departures::of(&[one; DESTINATIONS + 1]).is_none());
    }

    /// A count read back larger than the entry holds is read as the entry's
    /// width, and one read as nothing stays nothing. Both directions fail
    /// closed, which is what a value read out of shared memory has to do.
    #[test]
    fn a_count_that_cannot_be_read_is_read_as_fewer_and_never_as_more() {
        let mut words = Departures::of(&[Departure::of(Family::Four, 1, 443)])
            .expect("one is not too many")
            .words();
        words[0] = u64::MAX;
        assert_eq!(Departures::of_words(words).len(), DESTINATIONS);

        words[0] = 0;
        assert!(Departures::of_words(words).is_empty());
    }

    /// A family the map does not name is read as a destination nothing matches,
    /// rather than as one that matches anything.
    #[test]
    fn a_family_nobody_named_is_a_destination_nothing_matches() {
        let mut words = Departures::of(&[Departure::of(Family::Four, 0x0102_0304, 443)])
            .expect("one is not too many")
            .words();
        // A family number neither of the two is.
        words[3] = (777_u64 << 16) | 443;
        let read = Departures::of_words(words);
        assert!(!read.holds(Departure::of(Family::Four, 0x0102_0304, 443)));
    }
}
