//! **Everything this machine does, reached from a keyboard alone.**
//!
//! Task 3 of `docs/autonomy/v0-5-access-and-language-plan.md`, and clause 11.2's
//! of EN 301 549: everything operable from a keyboard, the focus never trapped,
//! the focus order the order a reader reads in.
//!
//! # A chord is a keyboard road, and it is not enough
//!
//! Every one of `alo-shortcuts`' actions already answers to a chord, so *is
//! there a keyboard road* is answered *yes* before this file exists. The
//! question worth asking is the next one: **can somebody who has not been told
//! the chord find it?** A person who cannot use a pointer and does not know that
//! the agent is `Super+A` has no road at all, and a list of chords in a manual
//! is not a road either — it is homework.
//!
//! So this holds a stronger thing than the clause asks: **every action has a
//! chord *and* a place the keyboard arrives at by pressing Tab**, and
//! [`the_tab_stop_for`] says where. Two of those stops did not exist when this
//! was written — the agent and the launcher — and `crate::tree` gained them in
//! the same change.
//!
//! # The focus order is the reading order
//!
//! Not a second list. [`focus_order`] is `crate::tree`'s reading order with
//! everything that cannot be used taken out, so a surface where the two disagree
//! is a surface that cannot be built: there is nowhere for them to disagree.
//! That is the whole of clause 11.2.4.3, and it is why the reading order was
//! worth getting right first.
//!
//! # Escape leaves, and never answers
//!
//! [`leaving`] says what Escape does on each of the nine surfaces, and the one
//! that matters is the approval: **Escape declines.** It must never approve —
//! ADR 0001 says what a person approves is the sentence, and a key pressed to
//! get out of the way is not somebody reading a sentence. It must not leave the
//! proposal unanswered either: a person who pressed Escape believes they have
//! dealt with it, and a change still waiting behind that belief is worse than
//! either answer.

use alo_shortcuts::Action;

use crate::tree::{Control, Surface};

/// **The keys keyboard-only operation is made of.**
///
/// Four, and no more: this is not a list of what a keyboard has, it is the list
/// of what a person must be able to do everything with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// To the next place the keyboard can be.
    Tab,
    /// To the one before it.
    TabBackwards,
    /// Use what the keyboard is on.
    Enter,
    /// Leave.
    Escape,
}

impl Key {
    /// All four, in the order this file argues about them.
    pub const ALL: [Self; 4] = [Self::Tab, Self::TabBackwards, Self::Enter, Self::Escape];
}

/// **Where the keyboard stops, on one surface, in order.**
///
/// The reading order with everything that cannot be used taken out. A control a
/// reader is told about but nobody can act on — a label, a heading, a line of
/// the record — is read and passed over, which is what a sighted person's eye
/// does with it.
#[must_use]
pub fn focus_order(surface: Surface) -> Vec<Control> {
    surface
        .read_aloud()
        .into_iter()
        .filter(|control| control.can_be_used())
        .collect()
}

/// **Where the keyboard goes after the last stop**, which is the first.
///
/// The focus is never trapped: Tab from the end of a surface arrives at its
/// beginning, and [`Key::Escape`] leaves the surface entirely. A surface with
/// nothing to use has nowhere for the keyboard to be trapped and answers
/// [`None`].
#[must_use]
pub fn after_the_last(surface: Surface) -> Option<Control> {
    focus_order(surface).into_iter().next()
}

/// **The place a keyboard arrives at, by pressing Tab, on the road to this
/// action.**
///
/// A stop may lead to more than one action — arranging a window is one stop and
/// several arrangements — and that is a road rather than a shortcoming: what a
/// person must not have to know is a chord, not a second keypress.
#[must_use]
pub const fn the_tab_stop_for(action: Action) -> Surface {
    match action {
        // The agent, and the windows a person moves between, are the desktop's.
        Action::TheAgent | Action::NextWindow | Action::PreviousWindow => Surface::Desktop,
        // Opening something, and moving between what is open, are the dock's.
        Action::Launcher | Action::NextApplication | Action::PreviousApplication => Surface::Dock,
        // Everything done to the window in front is where that window's own
        // controls are.
        Action::CloseWindow
        | Action::MinimiseWindow
        | Action::MaximiseWindow
        | Action::SnapLeft
        | Action::SnapRight => Surface::WindowControls,
    }
}

/// **What Escape does on a surface.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leaving {
    /// Back to the desktop, which is where a person came from.
    ReturnsToTheDesktop,
    /// Clears what was typed, because there is nowhere further back to go.
    ClearsWhatWasTyped,
    /// Already at the bottom: the desktop is what is left when everything is
    /// closed.
    AlreadyTheFloor,
    /// **Answers no.** Never yes, and never nothing.
    Declines,
}

/// **What Escape does on each surface**, and it does something on every one.
///
/// There is no variant for *nothing happens*: a surface a person cannot leave
/// with one key is a trap, which is the clause this file is about.
#[must_use]
pub const fn leaving(surface: Surface) -> Leaving {
    match surface {
        // There is nowhere further back than the screen a person reaches when
        // the desktop will not start: it is what is left when the workspace is
        // not there, so Escape leaves it where the desktop's Escape leaves the
        // desktop. Leaving it for real is restarting the machine, which is a
        // thing a person does to the machine and not a key
        // (`alo_shell::RecoveryKey` takes no Escape at all).
        Surface::Recovery => Leaving::AlreadyTheFloor,
        Surface::SignIn => Leaving::ClearsWhatWasTyped,
        Surface::Desktop => Leaving::AlreadyTheFloor,
        Surface::Dock
        | Surface::StatusArea
        | Surface::Record
        | Surface::Settings
        | Surface::WindowControls => Leaving::ReturnsToTheDesktop,
        Surface::Approval => Leaving::Declines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The focus order is the reading order with the labels taken out**, on
    /// every surface — held as the two being one list rather than two.
    #[test]
    fn the_keyboard_stops_where_a_reader_reads_and_in_that_order() {
        for surface in Surface::ALL {
            let read: Vec<Control> = surface.read_aloud();
            let stops = focus_order(surface);
            let mut expected = read.iter().filter(|one| one.can_be_used());
            for stop in &stops {
                assert_eq!(
                    Some(stop),
                    expected.next(),
                    "{surface:?} stops somewhere it is not read, or in another order"
                );
            }
            assert!(
                expected.next().is_none(),
                "{surface:?} reads a control the keyboard skips"
            );
        }
    }

    /// **Every surface can be left with one key**, and the approval's key
    /// answers no.
    #[test]
    fn every_surface_is_left_with_one_key_and_the_approval_is_never_approved_by_it() {
        for surface in Surface::ALL {
            let _: Leaving = leaving(surface);
        }
        assert_eq!(leaving(Surface::Approval), Leaving::Declines);
        assert_eq!(leaving(Surface::Settings), Leaving::ReturnsToTheDesktop);
        assert_eq!(leaving(Surface::SignIn), Leaving::ClearsWhatWasTyped);
    }

    /// **The keyboard is never trapped**: every surface with somewhere to be
    /// has somewhere to go after its last stop.
    #[test]
    fn the_keyboard_comes_back_round_rather_than_stopping() {
        for surface in Surface::ALL {
            let stops = focus_order(surface);
            assert_eq!(
                after_the_last(surface).is_some(),
                !stops.is_empty(),
                "{surface:?}"
            );
        }
    }
}
