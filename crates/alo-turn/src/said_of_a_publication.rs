//! **What a turn says on the journal about a publication**, so a walk on a
//! booted machine can check that alo published the folder, found nothing new,
//! or was stopped because nobody is signed in to GitHub — without reading the
//! screen (`alo_acting::said_of_publishing`).
//!
//! Said on the two roads a publication ends on at the person's own turns:
//! carried out (`crate::carrying`), after an approval or at full access; and
//! refused before anybody is asked, because `gh` had nobody signed in or could
//! not be asked (`crate::whose_site_it_is`). **A publication refused at the
//! sealed box or across a pairing says nothing here**: neither road asks `gh`
//! anything, and no walk publishes from either.
//!
//! **Never the folder or the account**, for `alo_acting::said_of_publishing`'s
//! reason. Written as the daemon's own stderr, which is the journal.

use alo_acting::said_of_publishing::{Publishing, a_publication};
use alo_publishing::NotPublished;

use crate::refusing::NotDone;
use crate::what_a_verb_did::WhatAVerbDid;

/// The sentence for a publication that was carried out, or tried.
#[must_use]
pub(crate) fn said(did: &Result<WhatAVerbDid, NotDone>) -> String {
    a_publication(match did {
        Ok(did) => match did.published() {
            Some(published) if published.nothing_new() => Publishing::NothingNew,
            Some(_) => Publishing::Published,
            // A publication that answered something other than a publication
            // is the machine contradicting itself, and is not one done.
            None => Publishing::NotDone,
        },
        Err(NotDone::CouldNotPublish(why)) => became_of(why),
        Err(_) => Publishing::NotDone,
    })
}

/// The sentence for a publication refused before anybody was asked.
#[must_use]
pub(crate) fn said_before_asking(why: &NotPublished) -> String {
    a_publication(became_of(why))
}

/// What a publication that did not happen became.
fn became_of(why: &NotPublished) -> Publishing {
    match why {
        NotPublished::NotSignedIn => Publishing::NotSignedIn,
        _ => Publishing::NotDone,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
#[path = "said_of_a_publication_tests.rs"]
mod tests;
