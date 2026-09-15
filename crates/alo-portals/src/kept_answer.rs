//! One answer, as a line of the answers file.
//!
//! When it was answered, the application named — or nobody, for a caller that
//! could not be named — the portal, and what it was answered with
//! ([`KeptOutcome`]). `docs/contracts/portal-answers-file.md` is this shape for
//! people reading the file without this crate.
//!
//! **The application is checked again on the way in.** A line whose
//! `application` is not an identifier is not an answer this backend could have
//! written, and is read back as a line that did not read rather than as an
//! application nobody can find.

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::answered::Answered;
use crate::kept_outcome::KeptOutcome;
use crate::portal::Portal;
use crate::request::identified;

/// One answer the backend gave, read back off the disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeptAnswer {
    /// When it was answered.
    at: SystemTime,
    /// The application that asked, when one was named.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    application: Option<String>,
    /// The portal it asked.
    portal: Portal,
    /// What it was answered with.
    #[serde(flatten)]
    outcome: KeptOutcome,
}

impl KeptAnswer {
    /// When it was answered.
    #[must_use]
    pub const fn at(&self) -> SystemTime {
        self.at
    }

    /// The application that asked, when one was named.
    #[must_use]
    pub fn application(&self) -> Option<&str> {
        self.application.as_deref()
    }

    /// The portal it asked.
    #[must_use]
    pub const fn portal(&self) -> Portal {
        self.portal
    }

    /// What it was answered with.
    #[must_use]
    pub const fn outcome(&self) -> &KeptOutcome {
        &self.outcome
    }

    /// The line this answer is, read — or [`None`] for a line that is not one.
    pub(crate) fn read(line: &str) -> Option<Self> {
        let answer: Self = serde_json::from_str(line).ok()?;
        match &answer.application {
            Some(named) if !identified(named).is_ok_and(|id| id.as_str() == named) => None,
            _ => Some(answer),
        }
    }

    /// This answer as one line, with no newline in it.
    pub(crate) fn line(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl From<&Answered> for KeptAnswer {
    fn from(answered: &Answered) -> Self {
        Self {
            at: answered.at(),
            application: answered
                .application()
                .map(|application| application.as_str().to_owned()),
            portal: answered.portal(),
            outcome: KeptOutcome::from(answered.outcome()),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::answered::{Outcome, Unanswered};
    use alo_capability::Applicant;
    use std::time::Duration;

    /// **A line is what it was**: time to the nanosecond, the application, the
    /// portal and the outcome — and nobody, for a caller nobody named, is a
    /// missing key rather than an invented name.
    #[test]
    fn an_answer_reads_back_as_it_was_written() {
        let at = SystemTime::UNIX_EPOCH + Duration::new(1_760_000_000, 123_456_789);
        for application in [Some(Applicant::named("org.gnome.Fractal")), None] {
            let answered = Answered::new(
                at,
                application.clone(),
                Portal::Secret,
                Outcome::Unanswered(Unanswered::KeyringUnavailable),
            );
            let kept = KeptAnswer::from(&answered);
            let line = kept.line().unwrap();
            assert!(!line.contains('\n'), "{line}");
            assert_eq!(
                line.contains("application"),
                application.is_some(),
                "{line}"
            );
            assert_eq!(KeptAnswer::read(&line), Some(kept));
        }
    }

    /// **A line naming something that is not an identifier does not read**,
    /// and neither does one missing what every answer has.
    #[test]
    fn a_line_no_backend_could_have_written_does_not_read() {
        for line in [
            r#"{"at":{"secs_since_epoch":1,"nanos_since_epoch":0},"application":"org.gnome Fractal","portal":"secret","answer":"unanswered","why":"not-identified"}"#,
            r#"{"at":{"secs_since_epoch":1,"nanos_since_epoch":0},"application":" org.gnome.Fractal","portal":"secret","answer":"unanswered","why":"not-identified"}"#,
            r#"{"at":{"secs_since_epoch":1,"nanos_since_epoch":0},"portal":"usb","answer":"unanswered","why":"not-identified"}"#,
            r#"{"at":{"secs_since_epoch":1,"nanos_since_epoch":0},"portal":"secret"}"#,
            r#"{"format":1}"#,
            "half a line",
        ] {
            assert_eq!(KeptAnswer::read(line), None, "{line}");
        }
        assert!(
            KeptAnswer::read(
                r#"{"at":{"secs_since_epoch":1,"nanos_since_epoch":0},"application":"org.gnome.Fractal","portal":"secret","answer":"unanswered","why":"not-identified"}"#
            )
            .is_some()
        );
    }
}
