//! Pointer protocol observations shared by socket fixtures.
use super::Events;
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, WEnum, delegate_noop,
    protocol::{wl_pointer, wl_region},
};

/// Wire facts, independent of compositor routing internals.
#[derive(Default)]
pub struct PointerEvents {
    /// Pointer proxy used for real cursor requests.
    pub proxy: Option<wl_pointer::WlPointer>,
    /// Most recent pointer enter serial.
    pub serial: u32,
    /// Latest button-event serial, used to request a real explicit popup grab.
    pub button_serial: u32,
    /// Surface protocol ID and local coordinates on entry.
    pub enters: Vec<(u32, f64, f64)>,
    /// Number of focus leaves.
    pub leaves: usize,
    /// Surface-local motion coordinates.
    pub motion: Vec<(f64, f64)>,
    /// Button transitions in wire order.
    pub buttons: Vec<(u32, wl_pointer::ButtonState)>,
    /// Scroll values in wire order.
    pub axes: Vec<(wl_pointer::Axis, f64)>,
    /// Finger scroll stop events.
    pub stops: usize,
    /// Completed pointer frames.
    pub frames: usize,
}
delegate_noop!(Events: ignore wl_region::WlRegion);
impl Dispatch<wl_pointer::WlPointer, ()> for Events {
    fn event(
        state: &mut Self,
        _: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let p = &mut state.pointer;
        match event {
            wl_pointer::Event::Enter {
                serial,
                surface,
                surface_x,
                surface_y,
                ..
            } => {
                p.serial = serial;
                p.enters
                    .push((surface.id().protocol_id(), surface_x, surface_y));
            }
            wl_pointer::Event::Leave { .. } => p.leaves += 1,
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => p.motion.push((surface_x, surface_y)),
            wl_pointer::Event::Button {
                serial,
                button,
                state: WEnum::Value(state),
                ..
            } => {
                p.button_serial = serial;
                p.buttons.push((button, state));
            }
            wl_pointer::Event::Axis {
                axis: WEnum::Value(axis),
                value,
                ..
            } => p.axes.push((axis, value)),
            wl_pointer::Event::AxisStop { .. } => p.stops += 1,
            wl_pointer::Event::Frame => p.frames += 1,
            _ => {}
        }
    }
}
