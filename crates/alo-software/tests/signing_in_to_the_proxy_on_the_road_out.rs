//! The road that installs, checks and updates applications, signing in to a
//! proxy that asks who this machine is.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 12: *a proxy an
//! organisation's description says wants a name is signed in to on **every road
//! out alo OS itself uses** — **installing and application updates**, the
//! system's own update, a provider's list and a turn's question — through
//! `alo_proxy::Carried::with_the_password` and through nothing else, held by a
//! test per road.* This is that test for this crate's three roads, built on
//! [ADR 0059](../../../docs/decisions/0059-where-a-machine-wide-proxy-password-is-kept.md).
//!
//! `tests/the_proxy_on_the_road_out.rs` is task 4's and showed the tool being
//! given the machine's proxy. What is new here is **where the password comes
//! from**: the machine's own credentials, read by
//! `alo_proxy::TheMachinesPasswords` and put on the road by
//! `alo_proxy::signed_in`, rather than typed into a test. A test that typed one
//! could not fail on a machine that never reads a credential at all.
//!
//! # It starts a program, because that is where this road is
//!
//! A machine with no rented tool on it is every machine this repository is
//! written on, so the program started here is a short script this file writes,
//! which prints the environment it was given and exits. What that shows is the
//! one thing worth showing: the child process really received the credential
//! the machine keeps, and really received nothing of this process's own.
//!
//! # And the refusal, which never starts a program at all
//!
//! A machine that cannot sign in has **no road to hand the tool**. There is no
//! `alo_proxy::Carried` to give it, so nothing is started — not going through
//! the proxy as somebody with no password, and not straight out around it. On a
//! company network with no other route out, both of those are worse than a
//! sentence somebody can act on.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_proxy::{
    Carried, ConfigurationAddress, NotEvaluated, NotSignedIn, ProxyAddress, Reaching, Road, Scheme,
    SpokenTo, TheEvaluator, TheMachinesPasswords, TheProxy, WhereThePasswordIs, signed_in, the_way,
};
use alo_software::{TheRentedTool, Tool};

/// The name the company's proxy asks this machine to sign in as.
const SIGNING_IN_AS: &str = "anna";

/// The name the machine's own password for that proxy is kept under.
const KEPT_UNDER: &str = "the company proxy";

/// The password itself, as a machine would have been given it.
const THE_PASSWORD: &str = "hunter2";

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

/// The company's proxy, asking who this machine is.
fn a_proxy_that_asks_who_you_are() -> TheProxy {
    TheProxy::one(
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080)
            .expect("an address")
            .signing_in(
                SIGNING_IN_AS,
                WhereThePasswordIs::named(KEPT_UNDER).expect("a name"),
            )
            .expect("a proxy that asks who you are"),
    )
}

/// A directory of this test's own, empty.
fn a_directory_of_its_own(named: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("alo-software-{named}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a directory");
    directory
}

/// The credentials a machine gave the unit taking this road, holding the
/// proxy's password.
fn given_the_password(named: &str) -> TheMachinesPasswords {
    let directory = a_directory_of_its_own(named);
    let at = directory.join(KEPT_UNDER);
    std::fs::write(&at, THE_PASSWORD).expect("a password");
    kept_the_way_a_machine_keeps_one(&at);
    TheMachinesPasswords::at(&directory)
}

/// The credentials of a unit that was given none.
fn given_nothing(named: &str) -> TheMachinesPasswords {
    TheMachinesPasswords::at(&a_directory_of_its_own(named).join("never-made"))
}

/// The permissions systemd gives a credential.
#[cfg(unix)]
fn kept_the_way_a_machine_keeps_one(at: &Path) {
    use std::os::unix::fs::PermissionsExt as _;

    std::fs::set_permissions(at, std::fs::Permissions::from_mode(0o400)).expect("a mode");
}

/// Nothing to do where a mode is not what says it.
#[cfg(not(unix))]
fn kept_the_way_a_machine_keeps_one(_at: &Path) {}

/// Where applications come from, as this machine reaches it.
fn the_place_applications_come_from() -> Reaching {
    Reaching::over(Scheme::Https, "dl.example.org").expect("a host")
}

/// The road out this machine takes for one of this crate's three roads: the way
/// decided, then signed in to. This is the whole of what a caller does.
fn the_road_out(
    proxy: &TheProxy,
    road: Road,
    signing_in: &TheMachinesPasswords,
) -> Result<Carried, NotSignedIn> {
    let way = the_way(
        proxy,
        road,
        &the_place_applications_come_from(),
        &NeverAsked,
    )
    .expect("nothing refuses the way");
    signed_in(way, signing_in)
}

/// A program that prints the environment it was given, one line each, and
/// ignores whatever arguments it was started with.
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

/// **All three of this crate's roads sign in**, with the password the machine
/// was given and through the one door a credential travels.
#[test]
fn installing_looking_for_updates_and_updating_all_sign_in_to_the_proxy() {
    let signing_in = given_the_password("all-three-roads");
    for road in [
        Road::InstallingAnApplication,
        Road::CheckingForApplicationUpdates,
        Road::UpdatingAnApplication,
    ] {
        let carried = the_road_out(&a_proxy_that_asks_who_you_are(), road, &signing_in)
            .expect("the machine was given the password");
        let tool = TheRentedTool::on_this_machine().taking(carried);
        let given = tool.environment();
        for name in ["http_proxy", "HTTP_PROXY", "https_proxy", "HTTPS_PROXY"] {
            assert_eq!(
                given
                    .iter()
                    .find(|(each, _)| *each == name)
                    .map(|(_, value)| value.as_str()),
                Some("http://anna:hunter2@proxy.example.com:8080"),
                "{road:?} was not signed in under {name}"
            );
        }
        assert_eq!(
            tool.through().shown(),
            Some("http://proxy.example.com:8080".to_owned()),
            "{road:?}: what a person reads carries the credential"
        );
        assert!(
            !format!("{tool:?}").contains(THE_PASSWORD),
            "{road:?}: the tool formatted carries the password"
        );
    }
}

/// **A machine that was never given the password takes no road at all**, on
/// every one of this crate's three.
///
/// There is no `alo_proxy::Carried` to hand the tool, so nothing is started:
/// neither the proxy nor the place applications come from is reached.
#[test]
fn a_machine_that_was_never_given_the_password_takes_none_of_the_three_roads() {
    let signing_in = given_nothing("given-nothing");
    for road in [
        Road::InstallingAnApplication,
        Road::CheckingForApplicationUpdates,
        Road::UpdatingAnApplication,
    ] {
        assert_eq!(
            the_road_out(&a_proxy_that_asks_who_you_are(), road, &signing_in)
                .expect_err("a road was taken with no credential"),
            NotSignedIn::NothingKeepsIt,
            "{road:?}"
        );
    }
}

/// **A password anybody on the machine could read takes no road either**, and
/// is told apart from one the machine never had.
#[cfg(unix)]
#[test]
fn a_password_anybody_could_read_takes_no_road_and_is_told_apart() {
    use std::os::unix::fs::PermissionsExt as _;

    let directory = a_directory_of_its_own("readable-by-anybody");
    let at = directory.join(KEPT_UNDER);
    std::fs::write(&at, THE_PASSWORD).expect("a password");
    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o444)).expect("a mode");

    assert_eq!(
        the_road_out(
            &a_proxy_that_asks_who_you_are(),
            Road::InstallingAnApplication,
            &TheMachinesPasswords::at(&directory),
        )
        .expect_err("a road was taken with a password anybody could read"),
        NotSignedIn::ReadableByAnybody
    );
}

/// **A proxy that asks for no name never reaches the store**, so the ordinary
/// company machine takes these roads exactly as it did — the credentials here
/// are a directory that does not exist.
#[test]
fn a_proxy_that_asks_for_no_name_never_reaches_the_store() {
    let nowhere = given_nothing("never-asked");
    let plain = TheProxy::one(
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).expect("an address"),
    );
    let carried = the_road_out(&plain, Road::InstallingAnApplication, &nowhere)
        .expect("nothing was asked of the store");
    assert_eq!(
        carried.as_an_address(),
        Some("http://proxy.example.com:8080".to_owned())
    );

    let straight = the_road_out(&TheProxy::None, Road::InstallingAnApplication, &nowhere)
        .expect("nothing was asked of the store");
    assert!(straight.is_straight());
}

/// **The program the tool starts really receives the credential**, and nothing
/// of this process's own environment.
///
/// Everything above is a list; this starts a process and reads what it actually
/// got, which is the only thing that shows the credential is carried rather
/// than described.
#[cfg(unix)]
#[test]
fn the_program_the_tool_starts_really_receives_the_credential_and_nothing_of_ours() {
    let carried = the_road_out(
        &a_proxy_that_asks_who_you_are(),
        Road::UpdatingAnApplication,
        &given_the_password("really-received"),
    )
    .expect("the machine was given the password");

    let program = a_program_that_prints_its_environment("alo-software-signing-in");
    let tool = TheRentedTool::at(program.to_str().expect("a path")).taking(carried);

    let printed = tool.open().expect("the program answers").join("\n");
    assert!(
        printed.contains("http_proxy=http://anna:hunter2@proxy.example.com:8080"),
        "the credential did not travel on the road that needs it: {printed}"
    );
    assert!(
        !printed.lines().any(|line| line.starts_with("HOME=")),
        "the caller's own environment reached the tool: {printed}"
    );
}
