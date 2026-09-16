//! How alo OS's own capture announces itself, so it is on the indicator.
//!
//! The plan's acceptance says **a screenshot is a use of the screen and appears
//! on task 1's indicator for its moment**, and there are two ways to make that
//! sentence true. One is for this crate to tell the indicator that a screenshot
//! is happening. The other is for the capture to be a stream on the machine's
//! own media server like anybody else's, so that `alo-in-use` reads it back out
//! of the server's record without knowing this crate exists.
//!
//! **It is the second**, and the difference is the whole of task 1. An
//! indicator fed by the things it is watching is an indicator that shows
//! whatever those things choose to say; `alo_in_use::InUse::read_from` has one
//! door and it is the media server, and a screenshot of ours that announced
//! itself to the indicator directly would be the first exception to that — from
//! inside the operating system, which is exactly where `docs/features.md`'s
//! *by any application, including ours* is pointed.
//!
//! So this file is short: it is the properties a capture of ours opens its
//! stream with, spelled in the names `alo_in_use::heard` reads, and nothing
//! else.
//!
//! # What it deliberately does not say
//!
//! **It does not stamp an application's identity.** That property is the
//! machine's to write about a sandboxed application, and `alo_in_use::heard`
//! reads it as *an application* and nothing else. A capture of alo OS's own
//! that wrote one would appear as an application nobody installed.
//!
//! **It does not name an agent.** A picture of the screen taken because a
//! person pressed a key is alo OS acting for them, not an agent acting: naming
//! an agent here would draw the line in terracotta (ADR 0010), which means *the
//! machine is doing this on your behalf as an agent* and would be a lie about
//! who asked. The agent's own screenshot is the plan's task 6, through a verb
//! that is approved, and it will name the agent because it is the agent's.

use alo_in_use::heard;

/// What the capture's own node is called on the machine's graph.
///
/// Not shown to anybody: what a person reads about a use is
/// `alo_in_use::Line`, which says *the screen is in use by alo OS itself*. This
/// is what somebody debugging the graph sees, and it says which part of alo OS
/// is on it.
pub const THE_NODE_NAME: &str = "alo-os-capture";

/// The properties a picture alo OS takes of its own screen is announced under.
///
/// Read back by `alo_in_use::heard` as [`alo_in_use::By::alo_os_itself`], which
/// is what puts the capture on the indicator for its moment.
pub const ANNOUNCED_AS: [(&str, &str); 2] = [
    (heard::WHAT_IT_CALLS_ITSELF, heard::ALO_OSS_OWN_IDENTIFIER),
    (heard::THE_NODE_NAME, THE_NODE_NAME),
];

/// The announcement, written the way the media server takes one.
///
/// A list of names and values between braces, which is the notation the rented
/// server reads a client's properties in.
#[must_use]
pub fn announced() -> String {
    let inside = ANNOUNCED_AS
        .iter()
        .map(|(named, value)| format!("{named} = {value}"))
        .collect::<Vec<String>>()
        .join(" ");
    format!("{{ {inside} }}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The capture says it is alo OS**, under the name the indicator reads
    /// identities by — one spelling of it, in `alo-in-use`, used here rather
    /// than written again.
    #[test]
    fn the_capture_says_it_is_alo_os() {
        assert!(
            ANNOUNCED_AS.contains(&(heard::WHAT_IT_CALLS_ITSELF, heard::ALO_OSS_OWN_IDENTIFIER)),
            "{ANNOUNCED_AS:?}"
        );
        assert_eq!(heard::WHAT_IT_CALLS_ITSELF, "application.id");
    }

    /// **It never stamps an application's identity.** That property is the
    /// machine's to write about a sandboxed application, and a capture of ours
    /// wearing one would appear on the indicator as an application nobody
    /// installed.
    #[test]
    fn the_capture_never_claims_to_be_an_application() {
        for (named, value) in ANNOUNCED_AS {
            assert!(
                !named.contains("portal"),
                "{named} is the machine's stamp to write"
            );
            assert!(!named.contains("app_id"), "{named}");
            assert!(!value.contains("portal"), "{value}");
        }
    }

    /// **It never names an agent.** A picture taken because a person pressed a
    /// key is alo OS acting for them; naming an agent would draw the line in
    /// terracotta and say the agent asked for it (ADR 0010).
    #[test]
    fn the_capture_never_names_an_agent() {
        for (named, _) in ANNOUNCED_AS {
            assert_ne!(named, heard::NAMING_THE_AGENT, "{named}");
        }
        assert!(!announced().contains(heard::NAMING_THE_AGENT));
    }

    /// **The announcement is written the way the server takes one**: names and
    /// values between braces.
    #[test]
    fn the_announcement_is_written_the_way_the_server_takes_one() {
        let announced = announced();
        assert!(announced.starts_with("{ "), "{announced}");
        assert!(announced.ends_with(" }"), "{announced}");
        assert!(
            announced.contains("application.id = os.alo.AloOs"),
            "{announced}"
        );
        assert!(
            announced.contains("node.name = alo-os-capture"),
            "{announced}"
        );
    }

    /// **The indicator reads this announcement as alo OS itself.** The record
    /// is written the way the media server writes one — a running screen source
    /// with our capture reading from it — and handed to `alo-in-use`, which
    /// knows nothing about this crate.
    #[test]
    fn the_indicator_reads_this_announcement_as_alo_os_itself() {
        let announced: serde_json::Map<String, serde_json::Value> = ANNOUNCED_AS
            .iter()
            .map(|(named, value)| {
                (
                    (*named).to_owned(),
                    serde_json::Value::String((*value).to_owned()),
                )
            })
            .collect();
        let record = serde_json::json!([
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
                "info": { "state": heard::RUNNING, "props": announced },
            },
            {
                "id": 42,
                "type": heard::A_LINK,
                "info": {
                    "props": { "link.output.node": 40, "link.input.node": 41 },
                },
            },
        ])
        .to_string();

        let in_use = heard::in_use_in(&record).unwrap_or_default();
        let ours = in_use
            .iter()
            .find(|used| used.at() == alo_in_use::UseId::recorded(41));
        assert!(
            ours.is_some_and(
                |used| used.what() == alo_in_use::Used::Screen && used.by().is_alo_os()
            ),
            "{in_use:?}"
        );
    }
}
