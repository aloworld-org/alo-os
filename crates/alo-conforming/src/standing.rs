//! Where a clause stands, and the evidence it stands on.
//!
//! Three standings and no fourth, and the whole design is in what each one is
//! **made of**:
//!
//! - [`Standing::Met`] carries an [`Evidence`] — a crate, a file and the name of
//!   a test in it. It cannot be made out of a sentence, because there is no
//!   variant that takes one. `tests/every_clause_that_is_met_names_a_test_that_exists.rs`
//!   then reads this repository and refuses any that names a test nobody wrote.
//! - [`Standing::NotYet`] carries a [`Waiting`] — a task number and the plan it
//!   is in — so *not yet* is a thing somebody can go and read, not a promise.
//! - [`Standing::NotApplicable`] carries the reason, because a clause dismissed
//!   without one is the easiest way to pass a standard and the least honest.
//!
//! **No clause may be met by a sentence.** That is the acceptance this crate was
//! written for, and it is held by the type rather than by a rule somebody has to
//! remember.

/// **A test in this workspace**: what a clause being met is made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Evidence {
    /// The crate the test is in, as its directory is named.
    pub crate_named: &'static str,
    /// The file, from the crate's own directory.
    pub file: &'static str,
    /// The test's own name.
    pub test: &'static str,
}

impl Evidence {
    /// Where the file is, from the root of the repository.
    #[must_use]
    pub fn at(&self) -> String {
        format!("crates/{}/{}", self.crate_named, self.file)
    }
}

/// **A task that has not been done**: what *not yet* is made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Waiting {
    /// The task's number in its plan.
    pub task: u32,
    /// The plan, from the root of the repository.
    pub plan: &'static str,
}

/// **Where a clause stands.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Standing {
    /// Met, by a test in this workspace.
    Met {
        /// The test.
        by: Evidence,
    },
    /// Not yet, because a task nobody has done.
    NotYet {
        /// The task.
        because: Waiting,
    },
    /// Not applicable, and why.
    NotApplicable {
        /// The reason, in this repository's own words.
        because: &'static str,
    },
}

/// **Whether a clause's number and reading have been checked against the
/// published text** by a person with the standard in front of them.
///
/// Every clause in this crate says [`Checked::NotAgainstTheText`] today. See
/// `crate::standard` for why that is written down rather than left to be
/// assumed: a list to check is useful, and a list that looks checked and is not
/// is worse than nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Checked {
    /// Nobody has read this clause against the standard's own text.
    NotAgainstTheText,
    /// Somebody has, and is named with when they did it.
    By {
        /// Who read it, and when.
        person: &'static str,
    },
}

impl Checked {
    /// Whether somebody has read this against the text.
    #[must_use]
    pub const fn against_the_text(self) -> bool {
        matches!(self, Self::By { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Evidence names a place a reader can open.**
    #[test]
    fn evidence_is_a_file_in_this_repository() {
        let evidence = Evidence {
            crate_named: "alo-access",
            file: "src/tree.rs",
            test: "every_surface_says_what_it_is_and_what_is_in_it",
        };
        assert_eq!(evidence.at(), "crates/alo-access/src/tree.rs");
    }

    /// **Nothing has been checked against the text yet**, and the type says
    /// which is which.
    #[test]
    fn checked_says_whether_somebody_has_read_the_standard_itself() {
        assert!(!Checked::NotAgainstTheText.against_the_text());
        assert!(
            Checked::By {
                person: "somebody, one day"
            }
            .against_the_text()
        );
    }
}
