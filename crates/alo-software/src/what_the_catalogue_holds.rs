//! What an agent is shown when it looks at the catalogue.
//!
//! The answer to [`crate::WHAT_CAN_BE_INSTALLED`], and **the place the one
//! thing this read may not say is refused by the type rather than by a
//! comment**: there is no constructor here that takes
//! `alo_applications::Installed`, a [`crate::Tool`], or anything else that
//! knows about this machine. The only way to make one is from a
//! [`Catalogue`] — a file this repository ships, byte-identical on every alo
//! machine — and a kind of work.
//!
//! # Why that line is where it is
//!
//! `crate::verbs`' header refused this read until 2026-10-17, in one bullet
//! covering two different questions: *looking at what is installed, or what
//! could be.* The reason it gave is right about the first and does not reach
//! the second, and that file now says so where the next reader of the verb list
//! will find it.
//!
//! The short of it: **what a person has installed is a fingerprint of who they
//! are, and the catalogue is a constant.** One is `include_str!` of a file in
//! this crate; the other is a scan of somebody's machine. A read that answered
//! both would have been the background reader `CLAUDE.md` calls a bug. A read
//! that answers only the first is a list of eight applications that is the same
//! on every machine sold.
//!
//! # What this means for *is it already here?*
//!
//! It is **not answered**, and an agent that wants to know cannot ask. That is
//! not an oversight and it is not free either: an agent proposing *install
//! Krita* for somebody who already has Krita is proposing a wasted approval.
//! What happens then is that the installation itself refuses —
//! `crate::installing` answers *it is already here* before anything leaves the
//! machine — and the person reads that rather than approving a download. One
//! wasted sentence is a better trade than a verb that enumerates a person's
//! software.
//!
//! The window a person opens knows, because a person may see their own
//! machine: `crate::Catalogue` plus what the host read off the sandbox is what
//! `alo_shell::MarketWindow` is handed. That is the person's own screen and not
//! an agent's answer, and the asymmetry is the point.

use crate::catalogue::{Catalogue, Listed};
use crate::category::Category;

/// What the catalogue holds under one kind of work, or under all of them.
///
/// **Owned rather than borrowed**, because the turn that asked is finished with
/// the `Catalogue` by the time this crosses a socket: `Catalogue::decided`
/// parses a compiled-in string, so the entries are cheap to copy and nothing is
/// kept alive by a borrow that has to outlive a reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatTheCatalogueHolds {
    /// The kind of work it was looked at under, and [`None`] for all of them.
    under: Option<Category>,
    /// What is on that shelf, in the order the catalogue keeps.
    shown: Vec<Listed>,
}

impl WhatTheCatalogueHolds {
    /// What is on one shelf of this catalogue, or on all of them.
    ///
    /// `None` is *everything*, which is what the ninth choice means and what
    /// the window a person opens shows first.
    #[must_use]
    pub fn under(catalogue: &Catalogue, kind: Option<Category>) -> Self {
        // **One expression for both, so the two answers cannot drift.** A
        // branch that built the whole list one way and a narrowed list another
        // is how *everything* comes to be in a different order from
        // *everything, narrowed and widened again*.
        let shown = catalogue
            .every()
            .iter()
            .filter(|one| kind.is_none_or(|under| one.category() == under))
            .cloned()
            .collect();
        Self { under: kind, shown }
    }

    /// The kind of work this was looked at under, and [`None`] for all of them.
    #[must_use]
    pub const fn kind(&self) -> Option<Category> {
        self.under
    }

    /// What is on the shelf, in the order the catalogue keeps.
    #[must_use]
    pub fn shown(&self) -> &[Listed] {
        &self.shown
    }

    /// How many there are.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.shown.len()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn the_catalogue() -> Catalogue {
        Catalogue::decided().unwrap()
    }

    /// **Everything is everything**, which is the answer the ninth choice gets
    /// and the one the window opens on.
    #[test]
    fn looking_under_everything_shows_the_whole_catalogue() {
        let catalogue = the_catalogue();
        let all = WhatTheCatalogueHolds::under(&catalogue, None);

        assert_eq!(all.kind(), None);
        assert_eq!(all.how_many(), catalogue.how_many());
        assert_eq!(all.shown(), catalogue.every());
    }

    /// **Narrowing shows that shelf and only that shelf**, and every shelf has
    /// something on it — which `category.rs` already holds the catalogue to, so
    /// a read can never answer a person with nothing.
    #[test]
    fn narrowing_shows_one_shelf_and_it_is_never_empty() {
        let catalogue = the_catalogue();
        let mut counted = 0;
        for kind in Category::EVERY {
            let shelf = WhatTheCatalogueHolds::under(&catalogue, Some(kind));

            assert_eq!(shelf.kind(), Some(kind));
            assert!(
                shelf.how_many() > 0,
                "nothing is on the {} shelf",
                kind.named()
            );
            for one in shelf.shown() {
                assert_eq!(
                    one.category(),
                    kind,
                    "{} is on the {} shelf",
                    one.application().identifier(),
                    kind.named()
                );
            }
            counted += shelf.how_many();
        }
        assert_eq!(
            counted,
            catalogue.how_many(),
            "the shelves together are not the catalogue"
        );
    }

    /// **The shelves keep the catalogue's own order**, which is what makes a
    /// narrowed answer and a widened one agree about which application is
    /// first.
    #[test]
    fn a_shelf_is_in_the_catalogues_order() {
        let catalogue = the_catalogue();
        for kind in Category::EVERY {
            let shelf = WhatTheCatalogueHolds::under(&catalogue, Some(kind));
            let under: Vec<&Listed> = catalogue.under(kind).collect();

            assert_eq!(shelf.how_many(), under.len());
            for (mine, theirs) in shelf.shown().iter().zip(under) {
                assert_eq!(mine, theirs);
            }
        }
    }

    /// **Every entry carries its four fields**, because an agent shown an
    /// application with no source or no licence could propose an installation
    /// naming neither — and the licence is the thing a person choosing between
    /// two of these actually weighs.
    #[test]
    fn every_entry_carries_what_an_agent_would_need_to_propose_it() {
        let catalogue = the_catalogue();
        let all = WhatTheCatalogueHolds::under(&catalogue, None);

        assert!(all.how_many() >= 8, "{} entries", all.how_many());
        for one in all.shown() {
            assert!(!one.application().identifier().is_empty());
            assert!(!one.source().as_str().is_empty());
            assert!(!one.version().is_empty());
            assert!(!one.licence().is_empty());
        }
    }
}
