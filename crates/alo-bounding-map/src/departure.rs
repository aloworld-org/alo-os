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
//!
//! # A link-local address is a destination only with its interface
//!
//! [ADR 0041](../../../docs/decisions/0041-a-link-local-departure-names-its-interface.md).
//! `fe80::1` is not one machine: it is one machine **on each link** this machine
//! is on, and a turn that could reach it on any of them would have been shown
//! one place and permitted several. So a departure keeps the interface an
//! address that needs one is on — `fe80::/10`, and multicast scoped to an
//! interface or a link, which is the kernel's own `__ipv6_addr_needs_scope_id`
//! — and [`Departure::on`] is the one door both halves go through, so that the
//! daemon and the programme cannot disagree about which addresses carry one.
//! Every other address carries none, whatever the caller handed in: a scope on
//! a global address is ignored by the kernel, and a check that compared it would
//! refuse a connection the kernel was always going to make to the same place.
//!
//! **A link-local address with no interface is a destination nothing holds**
//! ([`Departure::names_its_network`]). The kernel will not dial one; a datagram
//! to one leaves by whichever interface a socket option chose, which is not a
//! thing a departure can describe; so it is refused whether or not somebody
//! registered the same address with no interface beside it.
//!
//! # An IPv4 departure may be held to the interface it leaves by
//!
//! [ADR 0044](../../../docs/decisions/0044-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md).
//! `192.168.1.20` on the wired network and `192.168.1.20` on the Wi-Fi are two
//! machines whenever two routers hand out the same private range. The kernel
//! dials an IPv4 address with no interface named, so there is no scope to read
//! — but a socket **held to an interface** (`SO_BINDTOIFINDEX`) leaves by that
//! interface and no other, and that is how the daemon dials a paired machine
//! found on one network. So an IPv4 departure keeps whatever interface it is
//! given, and the programme gives it the one the socket is held to:
//!
//! - **held to an interface**, it is permitted only where the socket is held to
//!   that same interface ([`Departure::permits`]) — not on another, and not on a
//!   socket held to none, which leaves by whatever the route says at the moment;
//! - **held to none** — every provider's, and every IPv4 departure written before
//!   this — it is permitted however the socket is held, exactly as before: the
//!   route decides, and that is unchanged.
//!
//! An IPv6 address that is not link-local still keeps no interface: it names its
//! own network, and ADR 0041's rule for it is untouched.
//!
//! The interface travels in the top half of the word that already carried the
//! family and the port, which had been zeroes since the shape was written. The
//! map is not a byte wider, so [`WORDS`] and the kernel's own entry size are
//! unchanged, and an IPv4 destination held to no interface reads back exactly
//! as it did before.

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

    /// The interface the address is on, as the kernel numbers interfaces: for
    /// an address that needs one, and for an IPv4 address held to one — and
    /// zero for every other.
    interface: u32,
}

impl Departure {
    /// A destination somebody was shown, on no particular interface.
    ///
    /// Right for every address that names its own network. For one that does
    /// not — a link-local one — this is a destination nothing holds, because
    /// such an address is only somewhere once its interface is said:
    /// [`Departure::on`] is the door for it.
    #[must_use]
    pub const fn of(family: Family, address: u128, port: u16) -> Self {
        Self::on(family, address, port, 0)
    }

    /// A destination somebody was shown, on the interface the kernel numbers
    /// `interface`.
    ///
    /// **The interface is kept only where it can decide something**
    /// ([`keeps_its_interface`]): an address that needs one, and every IPv4
    /// address, whose interface is the one its socket is held to (ADR 0044).
    /// It is dropped everywhere else, so the daemon, which hands in whatever
    /// scope a resolved address carried, and the programme, which hands in
    /// whatever scope a `sockaddr_in6` carried, make the same value of the same
    /// destination.
    ///
    /// ```
    /// use alo_bounding_map::{Departure, Departures, Family};
    ///
    /// let studio = (0xfe80_u128 << 112) | 0x20;
    /// let on_the_cable = Departure::on(Family::Six, studio, 7_610, 3);
    /// let shown = Departures::of(&[on_the_cable]).expect("one is not too many");
    ///
    /// assert!(shown.holds(on_the_cable));
    /// // The same address on another interface is another machine.
    /// assert!(!shown.holds(Departure::on(Family::Six, studio, 7_610, 4)));
    /// // And with no interface at all it is nowhere.
    /// assert!(!shown.holds(Departure::of(Family::Six, studio, 7_610)));
    ///
    /// // A global address carries no interface, whatever was handed in.
    /// let global = (0x2001_0db8_u128 << 96) | 1;
    /// assert_eq!(
    ///     Departure::on(Family::Six, global, 443, 9),
    ///     Departure::of(Family::Six, global, 443)
    /// );
    /// ```
    #[must_use]
    pub const fn on(family: Family, address: u128, port: u16, interface: u32) -> Self {
        Self {
            address,
            port,
            family: family.number(),
            interface: if keeps_its_interface(family, address) {
                interface
            } else {
                0
            },
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

    /// The interface the address is on, or zero for an address that names its
    /// own network and an IPv4 address held to none — and for a link-local one
    /// nobody said the interface of, which [`Departure::names_its_network`]
    /// answers for.
    #[must_use]
    pub const fn interface(&self) -> u32 {
        self.interface
    }

    /// Whether this destination is somewhere: every address is, except one
    /// that needs an interface and was given none.
    ///
    /// Such an address could be on any of a machine's links, so it is refused
    /// as a destination rather than matched against one — [`Departures::holds`]
    /// answers false for it whatever was shown.
    #[must_use]
    pub const fn names_its_network(&self) -> bool {
        match self.family() {
            Some(family) => !needs_an_interface(family, self.address) || self.interface != 0,
            None => false,
        }
    }

    /// Whether being shown this destination permits going to `where_to`.
    ///
    /// The same destination, always. And one more, for one family: **an IPv4
    /// departure held to no interface permits the same address and port on any
    /// interface**, because that is what every IPv4 departure permitted before
    /// ADR 0044 and what a provider's still must — its address is on no one
    /// network, and the route decides. An IPv4 departure held to an interface
    /// permits that interface and nothing else, which is the whole of what
    /// holding it is for.
    ///
    /// ```
    /// use alo_bounding_map::{Departure, Family};
    ///
    /// let studio = 0xc0a8_0114; // 192.168.1.20
    /// let on_the_cable = Departure::on(Family::Four, studio, 7_610, 3);
    /// assert!(on_the_cable.permits(on_the_cable));
    /// // The same address on the Wi-Fi is another machine.
    /// assert!(!on_the_cable.permits(Departure::on(Family::Four, studio, 7_610, 4)));
    /// // And a socket held to nothing goes wherever the route says.
    /// assert!(!on_the_cable.permits(Departure::of(Family::Four, studio, 7_610)));
    ///
    /// // A provider's departure is held to nothing and permits what it did.
    /// let provider = Departure::of(Family::Four, 0x0102_0304, 443);
    /// assert!(provider.permits(Departure::on(Family::Four, 0x0102_0304, 443, 4)));
    /// ```
    #[must_use]
    pub const fn permits(&self, where_to: Departure) -> bool {
        let same_place = self.address == where_to.address
            && self.port == where_to.port
            && self.family == where_to.family;
        same_place
            && (self.interface == where_to.interface
                || (self.family == Family::INET && self.interface == 0))
    }

    /// The value as the map holds it: the address in two words, then the
    /// interface, the family and the port packed into a third — thirty-two,
    /// sixteen and sixteen bits, from the top.
    ///
    /// The order is the only thing keeping two separately compiled programs
    /// talking about the same destination, so it is decided here and nowhere
    /// else.
    #[must_use]
    pub const fn words(&self) -> [u64; WORDS_EACH] {
        [
            (self.address >> 64) as u64,
            self.address as u64,
            ((self.interface as u64) << 32) | ((self.family as u64) << 16) | self.port as u64,
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
            interface: (both >> 32) as u32,
        }
    }
}

/// Whether an address is only somewhere once the interface it is on is said.
///
/// The kernel's own rule, `__ipv6_addr_needs_scope_id`, written once for both
/// halves: an IPv6 **link-local** unicast address (`fe80::/10`), and an IPv6
/// **multicast** address whose scope is one interface (`ff01::/16` and its
/// flagged forms) or one link (`ff02::/16`). No IPv4 address is — the kernel
/// dials none of them by interface — and nor is any other IPv6 address.
#[must_use]
pub const fn needs_an_interface(family: Family, address: u128) -> bool {
    match family {
        Family::Four => false,
        Family::Six => {
            let link_local = address >> 118 == 0x3fa;
            let multicast = address >> 120 == 0xff;
            let scope = (address >> 112) & 0x0f;
            link_local || (multicast && (scope == 1 || scope == 2))
        }
    }
}

/// Whether a departure to this address keeps the interface it is given.
///
/// Every address that [`needs_an_interface`], and **every IPv4 address**: the
/// kernel dials one with no interface named, but a socket held to an interface
/// leaves by it, and a paired machine found on one network is dialled that way
/// (ADR 0044). The private ranges are not singled out, and deliberately: a
/// public address on an office network and a carrier's shared range are the
/// same question, and a departure held to nothing still permits any interface
/// ([`Departure::permits`]), so keeping the interface narrows only a departure
/// somebody held.
#[must_use]
pub const fn keeps_its_interface(family: Family, address: u128) -> bool {
    match family {
        Family::Four => true,
        Family::Six => needs_an_interface(family, address),
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
    interface: 0,
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
    ///
    /// Never for a destination that does not name its network — a link-local
    /// address with no interface — even if the same nothing was written into
    /// the entry: [`Departure::names_its_network`] says why. Each destination
    /// shown is asked [`Departure::permits`], which is where an IPv4 departure
    /// held to an interface is held to it (ADR 0044).
    #[must_use]
    pub fn holds(&self, where_to: Departure) -> bool {
        where_to.names_its_network() && self.each().any(|shown| shown.permits(where_to))
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

    /// The studio's link-local address, as reception measured it.
    const THE_STUDIO: u128 = (0xfe80_u128 << 112) | 0x0a1b_2c3d_4e5f_6071;

    /// The interface the studio was heard on.
    const THE_CABLE: u32 = 3;

    /// Another interface on the same machine, where nobody was shown anything.
    const ELSEWHERE: u32 = 4;

    /// **A link-local departure is permitted on the interface it was shown on,
    /// and refused on any other** — the same address on another link is
    /// another machine, and a departure widened to every link would have shown
    /// one place and permitted several.
    #[test]
    fn a_link_local_departure_is_held_on_its_interface_and_no_other() {
        let on_the_cable = Departure::on(Family::Six, THE_STUDIO, 7_610, THE_CABLE);
        let shown = Departures::of(&[on_the_cable]).expect("one is not too many");

        assert!(shown.holds(Departure::on(Family::Six, THE_STUDIO, 7_610, THE_CABLE)));
        assert!(
            !shown.holds(Departure::on(Family::Six, THE_STUDIO, 7_610, ELSEWHERE)),
            "a link-local departure shown on one interface was permitted on another"
        );
        assert!(!shown.holds(Departure::on(Family::Six, THE_STUDIO, 80, THE_CABLE)));

        // And the other way round: shown elsewhere, refused on the cable.
        let shown_elsewhere =
            Departures::of(&[Departure::on(Family::Six, THE_STUDIO, 7_610, ELSEWHERE)])
                .expect("one is not too many");
        assert!(shown_elsewhere.holds(Departure::on(Family::Six, THE_STUDIO, 7_610, ELSEWHERE)));
        assert!(!shown_elsewhere.holds(on_the_cable));
    }

    /// **A link-local address with no interface is nowhere**: refused as a
    /// destination, and refused even where the same nothing was written into
    /// the entry — because the interface it would leave by is whatever a socket
    /// option chose, which no departure describes.
    #[test]
    fn a_link_local_address_with_no_interface_is_held_by_nothing() {
        let nowhere = Departure::of(Family::Six, THE_STUDIO, 7_610);
        assert!(!nowhere.names_its_network());
        assert_eq!(nowhere.interface(), 0);

        let shown_nowhere = Departures::of(&[nowhere]).expect("one is not too many");
        assert!(!shown_nowhere.holds(nowhere));
        let shown_on_the_cable =
            Departures::of(&[Departure::on(Family::Six, THE_STUDIO, 7_610, THE_CABLE)])
                .expect("one is not too many");
        assert!(!shown_on_the_cable.holds(nowhere));

        // Multicast scoped to a link or to one interface needs one too.
        let link_multicast = (0xff02_u128 << 112) | 0xfb;
        let interface_multicast = (0xff01_u128 << 112) | 1;
        assert!(!Departure::of(Family::Six, link_multicast, 5_353).names_its_network());
        assert!(!Departure::of(Family::Six, interface_multicast, 5_353).names_its_network());
        assert!(Departure::on(Family::Six, link_multicast, 5_353, THE_CABLE).names_its_network());
    }

    /// **An address that names its own network carries no interface**, whatever
    /// the caller handed in — so a scope on a global address, which the kernel
    /// ignores, cannot make one destination two.
    #[test]
    fn an_address_that_names_its_own_network_keeps_no_interface() {
        let global = (0x2001_0db8_u128 << 96) | 1;
        let site_multicast = (0xff05_u128 << 112) | 1;
        let loopback = 1;
        for address in [global, site_multicast, loopback] {
            let scoped = Departure::on(Family::Six, address, 443, ELSEWHERE);
            assert_eq!(scoped.interface(), 0, "{address:x}");
            assert_eq!(scoped, Departure::of(Family::Six, address, 443));
            assert!(scoped.names_its_network());
        }
        // No IPv4 address needs an interface to be dialled — but since ADR 0044
        // one keeps the interface its socket is held to, which the tests below
        // hold.
        assert!(!needs_an_interface(Family::Four, 0xa9fe_0001));
        assert!(Departure::of(Family::Four, 0xa9fe_0001, 443).names_its_network());

        // The edges of `fe80::/10`, which is ten bits and not sixteen.
        assert!(needs_an_interface(Family::Six, 0xfebf_u128 << 112));
        assert!(!needs_an_interface(Family::Six, 0xfec0_u128 << 112));
        assert!(!needs_an_interface(Family::Six, 0xfe7f_u128 << 112));
    }

    /// **The interface survives the map, and an IPv4 destination's words are
    /// what they always were** — it travels in bits that were zero, so nothing
    /// written before this shape reads back differently.
    #[test]
    fn an_interface_survives_the_map_and_an_ipv4_destination_is_unchanged() {
        let on_the_cable = Departure::on(Family::Six, THE_STUDIO, 7_610, THE_CABLE);
        let shown = Departures::of(&[
            on_the_cable,
            Departure::on(Family::Six, THE_STUDIO, 7_610, u32::MAX),
        ])
        .expect("two is not too many");
        let read = Departures::of_words(shown.words());
        assert_eq!(read, shown);
        assert!(read.holds(on_the_cable));
        assert!(read.holds(Departure::on(Family::Six, THE_STUDIO, 7_610, u32::MAX)));
        assert_eq!(
            on_the_cable.words()[2],
            (u64::from(THE_CABLE) << 32) | (u64::from(Family::INET6) << 16) | 7_610
        );

        let frankfurt = Departure::of(Family::Four, 0x0102_0304, 443);
        assert_eq!(
            frankfurt.words(),
            [0, 0x0102_0304, (u64::from(Family::INET) << 16) | 443]
        );
    }

    /// The studio's private IPv4 address, `192.168.1.20`, which the Wi-Fi's
    /// router hands out too.
    const THE_STUDIO_OVER_IPV4: u128 = 0xc0a8_0114;

    /// **A private IPv4 departure held to the interface it was found on is
    /// permitted there and refused on another network carrying the same
    /// address** — and refused on a socket held to no interface, which leaves by
    /// whatever the route says at the moment (ADR 0044).
    #[test]
    fn an_ipv4_departure_held_to_an_interface_is_permitted_there_and_nowhere_else() {
        let on_the_cable = Departure::on(Family::Four, THE_STUDIO_OVER_IPV4, 7_610, THE_CABLE);
        assert_eq!(on_the_cable.interface(), THE_CABLE);
        assert!(on_the_cable.names_its_network());
        let shown = Departures::of(&[on_the_cable]).expect("one is not too many");

        assert!(shown.holds(on_the_cable));
        assert!(
            !shown.holds(Departure::on(
                Family::Four,
                THE_STUDIO_OVER_IPV4,
                7_610,
                ELSEWHERE
            )),
            "an IPv4 departure held to one interface was permitted on another"
        );
        assert!(
            !shown.holds(Departure::of(Family::Four, THE_STUDIO_OVER_IPV4, 7_610)),
            "an IPv4 departure held to an interface was permitted where the route chooses"
        );
        assert!(!shown.holds(Departure::on(
            Family::Four,
            THE_STUDIO_OVER_IPV4,
            80,
            THE_CABLE
        )));
        assert!(!shown.holds(Departure::on(
            Family::Four,
            THE_STUDIO_OVER_IPV4 + 1,
            7_610,
            THE_CABLE
        )));
        // And the IPv6 address holding the same numbers is not it either.
        assert!(!shown.holds(Departure::on(
            Family::Six,
            THE_STUDIO_OVER_IPV4,
            7_610,
            THE_CABLE
        )));
    }

    /// **A provider's departure is unchanged**: held to no interface, it permits
    /// the address and port however the socket is held — the route decides, as
    /// it did — and its words are the words it always had.
    #[test]
    fn an_ipv4_departure_held_to_no_interface_permits_what_it_always_did() {
        let frankfurt = Departure::of(Family::Four, 0x0102_0304, 443);
        let shown = Departures::of(&[frankfurt]).expect("one is not too many");
        for interface in [0, THE_CABLE, ELSEWHERE, u32::MAX] {
            assert!(
                shown.holds(Departure::on(Family::Four, 0x0102_0304, 443, interface)),
                "{interface}"
            );
        }
        assert!(!shown.holds(Departure::on(Family::Four, 0x0102_0304, 80, THE_CABLE)));
        assert_eq!(
            frankfurt.words(),
            [0, 0x0102_0304, (u64::from(Family::INET) << 16) | 443]
        );

        // Held to nothing is IPv4's alone: a link-local IPv6 departure with no
        // interface still permits nothing (ADR 0041).
        let unscoped = Departure::of(Family::Six, THE_STUDIO, 7_610);
        assert!(!unscoped.permits(Departure::on(Family::Six, THE_STUDIO, 7_610, THE_CABLE)));
        assert!(!keeps_its_interface(
            Family::Six,
            (0x2001_0db8_u128 << 96) | 1
        ));
        assert!(keeps_its_interface(Family::Four, THE_STUDIO_OVER_IPV4));
    }

    /// **The interface an IPv4 departure is held to survives the map**, in the
    /// bits ADR 0041 put a link-local interface in.
    #[test]
    fn an_ipv4_interface_survives_the_map() {
        let on_the_cable = Departure::on(Family::Four, THE_STUDIO_OVER_IPV4, 7_610, THE_CABLE);
        let shown = Departures::of(&[on_the_cable]).expect("one is not too many");
        let read = Departures::of_words(shown.words());
        assert_eq!(read, shown);
        assert!(read.holds(on_the_cable));
        assert!(!read.holds(Departure::on(
            Family::Four,
            THE_STUDIO_OVER_IPV4,
            7_610,
            ELSEWHERE
        )));
        assert_eq!(
            on_the_cable.words()[2],
            (u64::from(THE_CABLE) << 32) | (u64::from(Family::INET) << 16) | 7_610
        );
    }
}
