//! What an application's icon offers when asked, and what it never offers
//! without being asked.
//!
//! # On demand, never occupying space
//!
//! The owner's rule: these *appear on demand rather than occupying permanent
//! space in the Dock*. So there is no list of buttons here — there is a
//! function from *what is being asked about* to *what can be done to it*, and
//! nothing in this file can be drawn until somebody opens a menu.
//!
//! # The two that must never become buttons
//!
//! **Record this window** and **Give this window to alo** are offered on a
//! selected window and nowhere else. Neither becomes a permanent control, and
//! the Dock never becomes a dashboard for the agent — which is the owner's
//! sentence and is held by [`What::on_the_dock_permanently`] answering nothing
//! at all.
//!
//! That is a stronger guarantee than a comment: a later change that wanted a
//! recording button in the Dock would have to make that function return
//! something, and its test says it returns nothing.
//!
//! # Why an application's menu and a window's menu are different functions
//!
//! *Quit* applies to an application; *Record this window* applies to one window.
//! Asking one question and filtering afterwards would mean the answer for an
//! application briefly contained window actions — and *briefly contained* is how
//! something ends up drawn.

use crate::window::WindowId;

/// Something a person can ask for from an icon or a window preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum What {
    /// Open another window of this application.
    NewWindow,
    /// Show this application's windows — the same list [`crate::previews`]
    /// builds, reached from the menu rather than from a hover.
    ShowWindows,
    /// Keep this application on the Dock when it is closed.
    PinToDock,
    /// Stop keeping it there.
    UnpinFromDock,
    /// Put this window aside.
    Minimise,
    /// Fill the display with it.
    FullScreen,
    /// Close this application.
    Quit,
    /// Record what happens in this window.
    RecordThisWindow,
    /// Stop recording it.
    StopRecording,
    /// Let alo work in this window.
    GiveThisWindowToAlo,
    /// Stop alo working in it.
    StopAlo,
}

impl What {
    /// Whether this is drawn in the Dock whether or not anybody asked.
    ///
    /// **Nothing is.** The answer is a fixed empty list, and the test that says
    /// so is what stops *Record this window* quietly becoming a button — the
    /// owner's *neither feature adds a permanent Dock button*, and *the Dock
    /// should not become an agent dashboard*.
    #[must_use]
    pub const fn on_the_dock_permanently() -> &'static [Self] {
        &[]
    }

    /// Whether this action is about the agent.
    ///
    /// Named so the rule *the Dock never becomes a dashboard for the agent* can
    /// be asked rather than remembered.
    #[must_use]
    pub const fn is_about_the_agent(self) -> bool {
        matches!(self, Self::GiveThisWindowToAlo | Self::StopAlo)
    }
}

/// What is being asked about, which decides what can be done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AWindowsState {
    /// Whether it is being recorded.
    pub recording: bool,
    /// Whether alo is working in it.
    pub alo_is_working: bool,
    /// Whether it has been put aside already.
    pub put_aside: bool,
}

/// The menu for an application's icon.
///
/// **No window actions.** An icon is an application, and *Record this window*
/// has no window to mean.
#[must_use]
pub fn for_an_icon(is_pinned: bool, is_open: bool) -> Vec<What> {
    let mut offered = vec![What::NewWindow];
    if is_open {
        offered.push(What::ShowWindows);
    }
    offered.push(if is_pinned {
        What::UnpinFromDock
    } else {
        What::PinToDock
    });
    if is_open {
        offered.push(What::Quit);
    }
    offered
}

/// The menu for one window, chosen from a preview.
///
/// **This is where the two that must not become buttons live**, and each is
/// offered as its opposite once it is happening, so a person who started
/// something can always stop it from the same place they started it.
#[must_use]
pub fn for_a_window(_which: WindowId, state: AWindowsState) -> Vec<What> {
    let mut offered = Vec::new();
    if !state.put_aside {
        offered.push(What::Minimise);
        offered.push(What::FullScreen);
    }
    offered.push(if state.recording {
        What::StopRecording
    } else {
        What::RecordThisWindow
    });
    offered.push(if state.alo_is_working {
        What::StopAlo
    } else {
        What::GiveThisWindowToAlo
    });
    offered
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_window() -> WindowId {
        WindowId::numbered(1)
    }

    fn plain() -> AWindowsState {
        AWindowsState {
            recording: false,
            alo_is_working: false,
            put_aside: false,
        }
    }

    /// **Nothing is in the Dock permanently.** The owner's rule about recording
    /// and about alo, held as an empty list rather than as a comment: a change
    /// that wanted a button there would have to make this return something.
    #[test]
    fn nothing_is_drawn_in_the_dock_without_being_asked_for() {
        assert!(What::on_the_dock_permanently().is_empty());
    }

    /// **The Dock never becomes a dashboard for the agent.** Every action about
    /// alo is in a menu somebody opened, and none of them is permanent.
    #[test]
    fn every_action_about_the_agent_has_to_be_asked_for() {
        let about_alo: Vec<What> = for_a_window(a_window(), plain())
            .into_iter()
            .filter(|what| what.is_about_the_agent())
            .collect();
        assert_eq!(about_alo, [What::GiveThisWindowToAlo], "it is offered");

        for what in What::on_the_dock_permanently() {
            assert!(!what.is_about_the_agent(), "{what:?} is permanent");
        }
    }

    /// **An icon's menu holds no window actions**, because an icon is an
    /// application and *Record this window* has no window to mean.
    #[test]
    fn an_icons_menu_holds_nothing_about_a_particular_window() {
        for (pinned, open) in [(false, false), (false, true), (true, false), (true, true)] {
            let offered = for_an_icon(pinned, open);
            for what in [
                What::RecordThisWindow,
                What::StopRecording,
                What::GiveThisWindowToAlo,
                What::StopAlo,
                What::Minimise,
                What::FullScreen,
            ] {
                assert!(
                    !offered.contains(&what),
                    "{what:?} offered on an icon ({pinned}, {open})"
                );
            }
        }
    }

    /// Pinning and unpinning are one row that says what it would do, never both
    /// at once — a menu offering *Pin* and *Unpin* together is one a person
    /// cannot read.
    #[test]
    fn pinning_is_offered_as_whichever_it_would_do() {
        let unpinned = for_an_icon(false, true);
        assert!(unpinned.contains(&What::PinToDock));
        assert!(!unpinned.contains(&What::UnpinFromDock));

        let pinned = for_an_icon(true, true);
        assert!(pinned.contains(&What::UnpinFromDock));
        assert!(!pinned.contains(&What::PinToDock));
    }

    /// **A closed application cannot be quit or asked to show its windows**, and
    /// can still be given a new window and pinned — which is what a pinned,
    /// closed icon is for.
    #[test]
    fn a_closed_application_offers_only_what_makes_sense() {
        let closed = for_an_icon(true, false);
        assert!(closed.contains(&What::NewWindow));
        assert!(closed.contains(&What::UnpinFromDock));
        assert!(!closed.contains(&What::Quit));
        assert!(!closed.contains(&What::ShowWindows));
    }

    /// **Whatever was started can be stopped from where it was started.** Each
    /// of the two appears as its opposite once it is happening, so there is no
    /// hunting for a way to stop.
    #[test]
    fn what_is_happening_is_offered_as_the_way_to_stop_it() {
        let recording = AWindowsState {
            recording: true,
            ..plain()
        };
        let offered = for_a_window(a_window(), recording);
        assert!(offered.contains(&What::StopRecording));
        assert!(!offered.contains(&What::RecordThisWindow));

        let with_alo = AWindowsState {
            alo_is_working: true,
            ..plain()
        };
        let offered = for_a_window(a_window(), with_alo);
        assert!(offered.contains(&What::StopAlo));
        assert!(!offered.contains(&What::GiveThisWindowToAlo));
    }

    /// A window already put aside is not offered *Minimise*, and can still be
    /// recorded or given to alo — it is put away, not gone.
    #[test]
    fn a_window_put_aside_is_not_offered_to_be_put_aside_again() {
        let aside = AWindowsState {
            put_aside: true,
            ..plain()
        };
        let offered = for_a_window(a_window(), aside);
        assert!(!offered.contains(&What::Minimise));
        assert!(!offered.contains(&What::FullScreen));
        assert!(offered.contains(&What::RecordThisWindow));
        assert!(offered.contains(&What::GiveThisWindowToAlo));
    }

    /// Both roads a person might take are offered on an ordinary window, and
    /// neither is offered twice.
    #[test]
    fn an_ordinary_window_offers_each_thing_once() {
        let offered = for_a_window(a_window(), plain());
        for what in [
            What::Minimise,
            What::FullScreen,
            What::RecordThisWindow,
            What::GiveThisWindowToAlo,
        ] {
            assert_eq!(
                offered.iter().filter(|o| **o == what).count(),
                1,
                "{what:?}"
            );
        }
    }
}
