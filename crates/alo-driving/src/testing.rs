//! The fixtures this crate's tests are written against.
//!
//! **The real verbs, not a copy of them.** [`the_verbs`] is what `alo-agentd`
//! would put on a registry — `alo-files`' six and `alo-applications`' four — so
//! a test here scores an answer against the list a machine really has. A
//! fixture that declared ten verbs of its own would be a measurement of a
//! machine nobody ships, and the drift would be invisible.
//!
//! **The answers are written out**, never generated from the registry. A test
//! that built a correct call out of the same verbs it scores against would be
//! asserting that this crate agrees with itself.
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
/// do not stop the fixed set being put to a model.
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

/// One answer, as a model that got the shape right would have written it.
///
/// `door` is `read` or `propose`; each value is a JSON fragment, so a test can
/// send a number without quotes and something that is not JSON at all.
pub(crate) fn answering(door: &str, verb: &str, given: &[(&str, &str)]) -> String {
    let arguments: Vec<String> = given
        .iter()
        .map(|(named, is)| format!("{{\"named\":\"{named}\",\"is\":{is}}}"))
        .collect();
    format!(
        "{{\"format\":1,\"asks\":{{\"{door}\":{{\"verb\":\"{verb}\",\"given\":[{}]}}}}}}",
        arguments.join(",")
    )
}
