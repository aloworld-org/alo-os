#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use smithay::input::keyboard::xkb;

use super::{
    CONTROL_KEY, FIRST, KEYS_IN_A_LAYOUT, LAST, XKB_ABOVE_THE_KERNEL, the_layout_that_saves,
    the_layouts_for,
};

/// The layout, compiled as a client compiles it.
fn compiled(keymap: &str) -> xkb::Keymap {
    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    xkb::Keymap::new_from_string(
        &context,
        keymap.to_owned(),
        xkb::KEYMAP_FORMAT_TEXT_V1,
        xkb::KEYMAP_COMPILE_NO_FLAGS,
    )
    .expect("a layout XKB compiles")
}

/// What a client reads from these presses through these layouts, as text.
fn what_a_client_reads(text: &str) -> String {
    let mut read = String::new();
    for layout in the_layouts_for(text).unwrap() {
        let keymap = compiled(&layout.keymap);
        let state = xkb::State::new(&keymap);
        for press in layout.presses {
            let symbol = state.key_get_one_sym((press.key + XKB_ABOVE_THE_KERNEL).into());
            read.push(match xkb::keysym_get_name(symbol).as_str() {
                "Return" => '\n',
                "Tab" => '\t',
                _ => char::from_u32(xkb::keysym_to_utf32(symbol)).expect("a character"),
            });
        }
    }
    read
}

#[test]
fn the_count_of_keys_is_every_number_xkb_allows_after_control() {
    assert_eq!(KEYS_IN_A_LAYOUT, usize::try_from(LAST - FIRST + 1).unwrap());
}

#[test]
fn the_text_of_the_measurement_is_read_back_whole() {
    let text = "function f(a) {\n    return \"é 日本 — ✓\";\n}\n<div>\n\tend\n";
    assert_eq!(what_a_client_reads(text), text);
}

#[test]
fn capitals_and_symbols_need_no_shift() {
    let text = "ABC xyz !@#$%^&*()_+{}|:\"<>?~`";
    assert_eq!(what_a_client_reads(text), text);
    for layout in the_layouts_for(text).unwrap() {
        assert!(layout.presses.iter().all(|press| !press.with_control));
    }
}

#[test]
fn a_character_used_twice_is_one_key_pressed_twice() {
    let layouts = the_layouts_for("aaa").unwrap();
    assert_eq!(layouts.len(), 1);
    let keys: Vec<u32> = layouts[0].presses.iter().map(|press| press.key).collect();
    assert_eq!(keys.len(), 3);
    assert!(keys.iter().all(|key| *key == keys[0]));
}

#[test]
fn a_text_with_more_characters_than_one_layout_holds_is_typed_through_several() {
    let text: String = (0x4E00_u32..0x4E00 + 600)
        .filter_map(char::from_u32)
        .collect();
    let layouts = the_layouts_for(&text).unwrap();
    assert_eq!(layouts.len(), 3);
    assert_eq!(what_a_client_reads(&text), text);
}

#[test]
fn a_control_character_is_refused_before_anything_is_typed() {
    assert_eq!(the_layouts_for("ok\u{7}"), Err('\u{7}'));
    assert_eq!(the_layouts_for("line\r\n"), Err('\r'));
}

#[test]
fn an_empty_text_is_no_layout() {
    assert!(the_layouts_for("").unwrap().is_empty());
}

#[test]
fn saving_is_control_held_with_s() {
    let layout = the_layout_that_saves();
    let keymap = compiled(&layout.keymap);
    let state = xkb::State::new(&keymap);
    assert_eq!(layout.presses.len(), 1);
    let press = layout.presses[0];
    assert!(press.with_control);
    assert_eq!(
        xkb::keysym_get_name(state.key_get_one_sym((press.key + XKB_ABOVE_THE_KERNEL).into())),
        "s"
    );
    assert_eq!(
        xkb::keysym_get_name(state.key_get_one_sym((CONTROL_KEY + XKB_ABOVE_THE_KERNEL).into())),
        "Control_L"
    );
    assert_ne!(keymap.mod_get_index(xkb::MOD_NAME_CTRL), xkb::MOD_INVALID);
}
