//! The put-aside windows of one Place, grouped — and still one preview per window.
//!
//! Task 7 of `docs/autonomy/the-canvas-and-its-places.md`, as the owner ruled it on
//! 2026-10-03: **grouped by Place by default**, one consistent mode, the current Place
//! first and the others in World order, **without hiding individual windows behind an
//! application icon**. The second half is as much the instruction as the first, and the two
//! pull against each other — grouping is how a long list becomes readable, and the usual way
//! to group a list of windows is by the application that owns them.
//!
//! # Grouped by Place, never by application
//!
//! [`Preview`]'s own header settles why: *a preview is one window, never one application* —
//! the Dock answers *which applications are open*, and three put-aside Browser windows here
//! are three previews with three titles. Grouping by application would make this panel a
//! second Dock with worse information.
//!
//! A Place is *where the person was working*, so somebody who put four windows aside while
//! writing and two while reading has two groups that mean something to them. Nothing is
//! hidden: each group holds every window of that Place, individually.
//!
//! # This crate has no opinion about the order of Places
//!
//! The order comes from a [`World`] handed in, because the World **is** the layout.
//! `World`'s own header gives the reason this file would otherwise have had to invent:
//! *where a Place sits in the World is a thing a person may come to rely on, so it is data
//! rather than a function of a `BTreeMap`'s ordering.*
//!
//! The first version of this file returned `BTreeMap<Place, Vec<&Preview>>` and defended
//! that order as *two identical panels must produce one layout*. True, and the wrong
//! property: `Place` is a `u64`, so the order was the order Places were **made** in, and
//! **deterministic and meaningless are compatible.** The ruling forbids exactly that — *do
//! not inherit ordering accidentally from `BTreeMap`*.
//!
//! # A Place the World cannot name, which the ruling could not have anticipated
//!
//! `Server::the_world` is built from `the_frames_on_the_plane`, which reads the **mapped**
//! surfaces. A put-aside window is hidden and therefore not mapped — so **a Place whose
//! windows have all been put aside is absent from the World.** The World cannot order the
//! very Places this panel most needs ordered, and a person who puts away the last window on
//! a Place makes that Place vanish from the layout.
//!
//! So those Places come after the ones the World names, **in the panel's own order** — most
//! recently put aside first, which `Panel` decides and documents. That is meaningful rather
//! than merely convenient: the window somebody put away last is the one they are most likely
//! to want back, and it is the only ordering they can predict. Falling back to `Place`'s
//! `Ord` here would reintroduce creation order through the back door, which is the thing
//! this change exists to remove.
//!
//! # Two orderings in this file are one rule, and they agree by decision rather than by luck
//!
//! **Most recently put aside first** is chosen twice here, independently:
//!
//! - inside a group, by [`AGroup::previews`] — which keeps `Panel`'s own order untouched;
//! - between the Places the World cannot name, in [`a_place_groups_its_windows`] — where
//!   there is no layout to read and an order had to be picked.
//!
//! They agree, which is right, and until this note was written they agreed **because the
//! same person made the same call twice** rather than because one rule was stated once. That
//! is the two-spellings fault this repository keeps finding: each spelling correct, and
//! nobody obliged to notice when one moves.
//!
//! So: **they are the same rule.** The window somebody put away last is the one they are
//! most likely to want back, and it is the only ordering they can predict. If that reasoning
//! ever stops holding, it stops holding for both — and changing one without the other would
//! give a person two orders in one panel, the outer groups disagreeing with the rows inside
//! them, which is worse than either order alone.
//!
//! The reading lane raised this on a cold read of `#427`, having noticed the agreement was a
//! coincidence of two decisions rather than one.
//!
//! # Derived, never stored
//!
//! The answer is built from the panel's one list on every call. `Panel::put_aside` is
//! `previews.insert(0, …)` and `bring_back` is `previews.remove(at)`, so **every put-aside
//! shifts every existing index and every restore shifts every later one.** A stored grouping
//! holding positions would be wrong after each of them, would have to be repaired on both
//! roads, and the day somebody adds a third road it would be repaired on neither.

use alo_canvas::{Place, World};

use crate::Preview;
use crate::panel::Panel;

/// Whether the drawer puts a heading on each group.
///
/// A named answer rather than a count for the drawer to compare, because the rule — *one
/// represented Place takes no redundant heading, several take one each* — is the owner's and
/// belongs in one place. A surface writing `if groups.len() > 1` has copied the rule, and
/// the copy is what goes stale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Headings {
    /// One Place is represented: its windows, with no heading above them.
    NotNeeded,
    /// Several are: a heading per group, with that Place's windows underneath.
    OnePerGroup,
}

/// One Place's put-aside windows, in the panel's own order.
#[derive(Debug)]
pub struct AGroup<'a> {
    /// The Place these windows were put aside on.
    place: Place,
    /// Its windows, in the order the panel holds them.
    previews: Vec<&'a Preview>,
}

impl<'a> AGroup<'a> {
    /// Which Place these windows were put aside on.
    #[must_use]
    pub const fn place(&self) -> Place {
        self.place
    }

    /// Its windows, **most recently put aside first** — the panel's own order, untouched.
    ///
    /// This file does not re-sort. The panel is a surface people point at, so re-ordering
    /// here would move the preview somebody was reaching for, and would give the crate two
    /// answers to *what order are the previews in*.
    #[must_use]
    pub fn previews(&self) -> &[&'a Preview] {
        &self.previews
    }
}

/// The panel's windows grouped by Place, ordered as the owner ruled.
#[derive(Debug)]
pub struct ThePanelInGroups<'a> {
    /// One per represented Place, in the order the owner ruled.
    groups: Vec<AGroup<'a>>,
}

impl<'a> ThePanelInGroups<'a> {
    /// The groups: the current Place first when it holds anything, then the rest.
    #[must_use]
    pub fn groups(&self) -> &[AGroup<'a>] {
        &self.groups
    }

    /// Whether each group takes a heading.
    #[must_use]
    pub const fn headings(&self) -> Headings {
        if self.groups.len() > 1 {
            Headings::OnePerGroup
        } else {
            Headings::NotNeeded
        }
    }

    /// Whether anything is put aside at all.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
}

/// Group the panel's windows by Place: the current Place, then World order, then the Places
/// the World cannot name.
///
/// A Place with nothing put aside is **absent** rather than present and empty — the same
/// answer `crate::the_collapse_choice_per_place` gives, and for the same reason: a group for
/// a Place somebody merely looked at is a row that means nothing to them. That holds for
/// `looking_at` too, which takes no group when nothing has been put aside there.
///
/// Every window the panel holds appears exactly once across the groups.
#[must_use]
pub fn a_place_groups_its_windows<'a>(
    panel: &'a Panel,
    looking_at: Place,
    world: &World,
) -> ThePanelInGroups<'a> {
    // The Places to ask for, each at most once: where the person is, then the World's
    // layout, then whatever the panel holds that the World did not name.
    let mut order: Vec<Place> = Vec::new();
    let remember = |place: Place, order: &mut Vec<Place>| {
        if !order.contains(&place) {
            order.push(place);
        }
    };
    remember(looking_at, &mut order);
    for place in world.each() {
        remember(place, &mut order);
    }
    // The panel's own order for the remainder — most recently put aside first, which is what
    // iterating `previews()` gives. Never `Place`'s `Ord`, which is creation order wearing a
    // different hat.
    for preview in panel.previews() {
        remember(preview.place(), &mut order);
    }

    let groups = order
        .into_iter()
        .filter_map(|place| {
            let previews: Vec<&Preview> = panel
                .previews()
                .iter()
                .filter(|preview| preview.place() == place)
                .collect();
            (!previews.is_empty()).then_some(AGroup { place, previews })
        })
        .collect();
    ThePanelInGroups { groups }
}
