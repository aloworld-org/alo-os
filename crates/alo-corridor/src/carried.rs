//! What a verb is on the wire: its name and its typed arguments, and nothing
//! else.
//!
//! One object:
//!
//! ```text
//! {"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}
//! ```
//!
//! Two fields, and both are on the list task 8 of the local-network plan
//! names: the verb, and its typed arguments. The proof is the third thing on
//! that list and travels in a header, over exactly these bytes. A third field
//! in the body is refused rather than read around, for the reason
//! `alo-nearby` refuses an advertisement that says more than presence: reading
//! around it would let a later version quietly start saying more, and
//! everything on a network can read this.
//!
//! # The arguments are the local protocol's
//!
//! [`alo_protocol::Argument`] is one argument as it arrives at a daemon from
//! an agent on the same machine — a name and a [`alo_capability::Given`],
//! text or a whole number, with nothing checked about either. A verb crossing
//! a corridor arrives at exactly the same door on the other side, so it is
//! spelt exactly the same way: a second description of an argument would be
//! a second answer to *what can a model send*, and the closed list is
//! `alo-capability`'s. The list rather than an object, for that file's reason:
//! an object would deduplicate a name before the verb could refuse it.
//!
//! # Compact, and the bytes are the message
//!
//! [`Carried::said`] writes the object compactly and those bytes are what the
//! proof is made over and what is sent; [`Carried::read`] reads what arrived.
//! The receiving side checks the proof over the bytes that arrived *before*
//! reading them as a verb, so a body that was altered in transit is refused
//! as not from the machine it names rather than as not a verb.

use alo_capability::Given;
use alo_nearby::NotNearby;
use alo_protocol::Argument;
use serde::{Deserialize, Serialize};

/// The most bytes a verb on the wire may be.
///
/// Sixteen kibibytes: a verb is a name and a handful of paths and numbers,
/// and a message that says it is more is refused before its body is read.
pub const AT_MOST_A_VERB: usize = 16 * 1024;

/// A verb and its arguments, exactly as they cross.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Carried {
    /// The verb's name, as the asking agent wrote it.
    verb: String,
    /// What was given for each argument, duplicates and all.
    given: Vec<Argument>,
}

impl Carried {
    /// This verb with these arguments, as an agent asked for it.
    #[must_use]
    pub fn of(verb: &str, given: &[(&str, Given)]) -> Self {
        Self {
            verb: verb.to_owned(),
            given: given
                .iter()
                .map(|(named, is)| Argument::of(*named, is.clone()))
                .collect(),
        }
    }

    /// The bytes that cross: the object, compactly.
    ///
    /// What the proof is made over, and what the receiving machine checks it
    /// over. Cannot fail: a name and a list of text and numbers always
    /// serialise, and `serde_json` says so of every type without a map.
    #[must_use]
    pub fn said(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// A verb somebody sent, read back and checked for shape — and for
    /// nothing else: whether it is a verb this machine offers is the closed
    /// list's to answer, at the door.
    ///
    /// # Errors
    ///
    /// [`NotNearby::NotAMessage`] for anything that is not exactly the object
    /// [`said`](Self::said) writes, including a field this crate has no place
    /// for and a value that is neither text nor a whole number.
    pub fn read(said: &str) -> Result<Self, NotNearby> {
        serde_json::from_str(said)
            .map_err(|why| NotNearby::NotAMessage(format!("not a verb: {why}")))
    }

    /// The verb's name.
    #[must_use]
    pub fn verb(&self) -> &str {
        &self.verb
    }

    /// The arguments, as the door takes them: a name and what was given for
    /// it, in the order they were sent.
    #[must_use]
    pub fn given(&self) -> Vec<(&str, Given)> {
        self.given
            .iter()
            .map(|argument| (argument.named(), argument.given().clone()))
            .collect()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use alo_capability::Given;

    use super::Carried;

    /// A listing of one folder, as an agent asks for it.
    fn a_listing() -> Carried {
        Carried::of(
            "list_folder",
            &[("folder", Given::text("/home/anna/Invoices"))],
        )
    }

    /// **A verb written here is read back as it was written**, with the two
    /// fields and nothing beside them, and its arguments come out as the door
    /// takes them.
    #[test]
    fn a_verb_written_here_is_read_back_and_has_exactly_two_fields() {
        let said = a_listing().said();
        assert_eq!(
            said,
            r#"{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}"#
        );
        let read = Carried::read(&said).unwrap();
        assert_eq!(read, a_listing());
        assert_eq!(read.verb(), "list_folder");
        assert_eq!(
            read.given(),
            vec![("folder", Given::text("/home/anna/Invoices"))]
        );
    }

    /// **A field this crate has no place for is refused rather than read
    /// around**, and so is everything else that is not the object.
    #[test]
    fn a_verb_with_a_field_not_on_the_list_is_refused() {
        for not_one in [
            "",
            "{}",
            r#"{"verb":"list_folder"}"#,
            r#"{"given":[]}"#,
            r#"{"verb":"list_folder","given":[],"as":"root"}"#,
            r#"{"verb":"list_folder","given":[],"proof":"alo-os/1"}"#,
            r#"{"verb":"list_folder","given":[{"named":"most","is":true}]}"#,
            r#"{"verb":"list_folder","given":[{"named":"most","is":1.5}]}"#,
            r#"{"verb":"list_folder","given":[{"named":"most","is":1,"as":"root"}]}"#,
            r#"{"verb":"list_folder","given":{"folder":"/a"}}"#,
            r#"alo-os/1 proposal"#,
        ] {
            assert!(
                Carried::read(not_one).is_err(),
                "`{not_one}` was read as a verb"
            );
        }
    }

    /// A name given twice arrives twice, so the closed list can refuse it
    /// rather than this file choosing one.
    #[test]
    fn an_argument_given_twice_arrives_twice() {
        let twice = Carried::of(
            "rename_file",
            &[
                ("file", Given::text("/a")),
                ("file", Given::text("/b")),
                ("name", Given::text("c")),
            ],
        );
        let read = Carried::read(&twice.said()).unwrap();
        let names: Vec<&str> = read.given().iter().map(|(named, _)| *named).collect();
        assert_eq!(names, ["file", "file", "name"]);
    }
}
