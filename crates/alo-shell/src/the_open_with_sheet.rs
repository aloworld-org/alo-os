//! **The *Open with* sheet**: an application asked for the person to choose
//! what opens a link, and the desktop asks them — frame `25:52`, approved by
//! the owner on 2026-11-12 (`docs/design/default-apps-and-open-with.md`).
//!
//! The portal asks through the door's `choose-link` act and holds the
//! connection (`docs/contracts/application-door.md`); the door hands the
//! connection here (`AskedToChoose`) and this answers it when the person has.
//!
//! # Behaviour, from the frame's own note (`25:79`)
//!
//! - **Focus starts on the person's default**, which is the first offered.
//! - **Arrows choose, Enter opens, Escape cancels** and nothing opens.
//! - **Nothing here changes the default**: that stays in Settings → Default
//!   apps. What is answered is for this one link.
//! - Chosen or cancelled, the portal writes it in the record.
//!
//! # One at a time, and never for ever
//!
//! One sheet is up; up to `AS_MANY_AS_WAIT` more wait behind it, each with
//! its connection held, and one more than that is answered `cannot-show`. A
//! sheet nobody answers in `THE_LONGEST` is taken down and answered
//! `unanswered` — not `cancelled`, because nobody said no. A portal that hangs
//! up takes its sheet down with nothing answered.
//!
//! # It decides nothing
//!
//! What is offered is the portal's, from the same answers Settings → Default
//! apps draws, and the answer is one of those or none. The sheet opens
//! nothing: the portal records the choice and asks `open-link` for it.

use std::collections::VecDeque;
use std::io::{ErrorKind, Read as _, Write as _};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use alo_acting::{Chose, ToChooseALink};
use alo_applications::{Application, Link};

/// The longest a sheet stays up unanswered.
pub(crate) const THE_LONGEST: Duration = Duration::from_secs(5 * 60);

/// How many sheets may wait behind the one that is up.
pub(crate) const AS_MANY_AS_WAIT: usize = 4;

/// The portal asking the person to choose, with its connection held.
#[derive(Debug)]
pub struct AskedToChoose {
    /// What it asked.
    asked: ToChooseALink,
    /// Its connection, not blocking.
    talking: UnixStream,
}

impl AskedToChoose {
    /// The door read `asked` on `talking`.
    pub(crate) const fn of(asked: ToChooseALink, talking: UnixStream) -> Self {
        Self { asked, talking }
    }

    /// Answer it and let it go. A portal that has gone is not told.
    fn answered(mut self, chose: &Chose) {
        let _ = self.talking.write_all(chose.written().as_bytes());
        let _ = self.talking.write_all(b"\n");
        let _ = self.talking.flush();
    }

    /// Whether the portal has hung up. Anything it says after its line is not
    /// part of this act and is let go.
    fn has_hung_up(&mut self) -> bool {
        let mut said = [0_u8; 64];
        loop {
            match self.talking.read(&mut said) {
                Ok(0) => return true,
                Ok(_) => {}
                Err(why) if why.kind() == ErrorKind::WouldBlock => return false,
                Err(why) if why.kind() == ErrorKind::Interrupted => {}
                Err(_) => return true,
            }
        }
    }
}

/// A key, as the sheet understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SheetKey {
    /// Up or Left: the application before.
    Previous,
    /// Down or Right: the application after.
    Next,
    /// Home: the first.
    First,
    /// End: the last.
    Last,
    /// Enter or Space: open the link in the one chosen.
    Open,
    /// Escape: cancel, and nothing opens.
    Cancel,
    /// Anything else, which does nothing.
    Nothing,
}

impl SheetKey {
    /// What a key means here, read as Settings reads it — one reading of a
    /// key for every surface of this window's family.
    pub(crate) const fn of(key: crate::SettingsKey) -> Self {
        use crate::SettingsKey as K;
        match key {
            K::Previous | K::Less => Self::Previous,
            K::Next | K::More => Self::Next,
            K::First => Self::First,
            K::Last => Self::Last,
            K::Choose => Self::Open,
            K::Close => Self::Cancel,
            K::NextSection
            | K::PreviousSection
            | K::PutBackThisOne
            | K::PutBackAsShipped
            | K::Nothing => Self::Nothing,
        }
    }
}

/// The sheet that is up.
#[derive(Debug)]
struct Shown {
    /// What was asked, and where to answer.
    asked: AskedToChoose,
    /// Which application the keyboard is on.
    focus: usize,
    /// When it went up.
    since: Instant,
    /// Which sheet this is, counted from the first to go up: what a screen
    /// reader's press is held to, so a press heard on one sheet never answers
    /// the next.
    number: u64,
}

/// What the sheet shows, for drawing it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SheetShows<'a> {
    /// The application that asked.
    pub(crate) asking: &'a Application,
    /// The link.
    pub(crate) link: &'a Link,
    /// What is offered, the default first.
    pub(crate) offered: &'a [Application],
    /// Which one the keyboard is on.
    pub(crate) focus: usize,
    /// Which sheet this is (`Shown::number`).
    pub(crate) number: u64,
}

/// Every *Open with* sheet the desktop holds: the one up, and those waiting.
#[derive(Debug, Default)]
pub(crate) struct TheOpenWithSheet {
    /// The one up.
    shown: Option<Shown>,
    /// Those waiting behind it, oldest first.
    waiting: VecDeque<AskedToChoose>,
    /// How many sheets have gone up.
    went_up: u64,
}

impl TheOpenWithSheet {
    /// No sheet.
    pub(crate) fn none() -> Self {
        Self::default()
    }

    /// Whether a sheet is up, so every key is its.
    pub(crate) const fn is_up(&self) -> bool {
        self.shown.is_some()
    }

    /// What the sheet that is up shows.
    pub(crate) fn shows(&self) -> Option<SheetShows<'_>> {
        self.shown.as_ref().map(|shown| SheetShows {
            asking: shown.asked.asked.asking(),
            link: shown.asked.asked.link(),
            offered: shown.asked.asked.offered(),
            focus: shown.focus,
            number: shown.number,
        })
    }

    /// The portal asked these. `can_show` is whether this desktop can ask the
    /// person now; where it cannot, each is answered `cannot-show` at once.
    pub(crate) fn asked(&mut self, asks: Vec<AskedToChoose>, can_show: bool, now: Instant) {
        for ask in asks {
            if !can_show || self.waiting.len() >= AS_MANY_AS_WAIT {
                ask.answered(&Chose::CannotShow);
                continue;
            }
            self.waiting.push_back(ask);
            self.the_next_goes_up(now);
        }
    }

    /// Once a frame: a portal that hung up takes its sheet down, a sheet up
    /// for `THE_LONGEST` is answered `unanswered`, and the next goes up.
    pub(crate) fn waited(&mut self, now: Instant) {
        self.waiting.retain_mut(|ask| !ask.has_hung_up());
        if let Some(shown) = self.shown.as_mut() {
            if shown.asked.has_hung_up() {
                self.shown = None;
            } else if now.saturating_duration_since(shown.since) >= THE_LONGEST
                && let Some(shown) = self.shown.take()
            {
                shown.asked.answered(&Chose::Unanswered);
            }
        }
        self.the_next_goes_up(now);
    }

    /// A key while the sheet is up.
    pub(crate) fn pressed(&mut self, key: SheetKey, now: Instant) {
        let Some(shown) = self.shown.as_mut() else {
            return;
        };
        let last = shown.asked.asked.offered().len().saturating_sub(1);
        match key {
            SheetKey::Previous => shown.focus = shown.focus.saturating_sub(1),
            SheetKey::Next => shown.focus = (shown.focus + 1).min(last),
            SheetKey::First => shown.focus = 0,
            SheetKey::Last => shown.focus = last,
            SheetKey::Open => self.open_in(None, now),
            SheetKey::Cancel => self.cancelled(now),
            SheetKey::Nothing => {}
        }
    }

    /// The person pressed a row: the keyboard goes to it.
    pub(crate) fn pointed_at(&mut self, row: usize) {
        if let Some(shown) = self.shown.as_mut()
            && row < shown.asked.asked.offered().len()
        {
            shown.focus = row;
        }
    }

    /// *Open*: the link opens in `row`, or in the one the keyboard is on.
    pub(crate) fn open_in(&mut self, row: Option<usize>, now: Instant) {
        let Some(shown) = self.shown.take() else {
            return;
        };
        let at = row.unwrap_or(shown.focus);
        match shown.asked.asked.offered().get(at).cloned() {
            Some(chosen) => shown.asked.answered(&Chose::Chosen(chosen)),
            // Never reached: the focus is kept inside the offer. Answered as
            // no choice rather than as some other application.
            None => shown.asked.answered(&Chose::Cancelled),
        }
        self.the_next_goes_up(now);
    }

    /// *Cancel*, or Escape: nothing opens.
    pub(crate) fn cancelled(&mut self, now: Instant) {
        if let Some(shown) = self.shown.take() {
            shown.asked.answered(&Chose::Cancelled);
        }
        self.the_next_goes_up(now);
    }

    /// **A screen reader pressed `on`, on the sheet numbered `sheet`** — the
    /// same door a click takes: an application's row moves the keyboard to
    /// it, *Open* opens in the one the keyboard is on, *Cancel* cancels.
    ///
    /// **Held to the sheet it was heard on.** The tree a reader presses is a
    /// moment old, and a press heard on a sheet that has since been answered
    /// must not answer the next one — so a press for any other sheet, or for
    /// a row the sheet does not have, does nothing and says so with `false`.
    pub(crate) fn a_reader_pressed(
        &mut self,
        sheet: u64,
        on: crate::OnTheOpenWithSheet,
        now: Instant,
    ) -> bool {
        let Some(shown) = self.shown.as_ref() else {
            return false;
        };
        if shown.number != sheet {
            return false;
        }
        match on {
            crate::OnTheOpenWithSheet::TheApplication(row) => {
                if row >= shown.asked.asked.offered().len() {
                    return false;
                }
                self.pointed_at(row);
            }
            crate::OnTheOpenWithSheet::Open => self.open_in(None, now),
            crate::OnTheOpenWithSheet::Cancel => self.cancelled(now),
        }
        true
    }

    /// The desktop could not draw the sheet on this display: its question is
    /// answered `cannot-show` rather than every key held with nothing on the
    /// screen, the password sheet's rule.
    pub(crate) fn could_not_be_drawn(&mut self, now: Instant) {
        if let Some(shown) = self.shown.take() {
            shown.asked.answered(&Chose::CannotShow);
        }
        self.the_next_goes_up(now);
    }

    /// The oldest waiting goes up, when none is.
    fn the_next_goes_up(&mut self, now: Instant) {
        if self.shown.is_some() {
            return;
        }
        if let Some(asked) = self.waiting.pop_front() {
            self.went_up = self.went_up.wrapping_add(1);
            // **Focus starts on the person's default**, the first offered.
            self.shown = Some(Shown {
                asked,
                focus: 0,
                since: now,
                number: self.went_up,
            });
        }
    }
}

#[cfg(test)]
#[path = "the_open_with_sheet_tests.rs"]
mod tests;
