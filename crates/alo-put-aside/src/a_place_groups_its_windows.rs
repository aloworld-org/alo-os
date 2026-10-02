//! The put-aside windows of one Place, grouped — and still one preview per window.
//!
//! Task 7 of `docs/autonomy/the-canvas-and-its-places.md`: *grouping by Place **without
//! hiding individual windows behind an application icon***. The owner's instruction is
//! the second half as much as the first, and the two pull against each other — grouping
//! is how a list becomes readable, and the usual way to group a list of windows is by the
//! application that owns them, which is exactly what this must not do.
//!
//! # Grouped by Place, never by application
//!
//! [`Preview`]'s own header already settles why: *a preview is one window, never one
//! application* — the Dock answers *which applications are open*, and three put-aside
//! Browser windows here are three previews with three titles. Grouping by application
//! would make this panel a second Dock with worse information.
//!
//! Grouping by **Place** does not have that problem, because a Place is *where the person
//! was working*, and a person who put four windows aside while writing and two while
//! reading has two groups that mean something to them. Nothing is hidden: each group
//! holds every window of that Place, individually.
//!
//! # Derived, never stored
//!
//! [`a_place_groups_its_windows`] reads the panel's one list and builds the answer. The
//! panel keeps no second collection, and that is the point rather than an economy.
//!
//! `Panel::put_aside` is `previews.insert(0, …)` and `bring_back` is
//! `previews.remove(at)` — so **every put-aside shifts every existing index and every
//! restore shifts every later one.** A stored grouping holding positions would be wrong
//! after each of them, would have to be repaired on both roads, and the day somebody adds
//! a third road it would be repaired on neither. The same reason
//! `crate::the_collapse_choice_per_place` holds only what a person chose.
//!
//! `bringing_a_window_back_leaves_the_grouping_correct` is what makes *derived* a measured
//! property rather than a sentence here.
//!
//! # Place order between groups, the panel's own order inside one
//!
//! Groups come out in `Place` order, from a `BTreeMap`, because two identical panels must
//! produce one layout — a draw that depended on which window was put aside first would
//! move a person's previews under their hand for no reason they could see.
//!
//! **Inside** a group the panel's own order is kept, untouched: *most recently put aside
//! first*, which `Panel` decides and documents in four places. This file does not re-sort
//! and does not have an opinion. Two reasons, and the second is the one that would be
//! tempting to override: the most recent window is the one most likely to be wanted back,
//! and the panel is a surface people point at — so any re-ordering here would move the
//! preview somebody was reaching for, and would also give the crate two answers to *what
//! order are the previews in*.

use std::collections::BTreeMap;

use alo_canvas::Place;

use crate::Preview;
use crate::panel::Panel;

/// The put-aside windows of each Place that has any, in Place order.
///
/// A Place with nothing put aside is **absent** rather than present and empty — the same
/// answer `crate::the_collapse_choice_per_place` gives, and for the same reason: a group
/// for a Place a person merely looked at is a row in a list that means nothing to them.
///
/// Every window the panel holds appears exactly once across the groups. That is the
/// clause the owner named, and `every_window_appears_once_and_nothing_is_merged` is what
/// holds it.
#[must_use]
pub fn a_place_groups_its_windows(panel: &Panel) -> BTreeMap<Place, Vec<&Preview>> {
    let mut grouped: BTreeMap<Place, Vec<&Preview>> = BTreeMap::new();
    for preview in panel.previews() {
        grouped.entry(preview.place()).or_default().push(preview);
    }
    grouped
}
