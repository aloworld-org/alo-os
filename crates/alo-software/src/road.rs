//! The way out this machine takes to install, read from the machine's own file.
//!
//! Task 4 decided the setting, task 11 carried it to a turn's question and task
//! 13 gave it a password a person can set — and until this file existed **the
//! two roads this crate owns took whichever proxy their caller happened to pass
//! them.** [`crate::TheRentedTool::taking`] is handed an `alo_proxy::Carried`,
//! and nothing built one from the machine's own setting. On a company network
//! that is a person who set a proxy in Settings and whose applications install
//! straight out around it — or do not install at all, which is the same bug
//! read from the other end.
//!
//! # One reader, and no second spelling of the path
//!
//! The setting is read through `alo_networks::proxy_file::kept_on_this_machine`
//! from `alo_networks::proxy_file::THE_MACHINES_PROXY`, which is the same one
//! reader and the same one path `alo-looking-once` uses. A second reader of a
//! machine-wide file would be a second answer to *what proxy is this machine
//! on*, and there is one answer.
//!
//! # Nothing here decides which way a road goes
//!
//! `alo_proxy::the_way` decides, and this file contains no branch on the
//! setting's shape, no exception list, no loopback check and nowhere one could
//! be added. What it does is turn a place applications come from into the two
//! facts a proxy decision is made from — the scheme and the host, which is
//! `alo_proxy::Reaching` — ask, and hand the answer on.
//!
//! # A machine with no file goes straight out. A machine with an unreadable one does not.
//!
//! No file at all is an ordinary machine on an ordinary network, and it is
//! [`alo_proxy::TheProxy::None`].
//!
//! A file that **is** there and cannot be read — unreadable, or holding
//! something this machine did not write — is a **refusal**, and the errand is
//! not run. Reading an unreadable rule as *no rule* is how a company's traffic
//! goes around the company's own proxy with nobody told, and on a company
//! network the difference between the two is the difference between *this did
//! not work* and *this left the building without permission*. Only one of those
//! can be undone.
//!
//! # A proxy that asks who this machine is, is signed in to here
//!
//! `alo_proxy::signed_in` is the one door a proxy credential travels through,
//! and the password comes from the machine's own credentials
//! ([ADR 0059](../../../docs/decisions/0059-where-a-machine-wide-proxy-password-is-kept.md)),
//! given to the unit that carries these errands out. A machine that cannot sign
//! in **refuses the errand** — not reaching the proxy as somebody with no
//! password, and not going around it either.

use std::io::ErrorKind;
use std::path::Path;

use alo_networks::proxy_file::{THE_MACHINES_PROXY, kept_on_this_machine};
use alo_proxy::{
    Carried, NotSignedIn, Reaching, Road, Scheme, TheMachinesPasswords, TheProxy,
    TheRentedEvaluator, WhereThePasswordsAre, signed_in, the_way,
};

use crate::rented::TheRentedTool;

/// The unit that carries these errands out, which is where the machine puts the
/// credentials it gave it.
///
/// Written here rather than worked out, for `alo_proxy::provisioned`'s reason:
/// a name cannot be pointed somewhere by a variable. This crate is a library
/// rather than a program, so the unit named is the one that runs its roads
/// today — `alo-agentd` carries out the software verbs — and ADR 0059 wants
/// exactly this constant to exist, because *the constant in the crate is the
/// list of unit files that must carry the line*. A second unit that ever
/// installs an application has to carry it too, and would say so here.
pub const THE_UNIT: &str = "alo-agentd.service";

/// The scheme a place applications come from is reached over.
///
/// `crate::source::Source` only keeps a destination for an address whose host
/// this machine could read, and `host_of` reads only `https` and `http`
/// addresses — of which this is the one a proxy decision is made about, because
/// an install that went out unencrypted would be refused long before a proxy
/// was chosen.
const OVER: Scheme = Scheme::Https;

/// Why this machine could not decide a road out for an errand.
///
/// **On every one of these nothing is installed and nothing leaves.**
#[derive(Debug, thiserror::Error)]
pub enum NoRoad {
    /// The machine's proxy setting is there and could not be read.
    ///
    /// Including a file holding something this machine did not write, which is
    /// the case that matters: it is refused rather than read as *no proxy*.
    #[error("the machine's proxy at {path} could not be read, so nothing was installed: {why}")]
    NotRead {
        /// Where the setting is kept.
        path: String,
        /// What was wrong with it.
        why: String,
    },
    /// The place applications come from is not somewhere a road can be decided
    /// to.
    #[error("the place applications come from is not one a road can be decided to: {0}")]
    NotReachable(alo_proxy::NotReachable),
    /// The proxy is an automatic configuration and this machine could not work
    /// out from it where the road goes.
    ///
    /// **The road is then not taken**, never taken straight out instead.
    #[error(
        "the machine's proxy did not say where this road goes, so nothing was installed: {0:?}"
    )]
    NotDecided(alo_proxy::NotOnTheRoad),
    /// The proxy asks who this machine is, and it could not sign in.
    ///
    /// Written with `{0:?}` rather than `{0}` because `alo_proxy::NotSignedIn`
    /// has no `Display` — every one of these is about a credential, and its
    /// `Debug` carries none.
    #[error("this machine could not sign in to the proxy, so nothing was installed: {0:?}")]
    NotSignedIn(NotSignedIn),
}

/// The rented tool for one of this crate's roads, taking the way out this
/// machine's own setting decides.
///
/// This is what closes the gap the contract named: a caller that asks for the
/// tool by errand gets one carrying the machine's proxy, rather than one
/// carrying whatever it happened to pass.
///
/// # Errors
/// [`NoRoad`], on every one of which nothing is installed and nothing leaves.
pub fn the_tool_for(road: Road, host: &str) -> Result<TheRentedTool, NoRoad> {
    Ok(TheRentedTool::on_this_machine().taking(the_road_out(road, host)?))
}

/// The way out for one road to one place, from the machine's own file and the
/// machine's own credentials.
///
/// # Errors
/// [`NoRoad`].
pub fn the_road_out(road: Road, host: &str) -> Result<Carried, NoRoad> {
    road_out(
        &this_machines_proxy(Path::new(THE_MACHINES_PROXY))?,
        road,
        host,
        &TheMachinesPasswords::given_to(THE_UNIT),
    )
}

/// The same, with the setting and the machine's passwords named — which is what
/// a test can hand over, since neither the proxy file nor a credential exists on
/// a machine these tests run on.
///
/// `pub(crate)` on purpose: a production caller that could name either would be
/// a second answer to *what this machine's proxy is* and *where its credentials
/// are*, and there is one answer to each.
pub(crate) fn road_out(
    proxy: &TheProxy,
    road: Road,
    host: &str,
    signing_in: &dyn WhereThePasswordsAre,
) -> Result<Carried, NoRoad> {
    let going_to = Reaching::over(OVER, host).map_err(NoRoad::NotReachable)?;
    let way = the_way(
        proxy,
        road,
        &going_to,
        &TheRentedEvaluator::on_this_machine(),
    )
    .map_err(NoRoad::NotDecided)?;
    signed_in(way, signing_in).map_err(NoRoad::NotSignedIn)
}

/// The machine's proxy as the file at `at` holds it, or none where this machine
/// has no setting at all.
///
/// Takes the path rather than naming it, so that the three states a machine can
/// be in — no file, a file naming a proxy, a file this machine did not write —
/// are all measurable. [`the_road_out`] passes
/// `alo_networks::proxy_file::THE_MACHINES_PROXY` and nothing else does.
pub(crate) fn this_machines_proxy(at: &Path) -> Result<TheProxy, NoRoad> {
    let bytes = match std::fs::read(at) {
        Ok(bytes) => bytes,
        // No file is an ordinary machine on an ordinary network.
        Err(why) if why.kind() == ErrorKind::NotFound => return Ok(TheProxy::None),
        Err(why) => return Err(not_read(at, &why.to_string())),
    };
    kept_on_this_machine(&bytes)
        .map(|kept| kept.proxy().clone())
        .map_err(|why| not_read(at, &why.to_string()))
}

/// A setting that is there and could not be read, named by where it is.
fn not_read(at: &Path, why: &str) -> NoRoad {
    NoRoad::NotRead {
        path: at.display().to_string(),
        why: why.to_owned(),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    use alo_egress::{Destination, Errand, Indicator, OnItsOwn};

    use crate::tool::Tool as _;
    use alo_networks::proxy_file::machines;
    use alo_proxy::{ConfigurationAddress, Kept, ProxyAddress, SpokenTo, WhereThePasswordIs};

    use super::*;

    /// Where applications come from, as this machine reaches it.
    const THE_PLACE: &str = "dl.example.org";

    /// The two roads this task is about, and the third this crate takes.
    const THE_ROADS: [Road; 3] = [
        Road::InstallingAnApplication,
        Road::UpdatingAnApplication,
        Road::CheckingForApplicationUpdates,
    ];

    /// A directory of this test's own, empty.
    fn a_directory_of_its_own(named: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("alo-software-road-{named}"));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        directory
    }

    /// The company's proxy, asking nobody who they are.
    fn a_proxy() -> ProxyAddress {
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).expect("an address")
    }

    /// The machine's file, written the way this machine writes one.
    fn a_machine_whose_file_names(proxy: &TheProxy, named: &str) -> PathBuf {
        let at = a_directory_of_its_own(named).join("proxy.json");
        let kept = Kept::by_this_person(proxy.clone());
        std::fs::write(&at, machines(&kept).expect("a file this machine wrote")).unwrap();
        at
    }

    /// Credentials nobody was given, for a proxy that asks for nobody.
    fn given_nothing(named: &str) -> TheMachinesPasswords {
        TheMachinesPasswords::at(&a_directory_of_its_own(named).join("never-made"))
    }

    /// **A machine whose file names a proxy takes it on every road**, which is
    /// the whole of what this task was for.
    #[test]
    fn a_machine_whose_file_names_a_proxy_takes_it() {
        let at = a_machine_whose_file_names(&TheProxy::one(a_proxy()), "names-one");
        let proxy = this_machines_proxy(&at).expect("a file this machine wrote");

        for road in THE_ROADS {
            let carried = road_out(&proxy, road, THE_PLACE, &given_nothing("names-one-store"))
                .expect("a road out");
            assert_eq!(
                carried.as_an_address(),
                Some("http://proxy.example.com:8080".to_owned()),
                "{road:?} did not take the machine's proxy"
            );
            assert!(!carried.is_straight(), "{road:?} went straight out");
        }
    }

    /// **A machine with no file goes straight out**, which is what a machine on
    /// an ordinary network is.
    #[test]
    fn a_machine_with_no_file_goes_straight_out() {
        let nowhere = a_directory_of_its_own("no-file").join("proxy.json");
        let proxy = this_machines_proxy(&nowhere).expect("no file is not an error");
        assert_eq!(proxy, TheProxy::None);

        for road in THE_ROADS {
            let carried = road_out(&proxy, road, THE_PLACE, &given_nothing("no-file-store"))
                .expect("a road out");
            assert!(carried.is_straight(), "{road:?} did not go straight out");
        }
    }

    /// **A file this machine did not write refuses the errand**, and is never
    /// read as *no proxy*.
    ///
    /// This is the one that matters on a company network. A machine that read
    /// an unreadable rule as no rule would send a company's traffic around the
    /// company's own proxy with nobody told — and the refusal says where the
    /// file is, because whoever administers the machine is who can fix it.
    #[test]
    fn a_file_this_machine_did_not_write_refuses_rather_than_going_straight_out() {
        for (why, bytes) in [
            ("nothing at all", b"".as_slice()),
            ("something that is not this file", b"{}".as_slice()),
            (
                "a proxy spelled another way",
                br#"{"proxy":"http://p:8080"}"#.as_slice(),
            ),
            ("not even text", b"\x00\x01\x02".as_slice()),
        ] {
            let at = a_directory_of_its_own("not-ours").join("proxy.json");
            std::fs::write(&at, bytes).unwrap();
            let refused = this_machines_proxy(&at)
                .expect_err(&format!("a file holding {why} was read as a setting"));
            assert!(
                matches!(&refused, NoRoad::NotRead { path, .. } if path == &at.display().to_string()),
                "a file holding {why} was refused without saying where it is: {refused}"
            );
        }
    }

    /// **A proxy that asks who this machine is, and no credential, takes no
    /// road at all** — not to the proxy as nobody, and not around it.
    #[test]
    fn a_machine_that_was_never_given_the_password_takes_no_road() {
        let asks = TheProxy::one(
            a_proxy()
                .signing_in(
                    "anna",
                    WhereThePasswordIs::named("the company proxy").expect("a name"),
                )
                .expect("a proxy that asks who you are"),
        );
        let at = a_machine_whose_file_names(&asks, "asks-who");
        let proxy = this_machines_proxy(&at).expect("a file this machine wrote");

        for road in THE_ROADS {
            let refused = road_out(&proxy, road, THE_PLACE, &given_nothing("asks-who-store"))
                .expect_err("a road was taken with no credential");
            assert!(
                matches!(refused, NoRoad::NotSignedIn(NotSignedIn::NothingKeepsIt)),
                "{road:?} was refused for another reason: {refused:?}"
            );
        }
    }

    /// **Each errand on a machine whose file names a proxy gets a tool
    /// configured with it**, and the program really started really receives it.
    ///
    /// This is the half that earns the task. The tests above prove the *road*
    /// carries the machine's proxy; this proves an **errand** does — the two
    /// roads this plan owns, from the file, through `the_way`, onto the
    /// environment of the program that is actually started. A road that was
    /// decided correctly and never reached the tool would pass everything above
    /// and still install straight out.
    #[cfg(unix)]
    #[test]
    fn each_errand_on_a_machine_with_a_proxy_gets_a_tool_configured_with_it() {
        let at = a_machine_whose_file_names(&TheProxy::one(a_proxy()), "errands");
        let proxy = this_machines_proxy(&at).expect("a file this machine wrote");

        for road in [Road::InstallingAnApplication, Road::UpdatingAnApplication] {
            let carried = road_out(&proxy, road, THE_PLACE, &given_nothing("errands-store"))
                .expect("a road out");
            let tool = TheRentedTool::on_this_machine().taking(carried);
            let given = tool.environment();
            for name in ["http_proxy", "HTTP_PROXY", "https_proxy", "HTTPS_PROXY"] {
                assert_eq!(
                    given
                        .iter()
                        .find(|(each, _)| *each == name)
                        .map(|(_, value)| value.as_str()),
                    Some("http://proxy.example.com:8080"),
                    "{road:?} was not given {name} from the machine's own file"
                );
            }
        }
    }

    /// **And a machine with no file gets a tool told there is none**, rather
    /// than one left to inherit whatever started it.
    #[cfg(unix)]
    #[test]
    fn each_errand_on_a_machine_with_no_file_gets_a_tool_that_goes_straight_out() {
        let nowhere = a_directory_of_its_own("errands-no-file").join("proxy.json");
        let proxy = this_machines_proxy(&nowhere).expect("no file is not an error");

        for road in [Road::InstallingAnApplication, Road::UpdatingAnApplication] {
            let carried = road_out(
                &proxy,
                road,
                THE_PLACE,
                &given_nothing("errands-none-store"),
            )
            .expect("a road out");
            let tool = TheRentedTool::on_this_machine().taking(carried);
            let given = tool.environment();
            assert!(
                !given
                    .iter()
                    .any(|(name, value)| name.eq_ignore_ascii_case("http_proxy")
                        && !value.is_empty()),
                "{road:?} was given a proxy on a machine that has none: {given:?}"
            );
        }
    }

    /// **The indicator still names where the errand is really going, and never
    /// the proxy.**
    ///
    /// A person watching the indicator is being told where their machine is
    /// reaching. On a company network every errand goes *through* the proxy, so
    /// an indicator that named the proxy would say the same thing for every
    /// errand on the machine and tell nobody anything — and it would be a
    /// person's own destination replaced by their employer's equipment.
    #[test]
    fn the_indicator_names_where_the_errand_goes_and_never_the_proxy() {
        let at = a_machine_whose_file_names(&TheProxy::one(a_proxy()), "indicator");
        let proxy = this_machines_proxy(&at).expect("a file this machine wrote");
        let carried = road_out(
            &proxy,
            Road::InstallingAnApplication,
            THE_PLACE,
            &given_nothing("indicator-store"),
        )
        .expect("a road out");
        assert!(!carried.is_straight(), "the road did not take the proxy");

        let strings = crate::testing::in_english();
        let mut indicator = Indicator::default();
        let underway = indicator.beginning_on_its_own(
            OnItsOwn::for_(
                Errand::InstallingAnApplication,
                Destination::at(THE_PLACE).expect("a destination"),
            ),
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000),
        );
        let said: String = indicator
            .showing()
            .iter()
            .map(|shown| shown.showing().said(&strings).text().to_owned())
            .collect::<Vec<_>>()
            .join(" | ");
        indicator.ended_on_its_own(underway);

        assert!(
            said.contains(THE_PLACE),
            "the indicator did not name where the errand is going: {said}"
        );
        assert!(
            !said.contains("proxy.example.com"),
            "the indicator named the proxy rather than the destination: {said}"
        );
    }

    /// The credentials a machine gave this unit, holding that proxy's password.
    fn given_the_password(named: &str) -> TheMachinesPasswords {
        let directory = a_directory_of_its_own(named);
        let at = directory.join("the company proxy");
        std::fs::write(&at, "hunter2").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o400)).unwrap();
        }
        TheMachinesPasswords::at(&directory)
    }

    /// A program that prints the environment it was given, one line each.
    #[cfg(unix)]
    fn a_program_that_prints_its_environment(named: &str) -> PathBuf {
        use std::io::Write as _;
        use std::os::unix::fs::PermissionsExt as _;

        let program = a_directory_of_its_own(named).join("tool.sh");
        let mut written = std::fs::File::create(&program).unwrap();
        written
            .write_all(b"#!/bin/sh\nexec /usr/bin/env\n")
            .unwrap();
        drop(written);
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
        program
    }

    /// **The program the tool really starts really receives the credential**
    /// for a proxy the machine's own file named.
    ///
    /// Everything else here is a list of values; this starts a process and
    /// reads what it actually got. On this road the credential is delivered to
    /// a **rented program** through its environment, so the wire belongs to
    /// that program — what alo OS can be held to is that the password reached
    /// it, and that nothing of this process's own environment did.
    #[cfg(unix)]
    #[test]
    fn the_program_really_receives_the_credential_for_the_proxy_the_file_named() {
        let asks = TheProxy::one(
            a_proxy()
                .signing_in(
                    "anna",
                    WhereThePasswordIs::named("the company proxy").expect("a name"),
                )
                .expect("a proxy that asks who you are"),
        );
        let at = a_machine_whose_file_names(&asks, "really-received");
        let proxy = this_machines_proxy(&at).expect("a file this machine wrote");
        let carried = road_out(
            &proxy,
            Road::InstallingAnApplication,
            THE_PLACE,
            &given_the_password("really-received-store"),
        )
        .expect("the machine was given the password");

        let program = a_program_that_prints_its_environment("really-received-tool");
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

    /// **A proxy that is set and cannot be used refuses the errand**, and never
    /// falls back to going straight out.
    ///
    /// The case is an automatic configuration on a machine with nothing at the
    /// path that evaluates one. It is the sharpest form of the rule this whole
    /// file exists for: on a company network *this did not work* is recoverable
    /// and *this left the building without permission* is not, so where the
    /// machine cannot work out its road it takes none.
    ///
    /// There is no branch here that could do otherwise — `road_out` returns the
    /// road or an error and has no third answer — and this is the test that
    /// would fail the day somebody added one.
    #[test]
    fn a_proxy_that_is_set_and_cannot_be_worked_out_refuses_rather_than_going_straight() {
        let automatic = TheProxy::Automatic {
            at: ConfigurationAddress::checked("https://proxy.example.com/proxy.pac")
                .expect("an address a configuration is at"),
        };
        let at = a_machine_whose_file_names(&automatic, "automatic");
        let proxy = this_machines_proxy(&at).expect("a file this machine wrote");

        for road in THE_ROADS {
            let refused = road_out(&proxy, road, THE_PLACE, &given_nothing("automatic-store"))
                .expect_err("a road was taken under a configuration nothing could evaluate");
            assert!(
                matches!(refused, NoRoad::NotDecided(_)),
                "{road:?} was refused for another reason: {refused:?}"
            );
        }
    }

    /// **A place this machine cannot reach is refused** rather than decided
    /// about.
    #[test]
    fn a_place_no_road_can_be_decided_to_is_refused() {
        let refused = road_out(
            &TheProxy::None,
            Road::InstallingAnApplication,
            "",
            &given_nothing("nowhere-store"),
        )
        .expect_err("a road was decided to nowhere");
        assert!(matches!(refused, NoRoad::NotReachable(_)), "{refused:?}");
    }
}
