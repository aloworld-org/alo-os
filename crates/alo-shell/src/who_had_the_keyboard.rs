//! **Who had the person's keyboard, most recent first** — the one question
//! `front` asks the compositor: *which window was the person working in just
//! before this terminal?* (`docs/contracts/terminal-commands.md`, *front*).
//!
//! Until this file the compositor kept only who has the keyboard **now**
//! (`dock_windows.rs` said so: *nothing in this shell records when a person last
//! used a window*). So it is recorded at the two places the keyboard changes
//! hands: [`crate::surfaces::Surfaces::set_keyboard_focus`], for a window, and
//! `Server::the_person_went_to`, for alo's own terminal, which the compositor
//! draws itself and which has no Wayland surface to be focused.
//!
//! **Bounded and kept nowhere.** It holds the last [`AT_MOST`] changes, in
//! memory, for this session; it is never written to a disk and never sent
//! anywhere, and a window that closes drops out of it. It is asked only when a
//! person's own command asks `front` at the compositor's door.
//!
//! Generic over what a window is, so the order is tested without a display.

/// How many changes of hands are remembered.
pub(crate) const AT_MOST: usize = 32;

/// One holder of the keyboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HadTheKeyboard<W> {
    /// An application's window.
    Window(W),
    /// alo's own terminal, which the compositor draws itself.
    AlosOwnTerminal,
}

/// Who had the keyboard, most recent first, each once.
#[derive(Debug)]
pub(crate) struct WhoHadTheKeyboard<W> {
    /// Most recent first, with no holder twice.
    latest_first: Vec<HadTheKeyboard<W>>,
}

impl<W> Default for WhoHadTheKeyboard<W> {
    fn default() -> Self {
        Self {
            latest_first: Vec::new(),
        }
    }
}

impl<W: PartialEq> WhoHadTheKeyboard<W> {
    /// `who` has the keyboard now. Its older place in the list goes, so each
    /// holder is in it once, at the last time it held it.
    pub(crate) fn now_has(&mut self, who: HadTheKeyboard<W>) {
        self.latest_first.retain(|had| *had != who);
        self.latest_first.insert(0, who);
        self.latest_first.truncate(AT_MOST);
    }

    /// Forget every window `gone` says has closed.
    pub(crate) fn forget(&mut self, gone: impl Fn(&W) -> bool) {
        self.latest_first.retain(|had| match had {
            HadTheKeyboard::Window(window) => !gone(window),
            HadTheKeyboard::AlosOwnTerminal => true,
        });
    }

    /// The most recent holder that is not `this` — **never another chosen
    /// instead**: a window older than that one is not an answer to *the one
    /// before*, and [`None`] means there is none.
    pub(crate) fn latest_other_than(&self, this: &HadTheKeyboard<W>) -> Option<&HadTheKeyboard<W>> {
        self.latest_first.iter().find(|had| *had != this)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The order a person moved in, most recent first, each window once.
    #[test]
    fn the_window_before_the_terminal_is_the_last_one_the_person_was_in() {
        let mut who = WhoHadTheKeyboard::default();
        who.now_has(HadTheKeyboard::Window(1));
        who.now_has(HadTheKeyboard::Window(2));
        who.now_has(HadTheKeyboard::Window(1));
        who.now_has(HadTheKeyboard::AlosOwnTerminal);
        assert_eq!(
            who.latest_other_than(&HadTheKeyboard::AlosOwnTerminal),
            Some(&HadTheKeyboard::Window(1))
        );
        // A terminal window: the one before it is alo's own terminal.
        who.now_has(HadTheKeyboard::Window(9));
        assert_eq!(
            who.latest_other_than(&HadTheKeyboard::Window(9)),
            Some(&HadTheKeyboard::AlosOwnTerminal)
        );
    }

    /// **A window that closed is never answered, and nothing older stands in
    /// for it** — the caller asked which window, and the honest answer for a
    /// closed one is not the one before it. The forgetting is the caller's to
    /// do before it asks; what is asked after is only what is still open.
    #[test]
    fn a_closed_window_drops_out() {
        let mut who = WhoHadTheKeyboard::default();
        who.now_has(HadTheKeyboard::Window(1));
        who.now_has(HadTheKeyboard::Window(2));
        who.now_has(HadTheKeyboard::AlosOwnTerminal);
        who.forget(|window| *window == 2);
        assert_eq!(
            who.latest_other_than(&HadTheKeyboard::AlosOwnTerminal),
            Some(&HadTheKeyboard::Window(1))
        );
        who.forget(|_| true);
        assert_eq!(who.latest_other_than(&HadTheKeyboard::AlosOwnTerminal), None);
    }

    /// **Nothing before the terminal is none**, not the terminal itself.
    #[test]
    fn nothing_before_is_none() {
        let mut who: WhoHadTheKeyboard<u32> = WhoHadTheKeyboard::default();
        assert_eq!(who.latest_other_than(&HadTheKeyboard::AlosOwnTerminal), None);
        who.now_has(HadTheKeyboard::AlosOwnTerminal);
        assert_eq!(who.latest_other_than(&HadTheKeyboard::AlosOwnTerminal), None);
    }

    /// Bounded, so a long session costs a fixed amount.
    #[test]
    fn it_holds_at_most_its_bound() {
        let mut who = WhoHadTheKeyboard::default();
        for window in 0..(AT_MOST * 3) {
            who.now_has(HadTheKeyboard::Window(window));
        }
        assert_eq!(who.latest_first.len(), AT_MOST);
        assert_eq!(who.latest_first.first(), Some(&HadTheKeyboard::Window(AT_MOST * 3 - 1)));
    }
}
