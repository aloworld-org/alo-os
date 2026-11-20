//! **A card comes in front of a window alo just opened.**
//!
//! Main's walk at 71932e07 (run 44161055698, picture 92): at *ask each time*,
//! alo opened `asked.txt` in its own copy of VSCodium after the person's
//! approval, and that copy came in front with the keyboard
//! (`crate::a_file_alo_opened_comes_in_front`). The card asking to type into it
//! then rose in the task window **behind** the copy, where the person could not
//! see what they were asked to approve.

#[cfg(test)]
#[path = "the_card_comes_in_front_of_what_alo_opened_tests.rs"]
mod tests;
