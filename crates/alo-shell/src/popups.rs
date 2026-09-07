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
    /// Mapped toplevel parent.
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
    /// `Nested` renders these snapshots and pointer routing consumes them. Grabs, nested
    /// popup parents and reposition requests are explicitly dismissed for now.
    pub fn enable_popup_protocol(&mut self) {
        self.surfaces.popups.enabled = true;
    }

    /// Snapshot live configured popup buffers; this does not claim presentation.
    pub fn popup_surfaces(&self) -> Vec<Popup> {
        self.surfaces.popups.mapped().cloned().collect()
    }
}

impl Popups {
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
        let parent = role
            .get_parent_surface()
            .filter(|parent| parents.contains(parent));
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
            entry.dismiss();
        }
    }

    /// Parent loss dismisses once; client destruction removes owned bookkeeping.
    pub(crate) fn prune(&mut self, parents: &[WlSurface]) {
        self.entries.retain(|entry| entry.role.alive());
        for entry in &mut self.entries {
            if !parents.contains(&entry.popup.parent) {
                entry.dismiss();
            }
        }
    }

    /// Unsupported operations dismiss an existing popup instead of leaving it live.
    pub(crate) fn dismiss(&mut self, role: &PopupSurface) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| &entry.role == role) {
            entry.dismiss();
        } else {
            role.send_popup_done();
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
