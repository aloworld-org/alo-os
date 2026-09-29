//! Every window open on this machine, in the order they were last used.
//!
//! **The order is the whole point.** *Click an app to return to where you last
//! used it* is only answerable if something remembers which window that was, and
//! this is the something. Most recently used first, across every application, so
//! that *the most recent window of the Browser* is a question with one answer
//! rather than a guess.
//!
//! # One order, not one per application
//!
//! A single list, asked per application when the Dock needs it. Keeping a list
//! per application would mean the same fact written down twice — which of an
//! app's windows is newest, and which of all windows is newest — and two places
//! for it to be wrong. Asking the one list for an app's windows is a filter, and
//! a filter cannot disagree with what it filters.
//!
//! # A window put aside keeps its place in the order
//!
//! Putting a window away is not using it and not forgetting it. Its place in the
//! order is where it was, because *the most recently used window* is a fact about
//! when somebody last worked in it, and the Dock's rule then has an answer for
//! the row the owner's table calls *most recent window minimized*.
//!
//! # What this does not do, and whose it is
//!
//! **It does not survive a sign-out**, and it should. `leaving.toml` restores
//! what was open, and a use-order that is not restored with it means the first
//! click on every application the next morning goes somewhere arbitrary — which
//! breaks the rule on exactly the day a person most relies on it. That belongs
//! to whoever owns `leaving.toml` rather than to a crate about a dock, and it is
//! written down in `docs/design/what-a-click-on-the-dock-does.md` as owed rather
//! than fixed here.

use crate::window::{AppId, Window, WindowId};

/// Every window open on this machine, most recently used first.
///
/// **Two orders, because there are two questions.** Which window a click goes to
/// is *which did you use last*; where an application's icon sits is *when did it
/// open*, and that one must not move when somebody uses something
/// ([`crate::holding`]). One order cannot answer both, and a Dock built on the
/// use-order rearranges itself under the person's hand — which a test caught
/// doing exactly that before this second order existed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Windows {
    /// Newest use first. A window appears exactly once.
    in_use_order: Vec<Window>,
    /// The order they opened in, oldest first. The same windows, by number.
    in_open_order: Vec<WindowId>,
}

impl Windows {
    /// No windows open at all, which is a machine at sign-in.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// How many are open.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.in_use_order.len()
    }

    /// Whether none are.
    #[must_use]
    pub fn are_none(&self) -> bool {
        self.in_use_order.is_empty()
    }

    /// A window has opened, which counts as using it.
    ///
    /// **Replaces one already numbered the same**, so the list holds a window
    /// once however many times it is announced. A compositor re-announcing a
    /// window would otherwise put a second copy of it in the order, and the
    /// older copy would be an answer nobody could reach.
    pub fn opened(&mut self, window: Window) {
        let id = window.id();
        self.in_use_order.retain(|held| held.id() != id);
        self.in_use_order.insert(0, window);
        // Keeps the place it first took, so re-announcing a window does not
        // move its application's icon.
        if !self.in_open_order.contains(&id) {
            self.in_open_order.push(id);
        }
    }

    /// This window was used, so it goes to the front of the order.
    ///
    /// Says whether there was such a window. A window nothing knows about is not
    /// an error here: the compositor is the one that knows what is open, and a
    /// race between it and this list should not take the shell down.
    pub fn used(&mut self, id: WindowId) -> bool {
        let Some(at) = self.in_use_order.iter().position(|held| held.id() == id) else {
            return false;
        };
        let window = self.in_use_order.remove(at);
        self.in_use_order.insert(0, window);
        true
    }

    /// This window has closed. Says whether there was one.
    pub fn closed(&mut self, id: WindowId) -> bool {
        let before = self.in_use_order.len();
        self.in_use_order.retain(|held| held.id() != id);
        self.in_open_order.retain(|held| *held != id);
        self.in_use_order.len() != before
    }

    /// Every window, in the order they opened, oldest first.
    ///
    /// **What the Dock's order is made of.** Where an icon sits is a fact a
    /// person learns once, so it follows when the application arrived and never
    /// what they have been using.
    pub fn each_as_opened(&self) -> impl Iterator<Item = &Window> {
        self.in_open_order.iter().filter_map(|id| self.window(*id))
    }

    /// This window, as it stands.
    #[must_use]
    pub fn window(&self, id: WindowId) -> Option<&Window> {
        self.in_use_order.iter().find(|held| held.id() == id)
    }

    /// Put this window in the state it is now in, wherever it sits in the order.
    ///
    /// **Putting a window aside is not using it**, so this does not move it to
    /// the front. Says whether there was such a window.
    pub fn now_sits(&mut self, id: WindowId, sits: crate::window::HowItSits) -> bool {
        let Some(held) = self.in_use_order.iter_mut().find(|held| held.id() == id) else {
            return false;
        };
        *held = held.clone().now_sits(sits);
        true
    }

    /// Every window, most recently used first.
    pub fn each(&self) -> impl Iterator<Item = &Window> {
        self.in_use_order.iter()
    }

    /// This application's windows, most recently used first.
    ///
    /// This is what the Dock's hover list is made of, and the order is why the
    /// first row in it is the one a plain click would have gone to.
    ///
    /// The name is copied rather than borrowed for the life of the iterator, so
    /// that a caller can ask about an application it has just named without
    /// having to keep that name alive alongside the answer.
    pub fn of_the_app<'a>(&'a self, app: &AppId) -> impl Iterator<Item = &'a Window> + 'a {
        let app = app.clone();
        self.in_use_order
            .iter()
            .filter(move |held| held.app() == &app)
    }

    /// The window of this application the person used last.
    ///
    /// **The answer a plain click goes to.**
    #[must_use]
    pub fn most_recent_of(&self, app: &AppId) -> Option<&Window> {
        self.of_the_app(app).next()
    }

    /// How many windows this application has open.
    #[must_use]
    pub fn how_many_of(&self, app: &AppId) -> usize {
        self.of_the_app(app).count()
    }

    /// How many of this application's windows have been put aside.
    ///
    /// A screen reader announces this — *three windows open, one minimised* —
    /// so it is counted here rather than by whoever writes the sentence.
    #[must_use]
    pub fn how_many_of_them_put_aside(&self, app: &AppId) -> usize {
        self.of_the_app(app)
            .filter(|held| held.sits().was_put_aside())
            .count()
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
    use crate::window::HowItSits;

    fn app(name: &str) -> AppId {
        AppId::named(name).unwrap()
    }

    fn window(number: u64, of: &str, called: &str) -> Window {
        Window::of(
            WindowId::numbered(number),
            app(of),
            called,
            Patch::of(Spot::at(0, 0), 800, 600).unwrap(),
            HowItSits::OnTheCanvas,
        )
    }

    /// A desk: three Browser windows and one Blender, opened in that order.
    fn a_desk() -> Windows {
        let mut windows = Windows::none();
        windows.opened(window(1, "Browser", "Launch website"));
        windows.opened(window(2, "Browser", "Research"));
        windows.opened(window(3, "Blender", "Opening scene"));
        windows.opened(window(4, "Browser", "Preview"));
        windows
    }

    /// The newest is first and the list reads back in the order things happened.
    #[test]
    fn the_most_recently_opened_is_the_most_recent() {
        let windows = a_desk();
        assert_eq!(windows.how_many(), 4);
        let order: Vec<u64> = windows.each().map(|held| held.id().number()).collect();
        assert_eq!(order, [4, 3, 2, 1]);
        assert_eq!(
            windows.most_recent_of(&app("Browser")).unwrap().called(),
            "Preview"
        );
    }

    /// **Using a window moves it to the front**, which is the fact the whole
    /// click rule rests on.
    #[test]
    fn using_a_window_makes_it_the_most_recent() {
        let mut windows = a_desk();
        assert!(windows.used(WindowId::numbered(1)));
        assert_eq!(
            windows.most_recent_of(&app("Browser")).unwrap().called(),
            "Launch website"
        );
        assert_eq!(
            windows.each().next().unwrap().id(),
            WindowId::numbered(1),
            "and of everything, not only of its own app"
        );
        assert_eq!(windows.how_many(), 4, "nothing was added or lost");
    }

    /// **The order they opened in does not move when one is used**, which is the
    /// fact the Dock's stable icon positions rest on.
    ///
    /// A test in `crate::holding` caught this being wrong — the Dock was built
    /// from the use-order and rearranged itself when somebody looked at a
    /// window. This is the same statement asked of the order directly, so it is
    /// held in the file that owns it rather than only where it was noticed.
    #[test]
    fn the_order_they_opened_in_is_not_the_order_they_were_used() {
        let mut windows = a_desk();
        let opened: Vec<u64> = windows
            .each_as_opened()
            .map(|held| held.id().number())
            .collect();
        assert_eq!(opened, [1, 2, 3, 4], "oldest first");

        windows.used(WindowId::numbered(1));
        windows.now_sits(WindowId::numbered(3), HowItSits::PutAside);

        let after: Vec<u64> = windows
            .each_as_opened()
            .map(|held| held.id().number())
            .collect();
        assert_eq!(after, [1, 2, 3, 4], "and using or putting aside moved none");

        let used: Vec<u64> = windows.each().map(|held| held.id().number()).collect();
        assert_eq!(used, [1, 4, 3, 2], "while the use-order did move");
    }

    /// Closing takes a window out of both orders, so neither keeps a window
    /// nothing can reach.
    #[test]
    fn closing_takes_it_out_of_both_orders() {
        let mut windows = a_desk();
        assert!(windows.closed(WindowId::numbered(2)));

        let opened: Vec<u64> = windows
            .each_as_opened()
            .map(|held| held.id().number())
            .collect();
        assert_eq!(opened, [1, 3, 4]);
        assert_eq!(windows.each().count(), 3);
    }

    /// A window announced twice keeps the place it first took, so its
    /// application's icon does not move because a compositor said so again.
    #[test]
    fn a_window_announced_twice_keeps_the_place_it_first_opened_in() {
        let mut windows = a_desk();
        windows.opened(window(2, "Browser", "Research, again"));
        let opened: Vec<u64> = windows
            .each_as_opened()
            .map(|held| held.id().number())
            .collect();
        assert_eq!(opened, [1, 2, 3, 4]);
    }

    /// A window nothing knows about is answered rather than panicked over: the
    /// compositor is what knows, and a race with it must not take the shell
    /// down.
    #[test]
    fn using_or_closing_a_window_nobody_has_is_answered_not_fatal() {
        let mut windows = a_desk();
        assert!(!windows.used(WindowId::numbered(99)));
        assert!(!windows.closed(WindowId::numbered(99)));
        assert!(!windows.now_sits(WindowId::numbered(99), HowItSits::PutAside));
        assert_eq!(windows.how_many(), 4);
    }

    /// **A window appears once**, however many times it is announced, so there
    /// is never an older copy of it that nothing can reach.
    #[test]
    fn a_window_announced_twice_is_still_one_window() {
        let mut windows = a_desk();
        windows.opened(window(2, "Browser", "Research, again"));
        assert_eq!(windows.how_many(), 4);
        assert_eq!(
            windows.window(WindowId::numbered(2)).unwrap().called(),
            "Research, again"
        );
        assert_eq!(windows.each().next().unwrap().id(), WindowId::numbered(2));
    }

    /// **Putting a window aside is not using it.** It keeps its place in the
    /// order, so *the most recent window* can be one that was put away — which
    /// is the row the owner's table calls *most recent window minimized*.
    #[test]
    fn putting_a_window_aside_does_not_change_the_order() {
        let mut windows = a_desk();
        let before: Vec<u64> = windows.each().map(|held| held.id().number()).collect();

        assert!(windows.now_sits(WindowId::numbered(4), HowItSits::PutAside));

        let after: Vec<u64> = windows.each().map(|held| held.id().number()).collect();
        assert_eq!(before, after, "the order is untouched");
        let most_recent = windows.most_recent_of(&app("Browser")).unwrap();
        assert_eq!(most_recent.called(), "Preview");
        assert!(
            most_recent.sits().was_put_aside(),
            "and the most recent one is the one that was put away"
        );
    }

    /// An application's windows come back in the same order as the whole list,
    /// filtered — which is what stops the two ever disagreeing.
    #[test]
    fn an_applications_windows_are_the_one_order_filtered() {
        let windows = a_desk();
        let browsers: Vec<&str> = windows
            .of_the_app(&app("Browser"))
            .map(Window::called)
            .collect();
        assert_eq!(browsers, ["Preview", "Research", "Launch website"]);
        assert_eq!(windows.how_many_of(&app("Browser")), 3);
        assert_eq!(windows.how_many_of(&app("Blender")), 1);
        assert_eq!(windows.how_many_of(&app("Nothing")), 0);
    }

    /// What a screen reader is given — *three windows open, one minimised* —
    /// is counted here rather than by whoever writes the sentence.
    #[test]
    fn how_many_are_open_and_how_many_were_put_aside_are_both_counted() {
        let mut windows = a_desk();
        assert_eq!(windows.how_many_of_them_put_aside(&app("Browser")), 0);

        windows.now_sits(WindowId::numbered(2), HowItSits::PutAside);
        assert_eq!(windows.how_many_of(&app("Browser")), 3);
        assert_eq!(windows.how_many_of_them_put_aside(&app("Browser")), 1);
    }

    /// Closing takes it out of the order, and the next one along becomes the
    /// answer a click would go to.
    #[test]
    fn closing_a_window_hands_the_answer_to_the_next_one() {
        let mut windows = a_desk();
        assert!(windows.closed(WindowId::numbered(4)));
        assert_eq!(windows.how_many_of(&app("Browser")), 2);
        assert_eq!(
            windows.most_recent_of(&app("Browser")).unwrap().called(),
            "Research"
        );
    }

    /// A machine at sign-in has none, and says so plainly.
    #[test]
    fn a_machine_with_nothing_open_says_so() {
        let windows = Windows::none();
        assert!(windows.are_none());
        assert_eq!(windows.how_many(), 0);
        assert_eq!(windows.most_recent_of(&app("Browser")), None);
    }
}
