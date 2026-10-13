//! What one of the three acts asks of the windows there are — decided here,
//! carried out next door.
//!
//! **Split from [`crate::an_application_verb_is_carried_out`] because the two
//! halves are testable by different means.** Deciding *which window* from a
//! list is arithmetic over `alo_dock::Windows` and a unit test settles it.
//! Doing it needs a real compositor and a real client, which is the nested
//! fixture's road and `examples/a_real_application.rs`'s — `dock_windows_tests`
//! says so in as many words about the same boundary.
//!
//! Keeping them in one file would mean the only way to ask *does `close` ask
//! every window or one?* is to stand up a compositor, and that question has
//! nothing to do with a compositor.
//!
//! Nothing here touches a surface, a seat or the bus.

use alo_acting::Acting;
use alo_dock::{AppId, WindowId, Windows};

/// What the compositor is being asked to do about one application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WhatToDo {
    /// Ask the bus to start it. Only `open`, and only with no window open.
    StartIt,
    /// Reveal this window if it is put aside, then give it the keyboard.
    BringThisForward(WindowId),
    /// Ask each of these to close, oldest first.
    AskTheseToClose(Vec<WindowId>),
    /// There is no window of it here, and this act needs one.
    ThereIsNoWindow,
}

/// Decide, from the windows there are.
///
/// # `open` on something already running is *bring it up*
///
/// The same thing a press on its Dock icon does with one window open. An
/// `open` that started a second copy of a running application would be a verb
/// doing something nobody approved a sentence for.
///
/// # `focus` never starts anything
///
/// That is `open`'s. A `focus` that started an application would carry out a
/// different sentence from the one a person read.
///
/// # Several windows: the most recent, and never a list
///
/// A person pressing a Dock icon for an application with three windows is
/// offered a list, because they are standing there to choose from it. **An
/// agent is not standing there**, so `focus` and `open` bring forward the
/// window the person used most recently — the one they last looked at — which
/// is the only choice among the three that is about the person rather than
/// about the window list's order.
///
/// # `close` asks every window, oldest first
///
/// *close Geary* means Geary, not one of its windows, and a person who approved
/// *ask Geary to close* did not approve leaving two of its three windows open.
///
/// Oldest first — `each_as_opened`'s order — so that an application putting up
/// *save your changes?* puts it up for the window the person opened first, and
/// a second run of the same act asks the same windows in the same order. The
/// recency order would make the sequence depend on what the person clicked
/// while the act was in flight.
pub(crate) fn what_an_application_act_asks_for(
    windows: &Windows,
    app: &AppId,
    acting: &Acting,
) -> WhatToDo {
    match acting {
        Acting::Open(_) => match windows.most_recent_of(app) {
            Some(window) => WhatToDo::BringThisForward(window.id()),
            None => WhatToDo::StartIt,
        },
        Acting::Focus(_) => match windows.most_recent_of(app) {
            Some(window) => WhatToDo::BringThisForward(window.id()),
            None => WhatToDo::ThereIsNoWindow,
        },
        Acting::Close(_) => {
            let theirs: Vec<WindowId> = windows
                .each_as_opened()
                .filter(|window| window.app() == Some(app))
                .map(alo_dock::Window::id)
                .collect();
            if theirs.is_empty() {
                WhatToDo::ThereIsNoWindow
            } else {
                WhatToDo::AskTheseToClose(theirs)
            }
        }
    }
}
