//! The road out that installs, checks and updates applications, held to taking
//! the machine's one proxy.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 4: the machine's
//! proxy *reaches every road out alo OS itself uses — **installing**, updates,
//! providers — held by a test per road.* This is that test for installing, and
//! it is here because this is where the road is: `alo_software::TheRentedTool`
//! starts a program, and the only honest way to show a program was given
//! something is to start one and read what it got.
//!
//! Three of `alo_proxy::Road`'s seven are this crate's — installing an
//! application, looking for updates to one, and fetching one — and all three go
//! through the same door, so the test below is written about the door.
//!
//! **A machine with no rented tool on it is every machine this repository is
//! written on**, so the program started here is a short script this file writes,
//! which prints the environment it was given and exits. What that shows is
//! exactly what matters: the child process really received the machine's proxy,
//! and really received nothing of this process's own environment.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_proxy::{
    Carried, ConfigurationAddress, NotEvaluated, Password, ProxyAddress, Reaching, Road, Scheme,
    SpokenTo, TheEvaluator, TheProxy, WhereThePasswordIs, the_way,
};
use alo_software::{TheRentedTool, Tool};

/// An evaluator nothing in this file may ask: every setting here was typed by
/// somebody, so a decision that consulted a script would be one nobody asked for.
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

/// A program that prints the environment it was given, one line each, and
/// ignores whatever arguments it was started with.
///
/// A machine with no rented tool on it is every machine this repository is
/// written on, so the only way to show what a child process really received is
/// to be the child process.
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

/// Where applications come from, as this machine reaches it.
fn the_place_applications_come_from() -> Reaching {
    Reaching::over(Scheme::Https, "dl.example.org").expect("a host")
}

/// The way out this machine decided for one of this crate's three roads.
fn decided(proxy: &TheProxy, road: Road) -> Carried {
    Carried::of(
        the_way(
            proxy,
            road,
            &the_place_applications_come_from(),
            &NeverAsked,
        )
        .expect("nothing refuses it"),
    )
}

/// **The three roads this crate takes are given the machine's proxy**, and the
/// list the tool is started with is the whole of what it gets.
#[test]
fn installing_looking_for_updates_and_updating_are_all_given_the_machines_proxy() {
    for road in [
        Road::InstallingAnApplication,
        Road::CheckingForApplicationUpdates,
        Road::UpdatingAnApplication,
    ] {
        let tool =
            TheRentedTool::on_this_machine().taking(decided(&TheProxy::one(the_proxy()), road));
        let given = tool.environment();
        for name in ["http_proxy", "HTTP_PROXY", "https_proxy", "HTTPS_PROXY"] {
            assert_eq!(
                given
                    .iter()
                    .find(|(each, _)| *each == name)
                    .map(|(_, value)| value.as_str()),
                Some("http://proxy.example.com:8080"),
                "{road:?} was not given {name}"
            );
        }
        assert!(
            given
                .iter()
                .any(|(name, value)| *name == "no_proxy" && value.contains("127.0.0.1")),
            "{road:?} was not told to leave this machine alone"
        );
    }
}

/// **With no proxy set the tool is told there is none**, rather than being left
/// to find one — and it is still told to leave this machine alone.
#[test]
fn with_no_proxy_the_tool_is_told_there_is_none() {
    let tool = TheRentedTool::on_this_machine()
        .taking(decided(&TheProxy::None, Road::InstallingAnApplication));
    let given = tool.environment();
    for name in ["http_proxy", "HTTP_PROXY", "https_proxy", "HTTPS_PROXY"] {
        assert_eq!(
            given
                .iter()
                .find(|(each, _)| *each == name)
                .map(|(_, value)| value.as_str()),
            Some(""),
            "{name}"
        );
    }
    assert!(
        given.iter().any(|(name, _)| *name == "no_proxy"),
        "{given:?}"
    );
}

/// **A tool nobody handed a road to goes straight out**, which is what the
/// ordinary machine is, and the whole of what it is started with is this list.
#[test]
fn a_tool_nobody_handed_a_road_to_goes_straight_out() {
    let tool = TheRentedTool::on_this_machine();
    assert!(tool.through().is_straight());
    assert_eq!(tool.through().shown(), None);
    assert_eq!(
        tool.environment()
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>(),
        [
            "LC_ALL",
            "LANG",
            "PATH",
            "http_proxy",
            "HTTP_PROXY",
            "https_proxy",
            "HTTPS_PROXY",
            "no_proxy",
            "NO_PROXY"
        ]
    );
}

/// **The program really receives it.** Everything above is a list; this starts
/// a process and reads what it actually got, which is the only thing that shows
/// the road is taken rather than described.
#[cfg(unix)]
#[test]
fn the_program_the_tool_starts_really_receives_the_proxy_and_nothing_of_ours() {
    let program = a_program_that_prints_its_environment("alo-software-proxy-road");
    let tool = TheRentedTool::at(program.to_str().expect("a path")).taking(decided(
        &TheProxy::one(the_proxy()),
        Road::InstallingAnApplication,
    ));

    let printed = tool.open().expect("the program answers");
    let printed = printed.join("\n");
    assert!(
        printed.contains("http_proxy=http://proxy.example.com:8080"),
        "{printed}"
    );
    assert!(
        printed.contains("no_proxy=") && printed.contains("127.0.0.1"),
        "{printed}"
    );
    // `HOME` is in this process's own environment on any machine these tests run
    // on, and is on no list the tool is given. Its absence is the environment
    // having been cleared before the proxy was put on it.
    assert!(
        !printed.lines().any(|line| line.starts_with("HOME=")),
        "the caller's own environment reached the tool: {printed}"
    );
}

/// **A proxy that asks for a name is signed in to on this road, and the
/// credential goes nowhere else.**
///
/// The password reaches the child that needs it and appears in nothing a person
/// or a log would read — `alo_proxy::Carried` is where that promise lives, and
/// this is it holding at the far end of a real process.
#[cfg(unix)]
#[test]
fn a_proxy_that_asks_for_a_name_is_signed_in_to_and_the_password_goes_nowhere_else() {
    let signing_in = the_proxy()
        .signing_in(
            "anna",
            WhereThePasswordIs::named("the company proxy").expect("a name"),
        )
        .expect("a proxy that asks for a name");
    let carried = Carried::of(
        the_way(
            &TheProxy::one(signing_in),
            Road::UpdatingAnApplication,
            &the_place_applications_come_from(),
            &NeverAsked,
        )
        .expect("nothing refuses it"),
    )
    .with_the_password(Password::typed("hunter2").expect("a password"));

    let program = a_program_that_prints_its_environment("alo-software-proxy-credential");
    let tool = TheRentedTool::at(program.to_str().expect("a path")).taking(carried);

    assert_eq!(
        tool.through().shown(),
        Some("http://proxy.example.com:8080".to_owned()),
        "what a person reads carries no credential"
    );
    assert!(
        !format!("{tool:?}").contains("hunter2"),
        "the tool formatted says nothing about the password"
    );

    let printed = tool.open().expect("the program answers").join("\n");
    assert!(
        printed.contains("http_proxy=http://anna:hunter2@proxy.example.com:8080"),
        "the credential travels on the road that needs it: {printed}"
    );
}
