//! The words on the compositor's door for **`alo-tree` — where alo OS's own
//! surfaces are on the accessibility bus**.
//!
//! ```text
//! → alo-tree
//! ← at :1.42                                   (or: none)
//! ```
//!
//! alo's shell is not sandboxed, so nothing about its tree on the bus says
//! *this is alo* that another unsandboxed program could not also say. The
//! compositor publishes the tree itself, so it is the one that can say which
//! connection it is — and this door admits only the person's own login. A
//! person's command reaching *Settings* or *the record* reaches them at the name
//! answered here and nowhere else.

use crate::request::Request;

/// The act, alone on its line.
pub const ALO_TREE: &str = "alo-tree";

/// Whether a line is the `alo-tree` act.
#[must_use]
pub fn is_alo_tree(line: &str) -> bool {
    line.trim_end_matches(['\r', '\n']) == ALO_TREE
}

/// What the compositor answers to `alo-tree`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhereAlosTreeIs {
    /// The unique name, on the accessibility bus, the shell's tree answers as.
    At(String),
    /// The shell publishes no tree: no accessibility bus answered when the
    /// session began.
    Nowhere,
}

impl WhereAlosTreeIs {
    /// The tree at this unique bus name — [`WhereAlosTreeIs::Nowhere`] when
    /// it is not one: a unique name begins `:` and holds only letters, digits,
    /// `.`, `_` and `-`, at most 255 of them.
    #[must_use]
    pub fn at(name: &str) -> Self {
        let well_formed = name.len() <= 255
            && name.len() > 1
            && name.starts_with(':')
            && name
                .bytes()
                .skip(1)
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte));
        if well_formed {
            Self::At(name.to_owned())
        } else {
            Self::Nowhere
        }
    }

    /// The line, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        match self {
            Self::At(name) => format!("at {name}"),
            Self::Nowhere => "none".to_owned(),
        }
    }

    /// One line, read; [`None`] for anything else.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        let line = line.trim_end_matches(['\r', '\n']);
        if line.len() > Request::THE_MOST {
            return None;
        }
        if line == "none" {
            return Some(Self::Nowhere);
        }
        let name = line.strip_prefix("at ")?;
        match Self::at(name) {
            Self::At(name) => Some(Self::At(name)),
            Self::Nowhere => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_unique_name_is_written_and_read_back() {
        let at = WhereAlosTreeIs::at(":1.42");
        assert_eq!(at.written(), "at :1.42");
        assert_eq!(WhereAlosTreeIs::read("at :1.42\n"), Some(at));
        assert_eq!(
            WhereAlosTreeIs::read("none"),
            Some(WhereAlosTreeIs::Nowhere)
        );
        assert!(is_alo_tree("alo-tree\n"));
        assert!(!is_alo_tree("alo-tree org.a.B"));
    }

    /// **Only a unique name is an answer**: a well-known name could be claimed
    /// by any program, which is the whole of what this act exists to avoid.
    #[test]
    fn a_name_that_is_not_unique_is_never_an_answer() {
        for not in ["org.a11y.atspi.Registry", ":", ":1.42 x", ":1/2", ""] {
            assert_eq!(WhereAlosTreeIs::at(not), WhereAlosTreeIs::Nowhere, "{not}");
        }
        for not in ["at org.alo.Shell", "at", "at :1 2", ":1.42", "acted"] {
            assert_eq!(WhereAlosTreeIs::read(not), None, "{not}");
        }
    }
}
