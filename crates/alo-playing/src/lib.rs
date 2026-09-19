//! **What this machine plays, what it produces, and where the right to decode
//! came from.**
//!
//! [ADR 0051](../../../docs/decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md),
//! held as types rather than as prose. Its shape is that **encoding and decoding
//! are two decisions, and answering them together is the mistake**:
//!
//! - **What we encode, we choose.** A screen recording is a file this machine
//!   makes from nothing, so it is AV1 or VP9, Opus, Matroska — royalty-free,
//!   with AV1 only where hardware can encode it. [`producing`].
//! - **What we decode, somebody already sent.** A video in their mail, from
//!   their phone, from a colleague. Refusing it is refusing them their own file
//!   on their own machine. [`deciding`].
//!
//! | | |
//! |---|---|
//! | [`codec`] | the codecs the decision names, and which carry no royalty |
//! | [`producing`] | what this machine encodes, which is free on every machine |
//! | [`right`] | **where a right to decode comes from**, and the closed list of decoders the image ships |
//! | [`machine`] | what a machine has, handed in rather than read here |
//! | [`inside`] | the tracks in a file, because **a kind is the wrapping, not the codec** |
//! | [`deciding`] | the order, walked once per track |
//! | [`refusing`] | the one sentence, which belongs to `alo-opening` |
//!
//! # The right to decode comes from the hardware or from a licence, never from hope
//!
//! A machine may play an H.264 file because the chip in it holds a licence, or
//! because a company that paid for redistribution gave us a binary to pass on.
//! **It may not play one because we compiled a decoder and hoped.** The order —
//! free, then expired, then the silicon, then a redistributable decoder, then a
//! refusal — is not a preference: it is the order in which the right to decode
//! exists at all.
//!
//! # Which decoders the image itself carries
//!
//! [ADR 0058] closed the question ADR 0051 left open, and its rule is the one
//! thing worth remembering: **we ship a decoder when nobody can charge us for
//! shipping it** — because it was made free, because the patents expired, or
//! because a company that paid the royalty published a binary for us to pass
//! on. Everything else is the silicon's job, and where there is no silicon the
//! machine says so.
//!
//! So [`right::SoftwareDecoders`] names AV1, VP9 and every audio codec the
//! decision lists, and does not name H.264 or HEVC. MP3 and AAC-LC are in it
//! because their terms ran out, which is why [`right::Right::ByExpiry`] is its
//! own answer rather than folded into `RoyaltyFree`: **one row of the list rests
//! on a patent term**, and that is the sentence still with a lawyer, named at
//! [`right::SoftwareDecoders::THE_ONE_FOR_COUNSEL`]. The deadline is unchanged
//! and is a shipment rather than a version: before the certified laptop goes to
//! anybody outside this team.
//!
//! [ADR 0058]: ../../../docs/decisions/0058-which-software-decoders-the-image-ships.md
//!
//! # What this crate does not do
//!
//! - **It says nothing to a person.** There is no vocabulary here and there will
//!   not be one. A film this machine cannot play is reported through
//!   `alo_opening::Cannot::NothingHereOpens`, which is the sentence somebody
//!   already meets when a document cannot be opened. A second refusal shape for
//!   video would drift from the first, and a person would have to learn two ways
//!   of being told the same thing.
//! - **It plays nothing.** It decides. Moving bytes through the rented media
//!   stack is the work that reads this, and the test that a real sample file
//!   plays end to end waits on a machine — which is now all it waits on.
//! - **It reads no file.** What is inside one arrives as [`inside::Inside`],
//!   from whoever parsed the container.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod codec;
pub mod deciding;
pub mod inside;
pub mod machine;
pub mod producing;
pub mod refusing;
pub mod right;

pub use codec::{Audio, Video};
pub use deciding::{Plays, plays, the_right_to_sound, the_right_to_video};
pub use inside::Inside;
pub use machine::AMachine;
pub use producing::{Container, Meant, Produces};
pub use refusing::reported;
pub use right::{Right, SoftwareDecoders};
