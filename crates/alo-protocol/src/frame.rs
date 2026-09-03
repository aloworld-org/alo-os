//! One message, as it arrives: one line, a length, and a format number.
//!
//! Everything here is about the envelope rather than about the request inside
//! it, and all three of its rules exist because the thing reading them is a
//! privileged service that anything on the machine can talk to.
//!
//! # One message is one line
//!
//! A door that accepted more than one message at a time would answer the first
//! and throw the rest away, and *silently doing four fifths of what was asked*
//! is the failure a capability model is least able to survive. So a line break
//! inside a message is a refusal, and the one at the end that a line reader
//! leaves behind is not part of the message at all.
//!
//! # A message has a length before it has a meaning
//!
//! [`LONGEST`] is checked first, before anything is parsed, because a client
//! that can make `alo-agentd` allocate without a bound has taken the person's
//! machine away from them without ever being granted anything.
//!
//! # The format is read before the message
//!
//! A message from a newer alo OS is unreadable to this one *and* is worth a
//! different sentence from a message that is gibberish, so the number is read
//! first, out of a shape that tolerates fields this version has never heard of.
//! Reading the whole message first would answer a client from next year with
//! *this machine could not read that*, which sends whoever is holding it to
//! look for a bug rather than for an update.
//!
//! `docs/contracts/daemon-protocol.md` is where the number's rules live, and
//! they are `docs/contracts/record-file.md`'s: additive change does not raise
//! it, because a request this version has never heard of is refused rather than
//! misread.
//!
//! # The three rules hold in both directions, and one number does not
//!
//! What goes back is one line, of a bounded length, stating the format it is
//! written in — the same envelope, because the reasons are the same and a
//! second set of rules for the way back is a second place to get them right.
//!
//! The length is where the two directions genuinely differ. A request carries a
//! verb's name, a few paths or a question somebody composed; an **answer** can
//! carry a file, and `alo-files` bounds a read at [`alo_files::MOST_READ`] — a
//! megabyte, which JSON escaping can multiply by six on a file made of control
//! characters. One bound for both would therefore have been a bound that a
//! legitimate read cannot fit inside, so a verb that succeeded would produce a
//! message no client is allowed to read. [`LONGEST_ANSWER`] is derived from
//! that number rather than guessed beside it, and a test builds the worst case
//! and measures it.

use serde::{Deserialize, Serialize};

use crate::asked::Asked;
use crate::refusing::NotUnderstood;
use crate::told::Told;

/// The format this alo OS writes and reads.
///
/// A public surface: `docs/contracts/daemon-protocol.md` says when it rises and
/// when it does not.
pub const FORMAT: u32 = 1;

/// The most a message may be, in bytes.
///
/// One mebibyte. The largest thing a request can carry is a question for a
/// model, which is prose an agent composed; everything else is a verb's name
/// and a few paths. A bound is not a guess about what a client needs, it is
/// what stops one from deciding how much of this machine's memory it may have.
///
/// Bytes rather than characters, because what is being bounded is what the
/// machine has to hold rather than what somebody wrote.
pub const LONGEST: usize = 1024 * 1024;

/// The most an answer may be, in bytes.
///
/// Eight mebibytes, and the number is derived rather than chosen. The largest
/// thing an answer can carry is a file's contents, which `alo-files` bounds at
/// [`alo_files::MOST_READ`]; JSON writes a control character as six bytes, so
/// a file of a megabyte of them is six megabytes on the wire. Eight leaves the
/// envelope, the longest listing and the longest search comfortably inside it,
/// and `a_worst_case_read_fits_inside_the_bound` is the test that says so
/// rather than the sentence.
///
/// **Nothing this machine can answer with is longer than this**, so a reply
/// that exceeds it did not come from an alo OS verb — which is what a client
/// refusing it is really refusing.
pub const LONGEST_ANSWER: usize = 8 * 1024 * 1024;

/// One message, as it goes on the wire.
///
/// Public so that a client written in Rust builds a message from this type
/// rather than from a second description of the same format. What it holds is
/// `pub(crate)`, which is what keeps the two doors two.
#[derive(Debug, Serialize)]
pub(crate) struct Envelope {
    /// Which format this message is written in.
    format: u32,
    /// The request itself.
    asks: Asked,
}

/// One answer, as it goes on the wire.
///
/// Its own envelope name — `tells` beside the request's `asks` — so that a
/// message says which way it is going. A client that read an answer as a
/// request would be a client that could be handed one by anything on the
/// machine that can connect.
#[derive(Debug, Serialize)]
pub(crate) struct Reply {
    /// Which format this answer is written in.
    format: u32,
    /// The answer itself.
    tells: Told,
}

/// The same, as it is read back.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Heard {
    /// Which format it is written in. Checked already, and read again here so
    /// that an answer with no `format` at all cannot arrive by this road.
    #[expect(
        dead_code,
        reason = "read so that the field is required; its value was checked by `Stated`"
    )]
    format: u32,
    /// The answer itself.
    tells: Told,
}

/// The same, as it is read back — with the request left unread.
///
/// Two shapes rather than one, and the second is why: reading the format has to
/// succeed on a message whose request this version has never seen, so this one
/// tolerates what [`Envelope`] refuses.
#[derive(Debug, Deserialize)]
struct Stated {
    /// Which format the message claims to be written in.
    format: u32,
}

/// The one shape a message is read as, once the format is known.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Read {
    /// Which format it is written in. Checked already, and read again here so
    /// that a message with no `format` at all cannot arrive by this road.
    #[expect(
        dead_code,
        reason = "read so that the field is required; its value was checked by `Stated`"
    )]
    format: u32,
    /// The request itself.
    asks: Asked,
}

impl Envelope {
    /// A message carrying this request, in the format this alo OS writes.
    pub(crate) fn around(asks: Asked) -> Self {
        Self {
            format: FORMAT,
            asks,
        }
    }
}

/// Read one line as a message, and answer with the request inside it.
///
/// The three envelope rules in the order the header argues for: how long, how
/// many, and which format. What the request *is* comes last, because a message
/// that fails any of the three was never a request to begin with.
///
/// # Errors
/// [`NotUnderstood`], which says which of the four it was.
pub(crate) fn message(line: &str) -> Result<Asked, NotUnderstood> {
    if line.len() > LONGEST {
        return Err(NotUnderstood::TooLong {
            most: LONGEST,
            was: line.len(),
        });
    }
    // The line ending a reader hands back is not part of the message; a break
    // anywhere else means two messages arrived where one was expected.
    let line = line.trim_end_matches(['\n', '\r']);
    if line.contains('\n') || line.contains('\r') {
        return Err(NotUnderstood::MoreThanOneMessage);
    }

    let stated: Stated = serde_json::from_str(line).map_err(|_| NotUnderstood::NotReadable)?;
    if stated.format > FORMAT {
        return Err(NotUnderstood::FromANewerAloOs {
            format: stated.format,
        });
    }
    if stated.format < FORMAT {
        return Err(NotUnderstood::NotAFormat {
            format: stated.format,
        });
    }

    let read: Read = serde_json::from_str(line).map_err(|_| NotUnderstood::NotReadable)?;
    Ok(read.asks)
}

/// Write one request as the line that carries it.
///
/// `pub(crate)`: the two doors expose it, each for the requests its own side
/// may make.
///
/// # Errors
/// A `serde_json::Error`, which a request cannot cause — it is a name, some
/// text and some whole numbers. Handed back rather than swallowed, which is
/// what `alo-keeping` does with an entry it could not write down: a road that
/// cannot be taken is still not a road that invents an answer.
pub(crate) fn line(asks: Asked) -> Result<String, serde_json::Error> {
    serde_json::to_string(&Envelope::around(asks))
}

/// Read one line as an answer, and answer with what it says.
///
/// The same three rules as [`message`], in the same order, against
/// [`LONGEST_ANSWER`] — see this file's header for why that is a different
/// number and where it comes from.
///
/// # Errors
/// [`NotUnderstood`], which says which of the five it was.
pub(crate) fn reply(line: &str) -> Result<Told, NotUnderstood> {
    if line.len() > LONGEST_ANSWER {
        return Err(NotUnderstood::TooLong {
            most: LONGEST_ANSWER,
            was: line.len(),
        });
    }
    let line = line.trim_end_matches(['\n', '\r']);
    if line.contains('\n') || line.contains('\r') {
        return Err(NotUnderstood::MoreThanOneMessage);
    }

    let stated: Stated = serde_json::from_str(line).map_err(|_| NotUnderstood::NotReadable)?;
    if stated.format > FORMAT {
        return Err(NotUnderstood::FromANewerAloOs {
            format: stated.format,
        });
    }
    if stated.format < FORMAT {
        return Err(NotUnderstood::NotAFormat {
            format: stated.format,
        });
    }

    let heard: Heard = serde_json::from_str(line).map_err(|_| NotUnderstood::NotReadable)?;
    Ok(heard.tells)
}

/// Write one answer as the line that carries it.
///
/// `pub(crate)`: the two doors expose it, each for the answers its own side is
/// given.
///
/// # Errors
/// A `serde_json::Error`, handed back for [`line`]'s reason.
pub(crate) fn spoken(tells: Told) -> Result<String, serde_json::Error> {
    serde_json::to_string(&Reply {
        format: FORMAT,
        tells,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The ordinary path: one line, this format, a request inside it.
    #[test]
    fn a_message_of_this_format_is_the_request_inside_it() {
        let asked = message(r#"{"format":1,"asks":{"ask":{"question":"how many?"}}}"#).unwrap();
        assert_eq!(
            asked,
            Asked::Ask {
                question: "how many?".to_owned()
            }
        );
    }

    /// **A line reader's line ending is not part of the message.** A daemon
    /// that had to remember to trim would be a daemon that refuses every real
    /// client until somebody notices.
    #[test]
    fn the_line_ending_a_reader_leaves_behind_is_not_a_second_message() {
        for ending in ["\n", "\r\n", "\n\n"] {
            let line = format!(r#"{{"format":1,"asks":{{"ask":{{"question":"a"}}}}}}{ending}"#);
            assert!(message(&line).is_ok(), "{ending:?}");
        }
    }

    /// **Two messages are not one message.** The alternative is answering the
    /// first and dropping the rest, which is a service that silently does part
    /// of what it was asked.
    #[test]
    fn more_than_one_message_is_refused_rather_than_half_answered() {
        let two = concat!(
            r#"{"format":1,"asks":{"ask":{"question":"a"}}}"#,
            "\n",
            r#"{"format":1,"asks":{"ask":{"question":"b"}}}"#
        );
        assert_eq!(message(two), Err(NotUnderstood::MoreThanOneMessage));
    }

    /// **A message has a length before it has a meaning**, and the refusal
    /// carries both numbers for whoever draws them.
    #[test]
    fn a_message_longer_than_this_machine_holds_is_refused_before_it_is_read() {
        let enormous = format!(
            r#"{{"format":1,"asks":{{"ask":{{"question":"{}"}}}}}}"#,
            "a".repeat(LONGEST)
        );
        let was = enormous.len();
        assert_eq!(
            message(&enormous),
            Err(NotUnderstood::TooLong { most: LONGEST, was })
        );
    }

    /// **The format is read before the message**, so a client from a newer alo
    /// OS is told to update the machine rather than told it is unreadable —
    /// even when the request it carries is one this version has never seen.
    #[test]
    fn a_message_from_a_newer_alo_os_is_told_so_and_not_called_gibberish() {
        let newer = r#"{"format":2,"asks":{"reticulate":{"splines":3}},"and":"more"}"#;
        assert_eq!(
            message(newer),
            Err(NotUnderstood::FromANewerAloOs { format: 2 })
        );
    }

    /// A number no version ever wrote is a different thing to say from a number
    /// a later version will.
    #[test]
    fn a_format_nothing_ever_wrote_is_refused_as_that() {
        let never = r#"{"format":0,"asks":{"ask":{"question":"a"}}}"#;
        assert_eq!(message(never), Err(NotUnderstood::NotAFormat { format: 0 }));
    }

    /// A message with no format at all cannot get in by either road: it is
    /// refused when the number is read, and the shape that reads the request
    /// requires it too.
    #[test]
    fn a_message_that_names_no_format_is_not_a_message() {
        for line in [
            r#"{"asks":{"ask":{"question":"a"}}}"#,
            r#"{"format":"1","asks":{"ask":{"question":"a"}}}"#,
            "",
            "not json at all",
            "[]",
        ] {
            assert_eq!(message(line), Err(NotUnderstood::NotReadable), "{line}");
        }
    }

    /// A field nobody declared in the envelope is refused rather than ignored,
    /// once the format is one this version reads.
    #[test]
    fn a_field_nobody_declared_in_the_envelope_is_refused() {
        let extra = r#"{"format":1,"asks":{"ask":{"question":"a"}},"as":"root"}"#;
        assert_eq!(message(extra), Err(NotUnderstood::NotReadable));
    }

    /// What this crate writes is what it reads, so a client and a daemon built
    /// from it cannot disagree about the format.
    #[test]
    fn what_this_crate_writes_it_reads_back() {
        let asks = Asked::Approve { number: 7 };
        let written = line(asks.clone()).unwrap();
        assert!(written.contains("\"format\":1"), "{written}");
        assert_eq!(message(&written).unwrap(), asks);
    }

    /// An answer travels in an envelope of its own, and reads back as what was
    /// written.
    #[test]
    fn an_answer_is_written_and_read_in_its_own_envelope() {
        let tells = Told::Declined {};
        let written = spoken(tells.clone()).unwrap();
        assert_eq!(written, r#"{"format":1,"tells":{"declined":{}}}"#);
        assert_eq!(reply(&written).unwrap(), tells);
    }

    /// **A request is not an answer.** The two envelopes name the direction, so
    /// a client cannot be handed a request by whatever else on the machine can
    /// open a socket, and a daemon cannot be answered into.
    #[test]
    fn a_request_is_not_an_answer_and_an_answer_is_not_a_request() {
        let asked = line(Asked::Approve { number: 7 }).unwrap();
        assert_eq!(reply(&asked), Err(NotUnderstood::NotReadable));

        let told = spoken(Told::Declined {}).unwrap();
        assert_eq!(message(&told), Err(NotUnderstood::NotReadable));
    }

    /// The envelope's other three rules hold for an answer too: one line, a
    /// format this version reads, and nothing else in the envelope.
    #[test]
    fn an_answer_is_held_to_the_same_envelope_rules() {
        let two = concat!(
            r#"{"format":1,"tells":{"declined":{}}}"#,
            "\n",
            r#"{"format":1,"tells":{"declined":{}}}"#
        );
        assert_eq!(reply(two), Err(NotUnderstood::MoreThanOneMessage));

        assert!(reply(&format!("{}\r\n", spoken(Told::Declined {}).unwrap())).is_ok());

        assert_eq!(
            reply(r#"{"format":2,"tells":{"reticulated":{}}}"#),
            Err(NotUnderstood::FromANewerAloOs { format: 2 })
        );
        assert_eq!(
            reply(r#"{"format":0,"tells":{"declined":{}}}"#),
            Err(NotUnderstood::NotAFormat { format: 0 })
        );
        assert_eq!(
            reply(r#"{"format":1,"tells":{"declined":{}},"from":"root"}"#),
            Err(NotUnderstood::NotReadable)
        );
    }

    /// **The worst answer this machine can produce fits inside the bound.**
    /// The one thing the item did not contain and the tests did: a read is
    /// bounded at a megabyte and a message at a megabyte, so one bound for both
    /// directions would have meant a verb that succeeded and an answer nobody
    /// was allowed to read. A megabyte of control characters is the worst JSON
    /// escaping can do, and this is what it measures.
    #[test]
    fn a_worst_case_read_fits_inside_the_bound() {
        let most_read = usize::try_from(alo_files::MOST_READ).unwrap();
        let worst = "\u{1}".repeat(most_read);
        let written = spoken(Told::Did(crate::done::Done::Read { text: worst })).unwrap();
        assert!(written.len() > most_read * 5, "{}", written.len());
        assert!(
            written.len() <= LONGEST_ANSWER,
            "a read of {most_read} bytes crosses as {} and the bound is {LONGEST_ANSWER}",
            written.len()
        );
        assert_eq!(reply(&written).map(|_| ()), Ok(()));
    }

    /// And an answer longer than that is refused before it is parsed, for the
    /// reason a request is: nothing that reads a socket may be told how much
    /// memory to take.
    #[test]
    fn an_answer_longer_than_the_bound_is_refused_before_it_is_read() {
        let enormous = format!(
            r#"{{"format":1,"tells":{{"did":{{"read":{{"text":"{}"}}}}}}}}"#,
            "a".repeat(LONGEST_ANSWER)
        );
        let was = enormous.len();
        assert_eq!(
            reply(&enormous),
            Err(NotUnderstood::TooLong {
                most: LONGEST_ANSWER,
                was
            })
        );
    }
}
