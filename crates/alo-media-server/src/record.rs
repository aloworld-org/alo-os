//! **What the server said**, read.
//!
//! The record is a list of objects — every node, port, link, device and piece of
//! metadata the server holds — and this crate does nothing with them but hand
//! them over: what a node *means* is `alo-in-use`'s question, or `alo-sound`'s,
//! or `alo-cameras`', and the three of them mean different things by the same
//! object.
//!
//! # It is read as a stream of lists, not as one
//!
//! On 2026-09-17 a gate read a record with **trailing characters after the end
//! of the list** and every reading of the same machine a moment later parsed as
//! one list. What produced it was not caught, so nothing here claims to know;
//! `docs/quirks.md` records what was seen. What a reader must not do is turn it
//! into *this machine answered something unreadable*, because that sentence
//! takes an indicator off a screen while a camera may be on — so the record is
//! read as a **stream** of lists, and an object listed twice is taken as it was
//! listed last, which is what a second list of a changing graph would mean.
//!
//! A single list, which is every other reading there has ever been, goes through
//! this unchanged.

use serde_json::Value;

use crate::refusing::NotAsked;

/// **Everything the server listed**, in the order it listed it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheRecord {
    /// The objects.
    objects: Vec<Value>,
}

impl TheRecord {
    /// Everything, as the server wrote it.
    #[must_use]
    pub fn objects(&self) -> &[Value] {
        &self.objects
    }

    /// How many objects the server listed.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.objects.len()
    }

    /// Only the objects whose type ends with this — `":Node"`, `":Link"`,
    /// `":Metadata"` — which is how the server names what a thing is.
    pub fn of_type<'a>(&'a self, ending: &'a str) -> impl Iterator<Item = &'a Value> {
        self.objects.iter().filter(move |object| {
            object
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|named| named.ends_with(ending))
        })
    }
}

/// **Read a record.**
///
/// # Errors
/// [`NotAsked::ItAnsweredSomethingUnreadable`] where what came back is not a
/// list of objects at all.
pub fn read(said: &str) -> Result<TheRecord, NotAsked> {
    let mut objects: Vec<Value> = Vec::new();
    for read in serde_json::Deserializer::from_str(said).into_iter::<Value>() {
        let read = read.map_err(|why| NotAsked::ItAnsweredSomethingUnreadable {
            said: format!("the record is not readable: {why}"),
        })?;
        let Value::Array(listed) = read else {
            return Err(NotAsked::ItAnsweredSomethingUnreadable {
                said: "the record is not a list of objects".to_owned(),
            });
        };
        for object in listed {
            match the_same_one(&objects, &object) {
                Some(at) => {
                    if let Some(held) = objects.get_mut(at) {
                        *held = object;
                    }
                }
                None => objects.push(object),
            }
        }
    }
    Ok(TheRecord { objects })
}

/// Where this object already is in what has been read, by the server's own
/// number for it.
fn the_same_one(objects: &[Value], object: &Value) -> Option<usize> {
    let id = object.get("id").and_then(Value::as_u64)?;
    objects
        .iter()
        .position(|seen| seen.get("id").and_then(Value::as_u64) == Some(id))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    const ONE: &str = r#"[{"id": 31, "type": "PipeWire:Interface:Node", "info": {"state": "idle"}},
                          {"id": 32, "type": "PipeWire:Interface:Link"}]"#;

    /// **An ordinary record reads as itself.**
    #[test]
    fn one_list_of_objects_reads_as_that_list() {
        let record = read(ONE).expect("a record");
        assert_eq!(record.how_many(), 2);
        assert_eq!(record.of_type(":Node").count(), 1);
        assert_eq!(record.of_type(":Link").count(), 1);
        assert_eq!(record.of_type(":Metadata").count(), 0);
    }

    /// **A record in two lists is one machine, and the later line wins.**
    #[test]
    fn a_record_in_two_lists_is_one_machine() {
        let and_then = r#"[{"id": 31, "type": "PipeWire:Interface:Node",
                            "info": {"state": "running"}}]"#;
        let record = read(&format!("{ONE}\n{and_then}")).expect("two lists");
        assert_eq!(record.how_many(), 2, "the same object was counted twice");
        let node = record.of_type(":Node").next().expect("the node");
        assert_eq!(
            node.get("info").and_then(|info| info.get("state")),
            Some(&Value::String("running".to_owned())),
            "the earlier line won, so a change the server made was thrown away"
        );
    }

    /// **Something that is not a record says so**, rather than reading as a
    /// machine with nothing on it.
    #[test]
    fn something_that_is_not_a_record_is_refused_rather_than_read_as_an_empty_machine() {
        assert!(matches!(
            read("this is not a record"),
            Err(NotAsked::ItAnsweredSomethingUnreadable { .. })
        ));
        assert!(matches!(
            read(r#"{"id": 1}"#),
            Err(NotAsked::ItAnsweredSomethingUnreadable { .. })
        ));
        assert_eq!(read("").expect("nothing is nothing").how_many(), 0);
    }
}
