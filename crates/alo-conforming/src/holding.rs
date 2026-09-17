//! Every clause held to what it claims, against this repository as it is.
//!
//! The fifth in the family after `alo-reconciling` (promises against evidence),
//! `alo-by-hand` (verbs against their plain way), `alo-collected` (words against
//! the one vocabulary) and `alo-citing` (citations against the decisions that
//! exist). The shape is theirs: **the list is checked against the repository,
//! never against another list written by hand**, because two lists written by
//! hand only ever prove that two people agreed.
//!
//! # What is checked
//!
//! | | |
//! |---|---|
//! | A clause met by a test nobody wrote | [`Finding::ATestNobodyWrote`] |
//! | A clause waiting on a plan nobody wrote | [`Finding::APlanNobodyWrote`] |
//! | A clause waiting on a task that plan does not have | [`Finding::ATaskThatPlanDoesNotHave`] |
//! | Two clauses claiming one number | [`Finding::TwoClausesOneNumber`] |
//! | A number that is not one | [`Finding::ANumberThatIsNotOne`] |
//! | A requirement that is not one sentence | [`Finding::MoreThanOneSentence`] |
//! | A reason that says nothing | [`Finding::ADismissalWithNoReason`] |

use std::collections::BTreeMap;

use crate::clause::Clause;
use crate::standing::{Evidence, Standing, Waiting};

/// What the whole list adds up to, once every clause has been held to what it
/// claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Held {
    /// How many clauses there are.
    pub clauses: usize,
    /// How many are met by a test in this workspace.
    pub met: usize,
    /// How many are waiting on a task somebody has written down.
    pub waiting: usize,
    /// How many are not this machine's to answer.
    pub not_applicable: usize,
    /// How many have been read against the standard's own text by a person.
    ///
    /// Nought, today, and `crate::standard` says why that is written down.
    pub checked_against_the_text: usize,
}

/// What is wrong with a clause, or with the list.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Finding {
    /// A clause says a test meets it and no such test is in the workspace.
    #[error(
        "clause {number} is met by {test} in {at}, and this repository has no such test — a \
         clause met by a name nobody wrote is a clause met by a sentence"
    )]
    ATestNobodyWrote {
        /// The clause.
        number: &'static str,
        /// The test it named.
        test: &'static str,
        /// Where it said the test was.
        at: String,
    },
    /// A clause waits on a plan that is not in this repository.
    #[error("clause {number} waits on task {task} of {plan}, and there is no such plan")]
    APlanNobodyWrote {
        /// The clause.
        number: &'static str,
        /// The task it named.
        task: u32,
        /// The plan it named.
        plan: &'static str,
    },
    /// A clause waits on a task its plan does not have.
    #[error("clause {number} waits on task {task} of {plan}, and that plan has no such task")]
    ATaskThatPlanDoesNotHave {
        /// The clause.
        number: &'static str,
        /// The task it named.
        task: u32,
        /// The plan it named.
        plan: &'static str,
    },
    /// Two clauses under one number.
    #[error("{number} is claimed by two clauses")]
    TwoClausesOneNumber {
        /// The number.
        number: &'static str,
    },
    /// Something that is not a clause number.
    #[error("{number} is not a clause number")]
    ANumberThatIsNotOne {
        /// What was written.
        number: &'static str,
    },
    /// A requirement that is not one sentence a reader can take in.
    #[error("clause {number}'s requirement is not one sentence: {requirement}")]
    MoreThanOneSentence {
        /// The clause.
        number: &'static str,
        /// What was written.
        requirement: &'static str,
    },
    /// A clause dismissed without a reason.
    #[error("clause {number} is not applicable and does not say why")]
    ADismissalWithNoReason {
        /// The clause.
        number: &'static str,
    },
}

/// **Hold every clause to what it claims.**
///
/// `tests` are this repository's files that may hold a test — each as its path
/// from the root and its contents. `plans` are the plan documents, the same
/// way. Neither is a list this crate keeps: the test walks the repository and
/// hands over what it found.
///
/// # Errors
/// Every [`Finding`], so that a reader fixes them in one pass rather than one
/// run each.
pub fn held(
    clauses: &[Clause],
    tests: &[(String, String)],
    plans: &[(String, String)],
) -> Result<Held, Vec<Finding>> {
    let mut findings = Vec::new();
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();

    for clause in clauses {
        *seen.entry(clause.number).or_default() += 1;
        if !is_a_number(clause.number) {
            findings.push(Finding::ANumberThatIsNotOne {
                number: clause.number,
            });
        }
        if is_more_than_one_sentence(clause.requirement) {
            findings.push(Finding::MoreThanOneSentence {
                number: clause.number,
                requirement: clause.requirement,
            });
        }
        match clause.standing {
            Standing::Met { by } => {
                if !a_test_is_there(&by, tests) {
                    findings.push(Finding::ATestNobodyWrote {
                        number: clause.number,
                        test: by.test,
                        at: by.at(),
                    });
                }
            }
            Standing::NotYet { because } => {
                findings.extend(the_task_is_there(clause.number, because, plans));
            }
            Standing::NotApplicable { because } => {
                if because.trim().is_empty() {
                    findings.push(Finding::ADismissalWithNoReason {
                        number: clause.number,
                    });
                }
            }
        }
    }

    for (number, how_many) in seen {
        if how_many > 1 {
            findings.push(Finding::TwoClausesOneNumber { number });
        }
    }

    if !findings.is_empty() {
        return Err(findings);
    }
    Ok(Held {
        clauses: clauses.len(),
        met: clauses.iter().filter(|clause| clause.is_met()).count(),
        waiting: clauses.iter().filter(|clause| clause.is_waiting()).count(),
        not_applicable: clauses
            .iter()
            .filter(|clause| clause.is_not_applicable())
            .count(),
        checked_against_the_text: clauses
            .iter()
            .filter(|clause| clause.checked.against_the_text())
            .count(),
    })
}

/// Whether a test by this name is in the file this clause named.
fn a_test_is_there(evidence: &Evidence, tests: &[(String, String)]) -> bool {
    let at = evidence.at();
    tests
        .iter()
        .any(|(path, text)| path == &at && text.contains(&format!("fn {}(", evidence.test)))
}

/// Whether the plan is there and has the task, as findings.
fn the_task_is_there(
    number: &'static str,
    waiting: Waiting,
    plans: &[(String, String)],
) -> Vec<Finding> {
    let Some((_, text)) = plans.iter().find(|(path, _)| path == waiting.plan) else {
        return vec![Finding::APlanNobodyWrote {
            number,
            task: waiting.task,
            plan: waiting.plan,
        }];
    };
    if text.contains(&format!("### {}.", waiting.task)) {
        return Vec::new();
    }
    vec![Finding::ATaskThatPlanDoesNotHave {
        number,
        task: waiting.task,
        plan: waiting.plan,
    }]
}

/// Whether this is a clause number: digits, in parts, separated by dots.
fn is_a_number(number: &str) -> bool {
    let parts: Vec<&str> = number.split('.').collect();
    parts.len() >= 2
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|digit| digit.is_ascii_digit()))
}

/// Whether a requirement is more than the one sentence it is allowed.
///
/// One full stop, at the end. A reader comparing forty of these against a
/// document reads each at a glance or reads none of them.
fn is_more_than_one_sentence(requirement: &str) -> bool {
    let requirement = requirement.trim();
    requirement.is_empty()
        || !requirement.ends_with('.')
        || requirement.trim_end_matches('.').contains(". ")
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::standing::{Checked::NotAgainstTheText, Evidence};

    fn a_clause(number: &'static str, standing: Standing) -> Clause {
        Clause {
            number,
            requirement: "Something must be true.",
            standing,
            checked: NotAgainstTheText,
        }
    }

    fn a_test() -> (String, String) {
        (
            "crates/alo-access/src/tree.rs".to_owned(),
            "#[test]\nfn a_surface_says_what_it_is() { }".to_owned(),
        )
    }

    fn a_plan() -> (String, String) {
        (
            "docs/autonomy/a-plan.md".to_owned(),
            "### 3. Something\n\n**Status:** ready.".to_owned(),
        )
    }

    /// **A clause met by a test that exists holds; one met by a name nobody
    /// wrote does not.**
    #[test]
    fn a_clause_cannot_be_met_by_a_name_nobody_wrote() {
        let met = a_clause(
            "11.4.1.2",
            Standing::Met {
                by: Evidence {
                    crate_named: "alo-access",
                    file: "src/tree.rs",
                    test: "a_surface_says_what_it_is",
                },
            },
        );
        assert_eq!(
            held(&[met], &[a_test()], &[a_plan()]).map(|held| held.met),
            Ok(1)
        );

        let invented = a_clause(
            "11.4.1.2",
            Standing::Met {
                by: Evidence {
                    crate_named: "alo-access",
                    file: "src/tree.rs",
                    test: "a_test_nobody_wrote",
                },
            },
        );
        let findings = held(&[invented], &[a_test()], &[a_plan()]).expect_err("no such test");
        assert!(matches!(
            findings.first(),
            Some(Finding::ATestNobodyWrote { .. })
        ));
    }

    /// **Not yet is a task in a plan, and both have to be there.**
    #[test]
    fn not_yet_names_a_task_somebody_can_go_and_read() {
        let waiting = a_clause(
            "11.2.1.1",
            Standing::NotYet {
                because: Waiting {
                    task: 3,
                    plan: "docs/autonomy/a-plan.md",
                },
            },
        );
        assert_eq!(
            held(&[waiting], &[a_test()], &[a_plan()]).map(|held| held.waiting),
            Ok(1)
        );

        let elsewhere = a_clause(
            "11.2.1.1",
            Standing::NotYet {
                because: Waiting {
                    task: 3,
                    plan: "docs/autonomy/no-such-plan.md",
                },
            },
        );
        assert!(matches!(
            held(&[elsewhere], &[a_test()], &[a_plan()])
                .expect_err("no such plan")
                .first(),
            Some(Finding::APlanNobodyWrote { .. })
        ));

        let missing = a_clause(
            "11.2.1.1",
            Standing::NotYet {
                because: Waiting {
                    task: 9,
                    plan: "docs/autonomy/a-plan.md",
                },
            },
        );
        assert!(matches!(
            held(&[missing], &[a_test()], &[a_plan()])
                .expect_err("no such task")
                .first(),
            Some(Finding::ATaskThatPlanDoesNotHave { .. })
        ));
    }

    /// **A number is a number, a requirement is one sentence, and a dismissal
    /// has a reason.**
    #[test]
    fn the_shape_of_a_row_is_held_as_well_as_what_it_claims() {
        let not_a_number = a_clause(
            "eleven",
            Standing::NotApplicable {
                because: "a reason",
            },
        );
        assert!(matches!(
            held(&[not_a_number], &[], &[])
                .expect_err("not a number")
                .first(),
            Some(Finding::ANumberThatIsNotOne { .. })
        ));

        let mut two_sentences = a_clause(
            "5.2",
            Standing::NotApplicable {
                because: "a reason",
            },
        );
        two_sentences.requirement = "One thing. And another thing.";
        assert!(matches!(
            held(&[two_sentences], &[], &[])
                .expect_err("two sentences")
                .first(),
            Some(Finding::MoreThanOneSentence { .. })
        ));

        let dismissed = a_clause("5.3", Standing::NotApplicable { because: "   " });
        assert!(matches!(
            held(&[dismissed], &[], &[]).expect_err("no reason").first(),
            Some(Finding::ADismissalWithNoReason { .. })
        ));

        let twice = [
            a_clause(
                "5.4",
                Standing::NotApplicable {
                    because: "a reason",
                },
            ),
            a_clause(
                "5.4",
                Standing::NotApplicable {
                    because: "a reason",
                },
            ),
        ];
        assert!(
            held(&twice, &[], &[])
                .expect_err("two clauses, one number")
                .iter()
                .any(|finding| matches!(finding, Finding::TwoClausesOneNumber { .. }))
        );
    }
}
