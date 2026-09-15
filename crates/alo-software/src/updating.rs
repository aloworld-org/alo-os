//! Updating an application: offered, chosen, and never applied while it is open.
//!
//! # Offered, never applied by itself
//!
//! Looking for updates leaves the machine, so it is an errand of its own
//! ([`looking_for_updates`], then [`offered`] under the line it put on the
//! indicator). What comes back is a list of [`Offer`]s, and an offer is only
//! that — nothing here applies one because it exists. [`Offer`] has no public
//! constructor: an update that was never looked for on the indicator cannot be
//! applied, because there is nothing to apply it with.
//!
//! # Never while it is open
//!
//! [`applying`] refuses an offer for an application that is open, before
//! anything leaves, and [`apply`] asks again under the line — an application
//! opened between the two steps is still open. The refusal is the whole of the
//! policy: nothing here closes an application to make room for its update,
//! because closing it is the person's to do and unsaved work is the one thing
//! an update could lose that nobody could give back.
//!
//! # And never the machine
//!
//! An application's update is not the system's (`alo-keeping-up` is), and
//! nothing in this file — or in the rented tool's update of one application —
//! restarts anything. [`Updated`] says so to the person in those words, and has
//! no field that could say otherwise.

use alo_applications::Application;
use alo_egress::{Errand, OnItsOwn, Underway};
use alo_strings::{Filling, Said, Strings};

use crate::enabled::{Enabled, did_not_answer};
use crate::refusing::NotDone;
use crate::shown::{Stopped, held_to};
use crate::source::SourceName;
use crate::tool::Tool;
use crate::words;

/// A newer version of an installed application, found on the indicator and
/// waiting for the person to choose it.
///
/// Deliberately not `Clone`: one offer is chosen once, and an offer that could
/// be copied could be applied by whoever held the copy after the person moved
/// on.
#[derive(Debug, PartialEq, Eq)]
pub struct Offer {
    /// The application.
    application: Application,
    /// The place the newer version comes from.
    source: SourceName,
}

impl Offer {
    /// The application.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// The place the newer version comes from.
    #[must_use]
    pub const fn source(&self) -> &SourceName {
        &self.source
    }

    /// What a person is offered, in the language they read.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &words::UPDATE_OFFERED.key(),
            &Filling::of(words::APPLICATION, self.application.identifier()),
        )
    }
}

/// An application that has been updated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Updated {
    /// The application.
    application: Application,
}

impl Updated {
    /// The application.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// What a person is told, in the language they read.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &words::UPDATED.key(),
            &Filling::of(words::APPLICATION, self.application.identifier()),
        )
    }
}

/// The errand of looking for updates at this place, if the place may be used.
///
/// # Errors
/// [`NotDone`], when the place may not be used ([`Enabled::usable`]).
pub fn looking_for_updates(enabled: &Enabled, source: &str) -> Result<OnItsOwn, NotDone> {
    let (_, destination) = enabled.usable(source)?;
    Ok(OnItsOwn::for_(
        Errand::CheckingForApplicationUpdates,
        destination.clone(),
    ))
}

/// What this place has newer versions of, heard while the errand is on the
/// indicator.
///
/// Only what is installed is offered, and an identifier no verb could name is
/// passed over rather than offered: it could not be shown or chosen.
///
/// # Errors
/// [`Stopped::NotShown`] for a line that is not this look;
/// [`Stopped::Refused`] when the place may no longer be used or could not be
/// asked.
pub fn offered(
    during: &Underway,
    enabled: &Enabled,
    source: &str,
    tool: &impl Tool,
) -> Result<Vec<Offer>, Stopped> {
    let (source, destination) = enabled.usable(source)?;
    held_to(during, Errand::CheckingForApplicationUpdates, destination)?;
    let installed = tool.installed().map_err(did_not_answer)?;
    let newer = tool.updates(source.name()).map_err(did_not_answer)?;
    Ok(newer
        .iter()
        .filter(|identifier| installed.contains(identifier))
        .filter_map(|identifier| Application::identified(identifier).ok())
        .map(|application| Offer {
            application,
            source: source.name().clone(),
        })
        .collect())
}

/// Everything asked before a chosen update leaves, and the errand to show.
///
/// # Errors
/// [`NotDone`] — the place may no longer be used, the application is no longer
/// installed, it is open, or the tool could not say.
pub fn applying(enabled: &Enabled, offer: &Offer, tool: &impl Tool) -> Result<OnItsOwn, NotDone> {
    let (_, destination) = enabled.usable(offer.source.as_str())?;
    still_closed_and_here(offer, tool)?;
    Ok(OnItsOwn::for_(
        Errand::UpdatingAnApplication,
        destination.clone(),
    ))
}

/// Apply the update the person chose, while the errand is on the indicator.
///
/// Takes the offer by value: it is spent whether the update happens or not,
/// and a refused one is looked for again rather than retried from a stale
/// list.
///
/// # Errors
/// [`Stopped::NotShown`] for a line that is not this update;
/// [`Stopped::Refused`] for everything [`applying`] refuses, asked again, and
/// for what the rented tool refused.
pub fn apply(
    during: &Underway,
    enabled: &Enabled,
    offer: Offer,
    tool: &impl Tool,
) -> Result<Updated, Stopped> {
    let (source, destination) = enabled.usable(offer.source.as_str())?;
    held_to(during, Errand::UpdatingAnApplication, destination)?;
    still_closed_and_here(&offer, tool)?;
    tool.update(&offer.application)
        .map_err(|failed| failed.about(source.name(), &offer.application))?;
    Ok(Updated {
        application: offer.application,
    })
}

/// Refused when the application is no longer installed, or is open.
fn still_closed_and_here(offer: &Offer, tool: &impl Tool) -> Result<(), NotDone> {
    let identifier = offer.application.identifier();
    if !tool
        .installed()
        .map_err(did_not_answer)?
        .iter()
        .any(|here| here == identifier)
    {
        return Err(NotDone::not_installed(&offer.application));
    }
    if tool
        .open()
        .map_err(did_not_answer)?
        .iter()
        .any(|open| open == identifier)
    {
        return Err(NotDone::StillOpen {
            application: identifier.to_owned(),
        });
    }
    Ok(())
}
