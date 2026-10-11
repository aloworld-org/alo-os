//! Which surface the Dock has opened over the canvas, of the two it has.
//!
//! # Why one field rather than two
//!
//! The Dock opens *More apps* from its own control and a **window picker** from
//! an application's icon. Both hang over the canvas, both are dismissed by a
//! press outside, and **at most one of them is open**: opening either closes
//! the other, because a person who asked for the second did not ask to keep the
//! first.
//!
//! A `bool` beside an `Option<AppId>` can say otherwise. It can say both are
//! open, which draws two lists over each other; and it can say the picker is
//! open for no application, which draws a list of nothing. Neither is a state
//! anybody would choose, and both are states that compile.
//!
//! So the two are one value and the impossible pairs cannot be written down.
//! This replaced `Server::the_overflow_is_open`, which was a `bool`, on
//! 2026-10-11 — and it is a replacement rather than an addition, so no frame
//! gained a field.
//!
//! # It does not say where
//!
//! Only *what*. Where either list is drawn is the draw's, and a press is
//! classified against what the draw recorded — `crate::a_press_on_the_dock`
//! holds both halves. A type here that carried a rectangle would be a second
//! place deciding where a surface is.

/// Neither surface open, as something a frame can borrow.
///
/// `DesktopFrame` and `Shown` are `Copy`, and this type is not — it holds an
/// `AppId`. So a frame carries `&WhatTheDockHasOpen`, and the two states that
/// have nothing in them are statics rather than temporaries a literal would
/// have to keep alive.
pub static NOTHING: WhatTheDockHasOpen = WhatTheDockHasOpen::Nothing;

/// The overview open, as something a frame can borrow. See [`NOTHING`].
pub static MORE_APPS: WhatTheDockHasOpen = WhatTheDockHasOpen::MoreApps;

/// Which of the Dock's two surfaces is open.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum WhatTheDockHasOpen {
    /// Neither. The ordinary state, and the one a frame starts in.
    #[default]
    Nothing,
    /// The overview the *More apps* control opens.
    MoreApps,
    /// The windows of one application, opened from its icon.
    ///
    /// Carries the **rows**, not the application. A drawing needs each window's
    /// name and whether it was put aside, and neither is on the Dock's own
    /// list — `OnTheDock` is one entry per application and counts its windows
    /// rather than naming them.
    ///
    /// **Built fresh every frame**, by `Server::what_the_dock_has_open`, from
    /// the windows the Dock can see at that moment. So a window that closes
    /// while the list is open leaves the list, rather than leaving a row that
    /// names nothing — and there is no cache here that somebody has to remember
    /// to refresh.
    TheWindowsOf(alo_dock::picking::ThePicker),
}

/// Which surface the person asked for, which is all the `Server` keeps.
///
/// **Separate from [`WhatTheDockHasOpen`] because they are different facts.**
/// This is a choice and it lasts until the person makes another; that is a
/// frame's worth of rows and is worked out again every time anything is drawn.
/// Keeping the rows here instead would be a cache of what the windows were when
/// the list was opened, and the fault that follows is a row naming a window
/// that has closed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum WhatTheDockWasAskedFor {
    /// Neither.
    #[default]
    Nothing,
    /// The overview.
    MoreApps,
    /// The windows of this application.
    TheWindowsOf(alo_dock::AppId),
}

impl WhatTheDockHasOpen {
    /// Whether the overview is the one open.
    #[must_use]
    pub const fn is_more_apps(&self) -> bool {
        matches!(self, Self::MoreApps)
    }

    /// The window list that is open, if one is.
    #[must_use]
    pub const fn the_windows_of(&self) -> Option<&alo_dock::picking::ThePicker> {
        match self {
            Self::Nothing | Self::MoreApps => None,
            Self::TheWindowsOf(picker) => Some(picker),
        }
    }

    /// Whether anything at all is open.
    ///
    /// Named rather than left to `!matches!(.., Nothing)` at each site, because
    /// *is anything open* is the question a press outside asks and getting it
    /// from the wrong variants is how a surface comes to be undismissable.
    #[must_use]
    pub const fn is_anything(&self) -> bool {
        !matches!(self, Self::Nothing)
    }
}

/// Work out what to draw from what the person asked for and what is open now.
///
/// **Separate from `Server::what_the_dock_has_open` so it can be tested.** The
/// server's version needs a compositor to get its windows from; this needs a
/// list, and the thing worth checking is the conversion rather than where the
/// windows came from. The same split `crate::which_card_the_seat_holds` uses,
/// and for the same reason.
///
/// `room` is how far the list may grow before it scrolls. The caller that knows
/// is the draw, which knows where the bar is; this is handed whatever it is
/// given and does not decide.
pub(crate) fn what_that_means_now(
    asked: &WhatTheDockWasAskedFor,
    windows: &alo_dock::Windows,
    text: alo_appearance::TextScale,
    room: alo_dock::Room,
) -> WhatTheDockHasOpen {
    match asked {
        WhatTheDockWasAskedFor::Nothing => WhatTheDockHasOpen::Nothing,
        WhatTheDockWasAskedFor::MoreApps => WhatTheDockHasOpen::MoreApps,
        WhatTheDockWasAskedFor::TheWindowsOf(app) => {
            let picker = alo_dock::picking::ThePicker::of(
                alo_dock::previews::Previews::of(
                    windows,
                    app,
                    alo_dock::previews::Opened::ByPressingTheIcon,
                ),
                text,
                room,
            );
            // **A list that is not worth showing is not open.** The windows of
            // the application a person chose can close while its list is up,
            // and one window left is a list with nothing to choose between —
            // `alo_dock::clicking` would not have opened it for one. Closing is
            // the honest answer and it costs the person nothing: the window
            // they are left with is the one a plain press would now go to.
            if picker.is_worth_showing() {
                WhatTheDockHasOpen::TheWindowsOf(picker)
            } else {
                WhatTheDockHasOpen::Nothing
            }
        }
    }
}

#[cfg(test)]
#[path = "what_the_dock_has_open_tests.rs"]
mod tests;
