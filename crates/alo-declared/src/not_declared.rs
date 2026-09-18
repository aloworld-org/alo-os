//! Why the verbs alo OS ships could not be put on one list.

/// A crate whose verbs would not declare into the one registry.
///
/// Not a refusal anybody reads in their own language, and deliberately: the
/// thing that has gone wrong is alo OS's own list of what it can do, so there is
/// nothing to ask. It cannot happen on a machine that shipped — the test in
/// `tests/` runs the whole list — and whoever reads it is whoever is looking for
/// a file to open.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("alo OS's own verbs are wrong: {list} would not declare — {why}")]
pub struct NotDeclared {
    /// Which crate's verbs.
    list: &'static str,
    /// What that crate's own declaration said about it.
    why: String,
}

impl NotDeclared {
    /// The refusal, as the crate that would not declare and what it said.
    #[must_use]
    pub fn of(list: &'static str, why: String) -> Self {
        Self { list, why }
    }

    /// Which crate's verbs would not declare.
    #[must_use]
    pub fn list(&self) -> &'static str {
        self.list
    }

    /// What that crate said was wrong with them.
    #[must_use]
    pub fn why(&self) -> &str {
        &self.why
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The refusal names the crate and quotes what it said, because its reader
    /// is looking for a file to open.
    #[test]
    fn a_list_that_will_not_declare_names_the_crate() {
        let refused = NotDeclared::of("alo-nothing", "two of them are called open_file".to_owned());
        assert_eq!(refused.list(), "alo-nothing");
        assert_eq!(refused.why(), "two of them are called open_file");
        assert!(refused.to_string().contains("alo-nothing"), "{refused}");
        assert!(refused.to_string().contains("open_file"), "{refused}");
    }
}
