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
//! list of verbs is what refuses two different values for one argument. The
//! same value twice is one value (since 2026-11-14): there is no *which*.
//!
//! # And one object, when it cannot hide a duplicate
//!
//! **Since 2026-11-14 `given` may also arrive as one object**, `{"file":
//! "…", "into": "…"}`, because that is the shape a hosted model is held to
//! (`alo_turn::the_shape_a_provider_is_held_to`): under a strict schema a
//! model cannot write a property twice, and held to the list it named an
//! argument twice in every one of ten tries on the walk's own request, twice
//! with two different values (the every-try check, CI runs 43682925362 and
//! 43683523750). **The reason above still holds**, so the object is read
//! here key by key, in the order written, and **a key that comes twice is
//! refused** rather than one of them chosen ([`a_list_or_one_object`]).
//! Either form becomes the same list.
//!
//! [`SameArgumentTwice`]: alo_capability::CallError::SameArgumentTwice

use std::fmt;

use alo_capability::Given;
use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
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

/// **`given` read as a list of arguments, or as one object of them** — the
/// object key by key, in the order it was written, **refusing a key written
/// twice**, which a JSON reader would otherwise have settled by choosing one.
///
/// # Errors
/// The reader's own error: not a list or an object, an element that is not an
/// [`Argument`], a value that is not text or a whole number, or a key twice.
pub(crate) fn a_list_or_one_object<'de, D>(reading: D) -> Result<Vec<Argument>, D::Error>
where
    D: Deserializer<'de>,
{
    reading.deserialize_any(ListOrObject)
}

/// What [`a_list_or_one_object`] reads with.
struct ListOrObject;

impl<'de> Visitor<'de> for ListOrObject {
    type Value = Vec<Argument>;

    fn expecting(&self, said: &mut fmt::Formatter<'_>) -> fmt::Result {
        said.write_str("a list of {\"named\", \"is\"} or one object naming each argument once")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut list: A) -> Result<Self::Value, A::Error> {
        let mut read = Vec::new();
        while let Some(argument) = list.next_element::<Argument>()? {
            read.push(argument);
        }
        Ok(read)
    }

    fn visit_map<M: MapAccess<'de>>(self, mut object: M) -> Result<Self::Value, M::Error> {
        let mut read: Vec<Argument> = Vec::new();
        while let Some((named, is)) = object.next_entry::<String, Given>()? {
            if read.iter().any(|earlier| earlier.named == named) {
                return Err(serde::de::Error::custom(format!(
                    "{named} is named twice in one object, and which was meant is not this \
                     machine's to choose"
                )));
            }
            read.push(Argument { named, is });
        }
        Ok(read)
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

    /// What `given` is read as, standing alone.
    #[derive(Debug, Deserialize)]
    struct Read {
        /// The arguments.
        #[serde(deserialize_with = "super::a_list_or_one_object")]
        given: Vec<Argument>,
    }

    /// **One object is the same list, in the order it was written**, so the
    /// shape a hosted model is held to and the list a client writes make the
    /// same call.
    ///
    /// What would have to be true for this to pass while broken: a provider
    /// writing the object in an order other than the schema's, which changes
    /// nothing — the registry matches arguments by name, never by place.
    #[test]
    fn one_object_is_the_same_list_in_the_order_written() {
        let object: Read =
            serde_json::from_str(r#"{"given":{"file":"/home/anna/a.pdf","most":20}}"#).unwrap();
        let list: Read = serde_json::from_str(
            r#"{"given":[{"named":"file","is":"/home/anna/a.pdf"},{"named":"most","is":20}]}"#,
        )
        .unwrap();
        assert_eq!(object.given, list.given);
        let none: Read = serde_json::from_str(r#"{"given":{}}"#).unwrap();
        assert!(none.given.is_empty());
    }

    /// **A key written twice in one object is refused**, never settled by the
    /// reader choosing one — the reason `given` was a list in the first place.
    #[test]
    fn a_key_written_twice_in_one_object_is_refused() {
        for twice in [
            r#"{"given":{"file":"/a","file":"/b"}}"#,
            r#"{"given":{"file":"/a","file":"/a"}}"#,
        ] {
            assert!(serde_json::from_str::<Read>(twice).is_err(), "{twice}");
        }
    }

    /// **Nothing but text or a whole number is a value in the object form
    /// either**, and nothing but a list or an object is `given`.
    #[test]
    fn the_object_form_takes_no_other_value() {
        for wrong in [
            r#"{"given":{"file":true}}"#,
            r#"{"given":{"file":{"run":"sh"}}}"#,
            r#"{"given":{"file":1.5}}"#,
            r#"{"given":"file=/a"}"#,
        ] {
            assert!(serde_json::from_str::<Read>(wrong).is_err(), "{wrong}");
        }
    }
}
