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

use std::io::ErrorKind;

use alo_looking::Place;
use alo_networks::proxy_file::{THE_MACHINES_PROXY, kept_on_this_machine};
use alo_proxy::{Carried, Reaching, Road, Scheme, TheProxy, TheRentedEvaluator, the_way};

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
}

/// The way out to `place`, decided from this machine's own proxy setting.
///
/// # Errors
/// [`NoRoad`], on every one of which nothing is asked and nothing leaves.
pub fn the_road_out(place: &Place) -> Result<Carried, NoRoad> {
    let proxy = this_machines_proxy()?;
    let going_to = Reaching::over(Scheme::Https, place.host()).map_err(NoRoad::NotReachable)?;
    let way = the_way(
        &proxy,
        Road::CheckingForAnUpdate,
        &going_to,
        &TheRentedEvaluator::on_this_machine(),
    )
    .map_err(NoRoad::NotDecided)?;
    Ok(Carried::of(way))
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
