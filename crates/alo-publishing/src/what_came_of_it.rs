//! **What a publication's command did, read back as what the person is told** —
//! from the number it ended with ([`mod@crate::the_line`]'s table) and the lines it
//! printed.
//!
//! What a tool printed is passed on as its last few lines, in its own English,
//! because that is the part of a failure nobody here can word better than the
//! tool did — and cut short, because a reason is a sentence and not a log.

use alo_capability::{ItDidNotWork, WhatACommandDid};

use crate::not_published::NotPublished;
use crate::publication::{Publication, Published};
use crate::the_line::{MADE_THE_REPOSITORY, NOTHING_NEW, PUBLISHED, SIGNED_IN_AS};

/// The most of what a tool said that is carried into a reason, in characters.
const THE_MOST_OF_WHAT_IT_SAID: usize = 400;

/// How many of its last lines are carried.
const ITS_LAST_LINES: usize = 3;

/// What came of running `publication`'s command.
///
/// # Errors
/// [`NotPublished`], naming the step that failed.
pub fn what_came_of_it(
    publication: &Publication,
    did: &WhatACommandDid,
) -> Result<Published, NotPublished> {
    let said = did.said();
    let printed = |marker: &str| said.lines().any(|line| line.trim() == marker);
    match ItDidNotWork::of(did.returned(), did.took_too_long()) {
        None if printed(NOTHING_NEW) => Ok(Published::of(
            publication,
            true,
            printed(MADE_THE_REPOSITORY),
        )),
        None if printed(PUBLISHED) => Ok(Published::of(
            publication,
            false,
            printed(MADE_THE_REPOSITORY),
        )),
        // **Ended well and said neither**, which the line cannot do: told as
        // the unexpected end it is, never as published.
        None => Err(NotPublished::EndedWith {
            status: 0,
            said: its_last_lines(said),
        }),
        Some(ItDidNotWork::TookTooLong) => Err(NotPublished::TookTooLong),
        Some(ItDidNotWork::WasStopped) => Err(NotPublished::WasStopped),
        Some(ItDidNotWork::EndedWith(status)) => Err(failed_at(status, publication, said)),
        // Not answered by [`ItDidNotWork::of`]; read from the status below
        // like every other failure.
        Some(ItDidNotWork::NotSignedInToGitHub) => Err(NotPublished::NotSignedIn),
    }
}

/// Which step ended the run with `status`.
fn failed_at(status: i32, publication: &Publication, said: &str) -> NotPublished {
    match status {
        70 => NotPublished::NoGit,
        71 => NotPublished::NoGh,
        // **Asked of what `gh` said**, by the command road's own reading, so a
        // network that is down is not told as a sign-in that is missing.
        72 if alo_capability::not_signed_in_to_github::not_signed_in_to_github(said) => {
            NotPublished::NotSignedIn
        }
        72 => NotPublished::GitHubDidNotAnswer {
            said: its_last_lines(said),
        },
        73 => NotPublished::SignedInAsAnother {
            signed_in: said
                .lines()
                .find_map(|line| line.trim().strip_prefix(SIGNED_IN_AS))
                .unwrap_or_default()
                .to_owned(),
            account: publication.account().name().to_owned(),
        },
        75 => NotPublished::NoRepository {
            account: publication.account().name().to_owned(),
            said: its_last_lines(said),
        },
        76 => NotPublished::NoRoom,
        77 => NotPublished::NotPrepared {
            said: its_last_lines(said),
        },
        78 => NotPublished::Refused {
            said: its_last_lines(said),
        },
        79 => NotPublished::Empty,
        status => NotPublished::EndedWith {
            status,
            said: its_last_lines(said),
        },
    }
}

/// The last few lines a tool printed, on one line and cut short — leaving out
/// the run's own markers, which are not the tool's words.
pub(crate) fn its_last_lines(said: &str) -> String {
    let lines: Vec<&str> = said
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("alo-"))
        .collect();
    let from = lines.len().saturating_sub(ITS_LAST_LINES);
    let joined = lines.get(from..).unwrap_or_default().join(" ");
    if joined.chars().count() <= THE_MOST_OF_WHAT_IT_SAID {
        return joined;
    }
    let kept: String = joined.chars().take(THE_MOST_OF_WHAT_IT_SAID).collect();
    format!("{kept}…")
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::account::Account;
    use std::path::Path;

    fn anna() -> Publication {
        Publication::of(
            Path::new("/home/anna/site"),
            Account::named("anna").unwrap(),
        )
    }

    fn ended(status: Option<i32>, said: &str) -> Result<Published, NotPublished> {
        what_came_of_it(
            &anna(),
            &WhatACommandDid::of(status, said.to_owned(), false, false),
        )
    }

    #[test]
    fn a_run_that_published_is_published_and_says_whether_it_made_the_repository() {
        let made = ended(Some(0), "alo-made-the-repository\nalo-published\n").unwrap();
        assert!(!made.nothing_new());
        assert!(made.made_the_repository());
        assert_eq!(made.address(), "https://anna.github.io/");
        let same = ended(Some(0), "alo-nothing-new\n").unwrap();
        assert!(same.nothing_new());
        assert!(!same.made_the_repository());
    }

    /// **The refusal path, step by step**: every number the line ends with is
    /// its own reason, and nothing that did not print *published* is
    /// published.
    #[test]
    fn each_step_that_fails_is_its_own_reason_and_nothing_else_is_published() {
        assert_eq!(ended(Some(70), ""), Err(NotPublished::NoGit));
        assert_eq!(ended(Some(71), ""), Err(NotPublished::NoGh));
        assert_eq!(
            ended(
                Some(72),
                "To get started with GitHub CLI, please run:  gh auth login\n"
            ),
            Err(NotPublished::NotSignedIn)
        );
        assert_eq!(
            ended(Some(72), "error connecting to api.github.com\n"),
            Err(NotPublished::GitHubDidNotAnswer {
                said: "error connecting to api.github.com".to_owned()
            })
        );
        assert_eq!(
            ended(Some(73), "alo-signed-in-as:Bob\n"),
            Err(NotPublished::SignedInAsAnother {
                signed_in: "Bob".to_owned(),
                account: "anna".to_owned()
            })
        );
        assert!(matches!(
            ended(Some(75), "HTTP 422"),
            Err(NotPublished::NoRepository { .. })
        ));
        assert_eq!(ended(Some(76), ""), Err(NotPublished::NoRoom));
        assert!(matches!(
            ended(Some(77), "fatal"),
            Err(NotPublished::NotPrepared { .. })
        ));
        assert!(matches!(
            ended(Some(78), "! [rejected]"),
            Err(NotPublished::Refused { .. })
        ));
        assert_eq!(ended(Some(79), ""), Err(NotPublished::Empty));
        assert!(matches!(
            ended(Some(1), "odd"),
            Err(NotPublished::EndedWith { status: 1, .. })
        ));
        assert_eq!(ended(None, "alo-published"), Err(NotPublished::WasStopped));
        assert!(matches!(
            ended(Some(0), "something else"),
            Err(NotPublished::EndedWith { status: 0, .. })
        ));
        assert_eq!(
            what_came_of_it(
                &anna(),
                &WhatACommandDid::of(None, "alo-published".to_owned(), false, true)
            ),
            Err(NotPublished::TookTooLong)
        );
    }

    #[test]
    fn what_a_tool_said_is_its_last_lines_cut_short_without_the_runs_own_markers() {
        assert_eq!(
            its_last_lines("one\n\nalo-made-the-repository\ntwo\nthree\nfour\n"),
            "two three four"
        );
        let long = "x".repeat(1000);
        let cut = its_last_lines(&long);
        assert_eq!(cut.chars().count(), THE_MOST_OF_WHAT_IT_SAID + 1);
        assert!(cut.ends_with('…'));
    }
}
