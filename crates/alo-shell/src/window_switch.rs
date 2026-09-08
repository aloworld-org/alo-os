//! Stable window cycling, independent of presentation stacking.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::{InputError, Server, WindowActivationError};

/// Direction through the session's first-observed mapping order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowSwitchDirection {
    /// Select the following root, wrapping to the first.
    Forward,
    /// Select the preceding root, wrapping to the last.
    Backward,
}

/// Refusal or partial activation failure from trusted window cycling.
#[derive(Debug, thiserror::Error)]
pub enum WindowSwitchError {
    /// No live mapped toplevel can be selected; focus is unchanged.
    #[error("no mapped window to switch to")]
    Empty,
    /// Activation failed; preserves the activation error's partial-change detail.
    #[error(transparent)]
    Activation(#[from] WindowActivationError),
}

/// Session-owned ring; raising never edits it and retired roots are discarded.
#[derive(Default)]
pub(crate) struct SwitchOrder(Vec<WlSurface>);

impl SwitchOrder {
    /// Retain survivors and append new mappings in their observed stacking order.
    pub(crate) fn refresh<'a>(&mut self, mapped: impl Iterator<Item = &'a WlSurface>) {
        let mapped: Vec<_> = mapped.cloned().collect();
        self.0.retain(|root| mapped.contains(root));
        for root in mapped {
            if !self.0.contains(&root) {
                self.0.push(root);
            }
        }
    }

    /// Find a neighbor without changing either the ring or focus.
    fn select(
        &self,
        focus: Option<&WlSurface>,
        direction: WindowSwitchDirection,
    ) -> Option<WlSurface> {
        let position = focus.and_then(|focus| self.0.iter().position(|root| root == focus));
        let selected = match (direction, position) {
            (WindowSwitchDirection::Forward, Some(index)) => {
                self.0.get(index + 1).or_else(|| self.0.first())
            }
            (WindowSwitchDirection::Backward, Some(index)) => index
                .checked_sub(1)
                .and_then(|index| self.0.get(index))
                .or_else(|| self.0.last()),
            (WindowSwitchDirection::Forward, None) => self.0.first(),
            (WindowSwitchDirection::Backward, None) => self.0.last(),
        };
        selected.cloned()
    }
}

impl Server {
    /// Activate the next or previous visible window from trusted shell controls.
    ///
    /// The ring follows first observation at dispatch boundaries, not raising or
    /// activation history. New mappings append; observed unmap/disconnect removes
    /// them. Remapping after removal appends anew. No client work is dispatched
    /// inside this operation. Minimized roots are skipped without losing their
    /// original ring positions. With no focused root, forward selects the first
    /// visible root and backward the last. A sole visible root selects itself,
    /// preserving its popup grab.
    /// Popup focus counts as its owning root; switching away dismisses the grab
    /// and releases held keys through `activate_window` before recipient transfer.
    ///
    /// Missing keyboards refuse before changing state, even on an empty display.
    /// Empty displays refuse without changing focus. Success queues activation and
    /// raises the selected root; it does not prove client rendering. This exposes
    /// no agent endpoint, application grouping, shortcut binding or visual picker.
    pub fn switch_window(
        &mut self,
        direction: WindowSwitchDirection,
    ) -> Result<WlSurface, WindowSwitchError> {
        if self.surfaces.keyboard.is_none() {
            return Err(WindowActivationError::Input(InputError::Unavailable).into());
        }
        self.switch_order.refresh(self.surfaces.buffered());
        let visible = SwitchOrder(
            self.switch_order
                .0
                .iter()
                .filter(|root| self.surfaces.mapped_toplevel(root).is_some())
                .cloned()
                .collect(),
        );
        let root = visible
            .select(self.surfaces.keyboard_root().as_ref(), direction)
            .ok_or(WindowSwitchError::Empty)?;
        self.activate_window(&root)?;
        Ok(root)
    }
}
