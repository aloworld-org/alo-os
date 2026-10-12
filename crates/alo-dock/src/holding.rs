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

use crate::a_dock_entry::{ADockEntry, ANativeAction};
use crate::window::AppId;
use crate::windows::Windows;

/// One thing on the Dock.
///
/// **An application or a native action**, since the owner's ruling of
/// 2026-10-12. A native row has no windows and never will, so every count below
/// answers zero for it — which is true rather than a special case, and is why
/// the membership rule needs no arm for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnTheDock {
    /// Which thing.
    what: ADockEntry,
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
    /// Which application, if this row is one.
    ///
    /// **[`Option`] since 2026-10-12**, and the change is the point rather than
    /// a cost of it: a native row has no identifier, and answering with one
    /// would be the invented name `ADockEntry`'s header refuses. Every caller
    /// that draws an icon, announces a row or dispatches a press now has to say
    /// what a native row means to it.
    #[must_use]
    pub const fn app(&self) -> Option<&AppId> {
        self.what.application()
    }

    /// Which thing this row is.
    #[must_use]
    pub const fn what(&self) -> &ADockEntry {
        &self.what
    }

    /// The compositor action this row is, if it is one.
    #[must_use]
    pub const fn native(&self) -> Option<ANativeAction> {
        self.what.native()
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

    /// Whether this application belongs on the Dock at all.
    ///
    /// The owner's membership rule of 2026-10-11 —
    /// [ADR 0103](https://github.com/alo-os/alo-os/blob/main/docs/decisions/0103-the-dock-more-apps-and-the-minimised-panel-are-three-connected-surfaces.md).
    /// **Pinned is unconditional**; unpinned needs **a window that is not
    /// minimised**.
    ///
    /// # What this supersedes, and why it was never a rule anybody could check
    ///
    /// Until this method existed, [`Holding::showing`] gave an icon to every
    /// application that had a window and asked nothing about that window's
    /// state. So an unpinned application whose windows a person had all put
    /// aside stayed on the bar, with its windows *also* each in the panel — the
    /// same application answering in two places, one of which was supposed to be
    /// the record of it being away. That behaviour was written down nowhere.
    ///
    /// # "Not minimised" is a fact about the window, never about the camera
    ///
    /// A window on another Place, outside the view, or behind another window is
    /// **not** minimised and keeps its application here. This type cannot make
    /// the other mistake, because it holds counts and has never seen a camera —
    /// which is the architecture doing the work rather than a reviewer.
    #[must_use]
    pub const fn belongs_on_the_dock(&self) -> bool {
        matches!(self.pinned, Pinned::ByThePerson) || self.windows > self.put_aside
    }
}

/// The Dock's entries, in the order they are drawn.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Holding {
    /// Pinned, in the order they were pinned. These come first and stay put.
    ///
    /// **Applications and native actions in one list**, because the person's
    /// arrangement is one arrangement: Settings sits third because they put it
    /// third, and a separate list for native entries would make its position a
    /// property of its kind. The owner's ruling says the opposite — *native
    /// describes the implementation; it does not mean permanent* — and a
    /// second list is how *permanent* gets built without anybody deciding to.
    pinned: Vec<ADockEntry>,
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
        self.pin_entry(ADockEntry::Application(app));
    }

    /// Pin this entry, at the end of what is already pinned.
    ///
    /// [`Self::pin`]'s rule, for either kind: pinning something already pinned
    /// changes nothing and does not move it.
    pub fn pin_entry(&mut self, entry: ADockEntry) {
        if !self.pinned.contains(&entry) {
            self.pinned.push(entry);
        }
    }

    /// Stop pinning this application. Says whether it was pinned.
    pub fn unpin(&mut self, app: &AppId) -> bool {
        self.unpin_entry(&ADockEntry::Application(app.clone()))
    }

    /// Stop pinning this entry. Says whether it was pinned.
    ///
    /// **A native entry is removed by this and by nothing else**, which is the
    /// ruling's *it does not mean permanent* made into code: there is one
    /// removal and both kinds go through it.
    pub fn unpin_entry(&mut self, entry: &ADockEntry) -> bool {
        let before = self.pinned.len();
        self.pinned.retain(|held| held != entry);
        self.pinned.len() != before
    }

    /// Whether this application is pinned.
    #[must_use]
    pub fn is_pinned(&self, app: &AppId) -> bool {
        self.is_entry_pinned(&ADockEntry::Application(app.clone()))
    }

    /// Whether this entry is pinned.
    #[must_use]
    pub fn is_entry_pinned(&self, entry: &ADockEntry) -> bool {
        self.pinned.contains(entry)
    }

    /// Every entry pinned, in the order they were pinned.
    pub fn each_pinned(&self) -> impl Iterator<Item = &ADockEntry> {
        self.pinned.iter()
    }

    /// Every pinned **application**, in the order they were pinned, skipping
    /// the native entries.
    ///
    /// For callers that genuinely mean applications — matching a window to its
    /// icon, say. A caller that means *the person's arrangement* wants
    /// [`Self::each_pinned`], and the two are different questions.
    pub fn each_pinned_application(&self) -> impl Iterator<Item = &AppId> {
        self.pinned.iter().filter_map(ADockEntry::application)
    }

    /// What the Dock shows: pinned first, then every **other application with a
    /// window that is not minimised**, in the order it opened.
    ///
    /// The membership rule is [`OnTheDock::belongs_on_the_dock`], which is where
    /// its reasons are. Pinned applications are not filtered, because pinned is
    /// unconditional — a person who put something here meant it to stay.
    ///
    /// **Nothing here is ordered by use.** Where an icon sits is a fact a person
    /// learns once; which window a click goes to is a fact about this moment.
    /// Letting the second move the first would trade the Dock's only real
    /// advantage for a marginal one.
    #[must_use]
    pub fn showing(&self, windows: &Windows) -> Vec<OnTheDock> {
        let mut shown = self.in_the_overview(windows);

        // **An unpinned application whose windows are all minimised is not
        // here**, and its windows are each in the panel at the edge.
        // [`OnTheDock::belongs_on_the_dock`] carries the rule and the reasons.
        //
        // Applied to **every** row, not to the unpinned ones alone. Measured
        // 2026-10-11: with the filter on the unpinned rows only, deleting the
        // pinned clause from `belongs_on_the_dock` left all 219 tests passing —
        // the clause was correct, documented, and dead. A rule stated in one
        // place and applied to half its subjects survives every test written
        // against the behaviour, because the behaviour is what it gets right.
        //
        // Order is untouched: a filter keeps what it keeps where it was, so the
        // pinned section and the opening order both survive it.
        shown.retain(OnTheDock::belongs_on_the_dock);
        shown
    }

    /// **Everything *More apps* shows**: every pinned application in its saved
    /// order, then every other application with a window, in opening order.
    ///
    /// The owner's ruling of 2026-10-11, ADR 0103: *pinned applications first,
    /// in their saved order; other currently open applications after them,
    /// **including applications whose windows are all minimised**; each
    /// application exactly once.*
    ///
    /// # It is the Dock before the membership rule, and that is the point
    ///
    /// [`Self::showing`] is this list with
    /// [`OnTheDock::belongs_on_the_dock`] applied. Written as two walks they
    /// would be two answers to *which applications does this person have*, and
    /// the one place they are guaranteed to differ — an unpinned application
    /// whose windows are all minimised — is the exact case the ruling says must
    /// appear here and not there. A second walk would get that right on the day
    /// and drift afterwards.
    ///
    /// So the relationship is a line of code rather than a convention: **More
    /// apps is everything; the Dock is everything that belongs on it.**
    ///
    /// # What is not here
    ///
    /// Applications that are merely installed. The ruling is explicit —
    /// *installed applications that are neither pinned nor open do not belong
    /// in this grid; they remain available through Apps or the alo Bar* — and
    /// this type could not list them anyway: it is handed what is pinned and
    /// what is open and has never seen an installed-application list. **Absent
    /// by construction rather than by a filter somebody has to remember.**
    #[must_use]
    pub fn in_the_overview(&self, windows: &Windows) -> Vec<OnTheDock> {
        let mut shown: Vec<OnTheDock> = self
            .pinned
            .iter()
            .map(|entry| match entry {
                ADockEntry::Application(app) => self.one(app, Pinned::ByThePerson, windows),
                ADockEntry::Native(action) => Self::a_native_one(*action),
            })
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
            what: ADockEntry::Application(app.clone()),
            pinned,
            windows: windows.how_many_of(app),
            put_aside: windows.how_many_of_them_put_aside(app),
        }
    }

    /// A native entry's row.
    ///
    /// **Always `ByThePerson` and always no windows.** It is on the Dock only
    /// because somebody put it there, which is what `Pinned::ByThePerson`
    /// means; and the compositor's own surface is not a window this type can
    /// see, so the counts are zero. `belongs_on_the_dock` then keeps it for the
    /// pinned clause alone, which is the correct reason.
    const fn a_native_one(action: ANativeAction) -> OnTheDock {
        OnTheDock {
            what: ADockEntry::Native(action),
            pinned: Pinned::ByThePerson,
            windows: 0,
            put_aside: 0,
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
            .filter_map(|one| one.app().map(|app| app.name().to_owned()))
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

    /// **A native entry is removed exactly as an application is.**
    ///
    /// The owner's ruling of 2026-10-12: *native describes the implementation;
    /// it does not mean permanent.* Law 5 is what gives that teeth — **a change
    /// that takes a choice away from the person is a bug, whatever the reason
    /// given for it** — and the reason that would be given here is a good one:
    /// Settings always works if Settings is always there.
    ///
    /// So this is asserted beside an application's removal, in one test, rather
    /// than in a test of its own. Two tests would let the native one be deleted
    /// as redundant by somebody who had only read the other.
    #[test]
    fn either_kind_can_be_removed_and_a_removal_sticks() {
        let settings = ADockEntry::Native(ANativeAction::Settings);
        let mut holding = Holding::nothing();
        holding.pin(app("Docs"));
        holding.pin_entry(settings.clone());
        holding.pin(app("Browser"));
        assert_eq!(holding.each_pinned().count(), 3, "the premise");

        assert!(
            holding.unpin_entry(&settings),
            "a native entry refused to be removed, so it is permanent and law 5 is broken"
        );
        assert!(
            !holding.is_entry_pinned(&settings),
            "a native entry was removed and is still on the Dock"
        );
        assert!(
            !holding.unpin_entry(&settings),
            "and only once, as for an application"
        );

        // **And the arrangement around it is undisturbed**, which is what makes
        // the order the person's rather than a consequence of the kinds: Docs
        // and Browser were either side of it and are now neighbours.
        let left: Vec<&str> = holding.each_pinned_application().map(AppId::name).collect();
        assert_eq!(left, ["Docs", "Browser"]);

        // The person puts it back, which is the other half of *removable*: at
        // the end, because that is where pinning puts things, and they may move
        // it from there.
        holding.pin_entry(settings.clone());
        assert_eq!(
            holding.each_pinned().last(),
            Some(&settings),
            "a native entry a person restored did not come back"
        );
    }

    /// **A native entry is on the Dock with no windows, and keeps its row.**
    ///
    /// `belongs_on_the_dock` is *pinned, or more windows than are put aside*.
    /// A native row has no windows at all, so it survives on the pinned clause
    /// alone — which is the correct reason and needed no arm of its own. This
    /// test is what says that was checked rather than assumed: the membership
    /// rule's unpinned clause is the one that drops rows, and a native row has
    /// exactly the shape it drops.
    #[test]
    fn a_native_entry_with_no_windows_stays_on_the_dock() {
        let settings = ADockEntry::Native(ANativeAction::Settings);
        let mut holding = Holding::nothing();
        holding.pin_entry(settings.clone());
        let mut windows = Windows::none();
        windows.opened(window(1, "Browser", HowItSits::OnTheCanvas));

        let shown = holding.showing(&windows);

        assert_eq!(
            shown.first().map(OnTheDock::what),
            Some(&settings),
            "the Settings entry is not the first row, or is not there at all"
        );
        let its_row = row(&shown, 0);
        assert_eq!(
            its_row.app(),
            None,
            "a native row answered with an application"
        );
        assert_eq!(its_row.native(), Some(ANativeAction::Settings));
        assert!(
            !its_row.is_open(),
            "a native row claimed to have a window open"
        );
        assert_eq!(its_row.how_many_windows(), 0);
        assert_eq!(its_row.how_many_put_aside(), 0);
        assert!(
            its_row.belongs_on_the_dock(),
            "a native entry the person pinned was dropped by the membership rule"
        );
        // And the application beside it is still an ordinary row, so this
        // cannot pass by every row having become native.
        assert_eq!(row(&shown, 1).app().map(AppId::name), Some("Browser"));
    }

    /// Pinning twice changes nothing and moves nothing.
    #[test]
    fn pinning_something_twice_does_not_move_it() {
        let mut holding = Holding::nothing();
        holding.pin(app("Docs"));
        holding.pin(app("Browser"));
        holding.pin(app("Docs"));

        let pinned: Vec<&str> = holding.each_pinned_application().map(AppId::name).collect();
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

    /// **A pinned application that is running keeps its place and gains an
    /// indicator — it does not gain a second icon.**
    ///
    /// The owner's ruling of 2026-10-10: *a running pinned application stays in
    /// its existing position when launched, with a running indicator rather
    /// than a duplicate icon.* `showing` already did this — the second pass
    /// skips what `is_pinned` answers for — and it is the kind of promise a
    /// refactor breaks silently, because a Dock with two Browsers looks like a
    /// Dock that is working until somebody counts.
    #[test]
    fn launching_a_pinned_application_moves_nothing_and_duplicates_nothing() {
        let mut holding = Holding::nothing();
        holding.pin(app("files"));
        holding.pin(app("browser"));
        holding.pin(app("settings"));

        let closed = Windows::none();
        assert_eq!(
            names(&holding.showing(&closed)),
            ["files", "browser", "settings"],
            "the premise: three pinned applications, none of them running"
        );

        // The middle one starts, with two windows.
        let mut windows = Windows::none();
        windows.opened(window(1, "browser", HowItSits::OnTheCanvas));
        windows.opened(window(2, "browser", HowItSits::OnTheCanvas));
        let shown = holding.showing(&windows);

        assert_eq!(
            names(&shown),
            ["files", "browser", "settings"],
            "launching a pinned application moved an icon or added one"
        );
        assert_eq!(
            shown
                .iter()
                .filter(|one| one.app() == Some(&app("browser")))
                .count(),
            1,
            "the running pinned application has two icons"
        );

        let browser = row(&shown, 1);
        assert_eq!(browser.pinned(), Pinned::ByThePerson);
        assert!(
            browser.is_open(),
            "it is running, which is what a running indicator is drawn from"
        );
        assert_eq!(browser.how_many_windows(), 2);

        // And the two that are not running say so, in the same list.
        assert!(!row(&shown, 0).is_open());
        assert!(!row(&shown, 2).is_open());
    }

    /// **Pinned and running are two answers, and a drawing needs both.**
    ///
    /// The owner's wording is *a running indicator rather than a duplicate
    /// icon*, which is only drawable if the two facts arrive separately. They
    /// do, and this is what says a later change cannot collapse them into one
    /// *state* — which would make *pinned* and *running* mutually exclusive and
    /// leave a running pinned application describable only as one or the other.
    #[test]
    fn a_row_says_whether_it_is_pinned_and_whether_it_is_running_separately() {
        let mut holding = Holding::nothing();
        holding.pin(app("files"));
        let mut windows = Windows::none();
        windows.opened(window(1, "files", HowItSits::OnTheCanvas));
        windows.opened(window(2, "mail", HowItSits::OnTheCanvas));

        let shown = holding.showing(&windows);
        let four = [
            (row(&shown, 0).pinned(), row(&shown, 0).is_open()),
            (row(&shown, 1).pinned(), row(&shown, 1).is_open()),
        ];
        assert_eq!(
            four,
            [(Pinned::ByThePerson, true), (Pinned::No, true)],
            "a pinned running application and an unpinned running one must be told apart"
        );

        // The fourth combination, which only a closed pinned application gives.
        let shown = holding.showing(&Windows::none());
        assert_eq!(
            (row(&shown, 0).pinned(), row(&shown, 0).is_open()),
            (Pinned::ByThePerson, false)
        );
    }

    /// **An application that is only running disappears when it closes; a
    /// pinned one does not.** The owner's clause in its other direction.
    #[test]
    fn an_unpinned_application_leaves_the_dock_when_its_last_window_closes() {
        let mut holding = Holding::nothing();
        holding.pin(app("files"));
        let mut windows = Windows::none();
        windows.opened(window(1, "mail", HowItSits::OnTheCanvas));
        assert_eq!(names(&holding.showing(&windows)), ["files", "mail"]);

        assert_eq!(
            names(&holding.showing(&Windows::none())),
            ["files"],
            "the unpinned application stayed on the Dock after its last window closed, or the \
             pinned one left with it"
        );
    }
}

#[cfg(test)]
#[path = "holding_membership_tests.rs"]
mod membership_tests;
