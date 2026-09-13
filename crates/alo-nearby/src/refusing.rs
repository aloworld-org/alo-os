//! Every way this crate declines to go on, and the sentence each one is.
//!
//! Two kinds are deliberately kept in one list. Some are about a packet that
//! arrived — a stranger's, since anything on a network can send one — and some
//! are about this machine's own identity file. They are one enum because a
//! caller looking at a machine it found and a caller starting a machine up are
//! the same caller, and a second enum would only make it choose.
//!
//! None of them is a failure of the network. **Finding nothing is not an
//! error** — it is [`Looking::found`](crate::Looking::found) returning nothing,
//! the way `alo_models::found_on_this_machine` answers with an empty list
//! rather than a refusal. A machine with no network is a machine with no
//! neighbours, which is a true answer about the world.

use std::path::PathBuf;

/// Why something on the local network was not taken as a machine, or why this
/// machine could not say who it is.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum NotNearby {
    /// The packet ended in the middle of something it had promised.
    ///
    /// Ordinary on a network: a truncated datagram, a responder that is not
    /// speaking this service, a packet that was never meant for us.
    #[error("the advertisement ended before it finished saying what it was")]
    CutShort,

    /// A name in the packet pointed at itself, directly or round a loop.
    ///
    /// A name is allowed to point backwards into the packet it is in, which is
    /// how DNS keeps packets small, and a name that points at itself would read
    /// forever. This is the one refusal here that exists because a packet can
    /// be hostile rather than merely wrong.
    #[error("a name in the advertisement never ended")]
    NameNeverEnds,

    /// What was offered as a machine identity is not one.
    ///
    /// An identity is thirty-two hexadecimal characters and nothing else. A
    /// person's name, a hostname somebody typed, or a serial would all fail
    /// here, which is the point: see [`MachineId`](crate::MachineId).
    #[error(
        "`{0}` is not a machine identity — thirty-two hexadecimal characters, and nothing a \
         person chose"
    )]
    NotAMachineIdentity(String),

    /// The advertisement carried a field that is not on the list presence is
    /// allowed to carry.
    ///
    /// **This is the refusal this crate exists for.** Everything on a network
    /// can read an advertisement, including a machine nobody has paired with
    /// and nobody owns, so what an advertisement may carry is a closed list and
    /// anything else is refused rather than ignored. Ignoring it would let a
    /// later version quietly start saying more.
    #[error("the advertisement carries `{0}`, which is more than presence")]
    SaysMoreThanPresence(String),

    /// The advertisement is for some other service on the same network.
    ///
    /// Printers, media players and file shares all answer on the same address
    /// and port, so this is the commonest thing that happens here and it is not
    /// a fault in anything.
    #[error("the advertisement is for `{0}` rather than an alo machine")]
    NotAnAloMachine(String),

    /// Nothing in the advertisement said which machine sent it.
    #[error("nothing in the advertisement says which machine sent it")]
    SaysNothingAboutWhichMachine,

    /// What was offered as a machine's public half is not one.
    ///
    /// Sixty-four lowercase hexadecimal characters and nothing else; the
    /// length is carried rather than the bytes, because a stranger's bytes
    /// have no business in a sentence.
    #[error("{0} characters were offered as a machine's part in a pairing, which is not one")]
    NotAnOffer(usize),

    /// What arrived as a proof is not shaped like one.
    ///
    /// The shape only. Whether a proof that *is* shaped like one is true is
    /// `Proven::checked`'s to answer, and is not a `NotNearby`.
    #[error("what arrived as a proof is not one: {0}")]
    NotAProof(String),

    /// A name this machine was about to write into a packet cannot be written.
    ///
    /// A label over sixty-three characters or a name over two hundred and
    /// fifty-five: not reachable from an identity this crate made, and refused
    /// rather than truncated, because a truncated name is a different machine's.
    #[error("`{0}` cannot be written into an advertisement")]
    NotAName(String),

    /// The identity this machine keeps could not be read or written.
    ///
    /// Carried as a sentence rather than an `io::Error` so the type can be
    /// compared and tested; the reader is looking for which file, not for a
    /// kind.
    #[error("this machine's identity could not be read from {at}: {why}")]
    IdentityUnreadable {
        /// The file the identity is kept in.
        at: PathBuf,
        /// What the operating system said about it.
        why: String,
    },

    /// The randomness a new identity is made from was not available.
    #[error("this machine could not make an identity: {0}")]
    NoRandomness(String),

    /// A socket would not do what it was asked.
    ///
    /// **Not** *nothing was found*: finding nothing is an empty list, because a
    /// machine with no neighbours is a true answer about the world. This is the
    /// socket itself refusing — no interface, a port already held, a multicast
    /// group the machine will not join.
    #[error("this machine could not listen to its own network: {0}")]
    TheNetwork(String),
}

/// A short way to say what the operating system said, without carrying its
/// error type into an enum that is compared in tests.
pub(crate) fn because(why: &std::io::Error) -> String {
    why.to_string()
}

/// The crate's own name for itself in a sentence, so a caller printing a
/// refusal does not have to.
impl NotNearby {
    /// Whether this refusal is about a packet somebody else sent rather than
    /// about this machine.
    ///
    /// A caller looking at a network wants to go on to the next packet; a
    /// caller starting this machine up wants to stop. Nothing here decides
    /// which, but the difference is real and this is where it is written down.
    #[must_use]
    pub const fn is_about_a_stranger(&self) -> bool {
        matches!(
            *self,
            Self::CutShort
                | Self::NameNeverEnds
                | Self::NotAMachineIdentity(_)
                | Self::SaysMoreThanPresence(_)
                | Self::NotAnAloMachine(_)
                | Self::SaysNothingAboutWhichMachine
                | Self::NotAnOffer(_)
                | Self::NotAProof(_)
        )
    }
}
