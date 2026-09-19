//! **What a machine has**, as facts handed in rather than facts read here.
//!
//! Which hardware decoders a chip holds and whether a redistributable decoder
//! was installed are measurements, and they are measured by whoever can measure
//! them: the graphics stack, the installer, the update that brought the decoder.
//! This crate decides *given* them.
//!
//! That split is the same one `alo-choosing` makes with the variables it reads
//! and `alo-media-server` makes with where the server is listening, and it is
//! what lets every decision below be tested against machines nobody here owns —
//! a laptop with no video hardware at all, a workstation whose chip decodes
//! HEVC, an image built before a licensed decoder existed.

use crate::codec::{Audio, Video};

/// **One machine, as far as playing is concerned.**
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AMachine {
    /// The video codecs this machine's own hardware decodes.
    decodes_in_hardware: Vec<Video>,
    /// Whether a redistributable licensed decoder is installed, and for what.
    licensed: Vec<Video>,
    /// The audio codecs this machine's own hardware decodes.
    ///
    /// Rare and real: some embedded parts decode AAC without the processor.
    decodes_sound_in_hardware: Vec<Audio>,
}

impl AMachine {
    /// A machine with no video hardware and no licensed decoder.
    ///
    /// The honest default, and the shape of an ordinary build host — which is
    /// also, deliberately, the shape this crate is mostly tested against.
    #[must_use]
    pub const fn with_nothing() -> Self {
        Self {
            decodes_in_hardware: Vec::new(),
            licensed: Vec::new(),
            decodes_sound_in_hardware: Vec::new(),
        }
    }

    /// This machine, with these video codecs decoded by its own hardware.
    #[must_use]
    pub fn decoding_in_hardware(mut self, codecs: &[Video]) -> Self {
        self.decodes_in_hardware = codecs.to_vec();
        self
    }

    /// This machine, with these audio codecs decoded by its own hardware.
    #[must_use]
    pub fn decoding_sound_in_hardware(mut self, codecs: &[Audio]) -> Self {
        self.decodes_sound_in_hardware = codecs.to_vec();
        self
    }

    /// This machine, carrying a redistributable licensed decoder for these.
    ///
    /// In practice one entry: `openh264` decodes H.264 and nothing else. It is a
    /// list rather than a flag because a second such decoder appearing is a
    /// change of fact, not a change of shape.
    #[must_use]
    pub fn with_a_licensed_decoder_for(mut self, codecs: &[Video]) -> Self {
        self.licensed = codecs.to_vec();
        self
    }

    /// Whether this machine's own hardware decodes this video codec.
    #[must_use]
    pub fn decodes_video_in_hardware(&self, codec: Video) -> bool {
        self.decodes_in_hardware.contains(&codec)
    }

    /// Whether this machine's own hardware decodes this audio codec.
    #[must_use]
    pub fn decodes_audio_in_hardware(&self, codec: Audio) -> bool {
        self.decodes_sound_in_hardware.contains(&codec)
    }

    /// Whether a redistributable licensed decoder for this codec is installed.
    #[must_use]
    pub fn has_a_licensed_decoder_for(&self, codec: Video) -> bool {
        self.licensed.contains(&codec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A machine with nothing has nothing**, which is the default a build host
    /// and a plain laptop both are.
    #[test]
    fn a_machine_with_nothing_decodes_nothing_in_hardware() {
        let machine = AMachine::with_nothing();
        for codec in Video::EVERY {
            assert!(!machine.decodes_video_in_hardware(codec));
            assert!(!machine.has_a_licensed_decoder_for(codec));
        }
        for codec in Audio::EVERY {
            assert!(!machine.decodes_audio_in_hardware(codec));
        }
        assert_eq!(machine, AMachine::default());
    }

    /// **What a machine was told it has is what it answers**, and nothing near
    /// it.
    ///
    /// A chip that decodes H.264 does not thereby decode HEVC, and the two are
    /// separate silicon often enough that guessing would be wrong on real parts.
    #[test]
    fn a_machine_answers_for_the_codec_asked_about_and_no_other() {
        let machine = AMachine::with_nothing()
            .decoding_in_hardware(&[Video::H264Baseline])
            .with_a_licensed_decoder_for(&[Video::H264Main]);
        assert!(machine.decodes_video_in_hardware(Video::H264Baseline));
        assert!(!machine.decodes_video_in_hardware(Video::Hevc));
        assert!(!machine.decodes_video_in_hardware(Video::H264Main));
        assert!(machine.has_a_licensed_decoder_for(Video::H264Main));
        assert!(!machine.has_a_licensed_decoder_for(Video::H264Baseline));
    }
}
