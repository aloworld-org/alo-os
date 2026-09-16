//! The road out that stages the system's own update, held to taking the
//! machine's one proxy.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 4: the machine's
//! proxy *reaches every road out alo OS itself uses — installing, **updates**,
//! providers — held by a test per road.* This is that test for updates, and it
//! is here because this is where the road is: `alo_updating::TheBase` starts the
//! base's own program, that program pulls a system onto the disk, and the only
//! honest way to show a program was given something is to start one and read
//! what it got.
//!
//! On a company network with no other route out, a machine that staged updates
//! straight past the proxy would not update at all — and would say only that the
//! base could not be reached.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_proxy::{
    Carried, ConfigurationAddress, NotEvaluated, ProxyAddress, Reaching, Road, Scheme, SpokenTo,
    TheEvaluator, TheProxy, the_way,
};
use alo_updating::{Base, TheBase};

/// An evaluator nothing in this file may ask: every setting here was typed by
/// somebody, so a decision that consulted a script would be one nobody asked
/// for.
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

/// A program that prints the environment it was given and ignores its
/// arguments.
///
/// A machine with no base on it is every machine this repository is written on,
/// so the only way to show what the base's program really received is to be it.
#[cfg(unix)]
fn a_program_that_prints_its_environment(named: &str) -> PathBuf {
    use std::io::Write as _;
    use std::os::unix::fs::PermissionsExt as _;

    let program = std::env::temp_dir().join(format!("{named}.sh"));
    let mut written = std::fs::File::create(&program).expect("a program to start");
    written
        .write_all(b"#!/bin/sh\nexec /usr/bin/env\n")
        .expect("a program to start");
    drop(written);
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700))
        .expect("a program that can be started");
    program
}

/// The company's proxy, as somebody was handed it.
fn the_proxy() -> ProxyAddress {
    ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).expect("an address")
}

/// The way out this machine decided for the road an update is fetched on.
fn decided(proxy: &TheProxy) -> Carried {
    let going_to = Reaching::over(Scheme::Https, "updates.example.org").expect("a host");
    Carried::of(
        the_way(proxy, Road::CheckingForAnUpdate, &going_to, &NeverAsked)
            .expect("nothing refuses it"),
    )
}

/// **The base is given the machine's proxy**, under every name a program reads.
#[test]
fn the_base_is_given_the_machines_proxy() {
    let base = TheBase::on_this_machine().taking(decided(&TheProxy::one(the_proxy())));
    let given = base.environment();
    for name in ["http_proxy", "HTTP_PROXY", "https_proxy", "HTTPS_PROXY"] {
        assert_eq!(
            given
                .iter()
                .find(|(each, _)| *each == name)
                .map(|(_, value)| value.as_str()),
            Some("http://proxy.example.com:8080"),
            "the base was not given {name}"
        );
    }
    assert!(
        given
            .iter()
            .any(|(name, value)| *name == "no_proxy" && value.contains("127.0.0.1")),
        "the base was not told to leave this machine alone"
    );
}

/// **A base nobody handed a road to is told there is no proxy**, rather than
/// being left with whatever the service that started this process was carrying.
/// This program's environment is not cleared — it is the base's own, and the
/// base reads it — so saying so is the only thing that makes the setting the
/// machine's.
#[test]
fn a_base_nobody_handed_a_road_to_is_told_there_is_no_proxy() {
    let base = TheBase::on_this_machine();
    assert!(base.through().is_straight());
    assert_eq!(base.through().shown(), None);
    assert_eq!(
        base.environment(),
        vec![
            ("http_proxy", String::new()),
            ("HTTP_PROXY", String::new()),
            ("https_proxy", String::new()),
            ("HTTPS_PROXY", String::new()),
            ("no_proxy", alo_proxy::NEVER_THROUGH_A_PROXY.to_owned()),
            ("NO_PROXY", alo_proxy::NEVER_THROUGH_A_PROXY.to_owned()),
        ]
    );
}

/// **The program really receives it**, which is the only thing that shows the
/// road is taken rather than described.
#[cfg(unix)]
#[test]
fn the_program_the_base_starts_really_receives_the_proxy() {
    let program = a_program_that_prints_its_environment("alo-updating-proxy-road");
    let base = TheBase::at(&program).taking(decided(&TheProxy::one(the_proxy())));

    let printed = base.asked(&["status".to_owned()]).expect("it answers");
    let printed = String::from_utf8_lossy(&printed);
    assert!(
        printed.contains("http_proxy=http://proxy.example.com:8080"),
        "{printed}"
    );
    assert!(
        printed.contains("no_proxy=") && printed.contains("127.0.0.1"),
        "{printed}"
    );
}

/// **With no proxy set the program is told there is none**, which is what stops
/// an inherited setting deciding where a system update is fetched from. This
/// program's environment is the base's own and is not cleared, so *empty* is the
/// only thing that overrides what was already there.
#[cfg(unix)]
#[test]
fn with_no_proxy_the_program_is_told_there_is_none() {
    let program = a_program_that_prints_its_environment("alo-updating-no-proxy");
    let base = TheBase::at(&program).taking(decided(&TheProxy::None));

    let printed = base.asked(&["status".to_owned()]).expect("it answers");
    let printed = String::from_utf8_lossy(&printed);
    for name in ["http_proxy", "HTTP_PROXY", "https_proxy", "HTTPS_PROXY"] {
        assert!(
            printed.lines().any(|line| line == format!("{name}=")),
            "{name} was not said to be empty: {printed}"
        );
    }
}
