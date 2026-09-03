//! One argument, exactly as it arrived.
//!
//! A name and a value, and nothing has been checked about either. That is the
//! whole of this file, and the restraint is the design: `alo-capability` is
//! where an argument is validated, and a second validator here would be a
//! second answer to *is this a path* that could disagree with the first.
//!
//! # Why arguments are a list and not an object
//!
//! The obvious wire shape for arguments is a JSON object — `{"file": "…"}` —
//! and it is wrong for one reason. An object has no duplicates: a message
//! naming `file` twice arrives at the reader as one `file`, with the JSON
//! library having silently chosen which. `alo_capability::CallError` has a
//! refusal for exactly that — [`SameArgumentTwice`] — and a wire shape that
//! deduplicated before anybody looked would make it unreachable, in the one
//! place a person's approval sentence is built from.
//!
//! So arguments arrive as a list of these, duplicates and all, and the closed
//! list of verbs is what refuses them.
//!
//! [`SameArgumentTwice`]: alo_capability::CallError::SameArgumentTwice

use alo_capability::Given;
use serde::{Deserialize, Serialize};

/// One argument of a call, as it was written on the wire.
///
/// `Deserialize` is the point of it: this is the untrusted side of the
/// boundary, and [`Given`] is the type in this workspace whose whole job is
/// being read back off one. It serialises too, so that a client written in
/// Rust builds its requests from these rather than from a second description
/// of the same format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Argument {
    /// The name the verb declared this argument under.
    named: String,
    /// What was given for it: text or a whole number.
    is: Given,
}

impl Argument {
    /// An argument called this, with this given for it.
    pub fn of(named: impl Into<String>, is: Given) -> Self {
        Self {
            named: named.into(),
            is,
        }
    }

    /// The name it was given under.
    ///
    /// Not trimmed, not lowered, not matched loosely against anything: item 1's
    /// *identities are matched exactly* reaches the wire here, and the matching
    /// itself is the verb's.
    #[must_use]
    pub fn named(&self) -> &str {
        &self.named
    }

    /// What was given for it.
    #[must_use]
    pub fn given(&self) -> &Given {
        &self.is
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The two shapes a value can arrive in, and no third.
    #[test]
    fn an_argument_is_text_or_a_number() {
        let text: Argument =
            serde_json::from_str(r#"{"named":"file","is":"/home/anna/a.pdf"}"#).unwrap();
        assert_eq!(text.named(), "file");
        assert_eq!(text.given(), &Given::text("/home/anna/a.pdf"));

        let number: Argument = serde_json::from_str(r#"{"named":"most","is":20}"#).unwrap();
        assert_eq!(number.given(), &Given::number(20));
    }

    /// **Nothing else is an argument.** A value that is neither text nor a
    /// whole number has no shape to arrive in, which is `alo-capability`'s
    /// closed list of what a model can produce reaching the socket.
    #[test]
    fn nothing_that_is_not_text_or_a_number_is_an_argument() {
        for is in ["true", "null", "[1,2]", r#"{"run":"sh"}"#, "1.5"] {
            let message = format!(r#"{{"named":"file","is":{is}}}"#);
            assert!(
                serde_json::from_str::<Argument>(&message).is_err(),
                "{message}"
            );
        }
    }

    /// A field nobody declared is refused rather than ignored: a client that
    /// asked for something this machine does not do should be told so.
    #[test]
    fn a_field_nobody_declared_is_refused() {
        let extra = r#"{"named":"file","is":"/a","as":"root"}"#;
        assert!(serde_json::from_str::<Argument>(extra).is_err());
    }

    /// **The name is taken exactly as it was written.** Trimming it here would
    /// be this crate deciding that ` file` means `file`, which is a decision
    /// the verb registry makes or refuses.
    #[test]
    fn a_name_is_carried_exactly_as_it_was_written() {
        let spaced: Argument = serde_json::from_str(r#"{"named":" file ","is":"/a"}"#).unwrap();
        assert_eq!(spaced.named(), " file ");
    }

    /// A client writing a request and a daemon reading one are looking at the
    /// same format, because they are the same type.
    #[test]
    fn what_is_written_is_what_is_read() {
        let argument = Argument::of("name", Given::text("march-final.pdf"));
        let written = serde_json::to_string(&argument).unwrap();
        assert_eq!(
            serde_json::from_str::<Argument>(&written).unwrap(),
            argument
        );
    }
}
