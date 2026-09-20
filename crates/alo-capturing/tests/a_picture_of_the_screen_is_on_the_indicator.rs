//! A picture of the screen is a use of the screen, and it is on the indicator.
//!
//! The plan's fourth acceptance for this task: **a screenshot is a use of the
//! screen and appears on task 1's indicator for its moment, held by a test.**
//!
//! What makes it true is not a call from this crate to that one. It is that the
//! capture is a stream on the machine's own media server, announced under the
//! names `alo_in_use::heard` reads — so `alo-in-use` finds it in the server's
//! record, exactly as it finds a video call using the camera, and without
//! knowing this crate exists. This test is that path end to end: the properties
//! `alo_capturing::announcing` says a capture of ours is opened with go into a
//! record written the way the server writes one, and the answer comes back
//! through `InUse::read_from`, which is the only door there is.
//!
//! It is deliberately **not** a test that reads a boolean this crate set. A
//! test like that would pass on a machine where the indicator showed nothing.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::Token;
use alo_capturing::announcing;
use alo_in_use::{InUse, NotHeard, Streams, Use, Used, heard, in_use_words};
use alo_strings::Strings;

/// A media server whose record is this text.
struct AMachineWhoseRecordIs(String);

impl Streams for AMachineWhoseRecordIs {
    fn in_use_now(&mut self) -> Result<Vec<Use>, NotHeard> {
        // The record is read by the crate that owns reading one, and what the
        // objects in it mean is `alo-in-use`'s — the same two steps a real
        // machine takes.
        let read = alo_media_server::read(&self.0).map_err(|why| NotHeard::NotUnderstood {
            said: why.said().to_owned(),
        })?;
        heard::in_use_in(read.objects())
    }
}

/// The properties the capture announces itself with, as the record has them.
fn what_the_capture_announces() -> serde_json::Map<String, serde_json::Value> {
    announcing::ANNOUNCED_AS
        .iter()
        .map(|(named, value)| {
            (
                (*named).to_owned(),
                serde_json::Value::String((*value).to_owned()),
            )
        })
        .collect()
}

/// The machine's record while a screen cast is open and our capture is reading
/// one frame off it.
fn while_a_picture_is_being_taken() -> String {
    serde_json::json!([
        {
            "id": 40,
            "type": heard::A_NODE,
            "info": {
                "state": heard::RUNNING,
                "props": { "media.class": heard::VIDEO_INTO_THE_GRAPH },
            },
        },
        {
            "id": 41,
            "type": heard::A_NODE,
            "info": {
                "state": heard::RUNNING,
                "props": what_the_capture_announces(),
            },
        },
        {
            "id": 42,
            "type": heard::A_LINK,
            "info": { "props": { "link.output.node": 40, "link.input.node": 41 } },
        },
    ])
    .to_string()
}

/// The same machine a moment later, with nothing open at all.
fn once_it_has_been_taken() -> String {
    serde_json::json!([]).to_string()
}

/// **While the picture is being taken, the screen is in use — and the
/// indicator says alo OS is the one using it.** Read out of the media server's
/// own record, through the one door `alo-in-use` has.
#[test]
fn a_picture_being_taken_is_the_screen_in_use_by_alo_os() {
    let mut machine = AMachineWhoseRecordIs(while_a_picture_is_being_taken());
    let in_use = InUse::read_from(&mut machine).expect("the record is readable");

    assert!(!in_use.is_quiet(), "{in_use:?}");
    assert!(in_use.is_in_use(Used::Screen));
    assert!(!in_use.is_in_use(Used::Camera));
    assert!(!in_use.is_in_use(Used::Microphone));

    let ours = in_use
        .using(Used::Screen)
        .next()
        .expect("the screen is in use");
    assert!(
        ours.by().is_alo_os(),
        "alo OS's own picture is listed as {:?}",
        ours.by()
    );
}

/// **The line a person reads says the screen, and says alo OS.** In their own
/// language, with the mark and the place ADR 0010 asks for beside the words —
/// and not in terracotta, because terracotta means an agent is acting and
/// nobody asked an agent for this picture.
#[test]
fn the_line_says_the_screen_is_in_use_by_alo_os_itself() {
    let strings = Strings::of(in_use_words().expect("the indicator's own words"));
    let mut machine = AMachineWhoseRecordIs(while_a_picture_is_being_taken());
    let in_use = InUse::read_from(&mut machine).expect("the record is readable");

    let line = in_use.lines().into_iter().next().expect("one line");
    assert_eq!(line.what(), Used::Screen);

    let said = line.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("screen"), "{said}");
    assert!(said.text().contains("alo OS"), "{said}");

    assert_eq!(line.mark(), Used::Screen.mark());
    assert_eq!(line.position(), Used::Screen.position());
    assert_eq!(
        line.colour(),
        Token::Navy,
        "alo OS's own use is not the agent's"
    );
    assert!(!line.the_agents_dot());
}

/// **It is on the indicator for its moment and not after it.** A picture of the
/// screen is one frame; when the stream has gone, so has the line, and the
/// indicator says the room is quiet again.
#[test]
fn the_line_is_there_for_the_moment_and_then_it_is_not() {
    let mut taking = AMachineWhoseRecordIs(while_a_picture_is_being_taken());
    assert!(!InUse::read_from(&mut taking).expect("readable").is_quiet());

    let mut after = AMachineWhoseRecordIs(once_it_has_been_taken());
    let quiet = InUse::read_from(&mut after).expect("readable");
    assert!(quiet.is_quiet());
    assert_eq!(quiet.how_many(), 0);
}

/// **alo OS's own picture is on the list like anybody else's**, which is
/// `docs/features.md`'s ★ *by any application, including ours* meant rather than
/// claimed: the record is read the same way, the line is built the same way, and
/// nothing anywhere can leave ours off.
#[test]
fn alo_oss_own_picture_is_listed_like_anybody_elses() {
    let mut ours = AMachineWhoseRecordIs(while_a_picture_is_being_taken());
    let ours = InUse::read_from(&mut ours).expect("readable");

    // The same record, with a sandboxed application reading the screen cast
    // instead of us: one line, of the screen, either way.
    let an_application = serde_json::json!([
        {
            "id": 40,
            "type": heard::A_NODE,
            "info": {
                "state": heard::RUNNING,
                "props": { "media.class": heard::VIDEO_INTO_THE_GRAPH },
            },
        },
        {
            "id": 41,
            "type": heard::A_NODE,
            "info": {
                "state": heard::RUNNING,
                "props": {
                    "pipewire.access.portal.app_id": "com.example.VideoCall",
                    "application.name": "Video Call",
                },
            },
        },
        {
            "id": 42,
            "type": heard::A_LINK,
            "info": { "props": { "link.output.node": 40, "link.input.node": 41 } },
        },
    ])
    .to_string();
    let mut theirs = AMachineWhoseRecordIs(an_application);
    let theirs = InUse::read_from(&mut theirs).expect("readable");

    assert_eq!(ours.how_many(), theirs.how_many());
    assert_eq!(
        ours.lines().first().map(alo_in_use::Line::what),
        theirs.lines().first().map(alo_in_use::Line::what)
    );
    assert!(ours.uses().first().is_some_and(|one| one.by().is_alo_os()));
    assert!(
        theirs
            .uses()
            .first()
            .is_some_and(|one| one.by().application().is_some())
    );
}

/// **Nothing about the picture reaches the indicator.** The announcement is
/// three properties — who we are, what the node is called, and that it is a
/// video capture — and none of them says what was on the screen, what was
/// captured, or where the picture went.
#[test]
fn nothing_about_what_was_captured_reaches_the_indicator() {
    let announced = announcing::announced();
    for absent in [
        "window",
        "region",
        "title",
        "file",
        "clipboard",
        "png",
        "home",
    ] {
        assert!(
            !announced.to_lowercase().contains(absent),
            "{announced} names {absent}"
        );
    }
    // Named rather than counted: the guard is that these three are the whole
    // announcement, so a fourth property has to be argued for here before it
    // can reach a person's indicator.
    let named: Vec<&str> = announcing::ANNOUNCED_AS
        .iter()
        .map(|(named, _)| *named)
        .collect();
    assert_eq!(
        named,
        vec!["application.id", "node.name", "media.class"],
        "{announced}"
    );
}
