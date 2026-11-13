//! **Typing into the window in front, as key presses, to that one client** —
//! [ADR 0153](../../../docs/decisions/0153-alo-types-by-keys-in-its-own-copy-and-reads-the-text-back.md)
//! and `docs/contracts/application-door.md`, *`type-into` and `save-in`*.
//!
//! What it does, in order, and nothing else:
//!
//! 1. finds the window that holds the keyboard **now**, and refuses unless it
//!    is a window of the named application — the keyboard is never moved to
//!    carry this out;
//! 2. asks whoever answers for alo's copies whether that window's process may
//!    be typed into — alo's own copy at the default, the person's window only
//!    where they chose that;
//! 3. sends **that client's keyboards, and no other client's**, a layout of
//!    alo's own (`crate::a_layout_for_a_text`), every key pressed and let go,
//!    and then the person's layout and modifiers back.
//!
//! **It all happens inside one call on the compositor's thread**, which is the
//! thread every key of the person's is read on, so none of theirs can arrive
//! between the first layout and the last press. Nothing is sent through
//! Smithay's own keyboard state: the person's held keys, modifiers and layout
//! are never changed, and no other window is told anything.
//!
//! **The first character.** On 2026-11-13 the first character was lost when
//! `wtype` pressed keys the moment it handed sway a layout. Wayland has no
//! acknowledgement of a layout to wait for; what it guarantees is order on one
//! connection, and a client reads the layout before the first key that follows
//! it. Here both go on the target's own connection, in that order. **Whether
//! that is enough is measured on a booted machine**; the read-back the turn
//! makes before saving is what catches it if it is not.

use std::os::fd::AsFd as _;

use alo_acting::{ToType, Typed};
use alo_dock::AppId;
use smithay::input::keyboard::{KeymapFile, xkb};
use smithay::reexports::wayland_server::Resource as _;
use smithay::reexports::wayland_server::protocol::wl_keyboard::{
    KeyState, KeymapFormat, WlKeyboard,
};
use smithay::utils::SERIAL_COUNTER;

use crate::Server;
use crate::a_layout_for_a_text::{ALayout, CONTROL_KEY, the_layout_that_saves, the_layouts_for};

/// Whether the process behind the window in front may be typed into, for this
/// application — answered by whoever holds alo's copies and the person's
/// setting, which this compositor does not.
pub trait MayTypeInto {
    /// Whether the window of `application` whose process is `pid` may be typed
    /// into.
    fn may_type_into(&mut self, application: &str, pid: u32) -> bool;
}

/// The desktop answering [`MayTypeInto`], from what it holds of alo's copies
/// and the person's setting (`crate::TheDesktop::may_type_into`).
pub(crate) struct TheDesktopAnswers<'d>(pub(crate) &'d mut dyn crate::TheDesktop);

impl MayTypeInto for TheDesktopAnswers<'_> {
    fn may_type_into(&mut self, application: &str, pid: u32) -> bool {
        self.0.may_type_into(application, pid)
    }
}

/// One layout, compiled, with what is pressed through it.
struct Ready {
    /// The layout, as a file a client is handed.
    file: KeymapFile,
    /// The Control modifier's bit in it.
    control: u32,
    /// What is pressed.
    layout: ALayout,
}

/// Compile each layout before anything is sent, so a layout that will not
/// compile stops the typing before its first key.
fn ready(layouts: Vec<ALayout>) -> Option<Vec<Ready>> {
    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    layouts
        .into_iter()
        .map(|layout| {
            let keymap = xkb::Keymap::new_from_string(
                &context,
                layout.keymap.clone(),
                xkb::KEYMAP_FORMAT_TEXT_V1,
                xkb::KEYMAP_COMPILE_NO_FLAGS,
            )?;
            let index = keymap.mod_get_index(xkb::MOD_NAME_CTRL);
            let control = 1_u32.checked_shl(index)?;
            Some(Ready {
                file: KeymapFile::new(&keymap),
                control,
                layout,
            })
        })
        .collect()
}

/// Hand this keyboard this layout.
fn handed(keyboard: &WlKeyboard, file: &KeymapFile) -> bool {
    file.with_fd(keyboard.version() >= 7, |fd, size| {
        keyboard.keymap(
            KeymapFormat::XkbV1,
            fd.as_fd(),
            u32::try_from(size).unwrap_or(u32::MAX),
        );
    })
    .is_ok()
}

/// Press and let go of one key on this keyboard.
fn pressed(keyboard: &WlKeyboard, key: u32, time: u32) {
    keyboard.key(
        SERIAL_COUNTER.next_serial().into(),
        time,
        key,
        KeyState::Pressed,
    );
    keyboard.key(
        SERIAL_COUNTER.next_serial().into(),
        time,
        key,
        KeyState::Released,
    );
}

/// Every press of one layout, with Control held around a press that asks for it.
fn typed_through(keyboard: &WlKeyboard, ready: &Ready, time: u32) {
    for press in &ready.layout.presses {
        if press.with_control {
            keyboard.key(
                SERIAL_COUNTER.next_serial().into(),
                time,
                CONTROL_KEY,
                KeyState::Pressed,
            );
            keyboard.modifiers(SERIAL_COUNTER.next_serial().into(), ready.control, 0, 0, 0);
            pressed(keyboard, press.key, time);
            keyboard.modifiers(SERIAL_COUNTER.next_serial().into(), 0, 0, 0, 0);
            keyboard.key(
                SERIAL_COUNTER.next_serial().into(),
                time,
                CONTROL_KEY,
                KeyState::Released,
            );
        } else {
            pressed(keyboard, press.key, time);
        }
    }
}

impl Server {
    /// Type `text`, or save, into the window in front if it is a window of
    /// the application `asked` names and `may` lets it be typed into — and
    /// answer what happened. `text` is the text for [`ToType::Text`], and is
    /// not read for [`ToType::Save`].
    pub fn typed_into_the_window_in_front(
        &mut self,
        asked: &ToType,
        text: &str,
        may: &mut dyn MayTypeInto,
    ) -> Typed {
        let (layouts, done) = match asked {
            ToType::Text { .. } => match the_layouts_for(text) {
                Ok(layouts) => (layouts, Typed::Typed),
                Err(character) => return Typed::CannotType(character),
            },
            ToType::Save { .. } => (vec![the_layout_that_saves()], Typed::AskedToSave),
        };
        let Some(keyboard) = self.surfaces.keyboard.as_ref() else {
            return Typed::NoKeyboard;
        };
        let (handle, time, names) = (
            keyboard.handle.clone(),
            keyboard.time,
            keyboard.names.clone(),
        );
        let Some(root) = self.surfaces.keyboard_root() else {
            return Typed::NotInFront;
        };
        let identifier = asked.application().identifier();
        let Ok(app) = AppId::named(identifier) else {
            return Typed::NotInFront;
        };
        let id = crate::window_number::the_window_id_of(&root);
        let theirs = self
            .the_windows_the_dock_sees()
            .each_as_opened()
            .any(|window| window.id() == id && window.app() == Some(&app));
        if !theirs {
            return Typed::NotInFront;
        }
        let Some(client) = root.client() else {
            return Typed::NotInFront;
        };
        // **The kernel's account of the process**, recorded when it connected
        // — the same road the door asks of its own callers.
        let Ok(credentials) = client.get_credentials(&self.display_handle()) else {
            return Typed::NotAlosCopy;
        };
        let Ok(pid) = u32::try_from(credentials.pid) else {
            return Typed::NotAlosCopy;
        };
        if !may.may_type_into(identifier, pid) {
            return Typed::NotAlosCopy;
        }
        let Some(ready) = ready(layouts) else {
            return Typed::NoKeyboard;
        };
        let Some(theirs) = names.compiled().map(|keymap| KeymapFile::new(&keymap)) else {
            return Typed::NoKeyboard;
        };
        let keyboards: Vec<WlKeyboard> = handle.client_keyboards(&client).collect();
        if keyboards.is_empty() {
            return Typed::NoKeyboard;
        }
        let mods = handle.modifier_state().serialized;
        let mut whole = true;
        for keyboard in &keyboards {
            // Whatever the person is holding is let go for this client first,
            // so a held Shift does not change a typed letter.
            keyboard.modifiers(SERIAL_COUNTER.next_serial().into(), 0, 0, 0, 0);
            for one in &ready {
                // A layout that could not be handed over stops the typing
                // there: keys pressed through the person's layout would be
                // other characters. What did arrive is caught by the read-back.
                if !handed(keyboard, &one.file) {
                    whole = false;
                    break;
                }
                typed_through(keyboard, one, time);
            }
            // The person's layout back, whatever happened above.
            whole &= handed(keyboard, &theirs);
            keyboard.modifiers(
                SERIAL_COUNTER.next_serial().into(),
                mods.depressed,
                mods.latched,
                mods.locked,
                mods.layout_effective,
            );
        }
        if whole { done } else { Typed::NoKeyboard }
    }
}
