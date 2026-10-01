//! What the Dock holds: the applications on it, and which of them fit.
//!
//! **Icon positions stay stable.** An icon that moved when something else opened
//! would make the Dock a thing a person has to read before clicking, and the
//! whole value of a fixed navigation point is that it can be clicked without
//! being read. So the order here is *pinned first, in the order they were
//! pinned, then everything else that is open, in the order it opened* — and
//! nothing reorders because of use. The use-order decides **which window** a
//! click goes to ([`crate::clicking`]); it never decides where the icon is.
//!
//! # Pinned and open are two states, and they are not the same question
//!
//! **Pinned** means *stay on the Dock when this is closed*. **Open** means *this
//! has a window right now*. An application can be either, both, or — for a
//! moment while something is starting — neither, at which point it is not on the
//! Dock at all.
//!
//! They are told apart **without relying on colour**, which is the owner's
//! constraint and is why they are two separate answers here rather than one
//! enum with a colour attached. Whoever draws them gets two facts and has to
//! show both; a single *state* would let a drawing show one and imply the other.
//!
//! # Too many applications
//!
//! Icons keep a usable size and the rest go into a named overflow. **The size is
//! never traded away**: shrinking icons to fit more is how a dock becomes
//! unusable at exactly the moment a person has the most open, and an icon is a
//! thing somebody presses, held to a floor by
//! [`alo_appearance::targets::ENHANCED_TARGET`].
//!
//! **What that sentence was protecting is the target, and the owner has now
//! separated the two.** Their ruling of 2026-09-30 is that the Dock's glyph may
//! shrink from 32 to 28 as the bar fills, while a click target stays at least
//! 44 — so a little of the *drawing* is traded and none of the *pressing* is.
//! This file could not express that while a place was one number for both, and
//! `crate::places` still is: a place is always `ICON` across, so the thing drawn
//! and the thing pressed cannot move apart. That is the change this paragraph
//! is owed, and it is not made here.
//!
//! **What overflows is chosen from the end**, so that a pinned application a
//! person put there on purpose does not disappear because something else opened.

use crate::window::AppId;
use crate::windows::Windows;

/// One application on the Dock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnTheDock {
    /// Which application.
    app: AppId,
    /// Whether it stays here when it has no windows.
    pinned: Pinned,
    /// How many windows it has open, which may be none.
    windows: usize,
    /// How many of those were put aside.
    put_aside: usize,
}

/// Whether an application stays on the Dock when it is closed.
///
/// Named rather than a `bool` so that *pinned* and *open* cannot be handed over
/// the wrong way round: they are both yes-or-no facts about one application and
/// they read alike at a call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pinned {
    /// The person put it here; it stays when it is closed.
    ByThePerson,
    /// It is here because it is open, and goes when it closes.
    No,
}

impl OnTheDock {
    /// Which application.
    #[must_use]
    pub const fn app(&self) -> &AppId {
        &self.app
    }

    /// Whether it stays when it is closed.
    #[must_use]
    pub const fn pinned(&self) -> Pinned {
        self.pinned
    }

    /// Whether it has any windows at all.
    ///
    /// **A separate answer from [`Self::pinned`], on purpose.** A drawing has to
    /// show both, and the owner's rule is that they are distinguishable without
    /// colour — which a single combined state would quietly allow somebody to
    /// get away with not doing.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.windows > 0
    }

    /// How many windows it has open.
    #[must_use]
    pub const fn how_many_windows(&self) -> usize {
        self.windows
    }

    /// How many of them were put aside.
    #[must_use]
    pub const fn how_many_put_aside(&self) -> usize {
        self.put_aside
    }
}

/// The Dock's applications, in the order they are drawn.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Holding {
    /// Pinned, in the order they were pinned. These come first and stay put.
    pinned: Vec<AppId>,
}

impl Holding {
    /// A Dock with nothing pinned to it.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// Pin this application, at the end of what is already pinned.
    ///
    /// Pinning something already pinned changes nothing and does not move it:
    /// an icon that jumped because somebody pinned it twice would be the
    /// instability this file exists to prevent.
    pub fn pin(&mut self, app: AppId) {
        if !self.pinned.contains(&app) {
            self.pinned.push(app);
        }
    }

    /// Stop pinning this application. Says whether it was pinned.
    pub fn unpin(&mut self, app: &AppId) -> bool {
        let before = self.pinned.len();
        self.pinned.retain(|held| held != app);
        self.pinned.len() != before
    }

    /// Whether this application is pinned.
    #[must_use]
    pub fn is_pinned(&self, app: &AppId) -> bool {
        self.pinned.contains(app)
    }

    /// Every application pinned, in the order they were pinned.
    pub fn each_pinned(&self) -> impl Iterator<Item = &AppId> {
        self.pinned.iter()
    }

    /// What the Dock shows, given what is open: pinned first, then the rest of
    /// what is open, in the order it opened.
    ///
    /// **Nothing here is ordered by use.** Where an icon sits is a fact a person
    /// learns once; which window a click goes to is a fact about this moment.
    /// Letting the second move the first would trade the Dock's only real
    /// advantage for a marginal one.
    #[must_use]
    pub fn showing(&self, windows: &Windows) -> Vec<OnTheDock> {
        let mut shown: Vec<OnTheDock> = self
            .pinned
            .iter()
            .map(|app| self.one(app, Pinned::ByThePerson, windows))
            .collect();

        // The rest of what is open, in the order it opened — never the order it
        // was used, or an icon would move under the hand of somebody who merely
        // looked at a window.
        let mut rest: Vec<&AppId> = Vec::new();
        for window in windows.each_as_opened() {
            // **A window whose client named no application contributes no icon,
            // and the absence is honoured here rather than the window being
            // quietly lost.** The Dock's icons *are* applications: a window with
            // none has nothing to appear under, and inventing a name would
            // gather every unnamed window of every client beneath one false one.
            //
            // It stays reachable by every road that does not group by
            // application — the put-aside panel holds it by its own identity,
            // title and preview, and window switching lists windows rather than
            // applications. So this is the Dock declining to answer a question
            // about something that is not an application, not the Dock hiding a
            // window.
            let Some(app) = window.app() else {
                continue;
            };
            if !self.is_pinned(app) && !rest.contains(&app) {
                rest.push(app);
            }
        }
        shown.extend(
            rest.into_iter()
                .map(|app| self.one(app, Pinned::No, windows)),
        );
        shown
    }

    /// One application's row.
    fn one(&self, app: &AppId, pinned: Pinned, windows: &Windows) -> OnTheDock {
        OnTheDock {
            app: app.clone(),
            pinned,
            windows: windows.how_many_of(app),
            put_aside: windows.how_many_of_them_put_aside(app),
        }
    }
}

/// What is drawn on the Dock and what went into the overflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fitted {
    /// Drawn on the Dock itself, in order.
    on_the_dock: Vec<OnTheDock>,
    /// In the overflow list, in order.
    over: Vec<OnTheDock>,
}

impl Fitted {
    /// Drawn on the Dock itself.
    #[must_use]
    pub fn on_the_dock(&self) -> &[OnTheDock] {
        &self.on_the_dock
    }

    /// In the overflow list.
    #[must_use]
    pub fn over(&self) -> &[OnTheDock] {
        &self.over
    }

    /// Whether anything had to go into the overflow.
    #[must_use]
    pub fn anything_over(&self) -> bool {
        !self.over.is_empty()
    }
}

/// Fit these applications into room for `at_most` icons.
///
/// **The icon size is not the thing that gives**, so what does not fit goes into
/// a named list. What overflows is taken from the end, which is why pinned
/// applications come first: something a person put there on purpose should not
/// be pushed off by something that merely opened.
///
/// Room for none at all puts everything in the overflow rather than dropping
/// any: a Dock too small to draw is a Dock a person can still reach through one
/// control, and silently losing applications is not an option this returns.
#[must_use]
pub fn fit(showing: Vec<OnTheDock>, at_most: usize) -> Fitted {
    if showing.len() <= at_most {
        return Fitted {
            on_the_dock: showing,
            over: Vec::new(),
        };
    }
    // One place is given to the overflow control itself, or the last icon would
    // be replaced by the thing that says what is missing.
    let room = at_most.saturating_sub(1);
    let mut showing = showing;
    let over = showing.split_off(room);
    Fitted {
        on_the_dock: showing,
        over,
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported — \
              and since the workspace denies indexing a slice, a row that is not there says \
              which row was wanted rather than panicking with a bare index"
)]
mod tests {
    use super::*;
    use crate::on_the_canvas::{Patch, Spot};
    use crate::window::{HowItSits, Window, WindowId};

    fn app(name: &str) -> AppId {
        AppId::named(name).unwrap()
    }

    fn window(number: u64, of: &str, sits: HowItSits) -> Window {
        Window::of(
            WindowId::numbered(number),
            Some(app(of)),
            "A window",
            Patch::of(Spot::at(0, 0), 800, 600).unwrap(),
            sits,
        )
    }

    /// The names on the Dock, owned — so a test can hold an earlier answer
    /// beside a later one without keeping the whole earlier Dock alive.
    /// The row at this place. The workspace denies indexing a slice and a test
    /// is not exempt: a row that is not there should say which one was wanted
    /// and how long the dock was, rather than panic with a bare index.
    fn row(shown: &[OnTheDock], at: usize) -> &OnTheDock {
        shown
            .get(at)
            .unwrap_or_else(|| panic!("no row {at} on a dock of {}", shown.len()))
    }

    fn names(shown: &[OnTheDock]) -> Vec<String> {
        shown
            .iter()
            .map(|one| one.app().name().to_owned())
            .collect()
    }

    /// **Pinned first, in the order they were pinned**, whatever is open.
    #[test]
    fn what_is_pinned_comes_first_in_the_order_it_was_pinned() {
        let mut holding = Holding::nothing();
        holding.pin(app("Docs"));
        holding.pin(app("Browser"));

        let mut windows = Windows::none();
        windows.opened(window(1, "Blender", HowItSits::OnTheCanvas));

        assert_eq!(
            names(&holding.showing(&windows)),
            ["Docs", "Browser", "Blender"]
        );
    }

    /// **Opening something does not move an icon that was already there**, which
    /// is the whole of *icon positions stay stable*.
    #[test]
    fn opening_an_application_never_moves_the_icons_already_there() {
        let mut holding = Holding::nothing();
        holding.pin(app("Docs"));
        holding.pin(app("Browser"));

        let mut windows = Windows::none();
        let before = names(&holding.showing(&windows)).join(",");

        windows.opened(window(1, "Blender", HowItSits::OnTheCanvas));
        windows.opened(window(2, "Ptyxis", HowItSits::OnTheCanvas));
        let after = names(&holding.showing(&windows)).join(",");

        assert!(after.starts_with(&before), "{before} then {after}");
        assert_eq!(after, "Docs,Browser,Blender,Ptyxis");
    }

    /// **Using a window does not move any icon either.** The use-order decides
    /// which window a click goes to and nothing about where the icon sits.
    #[test]
    fn using_a_window_does_not_reorder_the_dock() {
        let holding = Holding::nothing();
        let mut windows = Windows::none();
        windows.opened(window(1, "Blender", HowItSits::OnTheCanvas));
        windows.opened(window(2, "Ptyxis", HowItSits::OnTheCanvas));
        let before = names(&holding.showing(&windows));

        windows.used(WindowId::numbered(1));
        assert_eq!(
            names(&holding.showing(&windows)),
            before,
            "the most recently used application did not jump to the front"
        );
    }

    /// **Pinned and open are two answers**, and every combination of them is
    /// reachable — which is what lets a drawing show both without colour.
    #[test]
    fn pinned_and_open_are_told_apart_separately() {
        let mut holding = Holding::nothing();
        holding.pin(app("Docs"));
        holding.pin(app("Browser"));

        let mut windows = Windows::none();
        windows.opened(window(1, "Browser", HowItSits::OnTheCanvas));
        windows.opened(window(2, "Blender", HowItSits::OnTheCanvas));

        let shown = holding.showing(&windows);
        let docs = row(&shown, 0);
        let browser = row(&shown, 1);
        let blender = row(&shown, 2);

        assert_eq!(docs.pinned(), Pinned::ByThePerson);
        assert!(!docs.is_open(), "pinned and closed");

        assert_eq!(browser.pinned(), Pinned::ByThePerson);
        assert!(browser.is_open(), "pinned and open");

        assert_eq!(blender.pinned(), Pinned::No);
        assert!(blender.is_open(), "open and not pinned");
    }

    /// An application that is neither pinned nor open is not on the Dock at all.
    #[test]
    fn something_neither_pinned_nor_open_is_not_there() {
        let holding = Holding::nothing();
        let windows = Windows::none();
        assert!(holding.showing(&windows).is_empty());
    }

    /// Unpinning leaves an open application there, and takes a closed one away.
    #[test]
    fn unpinning_keeps_it_only_while_it_is_open() {
        let mut holding = Holding::nothing();
        holding.pin(app("Browser"));
        holding.pin(app("Docs"));

        let mut windows = Windows::none();
        windows.opened(window(1, "Browser", HowItSits::OnTheCanvas));

        assert!(holding.unpin(&app("Browser")));
        assert_eq!(names(&holding.showing(&windows)), ["Docs", "Browser"]);

        assert!(holding.unpin(&app("Docs")));
        assert_eq!(names(&holding.showing(&windows)), ["Browser"]);

        assert!(!holding.unpin(&app("Docs")), "and only once");
    }

    /// Pinning twice changes nothing and moves nothing.
    #[test]
    fn pinning_something_twice_does_not_move_it() {
        let mut holding = Holding::nothing();
        holding.pin(app("Docs"));
        holding.pin(app("Browser"));
        holding.pin(app("Docs"));

        let pinned: Vec<&str> = holding.each_pinned().map(AppId::name).collect();
        assert_eq!(pinned, ["Docs", "Browser"]);
    }

    /// How many windows, and how many were put aside — what a screen reader is
    /// given for *three windows open, one minimised*.
    #[test]
    fn each_row_carries_what_a_screen_reader_announces() {
        let holding = Holding::nothing();
        let mut windows = Windows::none();
        windows.opened(window(1, "Browser", HowItSits::OnTheCanvas));
        windows.opened(window(2, "Browser", HowItSits::OnTheCanvas));
        windows.opened(window(3, "Browser", HowItSits::PutAside));

        let shown = holding.showing(&windows);
        assert_eq!(row(&shown, 0).how_many_windows(), 3);
        assert_eq!(row(&shown, 0).how_many_put_aside(), 1);
    }

    /// **Everything fits when there is room**, and nothing is put in an overflow
    /// that did not need one.
    #[test]
    fn nothing_overflows_when_there_is_room() {
        let mut holding = Holding::nothing();
        for name in ["Docs", "Browser", "Blender"] {
            holding.pin(app(name));
        }
        let fitted = fit(holding.showing(&Windows::none()), 8);
        assert_eq!(fitted.on_the_dock().len(), 3);
        assert!(!fitted.anything_over());
        assert!(fitted.over().is_empty());
    }

    /// **What does not fit goes into the overflow, taken from the end**, so a
    /// pinned application is not pushed off by something that merely opened.
    ///
    /// One place goes to the overflow control itself, or the last icon would be
    /// replaced by the thing that says what is missing.
    #[test]
    fn what_does_not_fit_is_taken_from_the_end() {
        let mut holding = Holding::nothing();
        for name in ["Docs", "Browser", "Blender", "Ptyxis", "Files"] {
            holding.pin(app(name));
        }
        let fitted = fit(holding.showing(&Windows::none()), 3);

        assert_eq!(names(fitted.on_the_dock()), ["Docs", "Browser"]);
        assert_eq!(names(fitted.over()), ["Blender", "Ptyxis", "Files"]);
        assert!(fitted.anything_over());
        assert_eq!(
            fitted.on_the_dock().len() + fitted.over().len(),
            5,
            "nothing was lost"
        );
    }

    /// **Nothing is ever silently dropped**, even asked to fit into no room at
    /// all: it all goes into the list rather than disappearing.
    #[test]
    fn room_for_none_puts_everything_in_the_list_rather_than_losing_it() {
        let mut holding = Holding::nothing();
        for name in ["Docs", "Browser"] {
            holding.pin(app(name));
        }
        for at_most in [0, 1] {
            let fitted = fit(holding.showing(&Windows::none()), at_most);
            assert!(fitted.on_the_dock().is_empty(), "{at_most}");
            assert_eq!(fitted.over().len(), 2, "{at_most}");
        }
    }
}
