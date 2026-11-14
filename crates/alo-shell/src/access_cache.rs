//! **The whole tree in one answer**: `org.a11y.atspi.Cache` at
//! `/org/a11y/atspi/cache`, as at-spi2-core 2.60's `xml/Cache.xml` describes it.
//!
//! libatspi — which Orca reads every application through — asks each
//! application it meets for `GetItems` there, once, and keeps the answer: one
//! entry per thing in the tree, with its name, role, states and where it hangs.
//! It then follows changes from the `AddAccessible` and `RemoveAccessible`
//! signals rather than asking again. **Until 2026-11-13 nothing answered at
//! that path**, and Orca on a booted machine logged *Error in GetItems … Unknown
//! object '/org/a11y/atspi/cache'* and read the tree one question at a time.
//!
//! # The shape, exactly
//!
//! `GetItems` answers `a((so)(so)(so)iiassusau)`: the thing, its application,
//! its parent, its index in that parent, how many children it has, the
//! interfaces it answers, its name, its role, its description and its states.
//! **libatspi does not check this signature before it reads** (`handle_get_items`
//! walks the reply field by field), so a field out of place is not refused — it
//! is misread. Each entry is built from the same [`ReadAloud`] that answers the
//! one-at-a-time questions, so the two cannot say different things.
//!
//! # Keeping a reader's copy true
//!
//! A reader holding the cache does not ask again, so when the tree is replaced
//! ([`crate::ReadAloudBus::now_showing`]) it has to be **told**: every entry
//! that is new or different is sent as `AddAccessible`, which libatspi applies
//! over what it held at that path, and every path that is gone is sent as
//! `RemoveAccessible`. [`what_changed`] decides which; the bus sends them.

use zbus::zvariant::OwnedObjectPath;

use crate::access_answers::{ReadAloud, Reference};

/// Where the cache is served, as at-spi2 fixes it.
pub(crate) const CACHE: &str = "/org/a11y/atspi/cache";

/// The interface the cache answers, and its two signals are sent on.
pub(crate) const CACHE_INTERFACE: &str = "org.a11y.atspi.Cache";

/// One entry, `((so)(so)(so)iiassusau)`: the thing, its application, its
/// parent, its index in that parent, its child count, its interfaces, its
/// name, its role, its description and its states.
pub(crate) type Entry = (
    Reference,
    Reference,
    Reference,
    i32,
    i32,
    Vec<String>,
    String,
    u32,
    String,
    Vec<u32>,
);

/// The entry one node is cached as.
pub(crate) fn entry_of(answers: &ReadAloud) -> Entry {
    (
        answers.itself.clone(),
        answers.application.clone(),
        answers.parent.clone(),
        answers.index_in_parent,
        answers.how_many_inside(),
        answers.interfaces(),
        answers.name.clone(),
        answers.role,
        answers.description.clone(),
        answers.states.clone(),
    )
}

/// What a reader holding `before` has to be told to hold `after`.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Changed {
    /// Entries that are new, or say something different at a path a reader
    /// already holds — each sent as `AddAccessible`, in tree order.
    pub(crate) added: Vec<Entry>,
    /// Things that are no longer in the tree — each sent as
    /// `RemoveAccessible`.
    pub(crate) removed: Vec<Reference>,
}

/// Compare two trees entry by entry, by the path each entry is at.
pub(crate) fn what_changed(before: &[Entry], after: &[Entry]) -> Changed {
    let path = |entry: &Entry| -> OwnedObjectPath { entry.0.1.clone() };
    let removed = before
        .iter()
        .filter(|old| !after.iter().any(|new| path(new) == path(old)))
        .map(|old| old.0.clone())
        .collect();
    let added = after
        .iter()
        .filter(|new| !before.iter().any(|old| old == *new))
        .cloned()
        .collect();
    Changed { added, removed }
}

/// The cache, served at [`CACHE`].
pub(crate) struct TheCache {
    /// Every entry, the shell itself first.
    pub(crate) entries: Vec<Entry>,
}

#[zbus::interface(name = "org.a11y.atspi.Cache")]
impl TheCache {
    /// Every thing in the tree, at once: `a((so)(so)(so)iiassusau)`.
    fn get_items(&self) -> Vec<Entry> {
        self.entries.clone()
    }

    /// Which version of the interface this is. `Cache.xml` names the property
    /// in lower case, unlike every other at-spi2 property.
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        1
    }
}

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    reason = "in a test, a panic on an index out of range is the failure being reported"
)]
#[path = "access_cache_tests.rs"]
mod tests;
