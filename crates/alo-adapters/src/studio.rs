//! alo Studio's semantic controls. Source consent stays in the system picker.

use alo_capability::Effect;
use alo_strings::Word;

use crate::{
    Adapter, AdapterArg, AdapterVerb, DBusMethod, Invocation, Kind, Loaded, Mechanism, NotLoaded,
    Offer, Part, Reaches,
};

/// The separately installed recording application.
pub const APPLICATION: &str = "org.aloworld.Studio";
/// The fixed recording-control object in Studio.
const OBJECT: &str = "/org/aloworld/Studio/Recorder";
/// The versioned semantic recording interface.
const INTERFACE: &str = "org.aloworld.Studio.Recorder1";
/// The effect description shared by recording changes.
const KEEP_VIDEOS: Word = Word::saying(
    "adapters.studio.keep-videos",
    "Existing videos stay in your private library.",
)
.noting("What existing recordings retain when a recording control is used.");
/// The translated purpose of the quality argument.
const QUALITY: Word = Word::saying("adapters.studio.quality", "the recording quality")
    .noting("The recording size and frame-rate choice shown before recording.");
/// The translated purpose of the library identifier.
const RECORDING: Word = Word::saying(
    "adapters.studio.recording",
    "the saved recording identifier",
)
.noting("The identifier of the saved video to open for review.");
/// The translated purpose of the recording title.
const TITLE: Word = Word::saying("adapters.studio.title", "the recording's new name")
    .noting("The title shown in the private recording library.");
/// The full-HD choice shown in approvals.
const FULL_HD: Word = Word::saying("adapters.studio.full-hd", "1080p, up to 30 fps")
    .noting("The recording size and maximum frame rate offered before starting capture.");
/// The HD choice shown in approvals.
const HD: Word = Word::saying("adapters.studio.hd", "720p, up to 30 fps")
    .noting("The recording size and maximum frame rate offered before starting capture.");
/// The original-size choice shown in approvals.
const ORIGINAL: Word = Word::saying("adapters.studio.original", "original size, up to 30 fps")
    .noting("The recording size and maximum frame rate offered before starting capture.");
/// The lower-cost quality choice shown in approvals.
const LIGHTWEIGHT: Word = Word::saying("adapters.studio.lightweight", "360p, up to 5 fps")
    .noting("A recording quality choice for slower devices.");
/// The closed set of named quality options.
const QUALITY_ARGS: &[AdapterArg] = &[AdapterArg {
    name: "quality",
    purpose: QUALITY,
    kind: Kind::Choice(&[
        Offer {
            name: "full_hd",
            words: FULL_HD,
        },
        Offer {
            name: "hd",
            words: HD,
        },
        Offer {
            name: "original",
            words: ORIGINAL,
        },
        Offer {
            name: "lightweight",
            words: LIGHTWEIGHT,
        },
    ]),
}];
/// A bounded library identifier, never a filesystem path.
const RECORDING_ARGS: &[AdapterArg] = &[AdapterArg {
    name: "recording",
    purpose: RECORDING,
    kind: Kind::Name { longest: 100 },
}];
/// A bounded recording title.
const TITLE_ARGS: &[AdapterArg] = &[AdapterArg {
    name: "title",
    purpose: TITLE,
    kind: Kind::Name { longest: 200 },
}];
/// The purpose of the status read.
const STATUS_PURPOSE: Word = Word::saying(
    "adapters.studio.status.purpose",
    "read alo Studio's current recording status",
)
.noting("Read whether recording is ready, running, paused, saving or complete.");
/// The sentence recorded for the status read.
const STATUS_SENTENCE: Word = Word::saying(
    "adapters.studio.status.sentence",
    "read alo Studio's recording status",
)
.noting("The read recorded when the person asks about their recording.");
/// The status read without an agent.
const STATUS_BY_HAND: Word = Word::saying(
    "adapters.studio.status.by-hand",
    "In alo Studio, read the status beside the preview and recording controls",
)
.noting("How to inspect the recording without an agent.");

/// Declare each fixed control with translated purpose, approval and effect words.
macro_rules! controls {
    ($(($name:ident, $key:literal, $purpose:literal, $sentence:literal, $manual:literal, $changes:literal, $method:literal, $args:expr, $parameters:expr)),* $(,)?) => {
        /// Every translated word this recording adapter declares.
        pub const WORDS: &[Word] = &[
            KEEP_VIDEOS, QUALITY, RECORDING, TITLE, FULL_HD, HD, ORIGINAL, LIGHTWEIGHT,
            STATUS_PURPOSE, STATUS_SENTENCE, STATUS_BY_HAND,
            $(
                Word::saying(concat!("adapters.studio.", $key, ".purpose"), $purpose).noting("What the recording control does."),
                Word::saying(concat!("adapters.studio.", $key, ".sentence"), $sentence).noting("The recording change the person approves."),
                Word::saying(concat!("adapters.studio.", $key, ".by-hand"), $manual).noting("How the person does this in alo Studio without an agent."),
                Word::saying(concat!("adapters.studio.", $key, ".changes"), $changes).noting("What accepting this recording change does."),
            )*
        ];

        /// Typed controls, each addressed to Studio's own named operation.
        pub const STUDIO: Adapter = Adapter {
            name: "studio", application: APPLICATION, releases: &["0.1"],
            mechanism: Mechanism::DBus, words: WORDS,
            verbs: &[
                AdapterVerb {
                    name: "status", purpose: STATUS_PURPOSE, effect: Effect::Read, sends: crate::adapter_verb::Sends::Never, args: &[],
                    reaches: Reaches::OnlyItsApplication, sentence: STATUS_SENTENCE,
                    by_hand: Some(STATUS_BY_HAND),
                    carried_out: Invocation::DBus(DBusMethod { object: OBJECT, interface: INTERFACE, method: "GetStatus", parameters: &[] }),
                },
                $(AdapterVerb {
                    name: stringify!($name),
                    purpose: Word::saying(concat!("adapters.studio.", $key, ".purpose"), $purpose),
                    effect: Effect::Change, sends: crate::adapter_verb::Sends::Never, args: $args, reaches: Reaches::OnlyItsApplication,
                    sentence: Word::saying(concat!("adapters.studio.", $key, ".sentence"), $sentence),
                    by_hand: Some(Word::saying(concat!("adapters.studio.", $key, ".by-hand"), $manual)),
                    carried_out: Invocation::DBus(DBusMethod { object: OBJECT, interface: INTERFACE, method: $method, parameters: $parameters }),
                },)*
            ],
        };

        /// Load recording controls with checked, translated change descriptions.
        ///
        /// # Errors
        /// [`NotLoaded`] if a declaration violates the adapter contract.
        pub fn loaded() -> Result<Loaded, NotLoaded> {
            let mut loaded = crate::load(&STUDIO)?;
            $(loaded = loaded.describing_change(stringify!($name), Word::saying(concat!("adapters.studio.", $key, ".changes"), $changes), Some(KEEP_VIDEOS))?;)*
            Ok(loaded)
        }
    };
}

controls! {
    (new_recording, "new-recording", "prepare a new alo Studio recording", "prepare a new recording in alo Studio", "In alo Studio, choose New recording", "Clears the previous source selection and turns the microphone option off.", "NewRecording", &[], &[]),
    (choose_screen, "choose-screen", "ask the person to choose a screen to record", "open alo Studio's system screen picker", "In alo Studio, choose A screen and Choose a screen", "Opens the system screen picker; no source is selected for the person.", "ChooseScreen", &[], &[]),
    (choose_window, "choose-window", "ask the person to choose a window to record", "open alo Studio's system window picker", "In alo Studio, choose A window and Choose a window", "Opens the system window picker; no source is selected for the person.", "ChooseWindow", &[], &[]),
    (start, "start", "start the selected recording after its performance check and countdown", "start the selected recording in alo Studio", "In alo Studio, choose Start recording", "Begins the performance check and countdown for the selected source.", "Start", &[], &[]),
    (pause, "pause", "pause the current recording and microphone", "pause the recording in alo Studio", "In alo Studio's recording controls, choose Pause", "Pauses video and microphone capture.", "Pause", &[], &[]),
    (resume, "resume", "resume a paused recording and its chosen microphone", "resume the recording in alo Studio", "In alo Studio's recording controls, choose Resume", "Resumes video and the recording's chosen microphone.", "Resume", &[], &[]),
    (finish, "finish", "finish capture and validate the saved recording", "finish the recording in alo Studio", "In alo Studio's recording controls, choose Stop recording", "Stops capture and begins finalizing the current recording.", "Finish", &[], &[]),
    (enable_microphone, "enable-microphone", "enable the microphone option for this recording", "enable the microphone for this alo Studio recording", "In alo Studio, turn on Record microphone", "Enables the microphone option; capture starts only when recording starts.", "EnableMicrophone", &[], &[]),
    (disable_microphone, "disable-microphone", "disable the microphone option before recording", "disable the microphone for this alo Studio recording", "In alo Studio, turn off Record microphone", "Disables the microphone option before recording starts.", "DisableMicrophone", &[], &[]),
    (set_quality, "set-quality", "choose recording quality before capture starts", "set alo Studio recording quality to {quality}", "In alo Studio, choose the named Quality option", "Changes the selected quality for a future recording.", "SetQuality", QUALITY_ARGS, &[Part::TextOf("quality")]),
    (countdown_off, "countdown-off", "record without a countdown", "turn off the countdown in alo Studio", "In alo Studio, choose Off under Countdown", "Selects no countdown before recording starts.", "SetCountdown", &[], &[Part::Number(0)]),
    (countdown_three, "countdown-three", "use a three-second recording countdown", "set a three-second countdown in alo Studio", "In alo Studio, choose 3 sec under Countdown", "Selects a three-second countdown before recording starts.", "SetCountdown", &[], &[Part::Number(3)]),
    (countdown_five, "countdown-five", "use a five-second recording countdown", "set a five-second countdown in alo Studio", "In alo Studio, choose 5 sec under Countdown", "Selects a five-second countdown before recording starts.", "SetCountdown", &[], &[Part::Number(5)]),
    (cancel_countdown, "cancel-countdown", "cancel a countdown before recording starts", "cancel alo Studio's recording countdown", "In alo Studio, choose Cancel during the countdown", "Cancels the countdown before recording begins.", "CancelCountdown", &[], &[]),
    (show_library, "show-library", "show the private recording library", "show alo Studio's recording library", "In alo Studio, choose Your recordings", "Closes any preview and opens the private recording library.", "ShowLibrary", &[], &[]),
    (review, "review", "review a completed recording by its library identifier", "review {recording} in alo Studio", "In alo Studio's library, open the saved recording", "Replaces any preview with the selected saved video.", "ReviewRecording", RECORDING_ARGS, &[Part::TextOf("recording")]),
    (rename, "rename", "name the completed recording being reviewed", "name the current alo Studio recording {title}", "In alo Studio's review, edit the recording title", "Changes the current recording's saved library name.", "RenameRecording", TITLE_ARGS, &[Part::TextOf("title")]),
    (export, "export", "ask where to save a copy of the reviewed recording", "open the save dialog for the current alo Studio recording", "In alo Studio's review, choose Save a copy", "Opens the system save dialog for the current recording.", "ExportRecording", &[], &[]),
}
