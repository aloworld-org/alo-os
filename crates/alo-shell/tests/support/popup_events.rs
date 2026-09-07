//! Popup requests and independent configure observations over real sockets.
use super::*;
use wayland_client::Proxy;
use wayland_protocols::xdg::shell::client::{xdg_popup, xdg_positioner};

/// Events kept separate from the toplevel's configure serial.
#[derive(Default)]
pub struct PopupEvents {
    /// Initial geometry configurations in wire order.
    pub geometry: Vec<(i32, i32, i32, i32)>,
    /// Popup-only XDG configure serial.
    pub serial: Option<u32>,
    /// Terminal dismissal count.
    pub done: usize,
    /// Dismissed role IDs in protocol order.
    pub done_order: Vec<u32>,
}

impl Dispatch<xdg_surface::XdgSurface, bool> for Events {
    fn event(
        state: &mut Self,
        _: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &bool,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            state.popups.serial = Some(serial);
        }
    }
}
impl Dispatch<xdg_popup::XdgPopup, ()> for Events {
    fn event(
        state: &mut Self,
        popup: &xdg_popup::XdgPopup,
        event: xdg_popup::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            xdg_popup::Event::Configure {
                x,
                y,
                width,
                height,
            } => state.popups.geometry.push((x, y, width, height)),
            xdg_popup::Event::PopupDone => {
                state.popups.done += 1;
                state.popups.done_order.push(popup.id().protocol_id());
            }
            _ => {}
        }
    }
}
delegate_noop!(Events: ignore xdg_positioner::XdgPositioner);

impl Application {
    /// Request unsupported repositioning with a valid positioner.
    pub fn reposition_popup(&self, popup: &xdg_popup::XdgPopup) {
        let positioner = self.shell.create_positioner(&self.queue.handle(), ());
        positioner.set_size(16, 16);
        positioner.set_anchor_rect(0, 0, 4, 4);
        popup.reposition(&positioner, 42);
        positioner.destroy();
    }
    /// Create a popup role without committing; offset can exercise integer refusal.
    pub fn popup(
        &self,
        parent: bool,
        offset: i32,
    ) -> (
        wl_surface::WlSurface,
        xdg_surface::XdgSurface,
        xdg_popup::XdgPopup,
    ) {
        self.popup_on(parent.then_some(&self.xdg), offset)
    }

    /// Create a popup on an explicit XDG parent, including another popup.
    pub fn popup_on(
        &self,
        parent: Option<&xdg_surface::XdgSurface>,
        offset: i32,
    ) -> (
        wl_surface::WlSurface,
        xdg_surface::XdgSurface,
        xdg_popup::XdgPopup,
    ) {
        let qh = self.queue.handle();
        let surface = self.compositor.create_surface(&qh, ());
        let xdg = self.shell.get_xdg_surface(&surface, &qh, true);
        let positioner = self.shell.create_positioner(&qh, ());
        positioner.set_size(16, 16);
        positioner.set_anchor_rect(2, 3, 4, 6);
        positioner.set_anchor(xdg_positioner::Anchor::BottomRight);
        positioner.set_gravity(xdg_positioner::Gravity::BottomRight);
        positioner.set_offset(offset, 1);
        let popup = xdg.get_popup(parent, &positioner, &qh, ());
        positioner.destroy();
        (surface, xdg, popup)
    }

    /// Attach the shared test buffer to a popup without silently acknowledging it.
    pub fn attach_popup(&self, surface: &wl_surface::WlSurface) {
        surface.attach(Some(&self.buffer), 0, 0);
        surface.frame(&self.queue.handle(), ());
        surface.commit();
    }

    /// Explicitly acknowledge the last popup configure received on the wire.
    pub fn ack_popup(&self, xdg: &xdg_surface::XdgSurface) {
        xdg.ack_configure(self.events.popups.serial.unwrap());
    }
}
