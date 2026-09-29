//! What one click on an application's icon does.
//!
//! The owner's rule, which everything here is an implementation of:
//!
//! > **Click an app to return to where you last used it. Choose a preview when
//! > you want a different window.**
//!
//! This file is the first sentence. The second is [`crate::Windows::of_the_app`],
//! which is the list a hover or a keyboard focus opens.
//!
//! # One behaviour, not a table of four
//!
//! The obvious design gives each case its own answer — an open window is
//! focused, a distant one is travelled to, one put aside is restored — and that
//! is three things to learn and three places to be surprised. **They are one
//! behaviour:** go to the window you used last, bring it back if it was put
//! away, and move the view if you could not already see it. The cases below are
//! that sentence meeting the facts, not four rules.
//!
//! # The two things a click never does
//!
//! **It never moves a window.** Travel is a change to the view and to what is in
//! front; where a window sits on the plane is the person's and is not touched by
//! looking at it.
//!
//! **It never opens a window by surprise.** An application with a window gets
//! that window. A new one is asked for — from the icon's menu, or by dragging
//! the icon onto the canvas, which shows where it would go before it goes there.
//!
//! # Clicking twice
//!
//! The second click answers [`WhatAClickDoes::BringForward`] for the window the
//! first one went to, which is to say it does nothing a person would notice. It
//! does not cycle to the next window, does not open another, and is not quietly
//! repurposed as *back*: going back is its own control, and a control that is
//! sometimes something else is one nobody can rely on.

use crate::on_the_canvas::TheView;
use crate::window::{AppId, HowItSits, WindowId};
use crate::windows::Windows;

/// What a click on an application's icon does.
///
/// Every answer names the window it is about, except the one where there is no
/// window yet — so a caller cannot act on a decision without knowing what it
/// acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatAClickDoes {
    /// Nothing of this application is open: open a window near what the person
    /// is looking at, and focus it.
    ///
    /// **Near the view, not at a remembered place.** There is nowhere it was
    /// last used, so the only honest answer is *where you are*: a window that
    /// opened out of sight would be a click that appeared to do nothing.
    OpenAWindowNearTheView,
    /// It is open, it is wholly in view: raise it. The canvas does not move.
    BringForward(WindowId),
    /// It is open and not wholly in view: raise it, and travel to it.
    TravelTo(WindowId),
    /// It was put aside, and its place is wholly in view: bring it back, and
    /// the canvas does not move.
    ///
    /// This is the case that only exists because both rules are taken
    /// seriously — *restore it, then travel to it* and *if it is already
    /// visible the canvas does not pan*. Travelling zero distance is still
    /// motion on a screen, and motion that communicates nothing is what people
    /// ask to be able to turn off.
    BringItBackWhereItIs(WindowId),
    /// It was put aside somewhere else: bring it back where it was, and travel.
    BringItBackAndTravelTo(WindowId),
}

impl WhatAClickDoes {
    /// Which window this is about, where there is one.
    #[must_use]
    pub const fn window(self) -> Option<WindowId> {
        match self {
            Self::OpenAWindowNearTheView => None,
            Self::BringForward(id)
            | Self::TravelTo(id)
            | Self::BringItBackWhereItIs(id)
            | Self::BringItBackAndTravelTo(id) => Some(id),
        }
    }

    /// Whether the canvas moves.
    ///
    /// Named rather than left to a caller matching four variants, because
    /// *does the view move* is the question a reduced-motion setting turns on
    /// and getting it from the wrong variants is how that setting comes to be
    /// half-kept.
    #[must_use]
    pub const fn travels(self) -> bool {
        matches!(self, Self::TravelTo(_) | Self::BringItBackAndTravelTo(_))
    }

    /// Whether a window that was put aside comes back.
    #[must_use]
    pub const fn brings_one_back(self) -> bool {
        matches!(
            self,
            Self::BringItBackWhereItIs(_) | Self::BringItBackAndTravelTo(_)
        )
    }
}

/// What a click on this application's icon does, given what is open and what the
/// person is looking at.
///
/// Answers rather than refuses: every state of the world has one of these as its
/// answer, and *this application has nothing open* is an answer rather than an
/// error.
#[must_use]
pub fn what_a_click_does(windows: &Windows, app: &AppId, view: TheView) -> WhatAClickDoes {
    let Some(window) = windows.most_recent_of(app) else {
        return WhatAClickDoes::OpenAWindowNearTheView;
    };
    let id = window.id();

    // A window filling a display is not being shown by the canvas, so there is
    // nothing to travel across: the person is taken to it as it is.
    if matches!(window.sits(), HowItSits::FillingTheScreen) {
        return WhatAClickDoes::BringForward(id);
    }

    let in_view = view.already_shows(window.at());
    match (window.sits().was_put_aside(), in_view) {
        (true, true) => WhatAClickDoes::BringItBackWhereItIs(id),
        (true, false) => WhatAClickDoes::BringItBackAndTravelTo(id),
        (false, true) => WhatAClickDoes::BringForward(id),
        (false, false) => WhatAClickDoes::TravelTo(id),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::on_the_canvas::{Patch, Spot};
    use crate::window::Window;

    fn app(name: &str) -> AppId {
        AppId::named(name).unwrap()
    }

    fn patch(x: i64, y: i64) -> Patch {
        Patch::of(Spot::at(x, y), 800, 600).unwrap()
    }

    /// What the person is looking at: the top-left of the plane.
    fn looking_here() -> TheView {
        TheView::showing(Patch::of(Spot::at(0, 0), 1_920, 1_080).unwrap())
    }

    fn window(number: u64, of: &str, called: &str, at: Patch, sits: HowItSits) -> Window {
        Window::of(WindowId::numbered(number), app(of), called, at, sits)
    }

    /// **Nothing open is a window near where you are looking**, because there is
    /// nowhere it was last used and a window that opened out of sight would be a
    /// click that appeared to do nothing.
    #[test]
    fn an_application_with_nothing_open_opens_one_where_the_person_is() {
        let did = what_a_click_does(&Windows::none(), &app("Browser"), looking_here());
        assert_eq!(did, WhatAClickDoes::OpenAWindowNearTheView);
        assert_eq!(did.window(), None);
        assert!(!did.travels());
        assert!(!did.brings_one_back());
    }

    /// **A window already in front of the person is raised and nothing moves.**
    #[test]
    fn a_window_already_in_view_is_brought_forward_without_moving_the_canvas() {
        let mut windows = Windows::none();
        windows.opened(window(
            1,
            "Browser",
            "Research",
            patch(100, 100),
            HowItSits::OnTheCanvas,
        ));

        let did = what_a_click_does(&windows, &app("Browser"), looking_here());
        assert_eq!(did, WhatAClickDoes::BringForward(WindowId::numbered(1)));
        assert!(!did.travels(), "the canvas does not move");
    }

    /// **A window out of view is travelled to**, and it is the *view* that
    /// travels: the answer says nothing about moving the window, because nothing
    /// moves it.
    #[test]
    fn a_window_out_of_view_is_travelled_to() {
        let mut windows = Windows::none();
        windows.opened(window(
            1,
            "Browser",
            "Research",
            patch(40_000, 30_000),
            HowItSits::OnTheCanvas,
        ));

        let did = what_a_click_does(&windows, &app("Browser"), looking_here());
        assert_eq!(did, WhatAClickDoes::TravelTo(WindowId::numbered(1)));
        assert!(did.travels());
        assert!(!did.brings_one_back());
    }

    /// **With several open, the click goes to the one used last** — not the
    /// first opened, not the nearest, and not a list.
    #[test]
    fn with_several_open_the_click_goes_to_the_one_used_last() {
        let mut windows = Windows::none();
        windows.opened(window(
            1,
            "Browser",
            "Launch website",
            patch(0, 0),
            HowItSits::OnTheCanvas,
        ));
        windows.opened(window(
            2,
            "Browser",
            "Research",
            patch(0, 0),
            HowItSits::OnTheCanvas,
        ));
        windows.opened(window(
            3,
            "Browser",
            "Preview",
            patch(0, 0),
            HowItSits::OnTheCanvas,
        ));

        let did = what_a_click_does(&windows, &app("Browser"), looking_here());
        assert_eq!(did.window(), Some(WindowId::numbered(3)));

        // And it follows the order rather than the numbering.
        windows.used(WindowId::numbered(1));
        let did = what_a_click_does(&windows, &app("Browser"), looking_here());
        assert_eq!(did.window(), Some(WindowId::numbered(1)));
    }

    /// **A window put aside comes back where it was, and the view goes to it.**
    #[test]
    fn a_window_put_aside_comes_back_at_its_own_place() {
        let mut windows = Windows::none();
        windows.opened(window(
            1,
            "Browser",
            "Research",
            patch(40_000, 30_000),
            HowItSits::PutAside,
        ));

        let did = what_a_click_does(&windows, &app("Browser"), looking_here());
        assert_eq!(
            did,
            WhatAClickDoes::BringItBackAndTravelTo(WindowId::numbered(1))
        );
        assert!(did.brings_one_back());
        assert!(did.travels());
    }

    /// **And when its place is already on screen, nothing travels.** Both rules
    /// are taken seriously at once, and travelling zero distance is still motion
    /// somebody may have asked not to see.
    #[test]
    fn a_window_put_aside_in_plain_view_comes_back_without_any_travel() {
        let mut windows = Windows::none();
        windows.opened(window(
            1,
            "Browser",
            "Research",
            patch(100, 100),
            HowItSits::PutAside,
        ));

        let did = what_a_click_does(&windows, &app("Browser"), looking_here());
        assert_eq!(
            did,
            WhatAClickDoes::BringItBackWhereItIs(WindowId::numbered(1))
        );
        assert!(did.brings_one_back());
        assert!(!did.travels(), "its place was already on screen");
    }

    /// **Clicking again does nothing a person would notice.** No cycling, no
    /// second window, and not quietly repurposed as going back.
    #[test]
    fn clicking_again_leaves_that_window_where_the_first_click_left_it() {
        let mut windows = Windows::none();
        windows.opened(window(
            1,
            "Browser",
            "Launch website",
            patch(0, 0),
            HowItSits::OnTheCanvas,
        ));
        windows.opened(window(
            2,
            "Browser",
            "Research",
            patch(0, 0),
            HowItSits::OnTheCanvas,
        ));

        let first = what_a_click_does(&windows, &app("Browser"), looking_here());
        assert_eq!(first.window(), Some(WindowId::numbered(2)));

        // The click focused it, which is a use.
        windows.used(WindowId::numbered(2));

        let again = what_a_click_does(&windows, &app("Browser"), looking_here());
        assert_eq!(again, first, "the same answer, and no travel in it");
        assert!(!again.travels());
        assert_eq!(windows.how_many(), 2, "and no window was created");
    }

    /// **A window filling a display is reached as it is.** The canvas is not
    /// what is showing it, so there is nothing to travel across.
    #[test]
    fn a_window_filling_the_screen_is_brought_forward_rather_than_travelled_to() {
        let mut windows = Windows::none();
        windows.opened(window(
            1,
            "Blender",
            "Opening scene",
            patch(40_000, 30_000),
            HowItSits::FillingTheScreen,
        ));

        let did = what_a_click_does(&windows, &app("Blender"), looking_here());
        assert_eq!(did, WhatAClickDoes::BringForward(WindowId::numbered(1)));
        assert!(
            !did.travels(),
            "its patch is far away and that is not what is showing it"
        );
    }

    /// **One application's windows are not another's.** A click on Blender is
    /// answered by Blender's most recent window even when a Browser window was
    /// used more recently than any of them.
    #[test]
    fn a_click_is_answered_by_that_applications_own_windows() {
        let mut windows = Windows::none();
        windows.opened(window(
            1,
            "Blender",
            "Opening scene",
            patch(0, 0),
            HowItSits::OnTheCanvas,
        ));
        windows.opened(window(
            2,
            "Browser",
            "Research",
            patch(0, 0),
            HowItSits::OnTheCanvas,
        ));

        assert_eq!(
            what_a_click_does(&windows, &app("Blender"), looking_here()).window(),
            Some(WindowId::numbered(1))
        );
        assert_eq!(
            what_a_click_does(&windows, &app("Browser"), looking_here()).window(),
            Some(WindowId::numbered(2))
        );
        assert_eq!(
            what_a_click_does(&windows, &app("Docs"), looking_here()),
            WhatAClickDoes::OpenAWindowNearTheView,
            "an application with nothing open, beside two that have windows"
        );
    }

    /// **Every answer either names a window or is the one that opens one**, so a
    /// caller can never act on a decision without knowing what it acts on.
    #[test]
    fn every_answer_that_is_about_a_window_names_it() {
        for (number, sits, at) in [
            (1_u64, HowItSits::OnTheCanvas, patch(100, 100)),
            (2, HowItSits::OnTheCanvas, patch(40_000, 30_000)),
            (3, HowItSits::PutAside, patch(100, 100)),
            (4, HowItSits::PutAside, patch(40_000, 30_000)),
            (5, HowItSits::FillingTheScreen, patch(100, 100)),
        ] {
            let mut windows = Windows::none();
            windows.opened(window(number, "Browser", "A window", at, sits));
            let did = what_a_click_does(&windows, &app("Browser"), looking_here());
            assert_eq!(
                did.window(),
                Some(WindowId::numbered(number)),
                "{did:?} did not name its window"
            );
        }
    }
}
