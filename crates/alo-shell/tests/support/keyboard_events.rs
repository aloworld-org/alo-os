//! Keyboard protocol observations for real socket and graphical fixtures.
use super::Events;
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
    protocol::{wl_keyboard, wl_seat},
};

/// Event facts retained without interpreting a compositor's internal state.
#[derive(Default)]
pub struct KeyboardEvents {
    /// Seat resource for explicit popup requests.
    pub seat: Option<wl_seat::WlSeat>,
    /// Surface IDs receiving keyboard focus, in wire order.
    pub surfaces: Vec<u32>,
    /// Advertised seat capabilities.
    pub capabilities: Option<wl_seat::Capability>,
    /// XKB keymap read from the received descriptor.
    pub keymap: String,
    /// Advertised rate and delay.
    pub repeat: Option<(i32, i32)>,
    /// Enter count and held-key arrays.
    pub enters: Vec<Vec<u8>>,
    /// Focus leaves.
    pub leaves: usize,
    /// Evdev code and press/release in wire order.
    pub keys: Vec<(u32, wl_keyboard::KeyState)>,
    /// Last key event serial, as received over the socket.
    pub key_serial: u32,
    /// Depressed modifier masks in wire order.
    pub modifiers: Vec<u32>,
}

impl Dispatch<wl_seat::WlSeat, ()> for Events {
    fn event(
        state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(caps),
        } = event
        {
            state.keyboard.capabilities = Some(caps);
            state.keyboard.seat = Some(seat.clone());
            if caps.contains(wl_seat::Capability::Pointer) {
                state.pointer.proxy = Some(seat.get_pointer(qh, ()));
            }
            if caps.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for Events {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let events = &mut state.keyboard;
        match event {
            wl_keyboard::Event::Keymap {
                format: WEnum::Value(wl_keyboard::KeymapFormat::XkbV1),
                fd,
                size,
            } => {
                use std::io::Read;
                let result = std::fs::File::from(fd)
                    .take(u64::from(size))
                    .read_to_string(&mut events.keymap);
                assert!(result.is_ok());
            }
            wl_keyboard::Event::RepeatInfo { rate, delay } => events.repeat = Some((rate, delay)),
            wl_keyboard::Event::Enter { keys, surface, .. } => {
                events.enters.push(keys);
                events.surfaces.push(surface.id().protocol_id());
            }
            wl_keyboard::Event::Leave { .. } => events.leaves += 1,
            wl_keyboard::Event::Key {
                serial,
                key,
                state: WEnum::Value(state),
                ..
            } => {
                events.key_serial = serial;
                events.keys.push((key, state));
            }
            wl_keyboard::Event::Modifiers { mods_depressed, .. } => {
                events.modifiers.push(mods_depressed)
            }
            _ => {}
        }
    }
}
