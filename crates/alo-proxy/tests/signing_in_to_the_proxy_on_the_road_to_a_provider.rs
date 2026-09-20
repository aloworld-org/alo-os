//! The road out to a provider's list, signing in to a proxy that asks who this
//! machine is.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 12: *a proxy an
//! organisation's description says wants a name is signed in to on **every road
//! out alo OS itself uses** — installing and application updates, the system's
//! own update, **a provider's list** and a turn's question — through
//! `alo_proxy::Carried::with_the_password` and through nothing else, held by a
//! test per road.* This is that test for the provider road, built on
//! [ADR 0059](../../../docs/decisions/0059-where-a-machine-wide-proxy-password-is-kept.md).
//!
//! `alo-models`' own `tests/the_proxy_on_the_road_to_a_provider.rs` is task 4's
//! and showed the request being configured with the machine's proxy. What is new
//! here is **where the password comes from** — the machine's own credentials,
//! read by `alo_proxy::TheMachinesPasswords` — and that it really goes on the
//! wire.
//!
//! # Why this road's test is in `alo-proxy` and not in `alo-models`
//!
//! Because `alo-models` belongs to `docs/autonomy/v0-5-the-models-measured-plan.md`,
//! which still has an unfinished task, and a lane does not write in another
//! plan's crate — `tools/kernel-loop/src/who_owns.rs` refuses it, and the reason
//! it exists is two machines editing one crate. So the road is exercised from
//! this side of it: `alo_models::Trying` is a dev-dependency here, the request
//! it makes is the production one, and what is asserted is what arrived at the
//! proxy.
//!
//! # It watches a socket, because a configured value is not a road taken
//!
//! A listener of this file's own stands for the company's proxy, and what is
//! asserted is what arrived on it: the first line of the `CONNECT`
//! conversation, and the header beside it that says who is asking. The listener
//! answers nothing, so the test never comes back; that is expected and is not
//! what is being measured.
//!
//! # And the refusal, where nothing is asked of anybody
//!
//! A machine that cannot sign in has no road to configure a request with, so
//! `alo_models::Trying` is never built and no socket is opened — not to the
//! proxy without the credential it asked for, and not around it.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::Read as _;
use std::net::{IpAddr, SocketAddr, TcpListener, UdpSocket};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use alo_models::{Provider, Region, SourcePolicy, Trying};
use alo_proxy::{
    Carried, ConfigurationAddress, NotEvaluated, NotSignedIn, ProxyAddress, Reaching, Road, Scheme,
    SpokenTo, TheEvaluator, TheMachinesPasswords, TheProxy, WhereThePasswordIs, signed_in, the_way,
};

/// The name the company's proxy asks this machine to sign in as.
const SIGNING_IN_AS: &str = "anna";

/// The name the machine's own password for that proxy is kept under.
const KEPT_UNDER: &str = "the company proxy";

/// The password itself, as a machine would have been given it.
///
/// Letters and digits on purpose: what goes on the wire is then exactly what is
/// written here, so a failure names the thing that is wrong rather than an
/// encoding.
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

/// The company's proxy at this address, asking who this machine is.
fn a_proxy_that_asks_who_you_are(at: SocketAddr) -> TheProxy {
    TheProxy::one(
        ProxyAddress::checked(SpokenTo::Http, &at.ip().to_string(), at.port())
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
    let directory = std::env::temp_dir().join(format!("alo-proxy-provider-{named}"));
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

/// A provider somewhere else, which is the only kind this road goes to.
fn a_provider(at: SocketAddr) -> Provider {
    Provider::checked(
        "Mine",
        &format!("https://{at}"),
        Region::Declared("the EU".to_owned()),
        None,
    )
    .expect("a provider")
}

/// The road out this machine takes to a provider: the way decided, then signed
/// in to. This is the whole of what a caller does.
fn the_road_out(
    proxy: &TheProxy,
    going_to: &SocketAddr,
    signing_in: &TheMachinesPasswords,
) -> Result<Carried, NotSignedIn> {
    let reaching = Reaching::over(Scheme::Https, &going_to.ip().to_string()).expect("a host");
    let way = the_way(proxy, Road::AskingAProvider, &reaching, &NeverAsked)
        .expect("nothing refuses the way");
    signed_in(way, signing_in)
}

/// This machine's own address, which is what a provider's and a proxy's have to
/// be: a loopback one is *this machine* and `alo_proxy::the_way` sends it
/// straight out whatever is set.
fn our_own_address() -> IpAddr {
    let asking = UdpSocket::bind("0.0.0.0:0").expect("a socket");
    match asking
        .connect("192.0.2.1:9")
        .and_then(|()| asking.local_addr())
    {
        Ok(ours) => ours.ip(),
        Err(_) => IpAddr::from([127, 0, 0, 1]),
    }
}

/// A listener this test owns, which reports **whether anybody came and what
/// they said** — [`None`] for a door nobody knocked on.
///
/// It answers nothing, so whatever opened the connection gets no reply and
/// gives up. What is kept is the first bytes it wrote, which is the only thing
/// that says who it thought it was talking to: a road out is handed its
/// addresses and resolves nothing, so a test watching addresses alone could not
/// tell a proxy from a provider at all.
fn a_listener_that_reports_what_it_was_told() -> (SocketAddr, JoinHandle<Option<Vec<u8>>>) {
    let listener = TcpListener::bind(SocketAddr::new(our_own_address(), 0)).expect("a listener");
    let at = listener.local_addr().expect("an address");
    listener.set_nonblocking(true).expect("a listener");
    let heard = std::thread::spawn(move || {
        let until = Instant::now() + Duration::from_secs(5);
        while Instant::now() < until {
            if let Ok((mut stream, _)) = listener.accept() {
                drop(stream.set_nonblocking(false));
                drop(stream.set_read_timeout(Some(Duration::from_secs(2))));
                // Read until the end of what was said rather than once: the
                // opening of a `CONNECT` conversation is written in several
                // goes, and one read can catch the first line without the
                // headers under it — which is exactly where the name this
                // machine signs in with lives.
                let mut said = Vec::new();
                let mut lump = [0_u8; 512];
                while said.len() < 4096 {
                    match stream.read(&mut lump) {
                        Ok(0) | Err(_) => break,
                        Ok(read) => {
                            said.extend_from_slice(lump.get(..read).unwrap_or_default());
                            if said.windows(4).any(|end| end == b"\r\n\r\n") {
                                break;
                            }
                        }
                    }
                }
                return Some(said);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        None
    });
    (at, heard)
}

/// **A provider's list is asked for through a proxy this machine signed in
/// to**, and the provider is never spoken to directly.
///
/// Read off the wire: the `CONNECT` line says the question is being carried to
/// the provider, and the header beside it says who is asking.
#[test]
fn a_providers_list_is_asked_for_through_a_proxy_this_machine_signed_in_to() {
    let (proxy, at_the_proxy) = a_listener_that_reports_what_it_was_told();
    let (provider, at_the_provider) = a_listener_that_reports_what_it_was_told();

    let carried = the_road_out(
        &a_proxy_that_asks_who_you_are(proxy),
        &provider,
        &given_the_password("signing-in"),
    )
    .expect("the machine was given the password");
    assert_eq!(
        carried.shown(),
        Some(format!("http://{proxy}")),
        "what a person reads carries the credential"
    );

    let refused = Trying::provider(&a_provider(provider), None)
        .taking(carried.for_a_request().expect("the client can use it"))
        .under(&SourcePolicy::Anywhere);
    assert!(
        refused.is_err(),
        "the listener standing for the proxy answered a real model list"
    );

    let said = what_was_said(at_the_proxy).expect("nobody came to the proxy's door");
    let said = String::from_utf8_lossy(&said).into_owned();
    assert!(
        said.starts_with(&format!("CONNECT {provider} ")),
        "the proxy was not asked to carry this to the provider: {said}"
    );
    assert!(
        said.contains(&format!(
            "Proxy-Authorization: Basic {}\r\n",
            signed_in_as()
        )),
        "this machine reached the company's proxy without saying who it is: {said}"
    );
    assert!(
        what_was_said(at_the_provider).is_none(),
        "the provider was spoken to directly on a machine told to use a proxy"
    );
}

/// **A machine that was never given the password asks nobody for anything.**
///
/// There is no road to configure a request with, so nothing is built and no
/// socket is opened: not to the proxy without the credential it asked for, and
/// not around it.
#[test]
fn a_machine_that_was_never_given_the_password_asks_nobody_for_anything() {
    let (proxy, at_the_proxy) = a_listener_that_reports_what_it_was_told();
    let (provider, at_the_provider) = a_listener_that_reports_what_it_was_told();

    let refused = the_road_out(
        &a_proxy_that_asks_who_you_are(proxy),
        &provider,
        &given_nothing("given-nothing"),
    )
    .expect_err("a road was taken with no credential");
    assert_eq!(refused, NotSignedIn::NothingKeepsIt);
    assert!(refused.nothing_was_sent());

    assert!(
        what_was_said(at_the_proxy).is_none(),
        "this machine knocked on the company's proxy with no password to give it"
    );
    assert!(
        what_was_said(at_the_provider).is_none(),
        "this machine went round the company's proxy rather than through it"
    );
}

/// **A password anybody on the machine could read asks nobody either**, and is
/// told apart from one the machine never had.
#[cfg(unix)]
#[test]
fn a_password_anybody_could_read_asks_nobody_and_is_told_apart() {
    use std::os::unix::fs::PermissionsExt as _;

    let (proxy, at_the_proxy) = a_listener_that_reports_what_it_was_told();
    let (provider, _at_the_provider) = a_listener_that_reports_what_it_was_told();

    let directory = a_directory_of_its_own("readable-by-anybody");
    let at = directory.join(KEPT_UNDER);
    std::fs::write(&at, THE_PASSWORD).expect("a password");
    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o444)).expect("a mode");

    let refused = the_road_out(
        &a_proxy_that_asks_who_you_are(proxy),
        &provider,
        &TheMachinesPasswords::at(&directory),
    )
    .expect_err("a road was taken with a password anybody could read");
    assert_eq!(refused, NotSignedIn::ReadableByAnybody);
    assert!(what_was_said(at_the_proxy).is_none());
}

/// **A proxy that asks for no name never reaches the store**, so an ordinary
/// machine asks a provider's list exactly as it did — the credentials here are
/// a directory that does not exist.
#[test]
fn a_proxy_that_asks_for_no_name_never_reaches_the_store() {
    let nowhere = given_nothing("never-asked");
    let provider = SocketAddr::new(our_own_address(), 9);
    let plain = TheProxy::one(
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).expect("an address"),
    );

    let carried =
        the_road_out(&plain, &provider, &nowhere).expect("nothing was asked of the store");
    assert_eq!(
        carried.as_an_address(),
        Some("http://proxy.example.com:8080".to_owned())
    );

    let straight =
        the_road_out(&TheProxy::None, &provider, &nowhere).expect("nothing was asked of the store");
    assert!(straight.is_straight());
    assert_eq!(straight.for_a_request().expect("nothing to refuse"), None);
}

/// What this machine says on the wire when it signs in, as the proxy reads it.
///
/// Worked out here rather than written down, so that a change to how a
/// credential is put on the wire fails as a difference on the socket rather
/// than as a constant nobody updated.
fn signed_in_as() -> String {
    let said = format!("{SIGNING_IN_AS}:{THE_PASSWORD}");
    let letters = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut written = String::new();
    for lump in said.as_bytes().chunks(3) {
        let byte = |place: usize| u32::from(lump.get(place).copied().unwrap_or_default());
        let held = byte(0) << 16 | byte(1) << 8 | byte(2);
        for place in 0..4 {
            if place <= lump.len() {
                let at = usize::try_from((held >> (18 - 6 * place)) & 0b11_1111).expect("a place");
                written.push(char::from(letters.get(at).copied().unwrap_or(b'=')));
            } else {
                written.push('=');
            }
        }
    }
    written
}

/// What was said at that door, waiting for the listener's own answer —
/// [`None`] where nobody came.
fn what_was_said(listener: JoinHandle<Option<Vec<u8>>>) -> Option<Vec<u8>> {
    listener.join().expect("the listener")
}
