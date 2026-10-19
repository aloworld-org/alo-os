//! The rule asked about a destination with nobody behind it.
//!
//! A question a person asked has no [`Leaving`](crate::Leaving) — nothing shows
//! it and nothing records it (ADR 0126) — and the organisation's rule still
//! applies to it. These are the tests that the rule is the *same* rule, and
//! that it is not a way past the indicator.

use alo_capability::Grantee;
use alo_models::{InferenceSource, Region};

use crate::destination::Destination;
use crate::leaving::{Leaving, Why};
use crate::policy::EgressPolicy;
use crate::refusing::Refusal;

/// A provider that says it runs in the named region.
fn in_region(region: &str) -> InferenceSource {
    InferenceSource::Hosted {
        provider: "Mine".to_owned(),
        region: Region::Declared(region.to_owned()),
    }
}

fn to(source: &InferenceSource) -> Destination {
    Destination::of(source).expect("a hosted provider is somewhere")
}

/// The same four rules, asked both ways, agreeing every time.
///
/// **This is the test that matters**, because the risk of a second door is that
/// it answers a slightly different question — and nothing about these rules has
/// ever read who is sending.
#[test]
fn every_rule_answers_the_same_with_an_egress_and_without_one() {
    let somewhere = in_region("the United States");
    let here = in_region("the EU");
    let rules = [
        EgressPolicy::Anywhere,
        EgressPolicy::InTheBuilding,
        EgressPolicy::InRegion("the EU".to_owned()),
        EgressPolicy::NothingLeaves,
    ];
    for rule in &rules {
        for source in [&somewhere, &here] {
            let leaving = Leaving::because(&Grantee::named("@mail"), Why::Asking, to(source));
            assert_eq!(
                rule.permits(&leaving),
                rule.permits_going_to(&to(source)),
                "{rule:?} disagreed with itself about {source:?}"
            );
        }
    }
}

/// Nothing may leave, so a person's question does not.
#[test]
fn a_machine_that_lets_nothing_leave_refuses_a_question_nobody_is_showing() {
    let source = in_region("the EU");
    let refused = EgressPolicy::NothingLeaves
        .refusal_going_to(&to(&source))
        .expect("nothing leaves at all");
    assert_eq!(refused, Refusal::NothingMayLeave);
}

/// The region is the organisation's, and a provider outside it is refused in
/// the words that name the region they set.
#[test]
fn a_provider_outside_the_region_is_refused_and_the_region_is_named() {
    let source = in_region("the United States");
    let refused = EgressPolicy::InRegion("the EU".to_owned())
        .refusal_going_to(&to(&source))
        .expect("a provider in the United States under a rule naming the EU");
    assert_eq!(
        refused,
        Refusal::OutsideTheRegion {
            region: "the EU".to_owned()
        }
    );
}

/// A provider inside the region is not refused, and neither is anything under
/// a rule that permits everything.
#[test]
fn a_rule_that_permits_this_destination_refuses_nothing() {
    let here = in_region("the EU");
    assert!(
        EgressPolicy::InRegion("the EU".to_owned())
            .refusal_going_to(&to(&here))
            .is_none()
    );
    assert!(
        EgressPolicy::Anywhere
            .refusal_going_to(&to(&here))
            .is_none()
    );
    let far = in_region("the United States");
    assert!(EgressPolicy::Anywhere.refusal_going_to(&to(&far)).is_none());
}

/// **The refusal is the same refusal the shown road makes**, which is what
/// keeps the person's sentence and the agent's one sentence.
#[test]
fn the_refusal_is_the_one_the_shown_road_would_have_made() {
    let source = in_region("the United States");
    let rule = EgressPolicy::InRegion("the EU".to_owned());
    let leaving = Leaving::because(&Grantee::named("@mail"), Why::Asking, to(&source));
    let shown = rule
        .refusal(&leaving)
        .expect("the shown road refuses this too");
    let unshown = rule
        .refusal_going_to(&to(&source))
        .expect("and so does the unshown one");
    assert_eq!(
        shown.why(),
        &unshown,
        "two doors onto one rule gave two refusals"
    );
}
