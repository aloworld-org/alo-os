//! The first line of the answers file: which format it is in, and — once a
//! shortening has removed anything — where it now starts and under which rule.
//!
//! The agent's record says the same two things in its own first line
//! (`docs/contracts/record-file.md`, `since` and `under`), for its reason: a
//! file whose first months have aged out and a file for a machine where no
//! application asked anything are otherwise the same file, and somebody
//! believes the second. **The mark is in the first line rather than an answer**
//! because an answer has a moment, and a later shortening would age a mark with
//! a moment out too.
//!
//! `alo_keeping::Head` is that crate's own line and is made only inside it, so
//! this is the answers file's, over the same two fields and with the same rule
//! for moving `since`: forwards, never back.

use std::path::Path;
use std::time::SystemTime;

use alo_keeping::Keeping;
use serde::{Deserialize, Serialize};

use crate::not_recorded::NotRecorded;

/// The format this backend writes, and the newest it adds to.
pub const THE_ANSWERS_FORMAT: u64 = 1;

/// What the answers file says about itself before any answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AnswersHead {
    /// Which shape the file is in. Required: it is what tells the first line
    /// from an answer, which has no such field.
    format: u64,
    /// The moment the file now starts at, where anything has been removed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    since: Option<SystemTime>,
    /// The rule that removed it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    under: Option<Keeping>,
}

impl AnswersHead {
    /// The first line of a file nothing has been removed from.
    pub(crate) const fn new() -> Self {
        Self {
            format: THE_ANSWERS_FORMAT,
            since: None,
            under: None,
        }
    }

    /// The first line `first`, read — refused unless it is a format line in a
    /// format this backend adds to.
    ///
    /// # Errors
    /// [`NotRecorded::NotAnAnswersFile`] for a line that is not a format line,
    /// or names format `0`; [`NotRecorded::ANewerFormat`] for one newer than
    /// [`THE_ANSWERS_FORMAT`].
    pub(crate) fn read(at: &Path, first: &str) -> Result<Self, NotRecorded> {
        let Ok(head) = serde_json::from_str::<Self>(first.trim_end_matches('\n')) else {
            return Err(NotRecorded::NotAnAnswersFile { at: at.to_owned() });
        };
        if head.format > THE_ANSWERS_FORMAT {
            return Err(NotRecorded::ANewerFormat {
                at: at.to_owned(),
                format: head.format,
            });
        }
        if head.format == 0 {
            return Err(NotRecorded::NotAnAnswersFile { at: at.to_owned() });
        }
        Ok(head)
    }

    /// The same file, now starting at `since` because `under` removed what
    /// came before.
    ///
    /// The **later** of the two moments is kept, so a file shortened twice says
    /// where it starts now, and a rule reaching further back than the last one
    /// cannot make the file claim what it removed has come back.
    pub(crate) fn shortened_to(&self, since: SystemTime, under: Keeping) -> Self {
        Self {
            format: self.format,
            since: Some(match self.since {
                Some(already) if already > since => already,
                _ => since,
            }),
            under: Some(under),
        }
    }

    /// This head as the first line of the file, with its newline.
    ///
    /// # Errors
    /// A `serde_json::Error`, which a number, a moment and a rule cannot cause;
    /// a `Result` rather than an unwrap because this runs inside the backend.
    pub(crate) fn line(&self) -> Result<String, serde_json::Error> {
        let mut line = serde_json::to_string(self)?;
        line.push('\n');
        Ok(line)
    }

    /// The moment the file starts at, where anything has been removed.
    pub(crate) const fn since(&self) -> Option<SystemTime> {
        self.since
    }

    /// The rule that removed what came before.
    pub(crate) const fn under(&self) -> Option<Keeping> {
        self.under
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// **A fresh file's first line is exactly what task 8 wrote**, so nothing
    /// written before this change reads differently after it.
    #[test]
    fn a_file_nothing_was_removed_from_begins_as_it_always_did() {
        assert_eq!(AnswersHead::new().line().unwrap(), "{\"format\":1}\n");
        let read = AnswersHead::read(Path::new("a"), "{\"format\":1}").unwrap();
        assert_eq!(read, AnswersHead::new());
        assert_eq!(read.since(), None);
        assert_eq!(read.under(), None);
    }

    /// **`since` moves forwards and never back**, and the rule named is the
    /// last one that removed anything.
    #[test]
    fn a_file_shortened_twice_starts_at_the_later_moment() {
        let day = Duration::from_secs(24 * 60 * 60);
        let week = Keeping::for_days(7).unwrap();
        let once = AnswersHead::new().shortened_to(noon(), week);
        let twice = once.shortened_to(noon() + day, week);
        assert_eq!(twice.since(), Some(noon() + day));
        let backwards = twice.shortened_to(noon() - day, Keeping::for_days(90).unwrap());
        assert_eq!(backwards.since(), Some(noon() + day));
        assert_eq!(backwards.under(), Keeping::for_days(90).ok());
    }

    /// **The line survives being written and read back**, in the notation the
    /// agent's record writes the same two fields in.
    #[test]
    fn a_shortened_head_reads_back_as_it_was_written() {
        let head = AnswersHead::new().shortened_to(noon(), Keeping::for_days(7).unwrap());
        let line = head.line().unwrap();
        assert_eq!(
            line,
            "{\"format\":1,\"since\":{\"secs_since_epoch\":1760000000,\"nanos_since_epoch\":0},\"under\":{\"for-days\":7}}\n"
        );
        assert_eq!(AnswersHead::read(Path::new("a"), &line).unwrap(), head);
    }

    /// **An answer is never read as a first line**, nor format `0`, nor a
    /// newer format.
    #[test]
    fn what_is_not_a_format_line_this_reads_is_refused() {
        let at = Path::new("a");
        for line in [
            r#"{"at":{"secs_since_epoch":1,"nanos_since_epoch":0},"portal":"secret"}"#,
            "{}",
            "{\"format\":0}",
            "{\"format\":1,\"under\":{\"for-days\":0}}",
            "half",
        ] {
            assert!(
                matches!(
                    AnswersHead::read(at, line),
                    Err(NotRecorded::NotAnAnswersFile { .. })
                ),
                "{line}"
            );
        }
        assert!(matches!(
            AnswersHead::read(at, "{\"format\":2}"),
            Err(NotRecorded::ANewerFormat { format: 2, .. })
        ));
    }
}
