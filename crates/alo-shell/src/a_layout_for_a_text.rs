//! **The keyboard layouts a text is typed through** —
//! [ADR 0153](../../../docs/decisions/0153-alo-types-by-keys-in-its-own-copy-and-reads-the-text-back.md).
//!
//! A key press carries a key's number, never a character: the application
//! looks the number up in the layout it was sent. The person's layout has no
//! key for `日` or `✓`, and may not have one for `é`. So the text is typed
//! through **layouts of its own**, each giving one key to each character it
//! holds — the way `wtype` typed every character of the measurement of
//! 2026-11-13 — and only the client being typed into is ever sent one
//! (`crate::typing_into_the_window_in_front`).
//!
//! One layout holds at most [`KEYS_IN_A_LAYOUT`] different characters, so a
//! text with more is typed through several, one after the other. Each layout
//! also holds one Control key, so `Ctrl+S` is typed the same way.
//!
//! **Nothing here touches a client.** This decides the layouts and the presses
//! as text and numbers, and is tested as such.

use std::collections::BTreeMap;

use smithay::input::keyboard::xkb;

/// The key the Control key is, in every layout here, by XKB's number.
const CONTROL: u32 = 9;

/// The first key a character is given, by XKB's number.
const FIRST: u32 = 10;

/// The last key number XKB's text format allows.
const LAST: u32 = 255;

/// How many different characters one layout holds.
pub(crate) const KEYS_IN_A_LAYOUT: usize = 246;

/// XKB numbers keys eight above the kernel's, which is what a client is sent.
const XKB_ABOVE_THE_KERNEL: u32 = 8;

/// One key pressed and let go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Press {
    /// The key, by the kernel's number — what `wl_keyboard.key` carries.
    pub(crate) key: u32,
    /// Whether Control is held while it is pressed.
    pub(crate) with_control: bool,
}

/// A layout, and the presses typed through it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ALayout {
    /// The layout, in XKB's text format.
    pub(crate) keymap: String,
    /// What is pressed, in order.
    pub(crate) presses: Vec<Press>,
}

/// The Control key, by the kernel's number, in every layout here.
pub(crate) const CONTROL_KEY: u32 = CONTROL - XKB_ABOVE_THE_KERNEL;

/// The name XKB knows a character's symbol by — or [`None`] for one no key
/// can carry: a control character other than a new line or a tab.
fn the_symbol_of(character: char) -> Option<String> {
    match character {
        '\n' => return Some("Return".to_owned()),
        '\t' => return Some("Tab".to_owned()),
        _ if character.is_control() => return None,
        _ => {}
    }
    let symbol = xkb::utf32_to_keysym(u32::from(character));
    if symbol.raw() == 0 {
        return None;
    }
    let name = xkb::keysym_get_name(symbol);
    (!name.is_empty()).then_some(name)
}

/// A layout giving these keys these symbols, with Control at [`CONTROL`].
fn written(symbols: &BTreeMap<u32, String>) -> String {
    let codes: String = symbols
        .keys()
        .map(|code| format!(" <K{code}> = {code};"))
        .collect();
    let named: String = symbols
        .iter()
        .map(|(code, name)| format!(" key <K{code}> {{ [ {name} ] }};"))
        .collect();
    let keycodes = format!("minimum = 8; maximum = {LAST}; <CTRL> = {CONTROL};{codes}");
    let keys = format!("key <CTRL> {{ [ Control_L ] }}; modifier_map Control {{ <CTRL> }};{named}");
    format!(
        "xkb_keymap {{\n\
         xkb_keycodes \"alo\" {{ {keycodes} }};\n\
         xkb_types \"alo\" {{ include \"complete\" }};\n\
         xkb_compatibility \"alo\" {{ include \"complete\" }};\n\
         xkb_symbols \"alo\" {{ {keys} }};\n\
         }};\n"
    )
}

/// The layouts `text` is typed through, in order — or the first character no
/// key can carry, in which case nothing of it is typed.
pub(crate) fn the_layouts_for(text: &str) -> Result<Vec<ALayout>, char> {
    let mut layouts = Vec::new();
    let mut given: BTreeMap<char, u32> = BTreeMap::new();
    let mut symbols: BTreeMap<u32, String> = BTreeMap::new();
    let mut presses = Vec::new();
    for character in text.chars() {
        let symbol = the_symbol_of(character).ok_or(character)?;
        if !given.contains_key(&character) && given.len() == KEYS_IN_A_LAYOUT {
            layouts.push(ALayout {
                keymap: written(&symbols),
                presses: std::mem::take(&mut presses),
            });
            given.clear();
            symbols.clear();
        }
        let next = FIRST + u32::try_from(given.len()).map_err(|_| character)?;
        let code = *given.entry(character).or_insert(next);
        symbols.entry(code).or_insert(symbol);
        presses.push(Press {
            key: code - XKB_ABOVE_THE_KERNEL,
            with_control: false,
        });
    }
    if !presses.is_empty() {
        layouts.push(ALayout {
            keymap: written(&symbols),
            presses,
        });
    }
    Ok(layouts)
}

/// The layout `Ctrl+S` is typed through.
pub(crate) fn the_layout_that_saves() -> ALayout {
    let symbols = BTreeMap::from([(FIRST, "s".to_owned())]);
    ALayout {
        keymap: written(&symbols),
        presses: vec![Press {
            key: FIRST - XKB_ABOVE_THE_KERNEL,
            with_control: true,
        }],
    }
}

#[cfg(test)]
#[path = "a_layout_for_a_text_tests.rs"]
mod tests;
