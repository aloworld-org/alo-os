//! Every road out of this machine, held to the one proxy — and held to being
//! every road.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 4: the machine's
//! proxy *reaches **every road out alo OS itself uses** — installing, updates,
//! providers — held by a test per road.* Each of those three roads is tested in
//! the crate that takes it, because that is where the test can watch a real
//! process being started:
//!
//! | | |
//! |---|---|
//! | installing, and application updates | `crates/alo-software/tests/the_proxy_on_the_road_out.rs` |
//! | the system's own update | `crates/alo-updating/tests/the_proxy_on_the_road_out.rs` |
//! | a provider | `crates/alo-models/tests/the_proxy_on_the_road_to_a_provider.rs` |
//!
//! What is here is the half those three cannot do: **holding the list of roads
//! to being the list of things this machine reaches the network for.** A road
//! that exists and is not on `alo_proxy::Road` would have no test to be missing
//! from, and *machine-wide* would quietly mean *the roads somebody remembered*.
//!
//! `alo_egress::Errand` is the other closed list of the same thing — every
//! reason alo OS reaches the network with no agent behind it — and the two are
//! compared here rather than hoped to agree. `alo-proxy` depends on nothing that
//! could say this at compile time, deliberately: it is a decision crate, and a
//! dependency on the indicator would be the first step towards the two answers
//! being one value read two ways.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_egress::{Destination, Errand, Indicator, OnItsOwn};
use alo_proxy::{
    Carried, ConfigurationAddress, NotEvaluated, ProxyAddress, Reaching, Road, Scheme, SpokenTo,
    TheEvaluator, TheProxy, Way, the_way,
};
use std::time::{Duration, SystemTime};

/// An evaluator nothing in this file may ask.
///
/// Every setting here is one somebody typed, so a decision that quietly
/// consulted a network's own script would be a decision nobody asked for.
struct NeverAsked;

impl TheEvaluator for NeverAsked {
    fn asked(
        &self,
        _: &ConfigurationAddress,
        address: &str,
        _: &str,
    ) -> Result<String, NotEvaluated> {
        unreachable!("nothing should have been asked about {address}")
    }
}

/// The moment every line in this file happens at.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

/// The company's proxy, as somebody was handed it.
fn the_proxy() -> ProxyAddress {
    ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).expect("an address")
}

/// **Every reason alo OS reaches the network is a road the proxy covers.**
///
/// The two lists are written in two crates that cannot see each other's
/// reasoning, and this is what stops them drifting: a seventh errand added to
/// `alo-egress` without a road here would be an egress that leaves the machine
/// with nobody having decided which way out it takes.
#[test]
fn every_errand_alo_os_runs_is_a_road_the_proxy_covers() {
    let errands: Vec<Errand> = Errand::EVERY.to_vec();
    let roads: Vec<Road> = Road::EVERY
        .into_iter()
        .filter(|road| road.is_an_errand())
        .collect();
    assert_eq!(
        errands.len(),
        roads.len(),
        "alo-egress has {} reasons to reach the network and alo-proxy covers {} of them",
        errands.len(),
        roads.len()
    );

    // Named one at a time, so that adding a member to either list fails here
    // with the name of the one that has no partner rather than with a number.
    for (errand, road) in errands.into_iter().zip(roads) {
        let named = match errand {
            Errand::SigningIn => Road::SigningIn,
            Errand::FetchingAModel => Road::FetchingAModel,
            Errand::CheckingForAnUpdate => Road::CheckingForAnUpdate,
            Errand::InstallingAnApplication => Road::InstallingAnApplication,
            Errand::CheckingForApplicationUpdates => Road::CheckingForApplicationUpdates,
            Errand::UpdatingAnApplication => Road::UpdatingAnApplication,
        };
        assert_eq!(named, road, "{errand:?} and {road:?} are not the same road");
    }
}

/// **One proxy, and the same answer on every road.** A setting that sent
/// updates one way and questions another would be two settings wearing one
/// name.
#[test]
fn one_setting_answers_every_road_the_same_way() {
    let proxy = TheProxy::one(the_proxy());
    let going_to = Reaching::over(Scheme::Https, "dl.example.org").expect("a host");
    for road in Road::EVERY {
        assert_eq!(
            the_way(&proxy, road, &going_to, &NeverAsked).expect("nothing refuses it"),
            Way::Through(the_proxy()),
            "{road:?}"
        );
    }
}

/// **The egress indicator names the real destination, not the proxy.**
///
/// *It went to the proxy* is not what a person needs to know. On a machine sold
/// on sovereignty the indicator answers *where did my work go*, and on a company
/// network the answer is the same whether or not a proxy is in the middle of it.
///
/// This is the test the plan's third acceptance line asks for, and it is here
/// rather than in `alo-proxy`'s own files because it needs both crates: a way
/// out, and the line a person actually reads.
#[test]
fn the_indicator_names_the_real_destination_and_never_the_proxy() {
    let strings = alo_strings::Strings::of(alo_egress::egress_words().expect("the egress words"));
    let going_to = Reaching::over(Scheme::Https, "dl.example.org").expect("a host");
    let way = the_way(
        &TheProxy::one(the_proxy()),
        Road::InstallingAnApplication,
        &going_to,
        &NeverAsked,
    )
    .expect("nothing refuses it");
    assert_eq!(way, Way::Through(the_proxy()));

    // The destination is built from where the road is going, which is the value
    // the caller already had. There is nothing on a `Way` that could be handed
    // here instead.
    let errand = OnItsOwn::for_(
        Errand::InstallingAnApplication,
        Destination::at(going_to.host()).expect("a destination"),
    );
    let mut indicator = Indicator::default();
    let underway = indicator.beginning_on_its_own(errand, noon());

    let line = underway.on_its_own().said(&strings);
    assert_eq!(
        line.text(),
        "alo OS is installing an application from dl.example.org"
    );
    assert!(!line.text().contains("proxy.example.com"), "{line}");
    assert!(!line.text().contains("8080"), "{line}");

    indicator.ended_on_its_own(underway);
    assert!(indicator.is_quiet());
}

/// **A road that goes through a proxy is still shown, and shown as itself.**
/// The same holds for a road the machine decided goes straight out: the
/// indicator line does not change because a proxy was or was not in the way.
#[test]
fn the_line_a_person_reads_is_the_same_whether_or_not_a_proxy_is_in_the_way() {
    let strings = alo_strings::Strings::of(alo_egress::egress_words().expect("the egress words"));
    let going_to = Reaching::over(Scheme::Https, "dl.example.org").expect("a host");
    let said = |proxy: &TheProxy| {
        let way = the_way(
            proxy,
            Road::CheckingForApplicationUpdates,
            &going_to,
            &NeverAsked,
        )
        .expect("nothing refuses it");
        let carried = Carried::of(way);
        let errand = OnItsOwn::for_(
            Errand::CheckingForApplicationUpdates,
            Destination::at(going_to.host()).expect("a destination"),
        );
        (carried.shown(), errand.said(&strings).into_text())
    };

    let (straight, without) = said(&TheProxy::None);
    let (through, with) = said(&TheProxy::one(the_proxy()));
    assert_eq!(straight, None);
    assert_eq!(through, Some("http://proxy.example.com:8080".to_owned()));
    assert_eq!(
        with, without,
        "the indicator says where the machine is reaching, not how"
    );
}
