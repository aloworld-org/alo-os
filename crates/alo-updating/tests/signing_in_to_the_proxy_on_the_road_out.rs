//! The road that stages the system's own update, signing in to a proxy that
//! asks who this machine is.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 12: *a proxy an
//! organisation's description says wants a name is signed in to on **every road
//! out alo OS itself uses** — installing and application updates, **the
//! system's own update**, a provider's list and a turn's question — through
//! `alo_proxy::Carried::with_the_password` and through nothing else, held by a
//! test per road.* This is that test for this crate's road, built on
//! [ADR 0059](../../../docs/decisions/0059-where-a-machine-wide-proxy-password-is-kept.md).
//!
//! `tests/the_proxy_on_the_road_out.rs` is task 4's and showed the base being
//! given the machine's proxy. What is new here is **where the password comes
//! from**: the machine's own credentials, read by
//! `alo_proxy::TheMachinesPasswords` and put on the road by
//! `alo_proxy::signed_in`, rather than typed into a test.
//!
//! On a company network with no other route out, a machine whose proxy asks who
//! it is and which could not answer would not update at all — and would say
//! only that the base could not be reached. The refusals below are what replace
//! that with a sentence somebody can act on.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_proxy::{
    Carried, ConfigurationAddress, NotEvaluated, NotSignedIn, ProxyAddress, Reaching, Road, Scheme,
    SpokenTo, TheEvaluator, TheMachinesPasswords, TheProxy, WhereThePasswordIs, signed_in, the_way,
};
use alo_updating::TheBase;

/// The name the company's proxy asks this machine to sign in as.
const SIGNING_IN_AS: &str = "anna";

/// The name the machine's own password for that proxy is kept under.
const KEPT_UNDER: &str = "the company proxy";

/// The password itself, as a machine would have been given it.
const THE_PASSWORD: &str = "hunter2";

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
    let directory = std::env::temp_dir().join(format!("alo-updating-{named}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a directory");
    directory
}

/// The credentials a machine gave the unit taking this road.
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

/// The road out this machine takes to the place its updates come from: the way
/// decided, then signed in to. This is the whole of what a caller does.
fn the_road_out(
    proxy: &TheProxy,
    road: Road,
    signing_in: &TheMachinesPasswords,
) -> Result<Carried, NotSignedIn> {
    let going_to = Reaching::over(Scheme::Https, "updates.example.org").expect("a host");
    let way = the_way(proxy, road, &going_to, &NeverAsked).expect("nothing refuses the way");
    signed_in(way, signing_in)
}

/// **Both of this crate's roads sign in** — asking whether there is an update,
/// and fetching one — with the password the machine was given.
#[test]
fn checking_for_an_update_and_fetching_one_both_sign_in_to_the_proxy() {
    let signing_in = given_the_password("both-roads");
    for road in [Road::CheckingForAnUpdate, Road::FetchingAnUpdate] {
        let carried = the_road_out(&a_proxy_that_asks_who_you_are(), road, &signing_in)
            .expect("the machine was given the password");
        let base = TheBase::on_this_machine().taking(carried);
        let given = base.environment();
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
            base.through().shown(),
            Some("http://proxy.example.com:8080".to_owned()),
            "{road:?}: what a person reads carries the credential"
        );
        assert!(
            !format!("{base:?}").contains(THE_PASSWORD),
            "{road:?}: the base formatted carries the password"
        );
    }
}

/// **A machine that was never given the password takes neither road**, so the
/// base's program is never started: not through the proxy as somebody with no
/// password, and not around it.
#[test]
fn a_machine_that_was_never_given_the_password_takes_neither_road() {
    let signing_in = given_nothing("given-nothing");
    for road in [Road::CheckingForAnUpdate, Road::FetchingAnUpdate] {
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
            Road::FetchingAnUpdate,
            &TheMachinesPasswords::at(&directory),
        )
        .expect_err("a road was taken with a password anybody could read"),
        NotSignedIn::ReadableByAnybody
    );
}

/// **A proxy that asks for no name never reaches the store**, so an ordinary
/// machine stages updates exactly as it did — the credentials here are a
/// directory that does not exist.
#[test]
fn a_proxy_that_asks_for_no_name_never_reaches_the_store() {
    let nowhere = given_nothing("never-asked");
    let plain = TheProxy::one(
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).expect("an address"),
    );
    let carried = the_road_out(&plain, Road::CheckingForAnUpdate, &nowhere)
        .expect("nothing was asked of the store");
    assert_eq!(
        carried.as_an_address(),
        Some("http://proxy.example.com:8080".to_owned())
    );

    let straight = the_road_out(&TheProxy::None, Road::CheckingForAnUpdate, &nowhere)
        .expect("nothing was asked of the store");
    assert!(straight.is_straight());
}
