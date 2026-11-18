//! Fixed recording requests on a machine with Studio installed and granted.
//!
//! The answers are scripted. These tries measure action selection and arguments;
//! the installed walk measures consent, actual state changes and saved media.

use alo_adapters::studio::{ACTION_NAMES, APPLICATION};
use alo_protocol::Done;
use serde_json::json;

use crate::a_road::{ARequest, Answered, Step};

/// One request's starting and resulting phase, and any exact argument it needs.
struct RecordingRequest {
    /// The stable request name printed in the model report.
    name: &'static str,
    /// What the person asks, without naming the protocol action.
    words: &'static str,
    /// What a status read shows before the requested action.
    before: &'static str,
    /// What a status read shows once the scripted action has completed.
    after: &'static str,
    /// Argument names and values this particular request decides.
    given: &'static [(&'static str, &'static str)],
}

/// In the adapter's declaration order. Tests require every declared action.
const REQUESTS: &[RecordingRequest] = &[
    RecordingRequest {
        name: "read Studio status",
        words: "alo Studio is open. Tell me whether my recording is paused.",
        before: "paused",
        after: "paused",
        given: &[],
    },
    RecordingRequest {
        name: "prepare a new Studio recording",
        words: "alo Studio is open in Review. Prepare a new recording; do not choose a source or start it.",
        before: "review",
        after: "ready",
        given: &[],
    },
    RecordingRequest {
        name: "choose a screen in Studio",
        words: "alo Studio is open. Show me its system screen picker so I can choose a screen to record. Do not start recording.",
        before: "ready",
        after: "choosing",
        given: &[],
    },
    RecordingRequest {
        name: "choose a window in Studio",
        words: "alo Studio is open. Show me its system window picker so I can choose one window to record. Do not start recording.",
        before: "ready",
        after: "choosing",
        given: &[],
    },
    RecordingRequest {
        name: "start a Studio recording",
        words: "In alo Studio I have already selected and approved the source, its preview is ready and the countdown is off. Start recording it.",
        before: "selected",
        after: "recording",
        given: &[],
    },
    RecordingRequest {
        name: "pause a Studio recording",
        words: "Pause the recording currently running in alo Studio.",
        before: "recording",
        after: "paused",
        given: &[],
    },
    RecordingRequest {
        name: "resume a Studio recording",
        words: "Resume my paused recording in alo Studio.",
        before: "paused",
        after: "recording",
        given: &[],
    },
    RecordingRequest {
        name: "finish a Studio recording",
        words: "Finish my running recording in alo Studio so I can review the saved video.",
        before: "recording",
        after: "review",
        given: &[],
    },
    RecordingRequest {
        name: "enable the Studio microphone",
        words: "My source is selected in alo Studio and recording has not started. Turn on Record microphone for this recording. Do not start it yet.",
        before: "selected",
        after: "selected",
        given: &[],
    },
    RecordingRequest {
        name: "disable the Studio microphone",
        words: "My source is selected in alo Studio and recording has not started. Turn off Record microphone. Do not start recording.",
        before: "selected",
        after: "selected",
        given: &[],
    },
    RecordingRequest {
        name: "set Studio recording quality",
        words: "My source is selected in alo Studio. Set its recording quality to Lightweight, 360p at up to 5 frames per second. Do not start recording.",
        before: "selected",
        after: "selected",
        given: &[("quality", "lightweight")],
    },
    RecordingRequest {
        name: "turn off the Studio countdown",
        words: "In alo Studio, turn off the countdown before my next recording. Do not start recording.",
        before: "selected",
        after: "selected",
        given: &[],
    },
    RecordingRequest {
        name: "set a three-second Studio countdown",
        words: "In alo Studio, set a three-second countdown for my next recording. Do not start recording.",
        before: "selected",
        after: "selected",
        given: &[],
    },
    RecordingRequest {
        name: "set a five-second Studio countdown",
        words: "In alo Studio, set a five-second countdown for my next recording. Do not start recording.",
        before: "selected",
        after: "selected",
        given: &[],
    },
    RecordingRequest {
        name: "cancel the Studio countdown",
        words: "Cancel alo Studio's countdown now, before it starts recording.",
        before: "countdown",
        after: "selected",
        given: &[],
    },
    RecordingRequest {
        name: "show the Studio library",
        words: "alo Studio is open and idle. Show me my private recording library.",
        before: "ready",
        after: "ready",
        given: &[],
    },
    RecordingRequest {
        name: "review a Studio recording",
        words: "In alo Studio, open the completed recording whose library identifier is 1000-20-1 for review.",
        before: "ready",
        after: "review",
        given: &[("recording", "1000-20-1")],
    },
    RecordingRequest {
        name: "rename a Studio recording",
        words: "In alo Studio I am reviewing a completed recording. Rename it Product demonstration.",
        before: "review",
        after: "review",
        given: &[("title", "Product demonstration")],
    },
    RecordingRequest {
        name: "export a Studio recording",
        words: "In alo Studio I am reviewing a completed recording. Open Save a copy so I can choose where to export it. Let me choose the destination.",
        before: "review",
        after: "review",
        given: &[],
    },
];

/// A request for every typed recording control, using its declared name.
pub(crate) fn the_requests() -> Vec<ARequest> {
    ACTION_NAMES
        .iter()
        .zip(REQUESTS)
        .map(|(&verb, request)| {
            let answer = if verb == status_name() {
                status(request.before, verb, false)
            } else {
                Answered::With(Done::ApplicationControlled {
                    application: APPLICATION.to_owned(),
                    action: verb.to_owned(),
                    reply: None,
                    unanswered: false,
                })
            };
            let mut action = Step::of(verb, answer);
            for &(argument, value) in request.given {
                action = action.expecting(argument, value);
            }
            let road = if verb == status_name() {
                vec![action]
            } else {
                vec![
                    Step::of(status_name(), status(request.before, verb, false)).a_look(),
                    action,
                ]
            };
            ARequest::asking(request.name, request.words, road)
                .or_looking_at(Step::of(status_name(), status(request.after, verb, true)))
        })
        .collect()
}

/// The adapter puts its status read first, without a second copy of its name.
fn status_name() -> &'static str {
    ACTION_NAMES.first().copied().unwrap_or_default()
}

/// Relevant states as the normal application's versioned status represents them.
fn status(phase: &str, action: &str, after: bool) -> Answered {
    let name = action.rsplit('.').next().unwrap_or_default();
    let microphone = match name {
        "disable_microphone" => !after,
        "enable_microphone" => after,
        _ => false,
    };
    let countdown = match (name, after) {
        ("countdown_three", true) => 3,
        ("countdown_five", true) => 5,
        ("countdown_off", false) => 3,
        _ => 0,
    };
    let preview = !matches!(phase, "ready" | "choosing" | "review");
    Answered::With(Done::ApplicationControlled {
        application: APPLICATION.to_owned(),
        action: status_name().to_owned(),
        reply: Some(
            json!({
                "version": 1, "phase": phase,
                "source_kind": if name == "choose_window" { "window" } else { "screen" },
                "microphone_enabled": microphone,
                "microphone_active": phase == "recording" && microphone,
                "quality": if after && name == "set_quality" { "lightweight" } else { "full_hd" },
                "countdown_seconds": countdown, "elapsed_ms": 0,
                "recording_id": if phase == "review" { Some("1000-20-1") } else { None },
                "preview_ready": preview, "export_pending": after && name == "export",
                "can_start": phase == "selected" && preview,
                "can_pause": phase == "recording", "can_resume": phase == "paused",
                "can_finish": matches!(phase, "preparing" | "recording" | "paused"),
            })
            .to_string(),
        ),
        unanswered: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Writing, answering, proposing, reading, the_verbs, the_words};
    use crate::{TheLevel, one_try_at};
    use alo_instructing::WhatItMayReach;

    #[test]
    fn every_studio_request_accepts_its_action_and_actual_status_reads_at_both_levels() {
        for level in [TheLevel::FullAccess, TheLevel::AskEachTime] {
            for ((verb, fixture), request) in ACTION_NAMES.iter().zip(REQUESTS).zip(the_requests()) {
                let mut lines = Vec::new();
                if *verb != status_name() {
                    lines.push(reading(status_name(), &[]));
                    lines.push(proposing(verb, fixture.given));
                }
                lines.push(reading(status_name(), &[]));
                lines.push(reading(status_name(), &[]));
                lines.push(answering("Done."));
                let mut model = Writing::these(&lines);
                let tried = one_try_at(
                    level, &request, &WhatItMayReach::nothing().at_full_access(true),
                    &the_verbs(), &the_words(), &mut model,
                );
                assert!(tried.took_the_road(), "{level:?}: {verb}: {tried:?}");
            }
        }
    }

    #[test]
    fn a_status_read_before_a_change_describes_the_state_before_the_change() {
        for ((verb, fixture), request) in ACTION_NAMES.iter().zip(REQUESTS).zip(the_requests()) {
            if *verb == status_name() {
                continue;
            }
            let mut model = Writing::these(&[
                reading(status_name(), &[]), proposing(verb, fixture.given), answering("Done."),
            ]);
            let tried = one_try_at(
                TheLevel::FullAccess, &request, &WhatItMayReach::nothing().at_full_access(true),
                &the_verbs(), &the_words(), &mut model,
            );
            assert!(tried.took_the_road(), "{verb}: {tried:?}");
            let Answered::With(done) = status(fixture.before, verb, false) else {
                panic!("a status is a protocol reply");
            };
            let expected = alo_protocol::ToAnAgent::did(done).written().expect("the reply writes");
            assert!(model.shown[1].contains(&expected), "{verb}: {}", model.shown[1]);
        }
    }

    #[test]
    fn status_alone_cannot_complete_a_requested_change() {
        for request in the_requests().into_iter().skip(1) {
            let mut model = Writing::these(&[reading(status_name(), &[]), answering("Done.")]);
            let tried = one_try_at(
                TheLevel::FullAccess, &request, &WhatItMayReach::nothing().at_full_access(true),
                &the_verbs(), &the_words(), &mut model,
            );
            assert!(!tried.took_the_road(), "{}: {tried:?}", request.named());
        }
    }
}
