//! Whether an object path, an interface or a method is a well-formed name.
//!
//! The rules are the D-Bus specification's, checked when an adapter is loaded
//! so that a malformed declaration is refused to its author rather than
//! discovered by the bus when a person has already approved something.

/// The longest an interface or a method name may be.
const LONGEST: usize = 255;

/// One element: a letter or underscore, then letters, digits and underscores.
fn is_an_element(element: &str) -> bool {
    element
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && element
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// An object path: `/`, or `/` followed by non-empty elements of letters,
/// digits and underscores separated by `/`.
#[must_use]
pub fn is_an_object(path: &str) -> bool {
    if path == "/" {
        return true;
    }
    path.strip_prefix('/').is_some_and(|rest| {
        rest.split('/').all(|element| {
            !element.is_empty()
                && element
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
    })
}

/// An interface: two or more elements separated by dots.
#[must_use]
pub fn is_an_interface(interface: &str) -> bool {
    interface.len() <= LONGEST
        && interface.split('.').count() >= 2
        && interface.split('.').all(is_an_element)
}

/// A method: one element.
#[must_use]
pub fn is_a_method(method: &str) -> bool {
    method.len() <= LONGEST && is_an_element(method)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_formed_names_are_and_others_are_not() {
        assert!(is_an_object("/org/gnome/TextEditor"));
        assert!(is_an_object("/"));
        for not in [
            "",
            "org/gnome",
            "/org//gnome",
            "/org/gnome/",
            "/org/gnome-x",
        ] {
            assert!(!is_an_object(not), "{not}");
        }
        assert!(is_an_interface("org.freedesktop.Application"));
        for not in ["", "org", "org..x", "org.1x", "org.x-y"] {
            assert!(!is_an_interface(not), "{not}");
        }
        assert!(is_a_method("ActivateAction"));
        for not in ["", "Open.Now", "1Open", "Open Now"] {
            assert!(!is_a_method(not), "{not}");
        }
    }
}
