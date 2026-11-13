//! **What a reader is told about a node besides its name, in the spelling the
//! rented tree reads** — object attributes and the shape of a description.
//!
//! These are identifiers on the wire, as a role number is: Orca reads a
//! choice's place from `posinset` and `setsize`, and a heading's level from
//! `level`. No person is told any of these words. They are here, beside
//! `crate::access_roles`, rather than in the file that builds a surface's tree,
//! so a surface's file — Settings' above all, which words nothing of its own
//! (`tests/settings_source.rs`) — never spells a protocol by hand.

/// A choice's place among its set, as a reader says it: *2 of 3*. `at` counts
/// from nought.
pub(crate) fn its_place(at: usize, of: usize) -> Vec<(String, String)> {
    vec![
        ("posinset".to_owned(), (at + 1).to_string()),
        ("setsize".to_owned(), of.to_string()),
    ]
}

/// A heading's level, one for a page's own heading and two for a section's.
pub(crate) fn a_heading_at(level: u8) -> (String, String) {
    ("level".to_owned(), level.to_string())
}

/// Several things drawn with a control, told as one description: each on a
/// line of its own, in the order they are drawn.
pub(crate) fn told_together(lines: &[String]) -> String {
    lines.join("\n")
}

/// The keys of a shortcut, told as one thing: *Ctrl + Space*.
pub(crate) fn keys_together(keys: &[String]) -> String {
    keys.join(" + ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A place counts from one, as a reader says it**, and a heading carries
    /// the level it was given.
    #[test]
    fn a_place_is_said_from_one_and_a_level_as_given() {
        assert_eq!(
            its_place(1, 3),
            vec![
                ("posinset".to_owned(), "2".to_owned()),
                ("setsize".to_owned(), "3".to_owned())
            ]
        );
        assert_eq!(a_heading_at(2), ("level".to_owned(), "2".to_owned()));
        assert_eq!(
            told_together(&["a".to_owned(), "b".to_owned()]),
            "a\nb".to_owned()
        );
        assert_eq!(
            keys_together(&["Ctrl".to_owned(), "Space".to_owned()]),
            "Ctrl + Space".to_owned()
        );
    }
}
