//! The line, said one way, and the names, declared once.

use super::*;

/// **The line names the row by its drawn name, quoted**, so a name with a
/// newline in it cannot start a journal line of its own.
#[test]
fn the_line_names_the_row_quoted() {
    assert_eq!(
        the_keyboard_is_on(ADD_A_PROVIDER),
        "alo-shell: in Settings the keyboard is on \"Add a provider\""
    );
    assert!(the_keyboard_is_on(ADD_A_PROVIDER).starts_with(IN_SETTINGS_THE_KEYBOARD_IS_ON));
    assert!(!the_keyboard_is_on("a\nalo-shell: forged").contains('\n'));
}

/// **The names are the English the screen draws**, each once.
#[test]
fn the_names_are_the_words_drawn() {
    assert_eq!(
        [ADD_A_PROVIDER, NAME, ADDRESS, KEY, MODEL, NEEDS_NO_KEY],
        [
            "Add a provider",
            "Name",
            "Address",
            "Key",
            "Model",
            "This service needs no key"
        ]
    );
}
