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
pub static MORE_APPS: WhatTheDockHasOpen = WhatTheDockHasOpen::MoreApps { focused: None };

/// Which of the Dock's two surfaces is open.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum WhatTheDockHasOpen {
    /// Neither. The ordinary state, and the one a frame starts in.
    #[default]
    Nothing,
    /// The overview the *More apps* control opens.
    MoreApps {
        /// Which application the keyboard is on, counting through both groups.
        ///
        /// Carried here rather than beside this value on the frame, because a
        /// focus and an open surface are one fact: a field saying *the third
        /// item* next to a tag saying *nothing is open* is a state the draw
        /// would have to decide what to do about, and there is no right answer
        /// to decide.
        focused: Option<usize>,
    },
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
    TheWindowsOf {
        /// The rows.
        picker: alo_dock::picking::ThePicker,
        /// Which row the keyboard is on. See [`Self::MoreApps`].
        focused: Option<usize>,
    },
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
    /// The overview, and which of its applications has keyboard focus.
    MoreApps {
        /// Which item the keyboard is on, counting through both groups.
        ///
        /// [`None`] until a person presses a key. Somebody who opened this
        /// with a pointer has focused nothing, and drawing a focus ring on an
        /// arbitrary item would tell them the keyboard was somewhere it was
        /// not.
        focused: Option<usize>,
    },
    /// The windows of this application, and which row has keyboard focus.
    TheWindowsOf {
        /// Whose windows.
        app: alo_dock::AppId,
        /// Which row the keyboard is on. [`None`] until a key is pressed.
        focused: Option<usize>,
    },
}

/// Which way a key moves the keyboard's focus.
///
/// **Directions and not key codes.** Which evdev code means *left* is the
/// router's business; what *left* does to a grid is this type's, and a grid
/// that knew about key codes would be a grid that had to change when somebody
/// added a second way to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AKeyMoves {
    /// The previous item on this row.
    Left,
    /// The next item on this row.
    Right,
    /// The item above, a whole row back.
    Up,
    /// The item below, a whole row on.
    Down,
}

/// Where the keyboard's focus goes, given where it is and what was pressed.
///
/// `how_many` is how many items there are and `across` how many fit on a row —
/// one for a list, four for the overview's grid at its drawn width.
///
/// **The first press focuses the first item, whichever key it was.** A person
/// who opened the surface with a pointer and then reached for the keyboard is
/// asking to start somewhere, and the start is the beginning — not the item a
/// *Left* would have landed on from nowhere.
///
/// **It stops at the ends rather than wrapping.** A grid that wraps takes
/// somebody from the last item to the first on one key press, which is a jump
/// nobody asked for and is hard to undo when the list is sixty long.
///
/// [`None`] when there is nothing to focus.
#[must_use]
pub(crate) fn where_focus_goes(
    from: Option<usize>,
    pressed: AKeyMoves,
    how_many: usize,
    across: usize,
) -> Option<usize> {
    if how_many == 0 {
        return None;
    }
    let across = across.max(1);
    let Some(from) = from else {
        return Some(0);
    };
    let from = from.min(how_many - 1);
    let to = match pressed {
        AKeyMoves::Left => from.checked_sub(1),
        AKeyMoves::Right => Some(from + 1),
        AKeyMoves::Up => from.checked_sub(across),
        // **Down from the last row stays**, rather than jumping to the last
        // item: a half-full bottom row would otherwise move the focus sideways
        // as well as down, which is not what the key says.
        AKeyMoves::Down => Some(from + across),
    };
    Some(to.filter(|to| *to < how_many).unwrap_or(from))
}

impl WhatTheDockHasOpen {
    /// Whether the overview is the one open.
    #[must_use]
    pub const fn is_more_apps(&self) -> bool {
        matches!(self, Self::MoreApps { .. })
    }

    /// Which item the keyboard is on, whichever surface is open.
    ///
    /// The draw asks this for the ring and for which cell to scroll to, and it
    /// does not care which of the two surfaces answered — the same reason
    /// `WhatTheDockWasAskedFor::focused` is one accessor over two variants.
    #[must_use]
    pub const fn focused(&self) -> Option<usize> {
        match self {
            Self::Nothing => None,
            Self::MoreApps { focused } | Self::TheWindowsOf { focused, .. } => *focused,
        }
    }

    /// The window list that is open, if one is.
    #[must_use]
    pub const fn the_windows_of(&self) -> Option<&alo_dock::picking::ThePicker> {
        match self {
            Self::Nothing | Self::MoreApps { .. } => None,
            Self::TheWindowsOf { picker, .. } => Some(picker),
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

impl WhatTheDockWasAskedFor {
    /// Whether anything is open at all.
    ///
    /// On the choice rather than on the drawn state, because the lane asks it
    /// before a frame is drawn — it decides whether the Dock takes the keys,
    /// and the answer must not depend on anything a draw worked out.
    #[must_use]
    pub(crate) const fn is_anything(&self) -> bool {
        !matches!(self, Self::Nothing)
    }

    /// Whether the open surface is the overview, which is the one with a
    /// search field in it.
    ///
    /// **A window list has no field**, so a letter pressed over one is the
    /// application's and goes the road it always went. Asked on the choice
    /// rather than the drawn state, for [`Self::is_anything`]'s reason.
    #[must_use]
    pub(crate) const fn is_the_overview(&self) -> bool {
        matches!(self, Self::MoreApps { .. })
    }

    /// Which item the keyboard is on, if any, whichever surface is open.
    ///
    /// One accessor for both surfaces because every caller wants the same
    /// thing of it — the ring to draw, the row to activate — and does not care
    /// which of the two it came from.
    #[must_use]
    pub(crate) const fn focused(&self) -> Option<usize> {
        match self {
            Self::Nothing => None,
            Self::MoreApps { focused } | Self::TheWindowsOf { focused, .. } => *focused,
        }
    }

    /// What Escape leaves open, or [`None`] when Escape is not the Dock's.
    ///
    /// The owner's ruling of 2026-10-11: *Escape dismisses the current picker
    /// first, then the parent popover.* So a window list goes back to the
    /// overview it was opened from, and the overview closes.
    ///
    /// **[`None`] is the important answer.** With nothing open, Escape belongs
    /// to whatever application has the keyboard — and taking it would mean a
    /// person could not cancel anything in any program while alo OS was
    /// running. This is the one state in which the Dock must not act, and it is
    /// the common one.
    ///
    /// A `bool` beside a new state would let a caller take the key and change
    /// nothing, or change the state and forward the key as well. One value
    /// cannot say both.
    #[must_use]
    pub(crate) fn when_escape_is_pressed(&self) -> Option<Self> {
        match self {
            // Not the Dock's key.
            Self::Nothing => None,
            Self::MoreApps { .. } => Some(Self::Nothing),
            // **Back, not out.** A person who opened a window list from the
            // overview asked for two things; Escape undoes the second.
            //
            // The overview it returns to has nothing focused: the person is
            // arriving there afresh, and restoring a focus they last had
            // several presses ago would put the ring somewhere they are not
            // thinking about.
            Self::TheWindowsOf { .. } => Some(Self::MoreApps { focused: None }),
        }
    }

    /// Where an arrow key leaves the focus, or [`None`] when it is not the
    /// Dock's key.
    ///
    /// `how_many` is how many items the open surface holds and `across` how
    /// many of them fit on a row. The caller works both out from what is open
    /// *now* rather than from what was open when the surface was opened, for
    /// the reason `what_that_means_now` gives: an application can close while
    /// its list is up.
    ///
    /// **[`None`] with nothing open, and [`None`] with nothing to focus**, so
    /// the arrow keys of a person using an application are never swallowed —
    /// the same shape and the same reason as
    /// [`Self::when_escape_is_pressed`]. An empty surface returns the key too:
    /// there is nothing for it to do and an application behind may want it.
    #[must_use]
    pub(crate) fn when_an_arrow_is_pressed(
        &self,
        pressed: AKeyMoves,
        how_many: usize,
        across: usize,
    ) -> Option<Self> {
        let to = where_focus_goes(self.focused(), pressed, how_many, across)?;
        match self {
            Self::Nothing => None,
            Self::MoreApps { .. } => Some(Self::MoreApps { focused: Some(to) }),
            Self::TheWindowsOf { app, .. } => Some(Self::TheWindowsOf {
                app: app.clone(),
                focused: Some(to),
            }),
        }
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
        WhatTheDockWasAskedFor::MoreApps { focused } => {
            WhatTheDockHasOpen::MoreApps { focused: *focused }
        }
        WhatTheDockWasAskedFor::TheWindowsOf { app, focused } => {
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
                WhatTheDockHasOpen::TheWindowsOf {
                    picker,
                    focused: *focused,
                }
            } else {
                WhatTheDockHasOpen::Nothing
            }
        }
    }
}

#[cfg(test)]
#[path = "what_the_dock_has_open_tests.rs"]
mod tests;
