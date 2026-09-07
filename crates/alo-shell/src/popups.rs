//! Opt-in popup protocol lifetimes, independent of presentation and grabs.

use smithay::{
    backend::renderer::utils::with_renderer_surface_state,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Rectangle},
    wayland::shell::xdg::{PopupSurface, PositionerState},
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
    /// Initial position and requested size in parent window coordinates.
    pub geometry: Rectangle<i32, Logical>,
}

/// Protocol ownership includes dismissed roles until destruction, preventing revival.
struct Entry {
    /// Smithay owns configure serial validation.
    role: PopupSurface,
    /// Immutable initial placement and parent.
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
}

impl crate::Server {
    /// Enable popup handshake tracking for a popup-aware backend or protocol fixture.
    ///
    /// `Nested` renders these snapshots and pointer routing consumes them.
    /// Pointer-triggered grabs are supported; reposition requests still dismiss.
    /// A root grab needs this seat's active pointer press serial on the parent
    /// tree. A submenu can inherit its topmost parent's grab serial. Keyboard
    /// and release-triggered initiation are not yet accepted.
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
        // Bounding every operand protects Smithay's i32 placement additions and
        // subtractions without patching it. This is a backend geometry limit,
        // not a memory allocation based on client-supplied dimensions.
        let safe = [
            positioner.rect_size.w,
            positioner.rect_size.h,
            positioner.anchor_rect.loc.x,
            positioner.anchor_rect.loc.y,
            positioner.anchor_rect.size.w,
            positioner.anchor_rect.size.h,
            positioner.offset.x,
            positioner.offset.y,
        ]
        .into_iter()
        .all(|value| (-1_000_000..=1_000_000).contains(&value));
        let Some(parent) = parent.filter(|_| self.enabled && safe) else {
            role.send_popup_done();
            return;
        };
        let geometry = positioner.get_geometry();
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
