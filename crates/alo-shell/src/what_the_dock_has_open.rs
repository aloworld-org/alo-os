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
    /// Carries **which** application, because a picker that did not know would
    /// be a list of nothing — and because the windows are re-read at every
    /// frame from the application rather than remembered, so a window that
    /// closes while the list is open leaves the list rather than leaving a row
    /// that names nothing.
    TheWindowsOf(alo_dock::AppId),
}

impl WhatTheDockHasOpen {
    /// Whether the overview is the one open.
    #[must_use]
    pub const fn is_more_apps(&self) -> bool {
        matches!(self, Self::MoreApps)
    }

    /// Which application's windows are open, if any are.
    #[must_use]
    pub const fn the_windows_of(&self) -> Option<&alo_dock::AppId> {
        match self {
            Self::Nothing | Self::MoreApps => None,
            Self::TheWindowsOf(app) => Some(app),
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

#[cfg(test)]
#[path = "what_the_dock_has_open_tests.rs"]
mod tests;
