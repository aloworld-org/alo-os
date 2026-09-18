//! One desktop: what it is called, which windows are on it, and how it is
//! divided.
//!
//! A desktop is identified by a [`DesktopId`] this crate makes, not by its
//! position: a person who reorders their desktops has the same desktops in a
//! different order, and a shortcut or a window that named *the third one* would
//! follow the reordering instead of following the desktop. Position is a
//! [`crate::Position`], read off the order and never stored here.
//!
//! **Each desktop keeps its own division.** A pair of windows split into halves
//! on desktop 1 stays split there while desktop 2 holds a single window filling
//! the screen; switching desktop does not re-lay-out anything. The division
//! itself — the tree of shares, what happens when a boundary moves, what a
//! closed window's share becomes — is `alo-dividing`'s and is not re-decided
//! here. What this crate adds is that there is one per desktop rather than one
//! per display.
//!
//! **What a desktop does not hold is a window's title or a document's name.**
//! It holds compositor window ids, exactly as a division does, for the reason
//! task 2 of this plan states: what is remembered about an arrangement is
//! applications and shares, never what was open in them.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};

use alo_dividing::{Area, Division, WindowId};
use alo_strings::{Filling, Said, Strings};

use crate::naming::Name;
use crate::position::Position;
use crate::words;

/// Where the next desktop's identity comes from.
///
/// One counter for the process, so two displays' desktops never share an id and
/// a desktop moved between them would keep the one it had.
static NEXT_DESKTOP: AtomicU64 = AtomicU64::new(1);

/// Which desktop.
///
/// Opaque: this crate compares two of them and nothing reads the number. It is
/// not a position, and not a number a person is ever shown — what a person reads
/// is [`Desktop::shown`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DesktopId(u64);

/// One of a person's desktops.
///
/// Deliberately not `Clone`: a desktop is one place, and a copy of one would be
/// two desktops claiming the same windows. It holds a `Division`, which is not
/// `Clone` for the same kind of reason.
#[derive(Debug, PartialEq, Eq)]
pub struct Desktop {
    /// Which desktop this is.
    id: DesktopId,
    /// What the person called it, when they have called it anything.
    name: Option<Name>,
    /// How this desktop's windows divide the display.
    division: Division,
    /// The windows on this desktop, and not the ones on every desktop.
    windows: BTreeSet<WindowId>,
}

impl Desktop {
    /// A new, empty desktop of a display.
    pub(crate) fn of(display: Area) -> Self {
        Self {
            id: DesktopId(NEXT_DESKTOP.fetch_add(1, Ordering::Relaxed)),
            name: None,
            division: Division::of(display),
            windows: BTreeSet::new(),
        }
    }

    /// Which desktop this is.
    #[must_use]
    pub const fn id(&self) -> DesktopId {
        self.id
    }

    /// What the person called it, or `None` when they have not called it
    /// anything.
    #[must_use]
    pub const fn name(&self) -> Option<&Name> {
        self.name.as_ref()
    }

    /// What this desktop is read as, in the language the person reads: their own
    /// name for it, or *Desktop 3* from `at`.
    ///
    /// A `String` rather than a `Said` for the reason `alo_shortcuts::Chord`
    /// gives for the same shape: half the answers are not strings anybody
    /// translates. A person's own name is their words, in whatever language
    /// they typed it, and a provenance on it would be a claim about a
    /// translation that nobody was ever going to make. Whether the *fallback*
    /// was translated is [`Desktop::numbered`]'s to answer.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    #[must_use]
    pub fn shown(&self, at: Position, strings: &Strings) -> String {
        match &self.name {
            Some(name) => name.as_str().to_owned(),
            None => Self::numbered(at, strings).into_text(),
        }
    }

    /// What a desktop nobody has named is called at this position, and whether
    /// anybody translated it.
    ///
    /// Public because the fallback is the case a translator's work shows up in,
    /// and a development build marking untranslated strings needs the `Said`
    /// that [`Desktop::shown`] flattens.
    #[must_use]
    pub fn numbered(at: Position, strings: &Strings) -> Said {
        strings.say(
            &words::DESKTOP_NUMBERED.key(),
            &Filling::of("number", at.number().to_string()),
        )
    }

    /// How this desktop's windows divide the display.
    #[must_use]
    pub const fn division(&self) -> &Division {
        &self.division
    }

    /// How this desktop's windows divide the display, to change.
    ///
    /// The division is `alo-dividing`'s and every rule about it is that crate's:
    /// a share is dropped, split, resized and closed there. What this crate
    /// holds is that this one belongs to this desktop.
    pub const fn division_mut(&mut self) -> &mut Division {
        &mut self.division
    }

    /// The windows on this desktop alone, in the order the compositor numbered
    /// them.
    ///
    /// **Not the whole of what a person sees there.** The windows on every
    /// desktop and the three promises are also on it —
    /// [`crate::OnADisplay::windows_on`] is that answer, because it is the
    /// display that knows them.
    pub fn windows(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.windows.iter().copied()
    }

    /// Whether this window is on this desktop alone.
    #[must_use]
    pub fn holds(&self, window: WindowId) -> bool {
        self.windows.contains(&window)
    }

    /// Put a window on this desktop.
    pub(crate) fn take(&mut self, window: WindowId) {
        self.windows.insert(window);
    }

    /// Take a window off this desktop, and out of its division.
    ///
    /// Whether it had a share is not a question worth asking: a window in no
    /// division is refused by `Division::close`, and that refusal is the
    /// ordinary case of a window that was never divided — most windows on a
    /// desktop are floating over it. So the answer is ignored deliberately,
    /// here and in [`Desktop::emptied`], and what is returned is whether the
    /// desktop held the window at all.
    pub(crate) fn let_go(&mut self, window: WindowId) -> bool {
        let _ = self.division.close(window);
        self.windows.remove(&window)
    }

    /// Everything on this desktop, and the empty division to go with it.
    pub(crate) fn emptied(&mut self) -> Vec<WindowId> {
        let windows: Vec<WindowId> = self.windows.iter().copied().collect();
        for window in &windows {
            let _ = self.division.close(*window);
        }
        self.windows.clear();
        windows
    }

    /// Call it this, or nothing.
    pub(crate) fn call_it(&mut self, name: Option<Name>) {
        self.name = name;
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{display, in_english, translated, window};

    /// Two desktops are never the same desktop, whatever they are called.
    #[test]
    fn two_desktops_are_never_the_same_desktop() {
        let one = Desktop::of(display());
        let two = Desktop::of(display());
        assert_ne!(one.id(), two.id());
        assert!(one.division().is_empty());
        assert_eq!(one.name(), None);
    }

    /// An unnamed desktop is read from its position, translated; a named one is
    /// read as the person's own words.
    #[test]
    fn an_unnamed_desktop_is_read_from_its_position() {
        let mut desktop = Desktop::of(display());
        let strings = in_english();
        let third = Position::numbered(3).unwrap();
        let said = Desktop::numbered(third, &strings);
        assert_eq!(said.text(), "Desktop 3");
        assert!(said.unfilled().is_empty());
        assert!(!said.is_a_bug());
        assert_eq!(desktop.shown(third, &strings), "Desktop 3");

        let german = translated(&[(words::DESKTOP_NUMBERED, "Arbeitsfläche {number}")]);
        assert!(Desktop::numbered(third, &german).is_translated());
        assert_eq!(desktop.shown(third, &german), "Arbeitsfläche 3");

        desktop.call_it(Some(Name::given("Post").unwrap()));
        assert_eq!(desktop.shown(third, &german), "Post");
        assert_eq!(desktop.name().map(Name::as_str), Some("Post"));
        desktop.call_it(None);
        assert_eq!(desktop.shown(third, &strings), "Desktop 3");
    }

    /// A window put on a desktop is held by it, let go of once, and emptying
    /// hands every window back.
    #[test]
    fn a_desktop_holds_the_windows_put_on_it() {
        let mut desktop = Desktop::of(display());
        desktop.take(window(1));
        desktop.take(window(2));
        assert!(desktop.holds(window(1)));
        assert_eq!(desktop.windows().count(), 2);
        assert!(desktop.let_go(window(1)));
        assert!(!desktop.let_go(window(1)));
        assert!(!desktop.holds(window(1)));
        assert_eq!(desktop.emptied(), vec![window(2)]);
        assert_eq!(desktop.windows().count(), 0);
        assert!(desktop.division().is_empty());
    }
}
