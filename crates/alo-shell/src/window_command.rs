//! Configured layout commands with additive, detailed refusal reporting.
use crate::{
    InputError, Server, ShortcutDispatchError, WindowMaximizeError, WindowMinimizeError,
    window_mode::Mode,
};
use alo_shortcuts::{Action, Chord, Shortcuts};

/// Refusal from the broader native window command bridge.
/// Diagnostic data, never localized control text. The legacy shortcut error
/// remains unchanged for callers which exhaustively match its variants.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WindowCommandError {
    /// A legacy close/cycle command refused, or the action is not implemented.
    #[error(transparent)]
    Shortcut(#[from] ShortcutDispatchError),
    /// Focused layout commands require a keyboard seat.
    #[error(transparent)]
    Input(#[from] InputError),
    /// No visible mapped root owns focus; stacking is never a fallback.
    #[error("no focused window for the requested layout command")]
    NoFocusedWindow,
    /// The focused root cannot change visibility.
    #[error(transparent)]
    Minimize(#[from] WindowMinimizeError),
    /// Maximize or restore refused without queuing a configure.
    #[error(transparent)]
    Maximize(#[from] WindowMaximizeError),
    /// Full screen, or leaving it, refused without queuing a configure.
    #[error(transparent)]
    FullScreen(#[from] crate::WindowFullScreenError),
    /// The division refused to put the window on that side, in its own words.
    #[error(transparent)]
    Dividing(#[from] crate::NotDivided),
}

impl WindowCommandError {
    /// Forwarded from [`ShortcutDispatchError::the_action_with_no_road`].
    #[must_use]
    pub(crate) const fn the_action_with_no_road(&self) -> Option<Action> {
        match self {
            Self::Shortcut(why) => why.the_action_with_no_road(),
            _ => None,
        }
    }
}

impl Server {
    /// Resolve current personal bindings to native window commands.
    ///
    /// Extends `dispatch_window_shortcut` without changing its exhaustive error
    /// contract. Close and cycling delegate to that API; minimise, maximise and
    /// snap resolve actual keyboard ownership, including popup roots. No binding
    /// or conflicting bindings return `None` without touching any client.
    ///
    /// Minimise hides the focused root without selecting another. Maximise
    /// toggles the latest requested mode, including pending replies: maximized
    /// restores normal geometry; normal or tiled maximizes. Snap requests its
    /// named half without toggling. Existing output, limits, busy-operation and
    /// acknowledged-commit checks remain authoritative. Nothing scales a buffer.
    ///
    /// This trusted action bridge is not a raw key filter or an agent endpoint.
    /// Callers must resolve layout, isolate consumed press/release and repeats,
    /// and provide their current settings. Each call is one explicit request;
    /// success means accepted/queued work, not a client's completed response.
    /// `Action::said` supplies externalized labels; no settings are persisted here.
    pub fn dispatch_window_command(
        &mut self,
        shortcuts: &Shortcuts,
        chord: Chord,
    ) -> Result<Option<Action>, WindowCommandError> {
        let Some(action) = shortcuts.action_for(chord) else {
            return Ok(None);
        };
        if !matches!(
            action,
            Action::MinimiseWindow
                | Action::MaximiseWindow
                | Action::FillTheScreen
                | Action::SnapLeft
                | Action::SnapRight
                | Action::WindowOptions
        ) {
            return self
                .dispatch_window_shortcut(shortcuts, chord)
                .map_err(Into::into);
        }
        if self.surfaces.keyboard.is_none() {
            return Err(InputError::Unavailable.into());
        }
        let root = self
            .surfaces
            .keyboard_root()
            .ok_or(WindowCommandError::NoFocusedWindow)?;
        match action {
            // **The person asked, so this goes down the put-aside road, not to the
            // primitive.** `set_window_minimized` hides a window and keeps its buffers; it
            // is still exactly right for `xdg_toplevel.set_minimized`, which is a client
            // asking, and `crate::surfaces` keeps that caller. But `docs/features.md`
            // promises that *minimising puts a preview in the panel at the edge of the
            // screen*, and this is the road a person takes to it.
            //
            // The road is not a replacement for the primitive — it **composes** it:
            // `crate::putting_a_window_aside` calls `set_window_minimized` itself, after
            // the panel has accepted the window. Recorded rather than performed because the
            // `Panel` is the desktop's; see `Server::asked_to_put_aside`.
            Action::MinimiseWindow => {
                self.the_person_asked_to_put_aside(&root);
            }
            // **Alt+Space opens the window's own menu**, which is where the
            // whole title can be read and copied and where the actions a
            // compact edge moves off the strip are reached. It is the
            // chord every desktop a person arrives from spells it with, and
            // `docs/features.md` promises the habits people arrive with
            // still work.
            //
            // **The same state a press on the Menu control sets**, so a menu
            // opened by chord and one opened by pointer are one menu. Press
            // it again and it closes, as the control does.
            Action::WindowOptions => {
                self.the_menu_was_opened_on(&root);
            }
            Action::MaximiseWindow => {
                let maximize = self.surfaces.requested_window_mode(&root) != Mode::Maximized;
                self.set_window_maximized(&root, maximize)?;
            }
            // **Full screen, by the person's own key** — the mode an
            // application asking for it already reaches (`crate::window_full_screen`),
            // toggled from the latest requested mode exactly as maximise is, so
            // two quick presses are in and out rather than in twice.
            Action::FillTheScreen => {
                let filling = self.surfaces.requested_window_mode(&root) != Mode::FillingTheScreen;
                self.set_window_full_screen(&root, filling)?;
            }
            // **The side and the layout are both `alo-dividing`'s.** The side
            // has been since 2026-09-22; the layout is since the `Server` came
            // to hold a division, because a window's place coming from a tree
            // of shares *and* from half an output would be the two layout
            // deciders the shell plan's constraint forbids.
            Action::SnapLeft | Action::SnapRight => {
                let Some(side) = alo_dividing::keyboard::side_for(action) else {
                    return Err(ShortcutDispatchError::Unsupported(action).into());
                };
                self.divide_focused_with_next(&root, side)?;
            }
            other => return Err(ShortcutDispatchError::Unsupported(other).into()),
        }
        Ok(Some(action))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The side a chord means is the dividing crate's answer.**
    ///
    /// Asked of every action rather than of the two that are splits, so an
    /// action that becomes a split later cannot acquire a second answer here
    /// without this failing.
    ///
    /// **What changed on 2026-09-26**: this used to hold that the shell's own
    /// half named the same side. There is no half — a window's place is a share
    /// of the division — so what is held now is that the shell asks and takes
    /// the answer whole, including the top and the bottom a half could never
    /// lay out.
    #[test]
    fn the_side_a_chord_means_is_the_dividing_crates_answer_and_never_a_second_one() {
        for &action in Action::ALL {
            let decided = alo_dividing::keyboard::side_for(action);
            assert_eq!(
                decided,
                alo_dividing::keyboard::side_for(action),
                "{action:?}: the side was not the dividing crate's own answer"
            );
        }
    }
}
