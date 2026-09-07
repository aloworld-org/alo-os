//! Keyboard protocol observations for real socket and graphical fixtures.
use super::Events;
use wayland_client::{
    Connection, Dispatch, QueueHandle, WEnum,
    protocol::{wl_keyboard, wl_seat},
};

/// Event facts retained without interpreting a compositor's internal state.
#[derive(Default)]
pub struct KeyboardEvents {
    /// Seat capabilities; no pointer is promised by this component.
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
            wl_keyboard::Event::Enter { keys, .. } => events.enters.push(keys),
            wl_keyboard::Event::Leave { .. } => events.leaves += 1,
            wl_keyboard::Event::Key {
                key,
                state: WEnum::Value(state),
                ..
            } => events.keys.push((key, state)),
            wl_keyboard::Event::Modifiers { mods_depressed, .. } => {
                events.modifiers.push(mods_depressed)
            }
            _ => {}
        }
    }
}
