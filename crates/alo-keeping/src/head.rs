//! The first line of a record, and where the record starts.
//!
//! Every alo OS record begins with one line that is not an entry. It says what
//! shape the file is in, and — once anything has ever been removed — the moment
//! the record now starts at and the rule that removed what came before.
//!
//! # Why the mark is here and not an entry
//!
//! Pruning removes evidence. The thing that must survive it is the fact that it
//! happened, or a record that lost its first six months answers *what did the
//! agent do in March* with *nothing*, and somebody believes it.
//!
//! An entry saying *this record was shortened* would be the obvious way to keep
//! that fact, and it is the wrong one: an entry has a moment, so a later prune
//! would age it out too, and after two rounds the record would look untouched
//! again. **A head cannot age out**, because it is not in the part of the file
//! that pruning walks. That is the whole reason this type exists rather than a
//! seventh `alo_record::Happened`.
//!
//! # The shape is a public surface
//!
//! `CLAUDE.md` names the image format and the update channel as public surfaces
//! that change additively. A record is one too: a security team's tooling, and
//! at v1 an export to their own console, read these files. So the first line
//! carries [`THE_FORMAT`], and a record written by a newer alo OS is **refused
//! rather than appended to** — adding a line in a shape this version does not
//! know would leave a file neither version can read.

use std::time::SystemTime;

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::keeping::Keeping;
use crate::words;

/// The shape of a record this version of alo OS writes and reads.
///
/// Additive changes keep this number. Anything that would stop an older alo OS
/// reading a record correctly raises it, and raising it is a deprecation with
/// the notice `CLAUDE.md` asks of a public surface.
pub const THE_FORMAT: u32 = 1;

/// What a record says about itself before it says what happened.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Head {
    /// Which shape the file is in.
    ///
    /// Required, and it is what tells a head from an entry: an entry has no
    /// such field, so a file whose first line is an entry is refused as not
    /// being a record rather than read as a record with no beginning.
    format: u32,
    /// The moment the record now starts at, where anything has been removed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    since: Option<SystemTime>,
    /// The rule that removed it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    under: Option<Keeping>,
}

impl Head {
    /// A record that has never had anything removed from it.
    pub(crate) fn new() -> Self {
        Self {
            format: THE_FORMAT,
            since: None,
            under: None,
        }
    }

    /// The same record, now starting at this moment because a rule removed what
    /// came before.
    ///
    /// The **later** of the two moments is kept, so a record shortened twice
    /// says where it starts now rather than where it started the first time.
    /// Nothing can move it back: a rule that removes less than the last one is
    /// a rule about what happens next, not a claim that what was removed has
    /// come back.
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

    /// This head as the first line of a file, with its newline.
    ///
    /// One place turns a head into a line, so the file a fresh record starts
    /// with and the file a shortened one is rewritten as cannot come apart.
    ///
    /// # Errors
    ///
    /// A `serde_json::Error`, which this type cannot cause: it holds a number,
    /// a moment and a rule, and none of those can fail to be written down. It
    /// is a `Result` rather than an unwrap because this runs inside the daemon.
    pub(crate) fn line(&self) -> Result<String, serde_json::Error> {
        let mut line = serde_json::to_string(self)?;
        line.push('\n');
        Ok(line)
    }

    /// Which shape the file is in.
    #[must_use]
    pub fn format(&self) -> u32 {
        self.format
    }

    /// The moment the record starts at, where anything has been removed.
    ///
    /// A date, so it is answered as a moment rather than written into the
    /// sentence beside it: how a date is written belongs to the reader's
    /// region, which is not the same thing as their language.
    #[must_use]
    pub fn since(&self) -> Option<SystemTime> {
        self.since
    }

    /// The rule that removed what came before.
    #[must_use]
    pub fn under(&self) -> Option<Keeping> {
        self.under
    }

    /// Whether this is still everything that ever happened.
    #[must_use]
    pub fn is_whole(&self) -> bool {
        self.since.is_none()
    }

    /// What this says, in the language the person reads.
    ///
    /// Two sentences, and which one is shown is the difference between *the
    /// agent did nothing in March* and *this record does not reach March*.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let key = if self.is_whole() {
            words::WHOLE.key()
        } else {
            words::SHORTENED.key()
        };
        strings.say(&key, &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, said};
    use std::time::Duration;

    /// A fixed moment, so that everything here is arithmetic.
    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// A fresh record says it is whole, and says so in words rather than by
    /// leaving a field empty for somebody to notice.
    #[test]
    fn a_fresh_record_says_nothing_has_been_removed_from_it() {
        let head = Head::new();
        assert_eq!(head.format(), THE_FORMAT);
        assert!(head.is_whole());
        assert_eq!(head.since(), None);
        assert_eq!(head.under(), None);
        assert!(said(&head.said(&in_english())).contains("nothing has been removed"));
    }

    /// **The sentence a shortened record shows is the point of the type.** It
    /// says the record does not go all the way back, so an absence in it is not
    /// read as an innocence.
    #[test]
    fn a_shortened_record_says_it_does_not_go_all_the_way_back() {
        let head = Head::new().shortened_to(noon(), Keeping::for_days(30).unwrap());
        assert!(!head.is_whole());
        assert_eq!(head.since(), Some(noon()));
        assert_eq!(head.under(), Keeping::for_days(30).ok());
        let message = said(&head.said(&in_english()));
        assert!(
            message.contains("does not go all the way back"),
            "{message}"
        );
    }

    /// Shortening twice moves the beginning forwards and never back, so a
    /// record cannot be made to claim it reaches further than it does.
    #[test]
    fn a_record_shortened_twice_starts_at_the_later_of_the_two_moments() {
        let day = Duration::from_secs(24 * 60 * 60);
        let thirty = Keeping::for_days(30).unwrap();
        let once = Head::new().shortened_to(noon(), thirty);
        let twice = once.shortened_to(noon() + day, thirty);
        assert_eq!(twice.since(), Some(noon() + day));

        // A rule that reaches back further does not move the beginning back:
        // what was removed is gone, and saying otherwise would be a claim about
        // evidence that is not there.
        let backwards = twice.shortened_to(noon() - day, Keeping::for_days(90).unwrap());
        assert_eq!(backwards.since(), Some(noon() + day));
        assert_eq!(backwards.under(), Keeping::for_days(90).ok());
    }

    /// The first line survives being written down and read back, which is the
    /// whole of its job.
    #[test]
    fn the_first_line_survives_being_written_down_and_read_back() {
        let head = Head::new().shortened_to(noon(), Keeping::for_days(7).unwrap());
        let written = serde_json::to_string(&head).unwrap();
        assert_eq!(serde_json::from_str::<Head>(&written).ok(), Some(head));

        // A fresh one writes only what it has to say, so a record that has
        // never been shortened has no field claiming it was.
        let fresh = serde_json::to_string(&Head::new()).unwrap();
        assert_eq!(fresh, "{\"format\":1}");
    }

    /// **A head and an entry cannot be mistaken for one another.** The format
    /// is required, so a file whose first line is an entry is refused as not
    /// being a record — rather than read as a record that has lost its
    /// beginning, which is the shape a truncated record would take.
    #[test]
    fn an_entry_cannot_be_read_as_a_beginning() {
        let entry = "{\"at\":{\"secs_since_epoch\":1,\"nanos_since_epoch\":0},\"happened\":{}}";
        assert!(serde_json::from_str::<Head>(entry).is_err());
        assert!(serde_json::from_str::<Head>("{}").is_err());
    }
}
