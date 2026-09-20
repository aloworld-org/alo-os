//! The media server's own record, read for what is in use.
//!
//! The record itself — running the tool, clearing its environment, and reading
//! an answer that may arrive as more than one list — is `alo-media-server`'s.
//! What is here is the half that is this crate's: **what those objects mean when
//! the question is *what is watching or listening*.**
//!
//! The rented media server keeps the graph of what is open on this machine and
//! will write it out as a list of objects: the nodes — devices and the streams
//! clients have open — and the links between them. This file is the one place
//! that turns that list into [`crate::Use`] values, and it is the only file in
//! this crate that knows anything about how the server writes things down.
//!
//! Nothing here is patched, extended or worked around. ADR 0011: the media
//! server is rented, configured and never written. We read what it says.
//!
//! # What counts as a use, and why it is read from the source and not the
//! stream
//!
//! A machine's graph has two sides to every capture: the **source** the picture
//! or the sound comes from, and the **stream** a client reads it through. Both
//! are in the record and either could be counted. This reads the source, for
//! one reason that decides it: on the stream side a client reading a camera and
//! a client receiving a shared screen are written down identically, and telling
//! them apart would mean guessing. On the source side they are two different
//! things and the server has already said which.
//!
//! So a use is a source the server says is **running**:
//!
//! | What the record says a node is | What is in use |
//! |---|---|
//! | an audio source | the microphone |
//! | a video source | the camera |
//! | a client producing video into the graph | the screen |
//!
//! and everything else in the record — sinks, ports, devices, clients, modules,
//! factories, and the streams on the other end of these — is not a use and is
//! passed over. A node the server says is idle or suspended is not in use
//! either: nothing is reading it, and the light on a camera is off.
//!
//! # And who is using it is whoever is reading from it
//!
//! Every link in the record joins one node's output to another's input. The
//! things using a source are the nodes on the input side of its links, and each
//! of them is a line of its own — two applications reading one camera are two
//! uses, because a person stopping one of them has to be able to say which.
//! Several links between one pair (a stereo capture is two) are one use.
//!
//! **A running source with nothing reading it is still a use.** It is listed as
//! [`crate::By::something_on_this_machine`], because the alternative is a use
//! that is not shown, and a use that is not shown is the whole failure this
//! indicator exists to prevent.
//!
//! # What the server vouches for, and what it merely repeats
//!
//! This is the security half of the plan's *rather than from what applications
//! say they are doing*, and the rule is one sentence: **nothing an application
//! can write about itself reaches the agent's answer or alo OS's.**
//!
//! - An application on alo OS is sandboxed (ADR 0005) and the machine stamps
//!   its identity onto its connection. Where that stamp is present it is the
//!   answer, and it can only ever say *this application*.
//! - Without that stamp the connection is from something not in a sandbox —
//!   alo OS's own components, or a program the person started in their own
//!   terminal (ADR 0043). A name written there is read as *which application*
//!   and nothing more, except for alo OS's own reserved identifier.
//!
//! The residual is honest and is written down rather than papered over: a
//! program **the person themselves started outside a sandbox** could write alo
//! OS's reserved identifier about itself and be listed as alo OS. It cannot
//! make itself invisible, which is the property that matters, and an
//! application — which is what arrives from outside the machine — cannot do
//! even this. Tightening it means comparing the connection's process against
//! alo OS's own, which is a thing to measure on a machine rather than to assume
//! in a file.
//!
//! # Why the names the record is written in are public
//!
//! They are read here and **announced elsewhere**. A capture alo OS takes of
//! its own screen has to appear on this indicator like anybody else's, and the
//! only honest way for that to be true is for the capture to open a stream on
//! the machine's own graph, announcing itself under the same names this file
//! reads back — `crates/alo-capturing` does exactly that. Two spellings of
//! `application.id`, one in each crate, would be a drift nobody notices until
//! the day alo OS's own screenshot is the one use this indicator does not show.
//!
//! So the names are declared once, here, where the argument about which of them
//! may be believed already lives. This file is still the only place that knows
//! how the server writes things down; what is public is the vocabulary, not a
//! second reading of it.

use std::collections::BTreeMap;

use alo_applications::Application;
use alo_capability::Grantee;
use serde_json::{Map, Value};

use crate::by::By;
use crate::refusing::NotHeard;
use crate::used::Used;
use crate::uses::{Use, UseId};

/// What the record calls a node.
pub const A_NODE: &str = "PipeWire:Interface:Node";

/// What the record calls a link.
pub const A_LINK: &str = "PipeWire:Interface:Link";

/// What the record calls an audio source: the microphone.
const AN_AUDIO_SOURCE: &str = "Audio/Source";

/// What the record calls a video source: the camera.
const A_VIDEO_SOURCE: &str = "Video/Source";

/// What the record calls a client producing video into the graph: a picture of
/// the screen, offered to whatever is reading it.
pub const VIDEO_INTO_THE_GRAPH: &str = "Stream/Output/Video";

/// What the record calls a client reading video out of the graph: a capture,
/// looking at a camera or at the picture of the screen somebody else offers.
///
/// **Not a use in itself.** This crate maps the source a capture reads from
/// and never the reader, so nothing here counts twice. It matters for the
/// other reason: it is what this machine's session manager sorts a stream by
/// when it goes looking for something to link it to, and **a video capture that
/// does not say this is linked to nothing**. Measured on the development PC,
/// 2026-09-20: a capture that named no kind reached the session manager as
/// `Stream/Input/Unknown`, which has no kind to search by, and was refused with
/// *target not found* and zero bytes.
pub const VIDEO_OUT_OF_THE_GRAPH: &str = "Stream/Input/Video";

/// The property a node's kind is written into.
pub const WHAT_KIND_IT_IS: &str = "media.class";

/// What the record says the state of a node that is capturing is.
pub const RUNNING: &str = "running";

/// The property the machine stamps a sandboxed application's identity into.
///
/// Written by the machine and not by the application, which is why it is the
/// only property here that is believed about identity.
const THE_STAMPED_IDENTITY: &str = "pipewire.access.portal.app_id";

/// The property a client writes its own identifier into.
pub const WHAT_IT_CALLS_ITSELF: &str = "application.id";

/// The property a client writes its own name into.
const WHAT_IT_IS_CALLED: &str = "application.name";

/// The name of a node, used as an identifier when a client wrote none.
pub const THE_NODE_NAME: &str = "node.name";

/// The identifier alo OS's own components connect under.
///
/// Reserved: an application cannot write it, because an application's identity
/// is stamped by the machine and this file never reads a stamped identity as
/// anything but an application.
pub const ALO_OSS_OWN_IDENTIFIER: &str = "os.alo.AloOs";

/// The property one of alo OS's own components names an agent in, when the use
/// belongs to an agent's turn rather than to the machine.
pub const NAMING_THE_AGENT: &str = "alo.agent";

/// What the media server's record says is in use.
///
/// # Errors
/// [`NotHeard::NotUnderstood`], when the record is not a list of objects or a
/// node or link in it cannot be read. Deliberately strict: an unreadable record
/// that was quietly skipped past would leave an indicator that reads exactly
/// like a quiet room, and being told the machine cannot answer is the only
/// honest alternative.
pub fn in_use_in(objects: &[Value]) -> Result<Vec<Use>, NotHeard> {
    let mut nodes: Vec<Node<'_>> = Vec::new();
    let mut links: Vec<(u32, u32)> = Vec::new();
    for object in objects {
        match object.get("type").and_then(Value::as_str) {
            Some(A_NODE) => {
                if let Some(node) = node_in(object)? {
                    nodes.push(node);
                }
            }
            Some(A_LINK) => {
                if let Some(link) = link_in(object)? {
                    links.push(link);
                }
            }
            _ => {}
        }
    }

    let by_number: BTreeMap<u32, &Node<'_>> =
        nodes.iter().map(|node| (node.number, node)).collect();
    let mut in_use = Vec::new();
    for node in &nodes {
        let Some(what) = what_is_used(node.class) else {
            continue;
        };
        if !node.running {
            continue;
        }
        let readers = readers_of(node.number, &links, &by_number);
        if readers.is_empty() {
            in_use.push(Use::of(
                UseId::recorded(node.number),
                what,
                By::something_on_this_machine(),
            ));
            continue;
        }
        for reader in readers {
            in_use.push(Use::of(
                UseId::recorded(reader.number),
                what,
                who_it_is(reader.props),
            ));
        }
    }
    Ok(in_use)
}

/// One node of the graph, as the record has it.
struct Node<'a> {
    /// The number the server records it under.
    number: u32,
    /// What the record says it is.
    class: &'a str,
    /// Whether the server says it is running.
    running: bool,
    /// What the record says about it.
    props: &'a Map<String, Value>,
}

/// What is in use when a node of this kind is running, or [`None`] when a node
/// of this kind is not a use at all.
fn what_is_used(class: &str) -> Option<Used> {
    match class {
        AN_AUDIO_SOURCE => Some(Used::Microphone),
        A_VIDEO_SOURCE => Some(Used::Camera),
        VIDEO_INTO_THE_GRAPH => Some(Used::Screen),
        _ => None,
    }
}

/// The node in this object, [`None`] if the object is a node that has gone.
///
/// # Errors
/// [`NotHeard::NotUnderstood`] when an object the record calls a node cannot be
/// read as one.
/// One object read as a node, or [`None`] where it is not one this crate holds.
fn node_in(object: &Value) -> Result<Option<Node<'_>>, NotHeard> {
    let number = number_in(object)?;
    let Some(info) = object.get("info") else {
        return Ok(None);
    };
    if info.is_null() {
        // The record's way of saying this node is gone.
        return Ok(None);
    }
    let Some(props) = info.get("props").and_then(Value::as_object) else {
        return Err(NotHeard::NotUnderstood {
            said: format!("the node {number} says nothing about itself"),
        });
    };
    Ok(Some(Node {
        number,
        class: props
            .get(WHAT_KIND_IT_IS)
            .and_then(Value::as_str)
            .unwrap_or_default(),
        running: info.get("state").and_then(Value::as_str) == Some(RUNNING),
        props,
    }))
}

/// The link in this object, [`None`] if the object is a link that has gone.
///
/// # Errors
/// [`NotHeard::NotUnderstood`] when an object the record calls a link cannot be
/// read as one.
fn link_in(object: &Value) -> Result<Option<(u32, u32)>, NotHeard> {
    let number = number_in(object)?;
    let Some(info) = object.get("info") else {
        return Ok(None);
    };
    if info.is_null() {
        return Ok(None);
    }
    let Some(props) = info.get("props").and_then(Value::as_object) else {
        return Err(NotHeard::NotUnderstood {
            said: format!("the link {number} says nothing about itself"),
        });
    };
    let (Some(from), Some(to)) = (
        whole_number(props.get("link.output.node")),
        whole_number(props.get("link.input.node")),
    ) else {
        return Err(NotHeard::NotUnderstood {
            said: format!("the link {number} does not say what it joins"),
        });
    };
    Ok(Some((from, to)))
}

/// The number an object is recorded under.
///
/// # Errors
/// [`NotHeard::NotUnderstood`] when there is none, or it is not a number this
/// machine can hold.
fn number_in(object: &Value) -> Result<u32, NotHeard> {
    whole_number(object.get("id")).ok_or_else(|| NotHeard::NotUnderstood {
        said: "the record has an object with no number on it".to_owned(),
    })
}

/// This value as a whole number, if it is one.
fn whole_number(value: Option<&Value>) -> Option<u32> {
    u32::try_from(value?.as_u64()?).ok()
}

/// The nodes reading from this one, each once, in the order the record has
/// them.
fn readers_of<'a>(
    number: u32,
    links: &[(u32, u32)],
    by_number: &BTreeMap<u32, &'a Node<'a>>,
) -> Vec<&'a Node<'a>> {
    let mut readers: Vec<&Node<'_>> = Vec::new();
    for (from, to) in links {
        if *from != number {
            continue;
        }
        let Some(reader) = by_number.get(to) else {
            continue;
        };
        if readers
            .iter()
            .any(|already| already.number == reader.number)
        {
            continue;
        }
        readers.push(reader);
    }
    readers
}

/// Who a node belongs to, from what the record says about it.
///
/// The rule this file's documentation states, in one place: a stamped identity
/// is an application and only an application; without one, alo OS's own
/// reserved identifier is alo OS or one of its agents, and anything else is an
/// application by whatever it calls itself — or, when it calls itself nothing
/// this machine can use, something it cannot name.
fn who_it_is(props: &Map<String, Value>) -> By {
    if let Some(stamped) = said_in(props, THE_STAMPED_IDENTITY) {
        return an_application(stamped, said_in(props, WHAT_IT_IS_CALLED));
    }
    if said_in(props, WHAT_IT_CALLS_ITSELF) == Some(ALO_OSS_OWN_IDENTIFIER) {
        return match said_in(props, NAMING_THE_AGENT).map(Grantee::named) {
            Some(agent) => By::the_agent(&agent).unwrap_or_else(By::alo_os_itself),
            None => By::alo_os_itself(),
        };
    }
    match said_in(props, WHAT_IT_CALLS_ITSELF).or_else(|| said_in(props, THE_NODE_NAME)) {
        Some(identifier) => an_application(identifier, said_in(props, WHAT_IT_IS_CALLED)),
        None => By::something_on_this_machine(),
    }
}

/// An application by this identifier and, where there is one, this name.
///
/// An identifier no verb could ever name is not made into an application: it is
/// [`By::something_on_this_machine`], which shows the use and says honestly
/// that the machine cannot put a name to it.
fn an_application(identifier: &str, called: Option<&str>) -> By {
    let application = match called {
        Some(called) => Application::called(identifier, called),
        None => Application::identified(identifier),
    };
    application.map_or_else(|_| By::something_on_this_machine(), By::an_application)
}

/// What the record says under this name, when it says something that is not
/// empty.
fn said_in<'a>(props: &'a Map<String, Value>, named: &str) -> Option<&'a str> {
    let said = props.get(named)?.as_str()?.trim();
    (!said.is_empty()).then_some(said)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A record, read the way a machine's is — through the crate that owns
    /// reading it.
    fn objects(record: &str) -> alo_media_server::TheRecord {
        alo_media_server::read(record).expect("a record this crate reads")
    }

    /// **A record that arrives as more than one list is still one machine**,
    /// end to end: `alo-media-server` reads the stream, and this crate counts
    /// the source once.
    #[test]
    fn a_record_in_two_lists_reads_as_one_machine() {
        let one = r#"[{"id": 31, "type": "PipeWire:Interface:Node",
             "info": {"state": "idle", "props": {"media.class": "Audio/Source"}}}]"#;
        let and_then = r#"[{"id": 31, "type": "PipeWire:Interface:Node",
             "info": {"state": "running", "props": {"media.class": "Audio/Source"}}}]"#;

        let quiet = in_use_in(objects(one).objects()).expect("one list reads");
        assert!(quiet.is_empty(), "an idle source is not a use");

        let text = format!("{one}\n{and_then}");
        let both = in_use_in(objects(&text).objects()).expect("two lists read");
        assert_eq!(both.len(), 1, "the same source was counted twice");
        assert_eq!(both.first().map(Use::what), Some(Used::Microphone));
    }
    use crate::testing::{a_record_of, a_recorded_link, a_recorded_node};

    /// **A running camera read by one application is one use, named.** The
    /// shape of every record this file reads, at its smallest.
    #[test]
    fn a_camera_read_by_an_application_is_one_use_named() {
        let record = a_record_of(&[
            a_recorded_node(30, A_VIDEO_SOURCE, RUNNING, &[]),
            a_recorded_node(
                42,
                "Stream/Input/Video",
                RUNNING,
                &[
                    (THE_STAMPED_IDENTITY, "com.example.VideoCall"),
                    (WHAT_IT_IS_CALLED, "Video Call"),
                ],
            ),
            a_recorded_link(50, 30, 42),
        ]);
        let in_use = in_use_in(objects(&record).objects()).unwrap();
        assert_eq!(in_use.len(), 1);
        let one = in_use.first().unwrap();
        assert_eq!(one.what(), Used::Camera);
        assert_eq!(one.at(), UseId::recorded(42));
        assert_eq!(
            one.by().application().map(Application::identifier),
            Some("com.example.VideoCall")
        );
    }

    /// **A source nothing is reading is not in use.** The server says it is
    /// idle, the light on the camera is off, and a line here would be a line
    /// that cries wolf.
    #[test]
    fn a_camera_nobody_is_reading_is_not_in_use() {
        let record = a_record_of(&[a_recorded_node(30, A_VIDEO_SOURCE, "idle", &[])]);
        assert!(in_use_in(objects(&record).objects()).unwrap().is_empty());
    }

    /// **A running source with nothing recorded reading it is still a use**,
    /// named as something this machine cannot name. The comfortable answer
    /// would be to leave it off; a use nobody can name is the one a person most
    /// needs to be told about.
    #[test]
    fn a_running_source_nothing_is_recorded_reading_is_still_shown() {
        let record = a_record_of(&[a_recorded_node(30, AN_AUDIO_SOURCE, RUNNING, &[])]);
        let in_use = in_use_in(objects(&record).objects()).unwrap();
        assert_eq!(in_use.len(), 1);
        let one = in_use.first().unwrap();
        assert_eq!(one.what(), Used::Microphone);
        assert!(one.by().is_something_it_cannot_name());
        assert_eq!(one.at(), UseId::recorded(30));
    }

    /// **The three kinds of source are the three things in use**, and nothing
    /// else in a record is one.
    #[test]
    fn the_three_kinds_of_source_are_the_three_things_in_use() {
        let record = a_record_of(&[
            a_recorded_node(10, AN_AUDIO_SOURCE, RUNNING, &[]),
            a_recorded_node(11, A_VIDEO_SOURCE, RUNNING, &[]),
            a_recorded_node(12, VIDEO_INTO_THE_GRAPH, RUNNING, &[]),
            a_recorded_node(13, "Audio/Sink", RUNNING, &[]),
            a_recorded_node(14, "Stream/Output/Audio", RUNNING, &[]),
            a_recorded_node(15, "Stream/Input/Video", RUNNING, &[]),
            a_recorded_node(16, "", RUNNING, &[]),
        ]);
        let what: Vec<Used> = in_use_in(objects(&record).objects())
            .unwrap()
            .iter()
            .map(Use::what)
            .collect();
        assert_eq!(
            what,
            vec![Used::Microphone, Used::Camera, Used::Screen],
            "a record's other nodes were counted as uses, or a source was not"
        );
    }

    /// **Two applications reading one camera are two uses**, and a pair of
    /// links between one pair of nodes — which is what a stereo capture is — is
    /// one.
    #[test]
    fn two_readers_are_two_uses_and_two_links_between_one_pair_are_one() {
        let record = a_record_of(&[
            a_recorded_node(30, AN_AUDIO_SOURCE, RUNNING, &[]),
            a_recorded_node(
                42,
                "Stream/Input/Audio",
                RUNNING,
                &[(THE_STAMPED_IDENTITY, "com.example.VideoCall")],
            ),
            a_recorded_node(
                43,
                "Stream/Input/Audio",
                RUNNING,
                &[(THE_STAMPED_IDENTITY, "com.example.Recorder")],
            ),
            a_recorded_link(50, 30, 42),
            a_recorded_link(51, 30, 42),
            a_recorded_link(52, 30, 43),
        ]);
        let in_use = in_use_in(objects(&record).objects()).unwrap();
        assert_eq!(in_use.len(), 2);
        assert_eq!(
            in_use.iter().map(Use::at).collect::<Vec<_>>(),
            vec![UseId::recorded(42), UseId::recorded(43)]
        );
    }

    /// **alo OS's own capture is on the list, named as alo OS.** The plan's
    /// *alo OS's own captures appear on it like anybody else's*, read at the
    /// place the record becomes a line.
    #[test]
    fn alo_oss_own_capture_is_read_as_alo_os() {
        let record = a_record_of(&[
            a_recorded_node(30, VIDEO_INTO_THE_GRAPH, RUNNING, &[]),
            a_recorded_node(
                42,
                "Stream/Input/Video",
                RUNNING,
                &[(WHAT_IT_CALLS_ITSELF, ALO_OSS_OWN_IDENTIFIER)],
            ),
            a_recorded_link(50, 30, 42),
        ]);
        let in_use = in_use_in(objects(&record).objects()).unwrap();
        assert_eq!(in_use.len(), 1);
        let one = in_use.first().unwrap();
        assert_eq!(one.what(), Used::Screen);
        assert!(one.by().is_alo_os());
    }

    /// **An agent's turn is read as the agent**, from alo OS's own reserved
    /// identifier and never from an application's.
    #[test]
    fn an_agents_turn_is_read_as_the_agent() {
        let record = a_record_of(&[
            a_recorded_node(30, VIDEO_INTO_THE_GRAPH, RUNNING, &[]),
            a_recorded_node(
                42,
                "Stream/Input/Video",
                RUNNING,
                &[
                    (WHAT_IT_CALLS_ITSELF, ALO_OSS_OWN_IDENTIFIER),
                    (NAMING_THE_AGENT, "@alo"),
                ],
            ),
            a_recorded_link(50, 30, 42),
        ]);
        let in_use = in_use_in(objects(&record).objects()).unwrap();
        let one = in_use.first().unwrap();
        assert!(one.by().is_the_agents());
        assert_eq!(one.by().agent().map(Grantee::as_str), Some("@alo"));
    }

    /// **A sandboxed application cannot claim to be alo OS or the agent.** The
    /// refusal path that matters most in this file: the machine stamps an
    /// application's identity onto its connection, a stamped identity is only
    /// ever read as an application, and an application writing alo OS's
    /// reserved identifier about itself gets a line naming itself rather than
    /// the agent's terracotta.
    #[test]
    fn an_application_cannot_claim_to_be_alo_os_or_the_agent() {
        let record = a_record_of(&[
            a_recorded_node(30, A_VIDEO_SOURCE, RUNNING, &[]),
            a_recorded_node(
                42,
                "Stream/Input/Video",
                RUNNING,
                &[
                    (THE_STAMPED_IDENTITY, "com.example.Pretender"),
                    (WHAT_IT_CALLS_ITSELF, ALO_OSS_OWN_IDENTIFIER),
                    (NAMING_THE_AGENT, "@alo"),
                    (WHAT_IT_IS_CALLED, "alo OS"),
                ],
            ),
            a_recorded_link(50, 30, 42),
        ]);
        let in_use = in_use_in(objects(&record).objects()).unwrap();
        let one = in_use.first().unwrap();
        assert!(!one.by().is_the_agents(), "it was drawn as the agent");
        assert!(!one.by().is_alo_os(), "it was drawn as alo OS");
        assert_eq!(
            one.by().application().map(Application::identifier),
            Some("com.example.Pretender"),
            "the stamped identity is not what was believed"
        );
    }

    /// **A client that says nothing usable about itself is still shown**, as
    /// something this machine cannot name — never left off.
    #[test]
    fn a_client_that_names_itself_unusably_is_still_shown() {
        for props in [
            vec![],
            vec![(WHAT_IT_CALLS_ITSELF, "")],
            vec![(WHAT_IT_CALLS_ITSELF, "not an identifier")],
            vec![(THE_STAMPED_IDENTITY, " ")],
        ] {
            let record = a_record_of(&[
                a_recorded_node(30, A_VIDEO_SOURCE, RUNNING, &[]),
                a_recorded_node(42, "Stream/Input/Video", RUNNING, &props),
                a_recorded_link(50, 30, 42),
            ]);
            let in_use = in_use_in(objects(&record).objects()).unwrap();
            assert_eq!(in_use.len(), 1, "{props:?}");
            let one = in_use.first().unwrap();
            assert_eq!(one.what(), Used::Camera, "{props:?}");
            assert!(one.by().is_something_it_cannot_name(), "{props:?}");
        }
    }

    /// **A node the record says has gone is not a use**, and does not stop the
    /// rest of the record being read.
    #[test]
    fn a_node_the_record_says_has_gone_is_passed_over() {
        let record = r#"[
            {"id": 30, "type": "PipeWire:Interface:Node", "info": null},
            {"id": 31, "type": "PipeWire:Interface:Node",
             "info": {"state": "running", "props": {"media.class": "Audio/Source"}}},
            {"id": 50, "type": "PipeWire:Interface:Link", "info": null},
            {"id": 60, "type": "PipeWire:Interface:Client",
             "info": {"props": {"application.name": "something"}}}
        ]"#;
        let in_use = in_use_in(objects(record).objects()).unwrap();
        assert_eq!(in_use.len(), 1);
        assert_eq!(in_use.first().map(Use::what), Some(Used::Microphone));
    }

    /// **An object that cannot be read is refused, not skipped.** Every one of
    /// these would otherwise leave an indicator that reads exactly like a quiet
    /// room, which is the one thing it may never do.
    ///
    /// Text that is not a record at all — nothing, something that is not JSON, a
    /// single object rather than a list — is refused a step earlier, by
    /// `alo-media-server`, and is held by that crate's own tests. What is left
    /// here is what this crate is the judge of: an object in a perfectly good
    /// record that does not say what this crate needs.
    #[test]
    fn an_object_that_cannot_be_read_is_refused_rather_than_read_as_quiet() {
        for unreadable in [
            r#"[{"type": "PipeWire:Interface:Node"}]"#,
            r#"[{"id": 1, "type": "PipeWire:Interface:Node", "info": {"state": "running"}}]"#,
            r#"[{"id": 1, "type": "PipeWire:Interface:Link", "info": {"props": {}}}]"#,
        ] {
            let refused = in_use_in(objects(unreadable).objects()).unwrap_err();
            assert!(
                matches!(refused, NotHeard::NotUnderstood { .. }),
                "{unreadable} was read as {refused:?}"
            );
            assert!(!refused.diagnosis().is_empty(), "{unreadable}");
        }
    }

    /// **An empty record is a quiet room**, which is a different answer from an
    /// unreadable one and is not a refusal.
    #[test]
    fn an_empty_record_is_a_quiet_room() {
        assert!(in_use_in(objects("[]").objects()).unwrap().is_empty());
    }
}
