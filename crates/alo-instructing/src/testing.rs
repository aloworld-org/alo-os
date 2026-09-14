//! The fixtures this crate's tests are written against.
//!
//! **The real verbs, not a copy of them.** [`the_verbs`] is what `alo-agentd`
//! would put on a registry — `alo-files`' six and `alo-applications`' four — so
//! a test here builds the words a real machine's model is shown. A fixture that
//! declared ten verbs of its own would describe a machine nobody ships, and the
//! drift would be invisible.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a test fixture, a panic on a None or an Err is the failure being reported"
)]

use alo_capability::{Arg, Effect, Requires, Takes, Verb, Verbs};
use alo_strings::Word;

/// The verbs alo OS itself offers: the six file verbs and the four application
/// verbs.
pub(crate) fn the_verbs() -> Verbs {
    let mut verbs = Verbs::default();
    alo_files::declare_into(&mut verbs).unwrap();
    alo_applications::declare_into(&mut verbs).unwrap();
    verbs
}

/// A verb this system does not ship, for the test that says an adapter's verbs
/// are told of like the machine's own.
pub(crate) fn a_verb_of_somebody_elses() -> Verb {
    Verb::checked(
        "water_the_plants",
        Word::saying(
            "testing.verb.water-the-plants.purpose",
            "water the plants in a room",
        ),
        Effect::Change,
        vec![Arg::taking(
            "room",
            Word::saying("testing.verb.water-the-plants.room", "the room to water"),
            Takes::name(64),
        )],
        Requires::nothing_because("a room is not a path, a file or an application"),
        Word::saying(
            "testing.verb.water-the-plants.sentence",
            "water the plants in {room}",
        ),
    )
    .unwrap()
}
