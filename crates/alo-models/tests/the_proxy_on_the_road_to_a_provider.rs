//! The road out to a provider, held to taking the machine's one proxy.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 4: the machine's
//! proxy *reaches every road out alo OS itself uses — installing, updates,
//! **providers** — held by a test per road.* This is that test for providers,
//! and it is here because this is where the road is: `alo_models::Trying` makes
//! the request, and a proxy that reached the other two roads and not this one
//! would mean a machine that can install applications on a company network and
//! cannot answer a question.
//!
//! # What it shows, and what it deliberately does not
//!
//! It shows that the request is configured with the way out this machine
//! decided, and that a road going straight out is configured **explicitly** —
//! which is the part that matters most. A client left to find its own proxy
//! reads the environment of whatever process it is in, and that is not a setting
//! anybody chose, cannot be shown to a person, and cannot be changed where they
//! would look for it. `alo-secrets` refuses to read an environment for the same
//! reason, in the file that says a function taking a uid cannot be pointed
//! somewhere by a variable.
//!
//! What it does not do is stand a proxy server up and watch a question go
//! through it. That is a machine test rather than a suite test, and the task's
//! report says so rather than claiming otherwise.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_models::{Provider, Region, Secret, SourcePolicy, Trying};
use alo_proxy::{
    Carried, ConfigurationAddress, NotEvaluated, Password, ProxyAddress, Reaching, Road, Scheme,
    SpokenTo, TheEvaluator, TheProxy, WhereThePasswordIs, the_way,
};

/// An evaluator nothing in this file may ask.
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

/// The company's proxy, as somebody was handed it.
fn the_proxy() -> ProxyAddress {
    ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).expect("an address")
}

/// A provider somewhere else, which is the only kind this road goes to.
fn a_provider() -> Provider {
    Provider::checked(
        "a provider",
        "https://api.example.com",
        Region::Declared("the EU".to_owned()),
        None,
    )
    .expect("a provider")
}

/// The way out this machine decided for the road to that provider.
fn decided(proxy: &TheProxy) -> Carried {
    let going_to = Reaching::over(Scheme::Https, "api.example.com").expect("a host");
    Carried::of(
        the_way(proxy, Road::AskingAProvider, &going_to, &NeverAsked).expect("nothing refuses it"),
    )
}

/// **The road to a provider takes the machine's proxy.**
#[test]
fn the_road_to_a_provider_takes_the_machines_proxy() {
    let carried = decided(&TheProxy::one(the_proxy()));
    assert_eq!(
        carried.shown(),
        Some("http://proxy.example.com:8080".to_owned())
    );

    let through = carried.for_a_request().expect("the client can use it");
    assert!(
        through.is_some(),
        "the request was configured with no proxy"
    );

    // And the request is built with it. Nothing is sent: the policy is asked
    // first and refuses, which is the door this road already goes through.
    let refused = Trying::provider(&a_provider(), None)
        .taking(through)
        .under(&SourcePolicy::ThisMachineOnly);
    assert!(refused.is_err(), "nothing leaves a machine set to this one");
}

/// **A road going straight out is said, rather than left to the client.** This
/// is what stops the environment of whatever process alo OS happens to be in
/// deciding where a question goes.
#[test]
fn a_road_going_straight_out_is_said_rather_than_left_to_the_client() {
    let carried = decided(&TheProxy::None);
    assert!(carried.is_straight());
    assert_eq!(
        carried.for_a_request().expect("nothing to refuse"),
        None,
        "straight out is None, and None is what the request is configured with"
    );
}

/// **This machine is never reached through the proxy**, so a model answering on
/// loopback (ADR 0007) goes on answering when somebody types a company address
/// into a settings panel.
#[test]
fn a_provider_on_this_machine_is_never_reached_through_the_proxy() {
    let here = Reaching::over(Scheme::Http, "127.0.0.1").expect("a host");
    let way = the_way(
        &TheProxy::one(the_proxy()),
        Road::AskingAProvider,
        &here,
        &NeverAsked,
    )
    .expect("nothing refuses it");
    assert!(way.is_straight());
    assert_eq!(
        Carried::of(way).for_a_request().expect("nothing to refuse"),
        None
    );
}

/// **A proxy that asks for a name is signed in to on this road too**, and the
/// key the question carries is unaffected by any of it.
#[test]
fn a_proxy_that_asks_for_a_name_is_signed_in_to_and_the_providers_key_is_untouched() {
    let signing_in = the_proxy()
        .signing_in(
            "anna",
            WhereThePasswordIs::named("the company proxy").expect("a name"),
        )
        .expect("a proxy that asks for a name");
    let carried = Carried::of(
        the_way(
            &TheProxy::one(signing_in),
            Road::AskingAProvider,
            &Reaching::over(Scheme::Https, "api.example.com").expect("a host"),
            &NeverAsked,
        )
        .expect("nothing refuses it"),
    )
    .with_the_password(Password::typed("hunter2").expect("a password"));

    assert!(
        carried
            .for_a_request()
            .expect("the client can use it")
            .is_some()
    );
    assert_eq!(
        carried.shown(),
        Some("http://proxy.example.com:8080".to_owned()),
        "what a person reads carries no credential"
    );

    let key = Secret::typed("a-provider-key").expect("a key");
    let refused = Trying::provider(&a_provider(), Some(&key))
        .taking(carried.for_a_request().expect("the client can use it"))
        .under(&SourcePolicy::ThisMachineOnly);
    assert!(refused.is_err(), "nothing leaves a machine set to this one");
}
