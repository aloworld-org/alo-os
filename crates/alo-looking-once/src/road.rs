//! The way out this machine takes to ask, decided rather than inherited.
//!
//! On a great many company networks there is no other route out, so a check
//! that ignored the machine's proxy would be a machine that can never find out
//! there is an update. `alo_proxy::the_way` decides it for
//! `alo_proxy::Road::CheckingForAnUpdate`, from the setting kept at
//! `alo_networks::proxy_file::THE_MACHINES_PROXY` — one file, one answer, and
//! no second spelling of that path here.
//!
//! **Said rather than left unsaid.** The road is handed to the client
//! explicitly ([`alo_looking::TheRegistry::taking`]), so what alo OS honours is
//! this machine's setting rather than whatever environment the service that
//! started this process happened to be carrying.
//!
//! **A machine with no proxy file goes straight out**, which is what a machine
//! on an ordinary network is. A machine whose proxy file is there and cannot be
//! read does **not**: it is refused, and the check is not made. Reading an
//! unreadable rule as *no rule* is how a company's traffic goes around its own
//! proxy with nobody told.
//!
//! # A proxy that asks who this machine is, is signed in to here
//!
//! `alo_proxy::signed_in` is the one door a proxy credential travels through,
//! and the password comes from the machine's own credentials at [`THE_UNIT`] —
//! [ADR 0059](../../../docs/decisions/0059-where-a-machine-wide-proxy-password-is-kept.md).
//! **Not the keyring ADR 0022 chose**, and this unit is the reason that record
//! exists: it runs at `multi-user.target` with nobody signed in, so a store on
//! the person's session bus would mean a machine on a company network never
//! finding out there is an update until somebody logs in.
//!
//! A machine that cannot sign in **refuses the check** — the same sentence as
//! the unreadable rule above, and for the same reason.

use std::io::ErrorKind;

use alo_looking::Place;
use alo_networks::proxy_file::{THE_MACHINES_PROXY, kept_on_this_machine};
use alo_proxy::{
    Carried, NotSignedIn, Reaching, Road, Scheme, TheMachinesPasswords, TheProxy,
    TheRentedEvaluator, WhereThePasswordsAre, signed_in, the_way,
};

/// This program's own unit, which is where the machine puts the credentials it
/// gave it.
///
/// Written here rather than worked out, for `alo_proxy::provisioned`'s reason:
/// a name cannot be pointed somewhere by a variable. It is also the line
/// `alo-looking-once.service` has to carry before a proxy that asks for a name
/// works on this road — ADR 0059 names it exactly.
pub const THE_UNIT: &str = "alo-looking-once.service";

/// Why this machine could not decide a road out to the place it asks.
#[derive(Debug, thiserror::Error)]
pub enum NoRoad {
    /// The machine's proxy setting is there and could not be read.
    #[error("the machine's proxy at {path} could not be read, so nothing was asked: {why}")]
    NotRead {
        /// Where the setting is kept.
        path: String,
        /// What was wrong with it.
        why: String,
    },
    /// The place the pin names is not somewhere a road can be decided to.
    #[error("the place this machine's updates come from is not one a road can be decided to: {0}")]
    NotReachable(alo_proxy::NotReachable),
    /// The proxy is an automatic configuration and this machine could not work
    /// out from it where the road goes.
    ///
    /// **The road is then not taken**, never taken straight out instead.
    #[error("the machine's proxy did not say where this road goes, so nothing was asked: {0:?}")]
    NotDecided(alo_proxy::NotOnTheRoad),
    /// The proxy asks who this machine is, and it could not sign in.
    ///
    /// **The road is then not taken**: not to the proxy as somebody with no
    /// password, and not around the proxy either. Written with `{0:?}` rather
    /// than `{0}` because `alo_proxy::NotSignedIn` has no `Display` — every one
    /// of these is about a credential, and its `Debug` carries none.
    #[error("this machine could not sign in to the proxy, so nothing was asked: {0:?}")]
    NotSignedIn(NotSignedIn),
}

/// The way out to `place`, decided from this machine's own proxy setting and
/// signed in to with the password this machine was given.
///
/// # Errors
/// [`NoRoad`], on every one of which nothing is asked and nothing leaves.
pub fn the_road_out(place: &Place) -> Result<Carried, NoRoad> {
    road_out(
        place,
        &this_machines_proxy()?,
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
    place: &Place,
    proxy: &TheProxy,
    signing_in: &dyn WhereThePasswordsAre,
) -> Result<Carried, NoRoad> {
    let going_to = Reaching::over(Scheme::Https, place.host()).map_err(NoRoad::NotReachable)?;
    let way = the_way(
        proxy,
        Road::CheckingForAnUpdate,
        &going_to,
        &TheRentedEvaluator::on_this_machine(),
    )
    .map_err(NoRoad::NotDecided)?;
    signed_in(way, signing_in).map_err(NoRoad::NotSignedIn)
}

/// The machine's proxy, or none where this machine has no setting at all.
fn this_machines_proxy() -> Result<TheProxy, NoRoad> {
    let bytes = match std::fs::read(THE_MACHINES_PROXY) {
        Ok(bytes) => bytes,
        Err(why) if why.kind() == ErrorKind::NotFound => return Ok(TheProxy::None),
        Err(why) => {
            return Err(NoRoad::NotRead {
                path: THE_MACHINES_PROXY.to_owned(),
                why: why.to_string(),
            });
        }
    };
    kept_on_this_machine(&bytes)
        .map(|kept| kept.proxy().clone())
        .map_err(|why| NoRoad::NotRead {
            path: THE_MACHINES_PROXY.to_owned(),
            why: why.to_string(),
        })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::path::{Path, PathBuf};

    use alo_proxy::{ProxyAddress, SpokenTo, WhereThePasswordIs};

    use super::*;

    /// Where this machine's updates come from, as the pin this repository
    /// ships names it.
    fn the_place() -> Place {
        Place::on_this_machine().expect("the pin this repository ships")
    }

    /// The company's proxy, asking who this machine is.
    fn a_proxy_that_asks_who_you_are() -> TheProxy {
        TheProxy::one(
            ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080)
                .expect("an address")
                .signing_in(
                    "anna",
                    WhereThePasswordIs::named("the company proxy").expect("a name"),
                )
                .expect("a proxy that asks who you are"),
        )
    }

    /// A directory of this test's own, empty.
    fn a_directory_of_its_own(named: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("alo-looking-once-{named}"));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        directory
    }

    /// The credentials a machine gave this unit, holding that proxy's password.
    fn given_the_password(named: &str) -> TheMachinesPasswords {
        let directory = a_directory_of_its_own(named);
        let at = directory.join("the company proxy");
        std::fs::write(&at, "hunter2").unwrap();
        held_the_way_a_machine_holds_one(&at);
        TheMachinesPasswords::at(&directory)
    }

    /// The permissions a machine keeps a credential with.
    #[cfg(unix)]
    fn held_the_way_a_machine_holds_one(at: &Path) {
        use std::os::unix::fs::PermissionsExt as _;

        std::fs::set_permissions(at, std::fs::Permissions::from_mode(0o400)).unwrap();
    }

    /// Nothing to do where a mode is not what says it.
    #[cfg(not(unix))]
    fn held_the_way_a_machine_holds_one(_at: &Path) {}

    /// **The one look this machine takes on its way up signs in to a proxy
    /// that asks who it is** — with the password the machine was given, before
    /// anybody has signed in, which is the whole reason ADR 0059 chose this
    /// store over a keyring on the person's own bus.
    #[test]
    fn the_look_this_machine_takes_on_its_way_up_signs_in_to_the_proxy() {
        let carried = road_out(
            &the_place(),
            &a_proxy_that_asks_who_you_are(),
            &given_the_password("signing-in"),
        )
        .expect("a road out");

        assert_eq!(
            carried.as_an_address(),
            Some("http://anna:hunter2@proxy.example.com:8080".to_owned())
        );
        assert_eq!(
            carried.shown(),
            Some("http://proxy.example.com:8080".to_owned()),
            "what a person reads carries the credential"
        );
        assert!(!format!("{carried:?}").contains("hunter2"));
    }

    /// **A machine that was never given the password refuses the check**, and
    /// never asks the place directly instead.
    #[test]
    fn a_machine_that_was_never_given_the_password_refuses_the_check() {
        let nowhere = a_directory_of_its_own("given-nothing").join("never-made");
        let refused = road_out(
            &the_place(),
            &a_proxy_that_asks_who_you_are(),
            &TheMachinesPasswords::at(&nowhere),
        )
        .expect_err("a road was taken with no credential");

        assert!(matches!(
            refused,
            NoRoad::NotSignedIn(NotSignedIn::NothingKeepsIt)
        ));
        assert!(!refused.to_string().contains("hunter2"), "{refused}");
    }

    /// **A proxy that asks for no name never reaches the store**, so an
    /// ordinary machine meets none of this — the credentials here are a
    /// directory that does not exist.
    #[test]
    fn a_proxy_that_asks_for_no_name_never_reaches_the_store() {
        let nowhere = a_directory_of_its_own("never-asked").join("never-made");
        let plain = TheProxy::one(
            ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).expect("an address"),
        );
        let carried = road_out(&the_place(), &plain, &TheMachinesPasswords::at(&nowhere))
            .expect("a road out");
        assert_eq!(
            carried.as_an_address(),
            Some("http://proxy.example.com:8080".to_owned())
        );

        let straight = road_out(
            &the_place(),
            &TheProxy::None,
            &TheMachinesPasswords::at(&nowhere),
        )
        .expect("a road out");
        assert!(straight.is_straight());
    }
}
