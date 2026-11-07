//! Attach encoding to an already running, permission-scoped capture stream.

use crate::{recorded_video::Progress, recorder::MediaError, words::t};
use gst::prelude::*;
use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

pub(crate) enum Audio {
    Off,
    Microphone,
    #[cfg(feature = "verification")]
    Generated,
}

pub(crate) struct Signals {
    pub(crate) fell_behind: Arc<AtomicBool>,
    pub(crate) paused: Arc<AtomicBool>,
}

pub(crate) fn attach(
    pipeline: &gst::Pipeline,
    path: &Path,
    width: i32,
    height: i32,
    frame_rate: i32,
    audio: Audio,
    signals: Signals,
) -> Result<Arc<Mutex<Progress>>, MediaError> {
    let microphone_clock = matches!(audio, Audio::Microphone);
    #[cfg(feature = "verification")]
    let generated_audio = matches!(audio, Audio::Generated);
    let audio_source = match audio {
        Audio::Off => None,
        Audio::Microphone => {
            // Recording tolerates more scheduling jitter than a live call. The
            // two-second ring still reads in 20 ms chunks; it does not wait for
            // the ring to fill before supplying sound.
            Some(
                "pulsesrc name=microphone client-name=alo-Studio do-timestamp=true buffer-time=2000000 latency-time=20000",
            )
        }
        #[cfg(feature = "verification")]
        Audio::Generated => Some(
            "audiotestsrc name=microphone is-live=true do-timestamp=true wave=sine volume=0.05",
        ),
    };
    // The explicitly chosen low-resource profile also avoids Opus's most
    // expensive search. The sample rate, channel count and bitrate stay fixed.
    let audio_encoding = crate::encoding_profile::audio(frame_rate);
    let audio = audio_source.map(|source| format!(
        " {source} ! queue name=audio-input ! audioconvert ! audioresample ! {audio_encoding} ! queue name=audio-buffer ! container."
    )).unwrap_or_default();
    // Drop frames before conversion and scaling so a lower chosen rate reduces
    // processing work as well as the encoder's load.
    // Resize in the source format first: converting a full display to I420
    // before downscaling spends CPU on pixels the chosen size will discard.
    let video_encoding = crate::encoding_profile::video(frame_rate);
    let bin = gst::parse::bin_from_description(&format!(
        "queue name=record-entry max-size-buffers=2 max-size-bytes=0 max-size-time=0 ! capsfilter name=record-format ! videorate skip-to-first=true drop-only=true ! videoscale ! videoconvert ! capsfilter name=size \
         ! queue name=record-buffer max-size-time=0 max-size-bytes=134217728 max-size-buffers=60 ! {video_encoding} \
         ! queue ! webmmux name=container offset-to-zero=true ! filesink name=output async=false {audio}"
    ), true).map_err(failed)?;
    let by_name = |name| {
        bin.by_name(name).ok_or_else(|| {
            MediaError(format!(
                "{} ({name}).",
                t("A recording component is unavailable")
            ))
        })
    };
    #[cfg(feature = "verification")]
    if let Some(source) = bin.by_name("microphone")
        && generated_audio
    {
        crate::verification_audio::delay(&source, pipeline)?;
    }
    let start = crate::recording_start::Start::observe(&bin)?;
    crate::audio_buffering::prepare(&bin, signals.fell_behind.clone())?;
    let tee = pipeline
        .by_name("frames")
        .ok_or_else(|| MediaError(t("The selected source is unavailable.")))?;
    // An active preview may have negotiated a rate below the chosen ceiling.
    // Drop-only conversion must preserve that rate rather than demand an upsample.
    let ceiling = gst::Fraction::new(frame_rate, 1);
    let selected = tee
        .static_pad("sink")
        .and_then(|pad| pad.current_caps())
        .ok_or_else(|| MediaError(t("Please wait for the live preview, then try again.")))?;
    by_name("record-format")?.set_property("caps", &selected);
    let rate = selected
        .structure(0)
        .and_then(|structure| structure.get::<gst::Fraction>("framerate").ok())
        .filter(|rate| *rate > gst::Fraction::new(0, 1) && *rate < ceiling)
        .unwrap_or(ceiling);
    let caps = gst::Caps::builder("video/x-raw")
        .field("format", "I420")
        .field("width", width)
        .field("height", height)
        .field("framerate", rate)
        .field("pixel-aspect-ratio", gst::Fraction::new(1, 1))
        .build();
    by_name("size")?.set_property("caps", &caps);
    by_name("output")?.set_property("location", path.to_string_lossy().as_ref());
    let progress = Arc::new(Mutex::new(Progress::default()));
    let observed = progress.clone();
    by_name("encoder")?
        .static_pad("src")
        .ok_or_else(|| MediaError(t("The video encoder is unavailable.")))?
        .add_probe(gst::PadProbeType::BUFFER, move |_, info| {
            if let Some(buffer) = info.buffer()
                && let Ok(mut progress) = observed.lock()
            {
                progress.observe(buffer);
            }
            gst::PadProbeReturn::Ok
        });
    // Retained video remains bounded independently of audio buffering.
    let video_fell_behind = signals.fell_behind;
    by_name("record-buffer")?.connect("overrun", false, move |_| {
        video_fell_behind.store(true, Ordering::Relaxed);
        None
    });
    let sink = bin
        .static_pad("sink")
        .ok_or_else(|| MediaError(t("The recording could not be connected.")))?;
    let pad = tee
        .request_pad_simple("src_%u")
        .ok_or_else(|| MediaError(t("The recording could not be connected.")))?;
    // Conversion belongs to this branch. Never ask the already selected native
    // stream to reconnect when an encoder negotiates its output size or rate.
    if pad
        .add_probe(gst::PadProbeType::EVENT_UPSTREAM, |_, info| {
            if info
                .event()
                .is_some_and(|event| matches!(event.view(), gst::EventView::Reconfigure(_)))
            {
                gst::PadProbeReturn::Drop
            } else {
                gst::PadProbeReturn::Ok
            }
        })
        .is_none()
    {
        tee.release_request_pad(&pad);
        return Err(MediaError(t("The recording could not be prepared.")));
    }
    let Some(blocked) = pad.add_probe(gst::PadProbeType::BLOCK_DOWNSTREAM, |_, _| {
        gst::PadProbeReturn::Ok
    }) else {
        tee.release_request_pad(&pad);
        return Err(MediaError(t("The recording could not be prepared.")));
    };
    // Block only the new branch while it is linked and brought to the parent's
    // state. The selected source and preview never disconnect or restart.
    let result = (|| {
        pipeline.add(&bin).map_err(failed)?;
        pad.link_full(
            &sink,
            gst::PadLinkCheck::DEFAULT | gst::PadLinkCheck::NO_RECONFIGURE,
        )
        .map_err(failed)?;
        if microphone_clock {
            crate::recording_clock::select(pipeline, &bin, &signals.paused)?;
        } else {
            bin.sync_state_with_parent().map_err(failed)?;
        }
        #[cfg(feature = "verification")]
        if std::env::var_os("ALO_STUDIO_VERIFY_PREPARE_DELAY").is_some() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        // State synchronization does not wait for a live microphone's first
        // sample. Gate video on that actual sample as well as branch readiness.
        let ready = pipeline.current_running_time().ok_or_else(|| {
            MediaError(t("The recording clock is unavailable. Please try again."))
        })?;
        start.admit_video(pipeline, &sink, ready, progress.clone())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = pad.unlink(&sink);
        let _ = bin.set_state(gst::State::Null);
        let _ = pipeline.remove(&bin);
        tee.release_request_pad(&pad);
    }
    pad.remove_probe(blocked);
    result?;
    Ok(progress)
}

fn failed(error: impl std::fmt::Display) -> MediaError {
    MediaError(error.to_string())
}
