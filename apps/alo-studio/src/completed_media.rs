//! Read the completed file with GStreamer's media discovery engine.

use std::{path::Path, time::Duration};

use crate::{recorder::MediaError, words::t};

/// Read playback duration after EOS, with a bounded discovery deadline.
/// Call off the UI thread. Missing, damaged, empty, or audio-only media refuses
/// rather than falling back to an interface timer or encoder timestamps.
pub fn duration(path: &Path) -> Result<Duration, MediaError> {
    gst::init().map_err(failed)?;
    let uri = gst::glib::filename_to_uri(path, None).map_err(failed)?;
    let discovery =
        gst_pbutils::Discoverer::new(gst::ClockTime::from_seconds(15)).map_err(failed)?;
    let info = discovery.discover_uri(uri.as_str()).map_err(failed)?;
    if info.result() != gst_pbutils::DiscovererResult::Ok || info.video_streams().is_empty() {
        return Err(MediaError(t("The saved video could not be verified.")));
    }
    info.duration()
        .filter(|duration| *duration > gst::ClockTime::ZERO)
        .map(|duration| Duration::from_nanos(duration.nseconds()))
        .ok_or_else(|| MediaError(t("The saved video's duration could not be read.")))
}

fn failed(error: impl std::fmt::Display) -> MediaError {
    MediaError(format!(
        "{} {error}",
        t("The saved video could not be verified.")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn missing_and_damaged_media_cannot_report_a_completed_duration()
    -> Result<(), Box<dyn std::error::Error>> {
        let path = std::env::temp_dir().join(format!(
            "alo-studio-media-refusal-{}.webm",
            std::process::id()
        ));
        assert!(duration(&path).is_err());
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(b"This is not a video")?;
        drop(file);
        let refused = duration(&path).is_err();
        std::fs::remove_file(path)?;
        assert!(refused);
        Ok(())
    }
}
