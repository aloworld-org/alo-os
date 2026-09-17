//! **Every rule that applies to a provider applies to ours, in the same words.**
//!
//! [ADR 0014](../../../docs/decisions/0014-alos-own-model-is-a-provider-like-any-other.md)
//! writes down the pressures before they arrive: make ours the default, make it
//! look safer, let the indicator treat it gently, let a failing local model
//! become a paid call. Each is refused here by comparing our entry against a
//! provider somebody else added and requiring the **same answer**, not merely a
//! correct one.
//!
//! That is the shape of every test in this file: `assert_eq!(ours, theirs)`. A
//! future change that makes ours quieter has to make everybody quieter, which
//! is a change somebody would notice.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_models::{InferenceSource, NotAllowed, Provider, Region, SourcePolicy};

/// A provider a person added, in the same region ours declares.
fn somebody_elses() -> Provider {
    Provider::checked(
        "Mistral",
        "https://api.mistral.ai/v1",
        Region::Declared("France".to_owned()),
        None,
    )
    .expect("a provider a person typed in")
}

/// Ours.
fn ours() -> Provider {
    alo_hosted::entry(None).expect("alo's own entry")
}

/// **A policy that refuses hosted inference refuses ours**, and says the same
/// thing about it.
#[test]
fn every_policy_treats_ours_exactly_as_it_treats_theirs() {
    let both = [ours(), somebody_elses()];
    for policy in [
        SourcePolicy::Anywhere,
        SourcePolicy::InTheBuilding,
        SourcePolicy::ThisMachineOnly,
        SourcePolicy::InRegion("France".to_owned()),
        SourcePolicy::InRegion("Switzerland".to_owned()),
    ] {
        let [mine, theirs] = both
            .each_ref()
            .map(|provider| policy.permits(&provider.source()));
        assert_eq!(
            mine, theirs,
            "{policy:?} treats alo's own service differently from another provider in the same \
             region"
        );
    }
}

/// **And it is refused in the same words**, which is what a person reads.
#[test]
fn a_refusal_names_ours_the_way_it_names_anybody() {
    let in_the_building = SourcePolicy::InTheBuilding;
    let [mine, theirs] = [ours(), somebody_elses()]
        .each_ref()
        .map(|provider| in_the_building.refusal(&provider.source()));
    assert_eq!(
        std::mem::discriminant(&mine.expect("ours is refused")),
        std::mem::discriminant(&theirs.expect("theirs is refused")),
        "alo's own service is refused with a different sentence from anybody else's"
    );

    // And a region that neither is in refuses both the same way.
    let elsewhere = SourcePolicy::InRegion("Switzerland".to_owned());
    let [mine, theirs] = [ours(), somebody_elses()]
        .each_ref()
        .map(|provider| elsewhere.refusal(&provider.source()));
    assert!(matches!(mine, Some(NotAllowed::OutsideTheRegion { .. })));
    assert_eq!(
        std::mem::discriminant(&mine.expect("ours")),
        std::mem::discriminant(&theirs.expect("theirs"))
    );
}

/// **Ours is hosted, and reports as hosted.** Not *this machine*, not a source
/// of its own — the same variant every provider gets.
#[test]
fn ours_is_hosted_like_any_other_provider() {
    assert_eq!(
        std::mem::discriminant(&ours().source()),
        std::mem::discriminant(&somebody_elses().source()),
    );
    assert_ne!(ours().source(), InferenceSource::ThisMachine);
}

/// **There is no default and no pre-selection.**
///
/// A machine nobody has configured has no provider at all — ours included —
/// and `alo-hosted` ships no list, no ordering and no *recommended* flag.
#[test]
fn nothing_here_is_offered_before_a_person_chooses_it() {
    let mut named: Vec<&str> = Vec::new();
    for symbol in [
        "DEFAULT",
        "RECOMMENDED",
        "PREFERRED",
        "FIRST",
        "fn default",
        "impl Default",
    ] {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        for file in std::fs::read_dir(&src)
            .expect("this crate's source")
            .flatten()
        {
            let text = std::fs::read_to_string(file.path()).unwrap_or_default();
            if text.contains(symbol) {
                named.push(symbol);
            }
        }
    }
    assert!(
        named.is_empty(),
        "alo-hosted offers itself: {named:?}. A person adds a provider; nothing here may arrive \
         already chosen (ADR 0014)"
    );
}

/// **Running out of credit with us reads exactly as running out anywhere else**
/// — and opens no door that a broken provider would not.
///
/// This is the pressure ADR 0014 names most sharply: a failing paid call is the
/// moment a product is tempted to suggest something. The sentence differs from
/// another provider's by the provider's name and by nothing else, and what may
/// happen next is identical.
#[test]
fn running_out_with_us_reads_and_behaves_as_running_out_anywhere() {
    use alo_answering::{Answering, WentWrong};
    use alo_strings::Strings;

    let words = alo_saying::everything_this_machine_can_say().expect("the machine's vocabulary");
    let strings = Strings::of(words);
    let policy = SourcePolicy::Anywhere;

    let said = |provider: &Provider| {
        Answering::chosen(provider.source(), &policy)
            .expect("nothing forbids a hosted provider here")
            .did_not_answer(WentWrong::RanOut, &[], &policy)
            .expect("running out is a thing that happens to a hosted provider")
            .said(&strings)
            .text()
            .to_owned()
    };

    let mine = said(&ours());
    let theirs = said(&somebody_elses());
    assert_eq!(
        mine.replace("alo", "PROVIDER"),
        theirs.replace("Mistral", "PROVIDER"),
        "running out with alo's own service reads differently from running out anywhere else"
    );
    assert!(
        mine.contains("nothing else about this machine has changed"),
        "{mine}"
    );
    assert!(
        !mine.to_lowercase().contains("subscri") && !mine.to_lowercase().contains("upgrade"),
        "a refusal from our own service sells something: {mine}"
    );
}

/// **Cancelling leaves a machine that works.**
///
/// A person who stops paying us has a machine that answers on its own hardware,
/// exactly as it did before they ever added a provider — so removing ours from
/// a person's settings is removing a provider, and nothing here is load-bearing
/// for anything else. Held by the fact that this crate ships a single provider
/// entry and no state: there is nothing to cancel *into*.
#[test]
fn cancelling_is_removing_a_provider_and_nothing_more() {
    // What the machine knows is an address, a region and a key reference —
    // nothing about an account, a balance or a plan, so there is nothing that
    // could stop working when the money stops.
    let entry = ours();
    let written = format!("{entry:?}").to_lowercase();
    for about_an_account in [
        "balance",
        "credit",
        "plan",
        "subscription",
        "expires",
        "tier",
    ] {
        assert!(
            !written.contains(about_an_account),
            "the machine holds `{about_an_account}` about alo's service; what it may know is an \
             address, a key, a region, and whether the last request was accepted"
        );
    }
}
