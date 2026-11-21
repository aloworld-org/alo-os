//! **The screen road is offered only once a read has shown it is needed**
//! (the owner's framing, 2026-11-20: the screen road is for the applications
//! alo cannot read, so it is offered once a read in that turn has shown one).
//!
//! **The one function both sides build the offer with**: the daemon's turn,
//! for the shape a provider holds its answer to (`alo_turn`), and the agent's
//! loop, for the verbs a model is told (`alo_agent::driving`) — and so the
//! every-try check, which drives the agent's own loop. Two copies would be two
//! offers that one day disagree.

use alo_capability::Verbs;

/// The verbs of the screen road, offered only after a read found a window
/// that describes nothing.
pub const THE_SCREEN_ROAD: [&str; 1] = [crate::working_in_an_application::PICTURE_WINDOW];

/// What a turn offers of `verbs`: all of them once `a_window_described_nothing`,
/// and otherwise all but the screen road's.
#[must_use]
pub fn the_verbs_offered(verbs: &Verbs, a_window_described_nothing: bool) -> Verbs {
    if a_window_described_nothing {
        return verbs.clone();
    }
    let mut offered = Verbs::default();
    for verb in verbs
        .all()
        .filter(|verb| !THE_SCREEN_ROAD.contains(&verb.name()))
    {
        // A verb already in one list cannot collide in a list made of them.
        let _ = offered.declare(verb.clone());
    }
    offered
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_capability::{Arg, Effect, Requires, Takes, Verb};
    use alo_strings::Word;

    fn named(name: &str) -> Verb {
        Verb::checked(
            name,
            Word::saying("instructing.test.purpose", "a test verb"),
            Effect::Read,
            vec![Arg::taking(
                "application",
                Word::saying("instructing.test.application", "an application"),
                Takes::Application,
            )],
            Requires::grants_over(["application"]),
            Word::saying("instructing.test.sentence", "read {application}"),
        )
        .unwrap()
    }

    fn the_verbs() -> Verbs {
        let mut verbs = Verbs::default();
        for name in [
            "accessible.read_window",
            crate::working_in_an_application::PICTURE_WINDOW,
        ] {
            verbs.declare(named(name)).unwrap();
        }
        verbs
    }

    /// **Not offered until a read found a window that describes nothing**, and
    /// every other verb offered either way.
    #[test]
    fn the_screen_road_is_offered_only_after_a_window_described_nothing() {
        let closed = the_verbs_offered(&the_verbs(), false);
        assert!(
            closed
                .of(crate::working_in_an_application::PICTURE_WINDOW)
                .is_none()
        );
        assert!(closed.of("accessible.read_window").is_some());
        let open = the_verbs_offered(&the_verbs(), true);
        assert!(
            open.of(crate::working_in_an_application::PICTURE_WINDOW)
                .is_some()
        );
        assert_eq!(open.len(), 2);
    }
}
