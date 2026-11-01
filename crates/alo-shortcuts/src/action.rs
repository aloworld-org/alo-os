//! What a shortcut does, and the closed list of what it can do.
//!
//! An [`Action`] is a thing the system itself does — summon the agent, close a
//! window, move to the next application. It is not a thing an *application*
//! does: `Ctrl+B` is bold in a document because the document received the keys,
//! and nothing in this crate knows or cares. The distinction is the whole reason
//! this list is short. Every action here costs a chord that no application on
//! the machine can ever see again, and a system that took thirty of them would
//! be a system whose applications behave strangely for reasons nobody can find.
//!
//! **The list is what `docs/features.md` promises at v0.01** and nothing else:
//! the agent overlay, the launcher, the window management a person does with the
//! keyboard, and switching between windows and applications. Dividing the screen
//! from the keyboard is v0.5 and is not here; screenshots are v0.5 and are not
//! here. An action arrives with the feature it belongs to, because a shortcut
//! bound to something that does not happen yet is a bug report waiting to be
//! filed.
//!
//! What each action *does* is the compositor's, which does not exist yet. What
//! it is *called* is here, because a person rebinding a shortcut is choosing
//! from this list and needs to read it.
//!
//! **In the language they read.** An [`Action`] has no `Display`: the only road
//! to words is [`Action::said`], which takes the strings this machine reads and
//! answers with a `Said` that says whether anybody translated it. A `Display`
//! would be an English row one `to_string()` away from a settings panel whose
//! author had no reason to think about it, and *hardcoded English is a bug*
//! rather than a preference. What the code holds instead is [`Action::word`] —
//! the declaration in [`crate::words`], which is the sentence a translator is
//! handed.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words::{self, Word};

/// Something the system does when a chord is pressed.
///
/// The names are a stored format — a settings file holds them beside the chord
/// a person chose — so they change additively or not at all. Removing one is
/// removing a feature, and a person's binding for it goes with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Bring up the agent over whatever is in front of you.
    TheAgent,
    /// Open the alo Bar — the field on the Dock whose prompt reads *Find,
    /// open, or ask…*.
    ///
    /// **It reached nothing from the day it was written until 2026-10-20**,
    /// and this doc said so: `Super`+`Space` was bound, and *which surface the
    /// launcher is* had never been decided. Two could have been meant.
    ///
    /// - The Dock's **More apps** overview is built, has a search field and a
    ///   grid, and is specified to hold applications **pinned or currently
    ///   open** — `docs/features.md` says in as many words that applications
    ///   *merely installed* are not there. A launcher that cannot start
    ///   something you have not already started is not one, so it was never
    ///   this.
    /// - **The alo Bar** — *ask, find, open, create or hand over, from one
    ///   place*, which `docs/features.md` carries as a starred differentiator.
    ///
    /// **The owner settled it on 2026-10-20**, in their own words: *it is
    /// already on the dock, we then need to add to open when the user clicks
    /// windows + space*. The Bar's field is on the Dock, it draws, and it has
    /// taken typing since the composer was built — what was missing was the
    /// chord reaching it, which `alo_shell::a_chord_opens_the_bar` now is.
    ///
    /// **What is still owed, and a reader should not assume otherwise:** the
    /// Bar opens and what is typed reaches the agent. Typing an application's
    /// name does **not** open that application yet. That is the *finding* half
    /// of the same promise and is the next change.
    Launcher,
    /// Open a terminal.
    ///
    /// **The only road to it, and that is why it exists.** `docs/features.md`
    /// promises *a terminal* — law 2's own words, *an operating system that does
    /// not trust its owner with a shell is a toy* — and `alo-software`'s
    /// `shipped.toml` installs one at first boot with `role = "terminal"`.
    /// Nothing could open it. It is not among the three a fresh machine pins
    /// (Files, Browser, Settings — the owner's own list, so a fourth would
    /// contradict it), an unpinned application with no window is not on the
    /// Dock at all, and the Dock's *More apps* overview is specified to hold
    /// pinned and **currently open** applications rather than merely installed
    /// ones. So an installed terminal was unreachable by pointer and by
    /// keyboard alike, which made that promise one with nothing behind it.
    ///
    /// **This makes an existing promise true rather than adding one**, which is
    /// the judgement `CLAUDE.md` asks a change to state: the terminal is
    /// promised, keyboard shortcuts are promised, and what was missing was a
    /// road between them.
    Terminal,
    /// Ask alo why the command in front of a terminal is waiting.
    ///
    /// **The only road to an answer that was already built, which is why it
    /// exists.** `docs/features.md` promises *a command looks stuck; **ask**, and
    /// get **waiting for the keyboard**, **for the network**, **for another
    /// process or a lock**, **still computing**, or **unknown with the evidence
    /// available***. The five answers are built, measured against real processes
    /// on a real kernel, each with a control; all four things to do about them
    /// are built. **Nothing could ask.** The promise's own word was *ask*, and
    /// there was no asking.
    ///
    /// **This makes an existing promise true rather than adding one**, which is
    /// the judgement `CLAUDE.md` asks a change to state: the answers are
    /// promised, keyboard shortcuts are promised, and what was missing was a
    /// road between them. It is the same judgement
    /// [`Self::Terminal`] was added under, for the same shape of gap.
    ///
    /// **Which key is an implementation and not a promise.** `docs/features.md`
    /// says *ask* and names no chord; the design file is silent; a person can
    /// rebind or clear this in Settings like any other. What is promised is that
    /// asking works.
    WhyIsItWaiting,
    /// Open Settings.
    ///
    /// **The road that needs no pointer**, which is why it is a shortcut as well
    /// as a place on the Dock: ADR 0085 takes both, because *a person who works
    /// by keyboard and a person who has never been told a chord exists are
    /// different people*.
    Settings,
    /// See what alo did: the record, opened by hand.
    ///
    /// **The road a person reaches the record by**, which nothing on a running
    /// machine offered until 2026-10-27: `alo_shell::RecordWindow` was built,
    /// routed and tested, and no key, row or Dock item opened it, so Demo 1
    /// stopped at *open the record*. The owner chose this key and a row in
    /// Settings that day.
    ///
    /// **This makes an existing promise true rather than adding one**:
    /// `docs/features.md` promises the record and *afterwards, ask what it
    /// did*, and ADR 0009 requires a plain road beside the agent's. It needs no
    /// model and no agent, so it works on a machine where neither was set up.
    SeeWhatAloDid,
    /// Show what can be done to the window in front.
    ///
    /// **The road a person reaches the window's own options by**, and the one
    /// the owner's ruling of 2026-10-10 makes necessary: the window menu is
    /// where the full title is selectable, which is the only place a person can
    /// read a long one slowly, copy it, or reach it by touch where there is no
    /// hover at all.
    WindowOptions,
    /// Close the window in front.
    CloseWindow,
    /// Put the window in front out of the way.
    MinimiseWindow,
    /// Make the window in front fill the screen, or put it back.
    MaximiseWindow,
    /// Make the window in front fill the whole screen, with the Dock and the
    /// panel giving way to it, or put it back.
    ///
    /// **Promised and missing until 2026-11-01.** The design gives *Enter or
    /// leave full screen* `⊞`+Enter, and full screen itself was built on
    /// 2026-09-30 for an application that asks for it — so a person could not
    /// ask for it from the keyboard. Maximise is not this: a maximised window
    /// keeps its own title bar and the Dock stays. This makes an existing
    /// promise true.
    FillTheScreen,
    /// Snap the window in front to the left half of the screen.
    SnapLeft,
    /// Snap the window in front to the right half of the screen.
    SnapRight,
    /// Move to the next window.
    NextWindow,
    /// Move to the previous window.
    PreviousWindow,
    /// Move to the next application.
    NextApplication,
    /// Move to the previous application.
    PreviousApplication,
    /// Zoom the canvas in.
    ZoomTheCanvasIn,
    /// Zoom the canvas out.
    ZoomTheCanvasOut,
    /// Show every window on the canvas at once.
    ShowAllOnTheCanvas,
    /// Put the canvas back to 100 %, its standard scale.
    ///
    /// **Promised and missing until 2026-11-01**, when a booted machine was
    /// pressed `⊞`+1 and nothing happened, 15 and 30 seconds later.
    /// `docs/design/the-shortcuts-and-the-edges.md` puts *Return the canvas to
    /// 100%* among the four canvas shortcuts to teach first, and
    /// `docs/design/the-canvas-as-a-workspace.md` §4 names *100%* among the
    /// essential zoom commands. [`Self::ZoomTheCanvasIn`] and
    /// [`Self::ZoomTheCanvasOut`] had a road; this one had none.
    ///
    /// **This makes an existing promise true rather than adding one**: the
    /// command and its key are the design's, and a person can rebind or clear
    /// it in Settings like every other action here.
    ReturnTheCanvasToLifeSize,
    /// Send the window in front to the next Place.
    ///
    /// **The road a person who cannot drag takes**, which is why it is a
    /// shortcut rather than only a gesture:
    /// `docs/autonomy/the-canvas-and-its-places.md` task 3 says *neither road is
    /// the only road*. The pointer road drags a frame out through the World; this
    /// one needs no pointer, no World and no aim.
    MoveTheWindowToTheNextPlace,
    /// Put this Place back as it was a moment ago.
    ///
    /// **One step back, not a named moment**, and the distinction is what keeps
    /// this inside the current release. `docs/features.md` holds two promises
    /// here at two tiers: *A Place remembers time* is `[v0.01]` and **the time
    /// ribbon** — the strip at the bottom edge that fades the desktop into the
    /// past with what changed glowing — is `[v1.1]`. Naming a moment needs that
    /// strip to name it in. Stepping back needs no surface at all, and walks the
    /// same series the ribbon will one day draw.
    ///
    /// So this is the road the capability has until the ribbon exists, exactly
    /// as [`Self::MoveTheWindowToTheNextPlace`] is the road for somebody who
    /// cannot drag: `docs/autonomy/the-canvas-and-its-places.md` says *neither
    /// road is the only road*.
    GoBackOnThisPlace,
    /// Step forward again: undo the last [`Self::GoBackOnThisPlace`].
    ///
    /// **Redo, on the design's `⊞`+Shift+Z**, beside going back on `⊞`+Z —
    /// `docs/design/the-shortcuts-and-the-edges.md` gives the two as *undo a
    /// canvas action* and *redo it*. Missing until 2026-11-01, when a person
    /// who stepped back twice could not step forward again without moving a
    /// window by hand. This makes an existing promise true.
    GoForwardOnThisPlace,
    /// Move the keyboard on to the next part of the screen.
    ///
    /// The canvas, the Dock, the panel of put-aside windows, notifications and
    /// the status area, in that order, and back to the canvas.
    /// `docs/design/the-shortcuts-and-the-edges.md` settles the route and the
    /// key: **F6**, which is what a landmark cycle is bound to on every system
    /// a person may be arriving from, so somebody who already knows it does not
    /// have to learn ours.
    ///
    /// **Without it the rest of this product's keyboard work is unreachable.**
    /// The Dock's surfaces take the arrows, Enter and Escape, and until this
    /// existed a person had to lift a hand to a pointer to reach them.
    TheNextArea,
    /// Move the keyboard back to the part of the screen before this one.
    ///
    /// Shift+F6. **Its own action rather than a modifier on the one above**,
    /// because a person rebinding one may want the other somewhere else — and
    /// because a settings panel has to be able to show them as two rows with
    /// two chords, which is what `Self::ALL` is for.
    ThePreviousArea,
}

impl Action {
    /// Everything a shortcut can do, in the order a settings panel lists them:
    /// the six that summon something, then the seven done to the window in
    /// front, then the four for moving between windows and applications, then
    /// the canvas's seven, then the two that move the keyboard between the
    /// parts of the screen.
    ///
    /// **The sentence above counts, and it had stopped.** It read *the three
    /// that summon something, then the window in front, then moving between
    /// windows* while four canvas actions sat under it unnamed — true of the
    /// list it was written for and quietly incomplete for every one added
    /// since. Corrected in the change that added the fifth canvas action rather
    /// than left for the sixth, because a counting sentence nobody recounts is
    /// worse than no sentence: it reads as checked.
    pub const ALL: &'static [Self] = &[
        Self::TheAgent,
        Self::Launcher,
        Self::Settings,
        Self::SeeWhatAloDid,
        Self::Terminal,
        Self::WhyIsItWaiting,
        Self::CloseWindow,
        Self::WindowOptions,
        Self::MinimiseWindow,
        Self::MaximiseWindow,
        Self::FillTheScreen,
        Self::SnapLeft,
        Self::SnapRight,
        Self::NextWindow,
        Self::PreviousWindow,
        Self::NextApplication,
        Self::PreviousApplication,
        Self::ZoomTheCanvasIn,
        Self::ZoomTheCanvasOut,
        Self::ShowAllOnTheCanvas,
        Self::ReturnTheCanvasToLifeSize,
        Self::MoveTheWindowToTheNextPlace,
        Self::GoBackOnThisPlace,
        Self::GoForwardOnThisPlace,
        Self::TheNextArea,
        Self::ThePreviousArea,
    ];

    /// The string this crate declares for it: the key a translator's file is
    /// sorted by, and the English beside it.
    #[must_use]
    pub fn word(self) -> Word {
        match self {
            Self::TheAgent => words::THE_AGENT,
            Self::Launcher => words::LAUNCHER,
            Self::Settings => words::SETTINGS,
            Self::SeeWhatAloDid => words::SEE_WHAT_ALO_DID,
            Self::Terminal => words::TERMINAL,
            Self::WhyIsItWaiting => words::WHY_IS_IT_WAITING,
            Self::CloseWindow => words::CLOSE_WINDOW,
            Self::WindowOptions => words::WINDOW_OPTIONS,
            Self::MinimiseWindow => words::MINIMISE_WINDOW,
            Self::MaximiseWindow => words::MAXIMISE_WINDOW,
            Self::FillTheScreen => words::FILL_THE_SCREEN,
            Self::SnapLeft => words::SNAP_LEFT,
            Self::SnapRight => words::SNAP_RIGHT,
            Self::NextWindow => words::NEXT_WINDOW,
            Self::PreviousWindow => words::PREVIOUS_WINDOW,
            Self::NextApplication => words::NEXT_APPLICATION,
            Self::PreviousApplication => words::PREVIOUS_APPLICATION,
            Self::ZoomTheCanvasIn => words::ZOOM_THE_CANVAS_IN,
            Self::ZoomTheCanvasOut => words::ZOOM_THE_CANVAS_OUT,
            Self::ShowAllOnTheCanvas => words::SHOW_ALL_ON_THE_CANVAS,
            Self::ReturnTheCanvasToLifeSize => words::RETURN_THE_CANVAS_TO_LIFE_SIZE,
            Self::MoveTheWindowToTheNextPlace => words::MOVE_THE_WINDOW_TO_THE_NEXT_PLACE,
            Self::GoBackOnThisPlace => words::GO_BACK_ON_THIS_PLACE,
            Self::GoForwardOnThisPlace => words::GO_FORWARD_ON_THIS_PLACE,
            Self::TheNextArea => words::THE_NEXT_AREA,
            Self::ThePreviousArea => words::THE_PREVIOUS_AREA,
        }
    }

    /// What this does, in the language the person reads — the row a settings
    /// panel draws.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not:
    /// there is always something to put on the screen, and what there was to
    /// say about where it came from is on the [`Said`]. A `Strings` that was
    /// never given [`crate::shortcut_words`] answers with the key, marked, and
    /// `Said::is_a_bug` — which is the honest answer to *the shell forgot to
    /// declare what this crate can say*.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use std::collections::BTreeSet;

    /// The list a settings panel walks holds every action once. An action
    /// missing from it is an action a person cannot rebind, which is the
    /// feature this crate exists for going quietly missing.
    #[test]
    fn the_list_holds_each_action_once() {
        let unique: BTreeSet<Action> = Action::ALL.iter().copied().collect();
        assert_eq!(unique.len(), Action::ALL.len());
        assert!(Action::ALL.contains(&Action::TheAgent));
    }

    /// Every action says what it does, and no two of them say the same thing —
    /// two rows reading "Next window" would be a list nobody could set from.
    #[test]
    fn every_action_says_what_it_does_and_no_two_say_the_same() {
        let strings = in_english();
        let mut said = BTreeSet::new();
        for action in Action::ALL {
            let row = action.said(&strings);
            assert!(!row.text().is_empty(), "{action:?}");
            assert!(!row.is_a_bug(), "{action:?} is not declared");
            assert!(
                said.insert(row.text().to_owned()),
                "two actions are both {row}"
            );
            assert_eq!(row.text(), action.word().says());
        }
    }

    /// **A row a person reads is the translation when there is one**, and says
    /// so — which is the whole of what moving this crate onto `alo-strings`
    /// bought.
    #[test]
    fn a_row_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(words::SNAP_LEFT, "Fenster auf die linke Hälfte legen")]);
        let said = Action::SnapLeft.said(&strings);
        assert_eq!(said.text(), "Fenster auf die linke Hälfte legen");
        assert!(said.is_translated());

        // And the one nobody translated is still English, and says it is.
        let untranslated = Action::SnapRight.said(&strings);
        assert_eq!(untranslated.text(), "Put the window on the right half");
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug());
    }

    /// A settings file holds the name, so a person's binding survives a release
    /// that adds an action above it in the list.
    #[test]
    fn a_settings_file_holds_the_name() {
        assert_eq!(
            serde_json::to_string(&Action::SnapLeft).unwrap(),
            r#""SnapLeft""#
        );
        assert_eq!(
            serde_json::from_str::<Action>(r#""TheAgent""#).unwrap(),
            Action::TheAgent
        );
        assert!(serde_json::from_str::<Action>(r#""RunAnything""#).is_err());
    }
}
