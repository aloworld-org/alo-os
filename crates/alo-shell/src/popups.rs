//! Opt-in popup protocol lifetimes, independent of presentation and grabs.

#[path = "popup_reactive.rs"]
mod reactive;

use smithay::{
    backend::renderer::utils::with_renderer_surface_state,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Physical, Rectangle, Size},
    wayland::{
        compositor::with_states,
        shell::xdg::{PopupSurface, PositionerState, XdgPopupSurfaceData},
    },
};

/// Configured popup buffer offered to a popup-aware backend.
///
/// Coordinates are relative to the parent's XDG window geometry, not its buffer.
/// This snapshot is compositor plumbing and exposes no agent context access.
#[derive(Clone)]
pub struct Popup {
    /// Popup surface tree root.
    pub surface: WlSurface,
    /// Mapped toplevel or popup parent.
    pub parent: WlSurface,
    /// Committed position and requested size in parent window coordinates.
    pub geometry: Rectangle<i32, Logical>,
}

/// Protocol ownership includes dismissed roles until destruction, preventing revival.
struct Entry {
    /// Smithay owns configure serial validation.
    role: PopupSurface,
    /// Committed placement and immutable parent.
    popup: Popup,
    /// A configured buffer has committed.
    buffered: bool,
    /// Dismissal is terminal even if the client commits another buffer.
    dismissed: bool,
}

/// Explicit opt-in keeps existing renderers from accepting invisible popups.
#[derive(Default)]
pub(crate) struct Popups {
    /// Enabled only by a backend prepared to consume popup snapshots.
    enabled: bool,
    /// Live protocol roles and terminal dismissal state.
    entries: Vec<Entry>,
    /// Last positive framebuffer extent, at compositor scale one.
    pub(crate) output_size: Option<Size<i32, Physical>>,
}

impl crate::Server {
    /// Enable popup handshake tracking for a popup-aware backend or protocol fixture.
    ///
    /// `Nested` renders these snapshots and pointer routing consumes them.
    /// Pointer and keyboard press/release grabs and explicit repositioning are
    /// supported. Initial and explicit placement use the last positive extent
    /// supplied to `render` and client-authorized constraints. Before an output exists,
    /// protocol fixtures use unconstrained placement. Mapped reactive popups are
    /// reconstrained after output or committed parent changes; scene geometry still
    /// changes only on an acknowledged commit. No agent authority is introduced.
    pub fn enable_popup_protocol(&mut self) {
        self.surfaces.popups.enabled = true;
    }

    /// Snapshot live configured popup buffers in parent-before-child creation order.
    /// This does not claim presentation; parent links form an acyclic forest.
    pub fn popup_surfaces(&self) -> Vec<Popup> {
        self.surfaces.popups.mapped().cloned().collect()
    }
}

impl Popups {
    /// A grab must precede the first buffer and use a tracked, undismissed role.
    pub(crate) fn grab_parent(&self, role: &PopupSurface) -> Option<WlSurface> {
        self.entries
            .iter()
            .find(|entry| {
                &entry.role == role && !entry.buffered && !entry.dismissed && entry.role.alive()
            })
            .map(|entry| entry.popup.parent.clone())
    }

    /// Pending grabs remain alive while waiting for the first configured buffer.
    pub(crate) fn live(&self, surface: &WlSurface) -> bool {
        self.entries
            .iter()
            .any(|entry| &entry.popup.surface == surface && !entry.dismissed && entry.role.alive())
    }

    /// Live popup roots shared by rendering and focus lifetime checks.
    pub(crate) fn mapped(&self) -> impl Iterator<Item = &Popup> {
        self.entries
            .iter()
            .filter(|entry| entry.buffered && !entry.dismissed && entry.role.alive())
            .map(|entry| &entry.popup)
    }
}

impl Popups {
    /// Accept only mapped parents and arithmetic-safe initial placement.
    pub(crate) fn insert(
        &mut self,
        role: PopupSurface,
        positioner: PositionerState,
        parents: &[WlSurface],
    ) {
        let parent = role.get_parent_surface().filter(|parent| {
            parents.contains(parent) || self.mapped().any(|p| &p.surface == parent)
        });
        let Some(parent) = parent.filter(|_| self.enabled) else {
            role.send_popup_done();
            return;
        };
        let Some(geometry) = self.placement(positioner, &parent, parents) else {
            role.send_popup_done();
            return;
        };
        role.with_pending_state(|state| {
            state.geometry = geometry;
            state.positioner = positioner;
        });
        self.entries.push(Entry {
            popup: Popup {
                surface: role.wl_surface().clone(),
                parent,
                geometry,
            },
            role,
            buffered: false,
            dismissed: false,
        });
    }

    /// Confirm a validated explicit request without changing committed placement.
    pub(crate) fn reposition(
        &mut self,
        role: &PopupSurface,
        positioner: PositionerState,
        token: u32,
        parents: &[WlSurface],
    ) {
        let geometry = self
            .entries
            .iter()
            .find(|entry| &entry.role == role && !entry.dismissed && entry.role.alive())
            .and_then(|entry| self.placement(positioner, &entry.popup.parent, parents));
        let Some(geometry) = geometry else {
            self.dismiss(role);
            return;
        };
        role.with_pending_state(|state| {
            state.geometry = geometry;
            state.positioner = positioner;
        });
        role.send_repositioned(token);
    }

    /// Translate output bounds through the same committed scene used for input.
    fn placement(
        &self,
        positioner: PositionerState,
        parent: &WlSurface,
        roots: &[WlSurface],
    ) -> Option<Rectangle<i32, Logical>> {
        let popups: Vec<_> = self.mapped().cloned().collect();
        crate::popup_placement::geometry(positioner, parent, roots, &popups, self.output_size)
    }

    /// Apply configure-before-buffer and terminal unmap semantics.
    pub(crate) fn commit(&mut self, surface: &WlSurface) {
        let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| &entry.popup.surface == surface)
        else {
            return;
        };
        if entry.dismissed {
            return;
        }
        let has_buffer =
            with_renderer_surface_state(surface, |state| state.buffer().is_some()).unwrap_or(false);
        if has_buffer {
            entry.buffered = entry.role.ensure_configured();
            if entry.buffered {
                // Smithay applies the acknowledged configure in its commit hook.
                // Requests and acknowledgements alone must never move the scene.
                with_states(surface, |states| {
                    if let Some(data) = states.data_map.get::<XdgPopupSurfaceData>() {
                        entry.popup.geometry = data
                            .lock()
                            .unwrap_or_else(|_| std::process::abort())
                            .current
                            .geometry;
                    }
                });
            }
        } else if entry.buffered
            || (!entry.role.is_initial_configure_sent() && entry.role.send_configure().is_err())
        {
            self.dismiss_tree(surface);
        }
    }

    /// Parent loss dismisses once; client destruction removes owned bookkeeping.
    pub(crate) fn prune(&mut self, parents: &[WlSurface]) {
        self.entries.retain(|entry| entry.role.alive());
        // Parents necessarily precede children: only already mapped roles can
        // become parents. Compute eligibility forward, send done child-first.
        let mut live = parents.to_vec();
        let mut lost = Vec::new();
        for entry in &self.entries {
            if !live.contains(&entry.popup.parent) {
                lost.push(entry.popup.surface.clone());
            } else if entry.buffered && !entry.dismissed {
                live.push(entry.popup.surface.clone());
            }
        }
        for entry in self.entries.iter_mut().rev() {
            if lost.contains(&entry.popup.surface) {
                entry.dismiss();
            }
        }
    }

    /// Unsupported operations dismiss the whole descendant chain, child-first.
    pub(crate) fn dismiss(&mut self, role: &PopupSurface) {
        if self.entries.iter().any(|entry| &entry.role == role) {
            self.dismiss_tree(role.wl_surface());
        } else {
            role.send_popup_done();
        }
    }

    /// Iterative traversal avoids client-controlled recursion depth.
    pub(crate) fn dismiss_tree(&mut self, surface: &WlSurface) {
        let mut lost = vec![surface.clone()];
        for entry in &self.entries {
            if lost.contains(&entry.popup.parent) {
                lost.push(entry.popup.surface.clone());
            }
        }
        for entry in self.entries.iter_mut().rev() {
            if lost.contains(&entry.popup.surface) {
                entry.dismiss();
            }
        }
    }
}

impl Entry {
    /// Terminal, idempotent dismissal; an old acknowledgement cannot revive it.
    fn dismiss(&mut self) {
        if !self.dismissed {
            self.role.send_popup_done();
        }
        self.dismissed = true;
        self.buffered = false;
    }
}
