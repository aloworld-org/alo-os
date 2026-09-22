//! Configured layout commands with additive, detailed refusal reporting.
use crate::{
    InputError, Server, ShortcutDispatchError, TileSide, WindowMaximizeError, WindowMinimizeError,
    WindowModeError, window_mode::Mode,
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
    /// Half-output tiling refused without queuing a configure.
    #[error(transparent)]
    Tile(#[from] WindowModeError),
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
            Action::MinimiseWindow | Action::MaximiseWindow | Action::SnapLeft | Action::SnapRight
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
            Action::MinimiseWindow => {
                self.set_window_minimized(&root, true)?;
            }
            Action::MaximiseWindow => {
                let maximize = self.surfaces.requested_window_mode(&root) != Mode::Maximized;
                self.set_window_maximized(&root, maximize)?;
            }
            // **Which side a chord means is `alo-dividing`'s answer, not this
            // file's.** It was decided here until 2026-09-22 — a second place
            // the same two actions became a side, which is the shape the shell
            // plan's constraint forbids. `window_tiling` stays the *mechanism*
            // that turns a side into half an output; what it no longer does is
            // decide which side, and a chord that maps to a side this mechanism
            // cannot lay out is refused rather than approximated.
            Action::SnapLeft | Action::SnapRight => {
                let Some(side) = tile_side_for(action) else {
                    return Err(ShortcutDispatchError::Unsupported(action).into());
                };
                self.set_window_tiled(&root, Some(side))?;
            }
            other => return Err(ShortcutDispatchError::Unsupported(other).into()),
        }
        Ok(Some(action))
    }
}

/// Which half of an output a chord puts the focused window on.
///
/// **The side is `alo-dividing`'s answer**, and this turns it into the half
/// `crate::window_tiling` can lay out. It was decided here until 2026-09-22 — a
/// second place the same two actions became a side, which is the shape the
/// shell plan's constraint forbids.
///
/// A side this mechanism has no layout for is [`None`] rather than something
/// near what was asked: halves of an output are left and right, and a chord
/// meaning a top or a bottom is refused.
fn tile_side_for(action: Action) -> Option<TileSide> {
    match alo_dividing::keyboard::side_for(action)? {
        alo_dividing::Side::Left => Some(TileSide::Left),
        alo_dividing::Side::Right => Some(TileSide::Right),
        alo_dividing::Side::Top | alo_dividing::Side::Bottom => None,
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected pairing is the failure being reported"
)]
mod tests {
    use super::*;

    /// **There is one place a chord becomes a side, and it is not this one.**
    ///
    /// Asked of every action rather than of the two that are splits, so an
    /// action that becomes a split later cannot acquire a second answer here
    /// without this failing. The shell may refuse a side it cannot lay out; what
    /// it may not do is name a different one from the crate that decides.
    #[test]
    fn the_side_a_chord_means_is_the_dividing_crates_answer_and_never_a_second_one() {
        for &action in Action::ALL {
            let decided = alo_dividing::keyboard::side_for(action);
            match (decided, tile_side_for(action)) {
                (Some(alo_dividing::Side::Left), Some(TileSide::Left))
                | (Some(alo_dividing::Side::Right), Some(TileSide::Right)) => {}
                // A side with no half to lay it out in is refused, which is a
                // narrowing of the decision and never a different one.
                (Some(alo_dividing::Side::Top | alo_dividing::Side::Bottom), None) => {}
                // Not a split at all, on either side of the question.
                (None, None) => {}
                (decided, took) => panic!(
                    "{action:?}: the dividing crate says {decided:?} and this shell took {took:?}"
                ),
            }
        }
    }
}
