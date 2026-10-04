//! The chord that opens Settings, which is the sixteenth action's other half.
//!
//! [ADR 0085](../../../docs/decisions/0085-how-a-person-reaches-settings.md)
//! decided that `⊞`+I opens Settings, and `alo_shortcuts::Action::Settings` is
//! what it is **called**. This is what it **does** — the division
//! `alo_shortcuts::action`'s own header draws: *what each action does is the
//! compositor's; what it is called is here.*
//!
//! # This layers the way `crate::canvas_command` does
//!
//! It answers the one action that is Settings' and delegates every other chord
//! onwards, which is the shape `dispatch_canvas_command` already has over
//! `dispatch_window_command`, and that one has over
//! `dispatch_window_shortcut`. A caller presses one key and reaches every
//! action; a second entry point to choose between would be a way of pressing a
//! key that reached only some of them.
//!
//! # Why the window is handed in rather than held here
//!
//! `Server` is the Wayland display, its clients and their surfaces. Settings is
//! none of those: it is a native window this compositor draws itself, and
//! `RecordWindow` — the only other one — is handed to `Nested::pump_record` by
//! whoever owns the frame loop rather than kept in `Server`. Keeping one here
//! would put a person's settings inside the client registry, where every
//! surface walk would have to learn to skip it.
//!
//! # Opening Settings asks nobody for anything
//!
//! [`SettingsWindow::opened_by_hand`] is the road a **person** takes, and it is
//! the only road this file knows. No agent, no grant, no approval and no record
//! entry: a person opening their own settings is not an action anybody needs
//! permission for, and ADR 0009's machine without an agent reaches Settings
//! exactly as this one does. The sections are read from where
//! [`SettingsPlaces`] says they are kept, and a file that is there and does not
//! read comes back in [`SettingsOpened`] rather than being swallowed — which is
//! why that value is inside the answer rather than beside it.

use std::time::SystemTime;

use alo_shortcuts::{Action, Chord, Shortcuts};

use crate::{CanvasCommandError, Server, SettingsOpened, SettingsPlaces, SettingsWindow};

/// What one chord did, when it did anything.
///
/// Settings' own answer carries [`SettingsOpened`], so a caller cannot act on
/// *Settings opened* without being handed what did not read while it opened.
/// Every other action is the layer below's and arrives as itself.
#[derive(Debug)]
pub enum WhatTheChordDid {
    /// Settings opened, and these files are there and did not read.
    SettingsOpened(SettingsOpened),
    /// An action one of the layers below answered.
    AnotherAction(Action),
}

impl WhatTheChordDid {
    /// Which action it was, whichever layer answered it.
    #[must_use]
    pub const fn action(&self) -> Action {
        match self {
            Self::SettingsOpened(_) => Action::Settings,
            Self::AnotherAction(action) => *action,
        }
    }
}

/// A chord that could not do what it names.
///
/// Settings itself has no refusal: a window a person opens by hand needs no
/// seat, no focused window and no output, so the only way this call fails is
/// the way the layers below already fail.
#[derive(Debug, thiserror::Error)]
pub enum SettingsCommandError {
    /// Any chord that is not Settings', answered by the layer below.
    #[error(transparent)]
    Canvas(#[from] CanvasCommandError),
}

impl Server {
    /// **Resolve one chord against the person's bindings, including Settings'.**
    ///
    /// This is the one entry point a caller needs: Settings is answered here and
    /// every other chord goes to [`Server::dispatch_canvas_command`] and onwards.
    /// No binding, or bindings that conflict, answer `Ok(None)` without opening
    /// anything and without touching a client.
    ///
    /// **Opening an open window opens it again**, which is
    /// `SettingsWindow::opened_by_hand`'s own rule and the right one: the chord
    /// means *show me my settings as they are*, and a person who presses it
    /// twice has asked for the second reading rather than for the window to
    /// close. Closing is `SettingsWindow::close`, which a key inside the window
    /// reaches and this chord never does.
    ///
    /// # Errors
    /// [`SettingsCommandError::Canvas`] for every refusal the layers below make.
    /// Settings makes none.
    pub fn dispatch_settings_command(
        &mut self,
        shortcuts: &Shortcuts,
        chord: Chord,
        window: &mut SettingsWindow,
        places: &SettingsPlaces,
        now: SystemTime,
    ) -> Result<Option<WhatTheChordDid>, SettingsCommandError> {
        let Some(action) = shortcuts.action_for(chord) else {
            return Ok(None);
        };
        if action == Action::Settings {
            return Ok(Some(WhatTheChordDid::SettingsOpened(
                window.opened_by_hand(places, now),
            )));
        }
        Ok(self
            .dispatch_canvas_command(shortcuts, chord, now)?
            .map(WhatTheChordDid::AnotherAction))
    }
}
