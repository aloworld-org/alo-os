//! The way out the unit that fetches a build takes, read from the machine's
//! own file rather than inherited from whatever started it.
//!
//! Staging an update pulls a whole system onto the disk, and on a great many
//! company networks there is no other route out. `alo-updating`'s own
//! documentation says the machine's proxy belongs on this road and leaves it to
//! the caller to supply one; on this road the caller is the unit in this crate,
//! so this is where it is supplied.
//!
//! The setting is read through `alo_networks::proxy_file::kept_on_this_machine`
//! from `alo_networks::proxy_file::THE_MACHINES_PROXY` — the same one reader
//! and the same one path `alo-looking-once` and `alo-software` use. A second
//! reader of a machine-wide file would be a second answer to *what proxy is
//! this machine on*, and there is one answer.
//!
//! **A machine with no file goes straight out. A machine with an unreadable one
//! does not.** No file is an ordinary machine on an ordinary network. A file
//! that is there and holds something this machine did not write is a refusal,
//! and nothing is fetched: reading an unreadable rule as *no rule* is how a
//! company's traffic goes around the company's own proxy with nobody told, and
//! on a company network *this did not work* can be undone while *this left the
//! building without permission* cannot.
//!
//! **Nothing here decides which way a road goes.** `alo_proxy::the_way` does,
//! for `alo_proxy::Road::FetchingAnUpdate`, and there is no branch on the
//! setting's shape in this file and nowhere one could be added.
//!
//! # Going back takes no road at all
//!
//! It fetches nothing: the build is already on the disk. So there is one road
//! here rather than two, and `alo-going-back.service` reads no proxy, needs no
//! credential and reaches no network.

use std::io::ErrorKind;
use std::path::Path;

use alo_networks::proxy_file::{THE_MACHINES_PROXY, kept_on_this_machine};
use alo_proxy::{
    Carried, NotSignedIn, Reaching, Road, Scheme, TheMachinesPasswords, TheProxy,
    TheRentedEvaluator, WhereThePasswordsAre, signed_in, the_way,
};

/// The unit that fetches a build, which is where the machine puts the
/// credentials it gave it.
///
/// Written here rather than worked out, for `alo_proxy::provisioned`'s reason:
/// a name cannot be pointed somewhere by a variable. ADR 0059 wants exactly
/// this constant to exist, because *the constant in the crate is the list of
/// unit files that must carry the line*.
pub const THE_UNIT: &str = "alo-applying-an-update.service";

/// The scheme a registry is reached over. A build fetched unencrypted is
/// refused long before a proxy is chosen.
const OVER: Scheme = Scheme::Https;

/// Why this machine could not decide a road out to where its builds come from.
///
/// **On every one of these nothing is fetched and nothing leaves.**
#[derive(Debug)]
pub enum NoRoad {
    /// The machine's proxy setting is there and could not be read — including
    /// a file holding something this machine did not write.
    NotRead {
        /// Where the setting is kept.
        path: String,
        /// What was wrong with it.
        why: String,
    },
    /// Where builds come from is not somewhere a road can be decided to.
    NotReachable(alo_proxy::NotReachable),
    /// The proxy is an automatic configuration and this machine could not work
    /// out from it where the road goes. **The road is then not taken**, never
    /// taken straight out instead.
    NotDecided(alo_proxy::NotOnTheRoad),
    /// The proxy asks who this machine is, and it could not sign in.
    ///
    /// Written with `{0:?}` rather than `{0}` because `alo_proxy::NotSignedIn`
    /// has no `Display` — every one of these is about a credential, and its
    /// `Debug` carries none.
    NotSignedIn(NotSignedIn),
}

impl std::fmt::Display for NoRoad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRead { path, why } => write!(
                f,
                "the machine's proxy at {path} could not be read, so nothing was fetched: {why}"
            ),
            Self::NotReachable(why) => write!(
                f,
                "where this machine's builds come from is not one a road can be decided to: {why}"
            ),
            Self::NotDecided(why) => write!(
                f,
                "the machine's proxy did not say where this road goes, so nothing was fetched: \
                 {why:?}"
            ),
            Self::NotSignedIn(why) => write!(
                f,
                "this machine could not sign in to the proxy, so nothing was fetched: {why:?}"
            ),
        }
    }
}

impl std::error::Error for NoRoad {}

/// The way out to `host`, from the machine's own file and the machine's own
/// credentials.
///
/// # Errors
/// [`NoRoad`], on every one of which nothing is fetched and nothing leaves.
pub fn the_road_out(host: &str) -> Result<Carried, NoRoad> {
    road_out(
        &this_machines_proxy(Path::new(THE_MACHINES_PROXY))?,
        host,
        &TheMachinesPasswords::given_to(THE_UNIT),
    )
}

/// The same, with the setting and the machine's passwords named — which is
/// what a test can hand over, since neither the proxy file nor a credential
/// exists on a machine these tests run on.
pub(crate) fn road_out(
    proxy: &TheProxy,
    host: &str,
    signing_in: &dyn WhereThePasswordsAre,
) -> Result<Carried, NoRoad> {
    let going_to = Reaching::over(OVER, host).map_err(NoRoad::NotReachable)?;
    let way = the_way(
        proxy,
        Road::FetchingAnUpdate,
        &going_to,
        &TheRentedEvaluator::on_this_machine(),
    )
    .map_err(NoRoad::NotDecided)?;
    signed_in(way, signing_in).map_err(NoRoad::NotSignedIn)
}

/// The machine's proxy as the file at `at` holds it, or none where this
/// machine has no setting at all.
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

    use alo_networks::proxy_file::machines;
    use alo_proxy::{ConfigurationAddress, Kept, ProxyAddress, SpokenTo, WhereThePasswordIs};

    use super::*;

    /// Where this machine's builds come from, for the road.
    const THE_PLACE: &str = "ghcr.io";

    /// A folder of this test's own, empty.
    fn a_folder(named: &str) -> PathBuf {
        let at =
            std::env::temp_dir().join(format!("alo-brokerd-road-{}-{named}", std::process::id()));
        drop(std::fs::remove_dir_all(&at));
        std::fs::create_dir_all(&at).unwrap();
        at
    }

    /// The company's proxy, asking nobody who they are.
    fn a_proxy() -> ProxyAddress {
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).expect("an address")
    }

    /// The machine's file, written the way this machine writes one.
    fn a_machine_whose_file_names(proxy: &TheProxy, named: &str) -> PathBuf {
        let at = a_folder(named).join("proxy.json");
        let kept = Kept::by_this_person(proxy.clone());
        std::fs::write(&at, machines(&kept).expect("a file this machine wrote")).unwrap();
        at
    }

    /// Credentials nobody was given.
    fn given_nothing(named: &str) -> TheMachinesPasswords {
        TheMachinesPasswords::at(&a_folder(named).join("never-made"))
    }

    /// **A machine whose file names a proxy fetches a build through it.**
    #[test]
    fn a_machine_whose_file_names_a_proxy_fetches_a_build_through_it() {
        let at = a_machine_whose_file_names(&TheProxy::one(a_proxy()), "names-one");
        let proxy = this_machines_proxy(&at).expect("a file this machine wrote");
        let carried =
            road_out(&proxy, THE_PLACE, &given_nothing("names-one-store")).expect("a road out");
        assert_eq!(
            carried.as_an_address(),
            Some("http://proxy.example.com:8080".to_owned())
        );
        assert!(!carried.is_straight());
    }

    /// **A machine with no file goes straight out**, which is what a machine
    /// on an ordinary network is.
    #[test]
    fn a_machine_with_no_file_goes_straight_out() {
        let nowhere = a_folder("no-file").join("proxy.json");
        let proxy = this_machines_proxy(&nowhere).expect("no file is not an error");
        assert_eq!(proxy, TheProxy::None);
        let carried =
            road_out(&proxy, THE_PLACE, &given_nothing("no-file-store")).expect("a road out");
        assert!(carried.is_straight());
    }

    /// **A file this machine did not write refuses the fetch**, and is never
    /// read as *no proxy*.
    #[test]
    fn a_file_this_machine_did_not_write_refuses_rather_than_going_straight_out() {
        for (why, bytes) in [
            ("nothing at all", b"".as_slice()),
            ("something that is not this file", b"{}".as_slice()),
            ("not even text", b"\x00\x01\x02".as_slice()),
        ] {
            let at = a_folder("not-ours").join("proxy.json");
            std::fs::write(&at, bytes).unwrap();
            let refused = this_machines_proxy(&at)
                .expect_err(&format!("a file holding {why} was read as a setting"));
            assert!(
                matches!(&refused, NoRoad::NotRead { path, .. } if path == &at.display().to_string()),
                "a file holding {why} was refused without saying where it is: {refused}"
            );
        }
    }

    /// **A proxy that asks who this machine is, and no credential, fetches
    /// nothing** — not through the proxy as nobody, and not around it.
    #[test]
    fn a_machine_that_was_never_given_the_password_fetches_nothing() {
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
        let refused = road_out(&proxy, THE_PLACE, &given_nothing("asks-who-store"))
            .expect_err("a road was taken with no credential");
        assert!(
            matches!(refused, NoRoad::NotSignedIn(NotSignedIn::NothingKeepsIt)),
            "{refused}"
        );
    }

    /// **A proxy that is set and cannot be worked out refuses rather than
    /// going straight out.**
    #[test]
    fn a_proxy_that_cannot_be_worked_out_refuses_rather_than_going_straight() {
        let automatic = TheProxy::Automatic {
            at: ConfigurationAddress::checked("https://proxy.example.com/proxy.pac")
                .expect("an address a configuration is at"),
        };
        let at = a_machine_whose_file_names(&automatic, "automatic");
        let proxy = this_machines_proxy(&at).expect("a file this machine wrote");
        let refused = road_out(&proxy, THE_PLACE, &given_nothing("automatic-store"))
            .expect_err("a road was taken under a configuration nothing could evaluate");
        assert!(matches!(refused, NoRoad::NotDecided(_)), "{refused}");
    }
}
