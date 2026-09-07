//! Reactive configure scheduling, separate from committed popup lifetimes.

use super::{Popups, WlSurface, XdgPopupSurfaceData, with_states};

impl Popups {
    /// Recompute against committed ancestors, never speculative parent configures.
    pub(crate) fn refresh(&mut self, roots: &[WlSurface]) {
        for index in 0..self.entries.len() {
            let Some(entry) = self.entries.get(index) else {
                break;
            };
            if !entry.buffered || entry.dismissed || !entry.role.alive() {
                continue;
            }
            // Smithay requires committed reactive permission for send_configure.
            // A pending explicit reposition may withdraw it: honor both states,
            // and defer newly granted permission until that request is committed.
            let reactive = with_states(&entry.popup.surface, |states| {
                states
                    .data_map
                    .get::<XdgPopupSurfaceData>()
                    .is_some_and(|data| {
                        data.lock()
                            .unwrap_or_else(|_| std::process::abort())
                            .current
                            .positioner
                            .reactive
                    })
            });
            if !reactive {
                continue;
            }
            // The latest server state includes outstanding explicit and automatic
            // configures. Comparing against it prevents a configure on every frame
            // while a slow client has not acknowledged the previous geometry yet.
            let pending = entry.role.with_pending_state(|state| *state);
            if !pending.positioner.reactive {
                continue;
            }
            let Some(geometry) = self.placement(pending.positioner, &entry.popup.parent, roots)
            else {
                let surface = entry.popup.surface.clone();
                self.dismiss_tree(&surface);
                continue;
            };
            if geometry == pending.geometry {
                continue;
            }
            entry
                .role
                .with_pending_state(|state| state.geometry = geometry);
            if entry.role.send_configure().is_err() {
                let surface = entry.popup.surface.clone();
                self.dismiss_tree(&surface);
            }
        }
    }
}
