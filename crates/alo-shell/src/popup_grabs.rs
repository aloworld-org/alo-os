//! Explicit pointer-triggered popup ownership, separate from protocol lifetimes.

use smithay::{
    input::Seat,
    reexports::wayland_protocols::xdg::shell::server::{xdg_popup, xdg_wm_base},
    reexports::wayland_server::{
        Resource,
        protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
    },
    utils::Serial,
    wayland::{compositor::get_parent, shell::xdg::PopupSurface},
};

use crate::surfaces::Surfaces;

/// One seat owns one ordered chain; only its topmost popup may add a child.
pub(crate) struct Grab {
    /// Original mapped window, used to restore focus without selecting another client.
    pub(crate) root: WlSurface,
    /// Parent-before-child roles, including a pending initial configure.
    chain: Vec<WlSurface>,
    /// Original active press authorizes submenus in this chain only.
    serial: Serial,
}

impl Surfaces {
    /// Accept an active pointer press on the parent, or the current chain's serial.
    pub(crate) fn grab_popup(&mut self, role: PopupSurface, seat: WlSeat, serial: Serial) {
        self.prune();
        if self
            .popups
            .mapped()
            .any(|popup| &popup.surface == role.wl_surface())
        {
            role.xdg_popup().post_error(
                xdg_popup::Error::InvalidGrab,
                "popup grab requested after mapping",
            );
            return;
        }
        let parent = self.popups.grab_parent(&role);
        if let Some(parent) = &parent {
            let popup_parent = self.popups.mapped().any(|popup| &popup.surface == parent);
            if popup_parent
                && self.popup_grab.as_ref().and_then(|grab| grab.chain.last()) != Some(parent)
            {
                role.wl_surface().post_error(
                    xdg_wm_base::Error::NotTheTopmostPopup,
                    "popup parent does not own the topmost grab",
                );
                return;
            }
        }
        let own_seat = self.keyboard.as_ref().is_some_and(|keyboard| {
            Seat::<Self>::from_resource(&seat).as_ref() == Some(&keyboard.seat)
        });
        let allowed = parent.as_ref().is_some_and(|parent| {
            if let Some(grab) = &self.popup_grab {
                grab.chain.last() == Some(parent)
                    && (serial == grab.serial || self.pointer_press_on(parent, serial))
            } else {
                self.mapped().any(|root| root == parent) && self.pointer_press_on(parent, serial)
            }
        });
        if !own_seat || !allowed {
            self.popups.dismiss(&role);
            self.prune();
            return;
        }
        let Some(parent) = parent else { return };
        // End the initiating implicit drag before changing recipients. Its later
        // physical release is unmatched and cannot land in the newly opened menu.
        let _ = self.clear_pointer();
        if let Some(grab) = &mut self.popup_grab {
            grab.chain.push(role.wl_surface().clone());
        } else {
            self.popup_grab = Some(Grab {
                root: parent,
                chain: vec![role.wl_surface().clone()],
                serial,
            });
        }
        let focus = self.popup_keyboard_focus();
        let _ = self.set_keyboard_focus(focus);
    }

    /// A client must destroy active grabbing children before their ancestors.
    pub(crate) fn popup_grab_destroyed(&mut self, role: &PopupSurface) {
        if let Some(grab) = &self.popup_grab
            && grab.chain.contains(role.wl_surface())
            && grab
                .chain
                .last()
                .is_some_and(|last| last != role.wl_surface() && self.popups.live(last))
            && role.wl_surface().is_alive()
        {
            role.wl_surface().post_error(
                xdg_wm_base::Error::NotTheTopmostPopup,
                "destroyed a popup below an active grabbing child",
            );
        }
    }

    /// Require the actual active press serial and a surface in the parent tree.
    fn pointer_press_on(&self, parent: &WlSurface, serial: Serial) -> bool {
        let Some(pointer) = &self.pointer else {
            return false;
        };
        if !pointer.handle.has_grab(serial) {
            return false;
        }
        let Some((mut surface, _)) = pointer.handle.grab_start_data().and_then(|data| data.focus)
        else {
            return false;
        };
        while let Some(ancestor) = get_parent(&surface) {
            surface = ancestor;
        }
        &surface == parent
    }

    /// Owner-events policy: other windows of this client remain pointer reachable.
    pub(crate) fn popup_allows_pointer(&self, surface: &WlSurface) -> bool {
        self.popup_grab
            .as_ref()
            .is_none_or(|grab| surface.id().same_client_as(&grab.root.id()))
    }

    /// Focus only mapped popups; a not-yet-buffered child cannot receive keys.
    pub(crate) fn popup_keyboard_focus(&self) -> Option<WlSurface> {
        let grab = self.popup_grab.as_ref()?;
        grab.chain
            .iter()
            .rev()
            .find(|surface| self.popups.mapped().any(|popup| &popup.surface == *surface))
            .cloned()
            .or_else(|| self.mapped().find(|root| *root == &grab.root).cloned())
    }

    /// Restore the surviving parent after destruction, dismissal or disconnect.
    pub(crate) fn prune_popup_grab(&mut self) {
        let Some(mut grab) = self.popup_grab.take() else {
            return;
        };
        let previous = grab.chain.last().cloned();
        let root_live = self.mapped().any(|root| *root == grab.root);
        if root_live {
            let live = grab
                .chain
                .iter()
                .take_while(|surface| self.popups.live(surface))
                .count();
            grab.chain.truncate(live);
        } else {
            grab.chain.clear();
        }
        let changed = previous != grab.chain.last().cloned();
        let root = root_live.then(|| grab.root.clone());
        if !grab.chain.is_empty() {
            self.popup_grab = Some(grab);
        }
        if changed {
            let _ = self.clear_pointer();
        }
        let focus = self.popup_keyboard_focus().or(root);
        let _ = self.set_keyboard_focus(focus);
    }

    /// Outside presses and backend focus loss dismiss child-first and consume input.
    pub(crate) fn dismiss_popup_grab(&mut self) {
        if let Some(first) = self
            .popup_grab
            .as_ref()
            .and_then(|grab| grab.chain.first())
            .cloned()
        {
            self.popups.dismiss_tree(&first);
            self.prune_popup_grab();
        }
    }
}
