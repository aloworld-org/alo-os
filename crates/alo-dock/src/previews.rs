//! The list of an application's windows, for choosing one that is not the last
//! one used.
//!
//! The second half of the owner's rule:
//!
//! > Click an app to return to where you last used it. **Choose a preview when
//! > you want a different window.**
//!
//! [`crate::clicking`] is the first half. This is what opens when the first half
//! is not what somebody wanted.
//!
//! # One list, three doors
//!
//! Hovering the icon, putting keyboard focus on it, and holding it all open
//! **the same list**. That is [`Opened`], and it is carried on the list rather
//! than changing it: a list that differed by how it was opened would be three
//! things to learn, and the keyboard one would be the one that quietly got less
//! attention. Everything the Dock shows on hover is reachable by keyboard focus,
//! which is the owner's constraint and is met here by construction rather than
//! by remembering.
//!
//! # The one used last is first, and says so
//!
//! **Marked, not merely first.** A person who cannot see the order — reading
//! with a screen reader, or glancing at a list of eight — needs the fact that
//! this is the one they were in last, and position alone does not carry it.
//! [`Preview::is_the_one_used_last`] is that fact, and it is on exactly one
//! preview.
//!
//! **It was called `is_the_one_used_last` until 2026-10-11**, and the
//! rename is the whole of what ADR 0103 cost this file. A click on the icon of
//! an application with several windows now opens *this list* rather than going
//! to a window, so *where a click would go* stopped being true of any row here
//! — while the thing the field computes, the window used last, did not change
//! at all. The name was the only wrong part, and a name that is wrong about a
//! mark a screen reader reads out is worse than one that is merely dated.
//!
//! What it is for now is the picker's **default**: the row a person lands on
//! when the list opens, which is still the one they were in last.
//!
//! # Windows put aside are on this list
//!
//! **And that is not the duplicate the owner refused.** The right panel holds
//! what was put aside, across every application, permanently, and answers *what
//! did I put aside*. This list is one application's, appears on demand, and
//! answers *which window of this one*. Leaving a put-aside window off it would
//! mean an application whose third window could not be reached at all except by
//! it happening to become the most recent.
//!
//! # Nothing here draws or decides what happens next
//!
//! A preview names a window and says where it sits. Choosing one is a click on
//! that window rather than on the application, and what that does is
//! [`crate::clicking`]'s answer about a window already chosen — so the two
//! halves of the rule cannot come to disagree about travel.

use crate::on_the_canvas::Patch;
use crate::window::{AppId, HowItSits, WindowId};
use crate::windows::Windows;

/// How the list was opened.
///
/// Carried so that whoever draws it can differ — a list under a pointer and a
/// list under keyboard focus are not placed the same way — while what is *in*
/// it cannot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opened {
    /// The pointer rested on the icon.
    ByHovering,
    /// The icon has keyboard focus.
    ByKeyboardFocus,
    /// The icon was held down.
    ByHolding,
}

impl Opened {
    /// Every way it can be opened, for a test that will not let one of them
    /// quietly answer differently.
    pub const ALL: [Self; 3] = [Self::ByHovering, Self::ByKeyboardFocus, Self::ByHolding];
}

/// One row: a window of this application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    /// Which window.
    id: WindowId,
    /// What the person calls it, which is what the row is headed with.
    called: String,
    /// Where it sits on the plane, so a row can show where it is.
    at: Patch,
    /// How it sits, so a row can say *put aside* without guessing.
    sits: HowItSits,
    /// Whether a plain click on the icon would have gone here.
    is_the_one_used_last: bool,
}

impl Preview {
    /// Which window.
    #[must_use]
    pub const fn id(&self) -> WindowId {
        self.id
    }

    /// What the person calls it.
    #[must_use]
    pub fn called(&self) -> &str {
        &self.called
    }

    /// Where it sits on the plane.
    #[must_use]
    pub const fn at(&self) -> Patch {
        self.at
    }

    /// How it sits.
    #[must_use]
    pub const fn sits(&self) -> HowItSits {
        self.sits
    }

    /// **Whether this is the window the person used last.**
    ///
    /// True on exactly one preview in a list that has any. Marked rather than
    /// left to position, because somebody reading the list one row at a time
    /// never sees the position.
    ///
    /// See this module's header for why it is no longer called
    /// `is_the_one_used_last`.
    #[must_use]
    pub const fn is_the_one_used_last(&self) -> bool {
        self.is_the_one_used_last
    }
}

/// An application's windows, in the order they were last used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Previews {
    /// Which application these belong to.
    app: AppId,
    /// How the list was opened.
    opened: Opened,
    /// The rows, most recently used first.
    rows: Vec<Preview>,
}

impl Previews {
    /// This application's windows, as a list opened this way.
    ///
    /// **The list is the same whichever way it was opened**, which is held by a
    /// test rather than by care.
    #[must_use]
    pub fn of(windows: &Windows, app: &AppId, opened: Opened) -> Self {
        let rows = windows
            .of_the_app(app)
            .enumerate()
            .map(|(place, window)| Preview {
                id: window.id(),
                called: window.called().to_owned(),
                at: window.at(),
                sits: window.sits(),
                // The first is the one the use-order puts first, which is by
                // definition where `crate::clicking` would have gone.
                is_the_one_used_last: place == 0,
            })
            .collect();
        Self {
            app: app.clone(),
            opened,
            rows,
        }
    }

    /// Which application these belong to.
    #[must_use]
    pub const fn app(&self) -> &AppId {
        &self.app
    }

    /// How this list was opened.
    #[must_use]
    pub const fn opened(&self) -> Opened {
        self.opened
    }

    /// The rows, most recently used first.
    #[must_use]
    pub fn rows(&self) -> &[Preview] {
        &self.rows
    }

    /// How many rows.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.rows.len()
    }

    /// Whether there is anything to choose between.
    ///
    /// **An application with one window has a list of one**, and that is not the
    /// same as having none: the row still names the window and says where it is,
    /// which is worth showing to somebody who wants to know before they travel.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// The row for the window the person used last, which is where the picker
    /// opens.
    #[must_use]
    pub fn the_one_used_last(&self) -> Option<&Preview> {
        self.rows.iter().find(|row| row.is_the_one_used_last())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::clicking::what_a_click_does;
    use crate::on_the_canvas::{Spot, TheView};
    use crate::window::Window;

    fn app(name: &str) -> AppId {
        AppId::named(name).unwrap()
    }

    fn patch(x: i64) -> Patch {
        Patch::of(Spot::at(x, 0), 800, 600).unwrap()
    }

    fn window(number: u64, of: &str, called: &str, sits: HowItSits) -> Window {
        Window::of(
            WindowId::numbered(number),
            Some(app(of)),
            called,
            patch(i64::from(u32::try_from(number).unwrap()) * 1_000),
            sits,
        )
    }

    /// The Browser of the owner's example: three windows, opened in this order.
    fn a_browser() -> Windows {
        let mut windows = Windows::none();
        windows.opened(window(
            1,
            "Browser",
            "Launch website",
            HowItSits::OnTheCanvas,
        ));
        windows.opened(window(2, "Browser", "Research", HowItSits::OnTheCanvas));
        windows.opened(window(3, "Browser", "Preview", HowItSits::OnTheCanvas));
        windows.opened(window(
            4,
            "Blender",
            "Opening scene",
            HowItSits::OnTheCanvas,
        ));
        windows
    }

    fn titles(previews: &Previews) -> Vec<String> {
        previews
            .rows()
            .iter()
            .map(|row| row.called().to_owned())
            .collect()
    }

    /// **The owner's example, named.** Launch website, Research and Preview are
    /// three rows a person picks between, most recently used first.
    #[test]
    fn an_applications_windows_are_named_most_recently_used_first() {
        let previews = Previews::of(&a_browser(), &app("Browser"), Opened::ByHovering);
        assert_eq!(titles(&previews), ["Preview", "Research", "Launch website"]);
        assert_eq!(previews.how_many(), 3);
        assert_eq!(previews.app(), &app("Browser"));
    }

    /// **One list, three doors.** Hover, keyboard focus and a long press open
    /// the same rows in the same order — held by a test rather than by care,
    /// because the keyboard one is the one that quietly gets less attention.
    #[test]
    fn every_way_of_opening_it_gives_the_same_list() {
        let windows = a_browser();
        let mut seen: Option<Vec<String>> = None;
        for opened in Opened::ALL {
            let previews = Previews::of(&windows, &app("Browser"), opened);
            assert_eq!(previews.opened(), opened, "it remembers which door");
            let rows = titles(&previews);
            match &seen {
                None => seen = Some(rows),
                Some(first) => assert_eq!(&rows, first, "{opened:?} gave a different list"),
            }
        }
        assert!(seen.is_some(), "no list was built at all");
    }

    /// **The one a click would go to is marked**, not merely first — because
    /// somebody reading one row at a time never sees the position.
    #[test]
    fn the_row_for_the_window_used_last_is_marked_and_there_is_one_of_it() {
        let previews = Previews::of(&a_browser(), &app("Browser"), Opened::ByKeyboardFocus);

        let marked: Vec<&str> = previews
            .rows()
            .iter()
            .filter(|row| row.is_the_one_used_last())
            .map(Preview::called)
            .collect();
        assert_eq!(marked, ["Preview"], "exactly one, and it is the first");
        assert_eq!(previews.the_one_used_last().unwrap().called(), "Preview");
    }

    /// **The mark follows the order, and the click rule no longer names a
    /// window to compare it against.**
    ///
    /// This test asked the two halves of the owner's sentence of one order so
    /// they could not disagree about which window is *where you last used it*,
    /// and it compared the mark with `what_a_click_does(..).window()`. ADR 0103
    /// made a click on a multi-window application open **this list**, so that
    /// call now answers `ChooseAWindow` and names no window — and this list is
    /// exactly the application for which there is no longer anything to
    /// compare.
    ///
    /// What is left to hold is the half that is still a fact: **the mark is the
    /// window used last**, which is where the picker opens. Asked of
    /// `Windows::most_recent_of`, which is the same source the click rule reads
    /// for the single-window case, so the two still cannot drift apart.
    ///
    /// And the pairing is checked where it still means something: a
    /// single-window application, where a click does go to a window and that
    /// window must be the marked one.
    #[test]
    fn the_marked_row_is_the_window_used_last() {
        let mut windows = a_browser();

        for used in [1_u64, 3, 2] {
            windows.used(WindowId::numbered(used));
            let previews = Previews::of(&windows, &app("Browser"), Opened::ByHovering);
            let marked = previews.the_one_used_last().unwrap().id();
            let recent = windows.most_recent_of(&app("Browser")).unwrap().id();
            assert_eq!(marked, recent, "after using {used}");
        }
    }

    /// **Where a click still names a window, it is the marked one.**
    ///
    /// The other half of the test above, kept alive on the case it survives:
    /// one window, so `what_a_click_does` answers about that window rather than
    /// opening a picker.
    #[test]
    fn with_one_window_the_click_goes_to_the_marked_row() {
        let mut windows = Windows::none();
        windows.opened(window(1, "Browser", "Preview", HowItSits::OnTheCanvas));
        let view = TheView::showing(Patch::of(Spot::at(0, 0), 1_920, 1_080).unwrap());

        let previews = Previews::of(&windows, &app("Browser"), Opened::ByHovering);
        let marked = previews.the_one_used_last().unwrap().id();
        let clicked = what_a_click_does(&windows, &app("Browser"), view)
            .window()
            .unwrap();
        assert_eq!(marked, clicked);
    }

    /// **A window put aside is on the list**, or an application's third window
    /// could not be reached at all — and the row says so, so a person choosing
    /// knows it will have to come back first.
    #[test]
    fn a_window_put_aside_is_on_the_list_and_says_that_it_is() {
        let mut windows = a_browser();
        windows.now_sits(WindowId::numbered(2), HowItSits::PutAside);

        let previews = Previews::of(&windows, &app("Browser"), Opened::ByHolding);
        assert_eq!(previews.how_many(), 3, "it is still reachable");

        let research = previews
            .rows()
            .iter()
            .find(|row| row.called() == "Research")
            .unwrap();
        assert!(research.sits().was_put_aside());
        assert!(!research.is_the_one_used_last());
    }

    /// **Putting one aside does not reorder the list**, because putting a window
    /// away is not using it — so the row a click would go to does not move
    /// under somebody who tidied their desk.
    #[test]
    fn putting_a_window_aside_does_not_move_the_rows() {
        let windows = a_browser();
        let before = titles(&Previews::of(&windows, &app("Browser"), Opened::ByHovering));

        let mut windows = windows;
        windows.now_sits(WindowId::numbered(3), HowItSits::PutAside);
        let after = titles(&Previews::of(&windows, &app("Browser"), Opened::ByHovering));

        assert_eq!(before, after);
        assert_eq!(
            Previews::of(&windows, &app("Browser"), Opened::ByHovering)
                .the_one_used_last()
                .unwrap()
                .called(),
            "Preview",
            "and the marked row is still the one that was used last"
        );
    }

    /// **A row says where its window is**, which is what a small map of the
    /// canvas is drawn from.
    #[test]
    fn a_row_says_where_its_window_sits() {
        let previews = Previews::of(&a_browser(), &app("Browser"), Opened::ByHovering);
        for row in previews.rows() {
            assert_eq!(row.at().wide(), 800);
        }
        let first = previews.the_one_used_last().unwrap();
        assert_eq!(first.at().corner().x(), 3_000, "the third window's place");
    }

    /// **One application's list holds no other application's windows**, even
    /// when another was used more recently.
    #[test]
    fn one_applications_list_is_its_own() {
        let windows = a_browser();
        let blender = Previews::of(&windows, &app("Blender"), Opened::ByHovering);
        assert_eq!(titles(&blender), ["Opening scene"]);
        assert!(
            blender.the_one_used_last().is_some(),
            "a list of one still marks its row"
        );
    }

    /// **An application with nothing open has an empty list**, and that is not
    /// an error: a click on it opens a window, and there was never anything to
    /// choose between.
    #[test]
    fn an_application_with_nothing_open_has_no_rows() {
        let previews = Previews::of(&a_browser(), &app("Docs"), Opened::ByKeyboardFocus);
        assert!(previews.is_empty());
        assert_eq!(previews.how_many(), 0);
        assert_eq!(previews.the_one_used_last(), None);
    }
}
