//! What a thing in an application's window is, in the few kinds that matter
//! to reading and pressing it.
//!
//! The rented accessibility layer names well over a hundred roles
//! (`AtspiRole`, whose numbers are part of its published interface and do not
//! change). An agent needs to know far fewer: whether something is a window,
//! something that can be pressed and which kind, words, a field a person types
//! into, **a password field**, a document, a picture, or an area the
//! application draws itself. Everything else — the panels, boxes and fillers a
//! window is laid out with — is a [`Role::Part`], which is walked through and
//! not described.
//!
//! **A password field is its own role and never a kind of text field**, so no
//! code deciding what to read can treat it as one by matching on text fields.

/// What one thing in a window is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// A window, a dialogue or an alert.
    Window,
    /// A button.
    Button,
    /// A box that is ticked or not.
    CheckBox,
    /// One of a set of options.
    RadioButton,
    /// A switch.
    Switch,
    /// One item of a menu.
    MenuItem,
    /// One tab of a set.
    Tab,
    /// A link.
    Link,
    /// A drop-down: one choice of several, shown closed.
    DropDown,
    /// One item of a list, a table or a tree — a file in a folder, a row of
    /// an archive.
    Item,
    /// A number within a range: a slider, a progress bar or a meter.
    Range,
    /// Words that are read: a label, a heading, a paragraph.
    Label,
    /// A field a person types into, whose contents are shown.
    TextField,
    /// A field a person types a password into. **Its contents are never read.**
    PasswordField,
    /// A document an application shows.
    Document,
    /// A picture or an icon.
    Image,
    /// An area the application draws itself, which describes nothing inside it.
    Canvas,
    /// Anything else: how a window is laid out, walked through and not described.
    Part,
}

impl Role {
    /// The role an `AtspiRole` number is.
    #[must_use]
    pub const fn of_the_tree(number: u32) -> Self {
        match number {
            // ALERT, DIALOG, FRAME, WINDOW
            2 | 16 | 23 | 69 => Self::Window,
            // PUSH_BUTTON, TOGGLE_BUTTON, PUSH_BUTTON_MENU
            43 | 62 | 129 => Self::Button,
            // CHECK_BOX
            7 => Self::CheckBox,
            // RADIO_BUTTON
            44 => Self::RadioButton,
            // SWITCH
            130 => Self::Switch,
            // CHECK_MENU_ITEM, MENU, MENU_ITEM, RADIO_MENU_ITEM — a menu is the
            // heading of one, *File* in a menu bar, which a press opens. One
            // with no name is the list a heading opens, and the walk passes it
            // as layout (`crate::walking`).
            8 | 33 | 35 | 45 => Self::MenuItem,
            // PAGE_TAB
            37 => Self::Tab,
            // LINK
            88 => Self::Link,
            // COMBO_BOX
            11 => Self::DropDown,
            // LIST_ITEM, TABLE_CELL, TREE_ITEM
            32 | 56 | 91 => Self::Item,
            // PROGRESS_BAR, SLIDER, LEVEL_BAR
            42 | 51 | 103 => Self::Range,
            // LABEL, PARAGRAPH, CAPTION, HEADING, STATIC
            29 | 73 | 81 | 83 | 116 => Self::Label,
            // SPIN_BUTTON, TEXT, ENTRY
            52 | 61 | 79 => Self::TextField,
            // PASSWORD_TEXT
            40 => Self::PasswordField,
            // DOCUMENT_FRAME, DOCUMENT_SPREADSHEET … DOCUMENT_EMAIL
            82 | 92..=96 => Self::Document,
            // ICON, IMAGE
            26 | 27 => Self::Image,
            // CANVAS, DRAWING_AREA
            6 | 18 => Self::Canvas,
            _ => Self::Part,
        }
    }

    /// Whether its text is read, when it has any.
    ///
    /// **Never for a password field**, and this is the one place that says so
    /// for the walk; `crate::shown` holds it a second time for what is handed
    /// back.
    #[must_use]
    pub const fn text_is_read(self) -> bool {
        matches!(
            self,
            Self::Label | Self::TextField | Self::Document | Self::DropDown | Self::Item
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_password_field_is_its_own_role_and_its_text_is_never_read() {
        assert_eq!(Role::of_the_tree(40), Role::PasswordField);
        assert!(!Role::PasswordField.text_is_read());
        assert_eq!(Role::of_the_tree(61), Role::TextField);
        assert!(Role::TextField.text_is_read());
        assert_eq!(Role::of_the_tree(20), Role::Part);
        assert_eq!(Role::of_the_tree(u32::MAX), Role::Part);
    }
}
