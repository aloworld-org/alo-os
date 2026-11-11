//! **alo in a window**: the window a person offered with *Help with this
//! window*, the request that carries it, and the window each task works in.
//!
//! Two promises meet here and are one surface
//! (`docs/design/the-window-controls.md`, `11:146`; `docs/features.md`, *The
//! agent's presence*):
//!
//! 1. **Help with this window** is asked in the window's own panel: its alo
//!    row is a field, *alo · Ask about this window…*, and **reaching it asks
//!    nothing** (`crate::the_window_panels_field`; the owner, 2026-11-11: *"the
//!    user should be able to type in here not to open another window"*). The
//!    words are held with the window whose panel it is
//!    (`AloInAWindow::the_field`) while the person types; what travels to
//!    alo — the window's title and its application's identifier — is read
//!    **at the moment they send it**, and never before or after (`CLAUDE.md`,
//!    *context is offered, never watched*). Escape, the panel closing, AI
//!    switched off, or the window closing all let the words go, and nothing
//!    is asked.
//! 2. **The window alo is working in wears its presence** for the turn, on no
//!    other window, and is bare again when the turn ends. A task asked about a
//!    window remembers which one (`AloInAWindow::works_in`); what it is
//!    doing now is the task's own reading
//!    (`alo_tasks::ATask::what_is_happening`), so the window's band and the
//!    task window can never disagree about whether alo is working.
//!
//! # What this never does
//!
//! - **It grants nothing.** Offering a window is a hint for the model
//!   (`alo_instructing::TheWindowOffered`); every verb is still decided by
//!   `alo-agentd` at the person's own level, full access acting without the OK
//!   (ADR 0149) and every other level asking as it always did.
//! - **It reads nothing in the window.** A title and an identifier, which the
//!   band already shows, at one moment the person chose.
//! - **It never stops a turn on its own.** A window that closes mid-turn takes
//!   its presence with it and leaves the work to the task window, where the
//!   person can still see it and stop it; stopping is the person's.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use alo_instructing::TheWindowOffered;

use crate::the_tasks_a_person_opened::WhichTask;
use crate::the_window_panels_field::TheWindowField;

/// One request a person sent about a window, waiting to become a task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskedAbout {
    /// What they asked, in their own words.
    pub request: String,
    /// The window as it was when they sent it.
    pub window: TheWindowOffered,
    /// Which window it is, so the task can wear it.
    pub surface: WlSurface,
}

/// alo and the windows it is offered and works in.
#[derive(Debug, Default)]
pub struct AloInAWindow {
    /// What is typed into a window panel's field, and whose panel it is, while
    /// that panel is open. Nothing has been asked.
    the_field: Option<(WlSurface, TheWindowField)>,
    /// Requests sent about a window, waiting for the frame loop.
    asked_about: Vec<AskedAbout>,
    /// Which window each task was asked about, oldest first.
    works_in: Vec<(WhichTask, WlSurface)>,
    /// Whether alo may be asked now: something is chosen to answer, and AI is
    /// not set to *Not at all*. Told by the frame; `false` until it is.
    may_be_asked: bool,
}

/// How alo is in a window now, which is all that changes how its band looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowAloIsHere {
    /// alo is acting: deep teal, with its mark and its word, and Stop.
    Working,
    /// alo has asked the person something and is not acting: navy, as the task
    /// window draws it, and Stop — which lets the question lapse unanswered.
    WaitingForThePerson,
    /// Stop was pressed and the work has not yet ended: still teal, because
    /// alo has not gone, and Stop greyed so it cannot be pressed twice.
    Stopping,
}

/// What a window's band shows while alo is in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AloHere {
    /// The task working in it.
    pub which: WhichTask,
    /// How it is.
    pub how: HowAloIsHere,
    /// The task's status, in its own words — *alo is working*.
    pub status: alo_strings::Word,
    /// What the person asked, which is the plain-language description of the
    /// work: their own words, never a model's summary of them.
    pub what: String,
}

impl AloInAWindow {
    /// What is typed into this window's panel field, if anything has been.
    #[must_use]
    pub fn the_field_of(&self, surface: &WlSurface) -> Option<&TheWindowField> {
        self.the_field
            .as_ref()
            .filter(|(its, _)| its == surface)
            .map(|(_, field)| field)
    }

    /// This window's panel field, to type in: a fresh one where another
    /// window's was held, since one panel is open at a time.
    pub(crate) fn the_field_to_type_in(&mut self, surface: &WlSurface) -> &mut TheWindowField {
        if self
            .the_field
            .as_ref()
            .is_none_or(|(its, _)| its != surface)
        {
            self.the_field = None;
        }
        let (_, field) = self
            .the_field
            .get_or_insert_with(|| (surface.clone(), TheWindowField::default()));
        field
    }

    /// Whether alo may be asked now.
    #[must_use]
    pub const fn may_be_asked(&self) -> bool {
        self.may_be_asked
    }

    /// Which window this task was asked about, if it was asked about one.
    #[must_use]
    pub fn the_window_of(&self, which: WhichTask) -> Option<&WlSurface> {
        self.works_in
            .iter()
            .find(|(task, _)| *task == which)
            .map(|(_, surface)| surface)
    }

    /// The tasks asked about this window, newest first.
    fn tasks_in(&self, surface: &WlSurface) -> impl Iterator<Item = WhichTask> + '_ {
        let surface = surface.clone();
        self.works_in
            .iter()
            .rev()
            .filter(move |(_, its)| *its == surface)
            .map(|(which, _)| *which)
    }

    /// Let go of everything about windows that are gone, and of words typed
    /// into a panel that has closed or about which alo may no longer be asked.
    fn pruned(&mut self, alive: &[WlSurface], panel_open: Option<&WlSurface>) {
        let keep = self.the_field.as_ref().is_some_and(|(its, _)| {
            self.may_be_asked && alive.contains(its) && panel_open == Some(its)
        });
        if !keep {
            self.the_field = None;
        }
        self.works_in.retain(|(_, surface)| alive.contains(surface));
    }
}

/// How a task is, read from its own status word, or [`None`] once its work is
/// over — complete or stopped — when no window wears it.
fn how_it_is(status: alo_strings::Word) -> Option<HowAloIsHere> {
    if status == alo_tasks::words::ALO_IS_WORKING {
        Some(HowAloIsHere::Working)
    } else if status == alo_tasks::words::WAITING_FOR_YOUR_PERMISSION {
        Some(HowAloIsHere::WaitingForThePerson)
    } else if status == alo_tasks::words::STOPPING {
        Some(HowAloIsHere::Stopping)
    } else {
        None
    }
}

impl crate::Server {
    /// **The frame says whether alo may be asked**, and everything about
    /// windows that are gone is let go.
    ///
    /// Asked of the desktop once a frame — the same answer that decides whether
    /// the launcher offers to ask alo — because whether anything answers is a
    /// reading of the person's settings, which this crate never makes.
    pub fn alo_may_be_asked(&mut self, may: bool) {
        self.alo_in_a_window.may_be_asked = may;
        let alive: Vec<WlSurface> = self.mapped_surfaces().cloned().collect();
        let open = self.whose_panel_is_open();
        self.alo_in_a_window.pruned(&alive, open.as_ref());
    }

    /// **Help with this window, chosen**: this window's panel opens, if it is
    /// not open, with the keyboard in its field, and nothing is asked. Answers
    /// whether the field has the keyboard: never while alo may not be asked,
    /// which is also why the field is not in the panel then, and never for a
    /// window that is not on the screen.
    ///
    /// What a click on the field does; the arrows reach it by walking the
    /// panel, which is the same place by the other road.
    pub fn help_with_this_window(&mut self, surface: &WlSurface) -> bool {
        if !self.alo_in_a_window.may_be_asked {
            return false;
        }
        if !self.mapped_surfaces().any(|it| it == surface) {
            return false;
        }
        // **The keyboard goes to the field**, so the person types their
        // request at once — and away from a task window, which would otherwise
        // be asked for every key first.
        self.the_task_gave_the_keyboard_up();
        self.the_keyboard_goes_to_the_window_field(surface);
        true
    }

    /// What is typed in this window's panel field, and the caret, for
    /// drawing it. [`None`] until something is typed.
    #[must_use]
    pub fn what_is_typed_in_the_window_field(
        &self,
        surface: &WlSurface,
    ) -> Option<crate::window_field_paint::TheFieldAsTyped<'_>> {
        self.alo_in_a_window
            .the_field_of(surface)
            .map(|field| crate::window_field_paint::TheFieldAsTyped {
                typed: field.typed(),
                caret: field.caret(),
            })
    }

    /// **The person sent a request about this window**: read the window now,
    /// at the moment they sent it, and record the asking.
    ///
    /// Answers whether anything was asked. **Nothing is** when the request is
    /// empty, or the window has gone or names nothing — and then the person's
    /// words stay where they typed them, to send again or not.
    pub(crate) fn a_person_asked_alo_about_the_window(
        &mut self,
        surface: &WlSurface,
        request: &str,
    ) -> bool {
        let request = request.trim();
        // **Asked first, and nothing read from a window that has gone.**
        if request.is_empty() || !self.mapped_surfaces().any(|it| it == surface) {
            return false;
        }
        let title = self
            .the_name_of(surface)
            .its_own_words()
            .map(str::to_owned)
            .unwrap_or_default();
        let application = self.the_app_id_of(surface).unwrap_or_default();
        let Some(window) = TheWindowOffered::of(&title, &application) else {
            return false;
        };
        self.alo_in_a_window.asked_about.push(AskedAbout {
            request: request.to_owned(),
            window,
            surface: surface.clone(),
        });
        true
    }

    /// Take the requests sent about a window, waiting to become tasks.
    ///
    /// **Taken rather than read**, for `Server::take_what_was_asked_of_alo`'s
    /// reason: one Enter, one task, one request.
    pub fn take_what_was_asked_about_a_window(&mut self) -> Vec<AskedAbout> {
        std::mem::take(&mut self.alo_in_a_window.asked_about)
    }

    /// **This task works in this window**, which it now wears while it works.
    pub fn alo_works_in(&mut self, which: WhichTask, surface: WlSurface) {
        self.alo_in_a_window.works_in.push((which, surface));
    }

    /// The window a task was asked about, as it reads now, for a follow-up
    /// typed in that task's window: read at the moment the person sends it, and
    /// [`None`] once the window has gone.
    #[must_use]
    pub fn the_window_a_task_is_about(&self, which: WhichTask) -> Option<TheWindowOffered> {
        let surface = self.alo_in_a_window.the_window_of(which)?;
        if !self.mapped_surfaces().any(|it| it == surface) {
            return None;
        }
        let title = self
            .the_name_of(surface)
            .its_own_words()
            .map(str::to_owned)
            .unwrap_or_default();
        TheWindowOffered::of(&title, &self.the_app_id_of(surface).unwrap_or_default())
    }

    /// **What this window's band shows of alo**, or [`None`] when alo is not
    /// at work in it.
    ///
    /// The newest task asked about this window whose work is not over: working,
    /// waiting on the person, or stopping. Nothing for any other window, and
    /// nothing once the work has ended, complete or stopped.
    #[must_use]
    pub fn alo_in_this_window(&self, surface: &WlSurface) -> Option<AloHere> {
        self.alo_in_a_window.tasks_in(surface).find_map(|which| {
            if !self.the_tasks.is_open(which) {
                return None;
            }
            let task = self.the_tasks.the(which)?;
            let status = task.what_is_happening();
            let how = how_it_is(status)?;
            let what = task
                .conversation()
                .the_request()
                .map(|turn| turn.said().to_owned())
                .unwrap_or_default();
            Some(AloHere {
                which,
                how,
                status,
                what,
            })
        })
    }
}

#[cfg(test)]
#[path = "alo_in_a_window_tests.rs"]
mod tests;
