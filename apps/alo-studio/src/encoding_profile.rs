//! Keep preflight and recording on the same selected encoding configuration.

/// The selected video encoder, shared by preflight and actual recording.
pub(crate) fn video(frame_rate: i32) -> &'static str {
    if frame_rate <= 5 {
        "vp8enc name=encoder deadline=1 cpu-used=16 threads=2 lag-in-frames=0 target-bitrate=1500000 keyframe-max-dist=30"
    } else {
        "vp9enc name=encoder deadline=1 cpu-used=8 threads=4 row-mt=true lag-in-frames=0 target-bitrate=4500000 keyframe-max-dist=60"
    }
}

/// The selected microphone format and voice-encoding cost.
pub(crate) fn audio(frame_rate: i32) -> &'static str {
    if frame_rate <= 5 {
        "audio/x-raw,format=S16LE,rate=16000,channels=1 ! opusenc bitrate=24000 complexity=0 audio-type=voice"
    } else {
        "audio/x-raw,rate=48000,channels=1 ! opusenc bitrate=96000 complexity=10"
    }
}
