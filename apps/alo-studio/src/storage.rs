//! Recordings belong to the person. Completed videos are never overwritten.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// A completed recording in this application's private library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    /// A filename-safe local identity, never a user-supplied path.
    pub id: String,
    /// The person's editable title.
    pub title: String,
    /// When recording began, as Unix seconds.
    pub created: u64,
    /// Active recording time, excluding pauses.
    pub duration_ms: u64,
    /// Encoded width.
    pub width: i32,
    /// Encoded height.
    pub height: i32,
    /// Whether the person chose their microphone for this clip.
    pub microphone: bool,
}

impl Clip {
    /// The active duration shown beside the video.
    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.duration_ms)
    }
}

/// A library can fail to read or save, but never claim a video was saved anyway.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// An operating-system failure, retaining the path for useful diagnostics.
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Metadata has an unknown or damaged shape.
    #[error("The recording information could not be read: {0}")]
    Metadata(#[from] serde_json::Error),
    /// A path-like identity from damaged metadata must never escape the library.
    #[error("The recording has an invalid identifier.")]
    InvalidIdentity,
    /// The encoder produced nothing, so this cannot become a completed clip.
    #[error("No video frames were saved. Please choose the source again and retry.")]
    EmptyVideo,
}

fn io(path: &Path, source: std::io::Error) -> StorageError {
    StorageError::Io {
        path: path.to_owned(),
        source,
    }
}

/// One private folder. Export happens only after the person chooses a destination.
#[derive(Clone)]
pub struct Library {
    root: PathBuf,
}

impl Library {
    /// Create the directory if necessary. No recording device is touched.
    pub fn at(root: PathBuf) -> Result<Self, StorageError> {
        fs::create_dir_all(&root).map_err(|error| io(&root, error))?;
        Ok(Self { root })
    }
    /// Where the video is kept. Identity validation precedes every path join.
    pub fn video(&self, id: &str) -> Result<PathBuf, StorageError> {
        self.path(id, "webm")
    }
    /// The pending file remains distinguishable after a power loss.
    pub fn pending(&self, id: &str) -> Result<PathBuf, StorageError> {
        self.path(id, "webm.part")
    }
    /// Optional thumbnail saved from an actual captured frame.
    pub fn thumbnail(&self, id: &str) -> Result<PathBuf, StorageError> {
        self.path(id, "png")
    }
    /// Reserve a new output, refusing even an unlikely identity collision.
    pub fn begin(&self, width: i32, height: i32, microphone: bool) -> Result<Clip, StorageError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let id = format!(
            "{}-{}-{}",
            now.as_secs(),
            now.subsec_nanos(),
            std::process::id()
        );
        let pending = self.pending(&id)?;
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&pending)
            .map_err(|error| io(&pending, error))?;
        Ok(Clip {
            id,
            title: "Untitled recording".to_owned(),
            created: now.as_secs(),
            duration_ms: 0,
            width,
            height,
            microphone,
        })
    }
    /// Called after EOS, when the media container has been sealed successfully.
    pub fn finish(&self, clip: &Clip) -> Result<(), StorageError> {
        let pending = self.pending(&clip.id)?;
        if fs::metadata(&pending)
            .map_err(|error| io(&pending, error))?
            .len()
            == 0
        {
            return Err(StorageError::EmptyVideo);
        }
        let video = self.video(&clip.id)?;
        OpenOptions::new()
            .write(true)
            .open(&pending)
            .and_then(|file| file.sync_all())
            .map_err(|error| io(&pending, error))?;
        // hard_link fails if the destination exists: unlike rename, it cannot
        // replace an existing video on Unix. Both files are in this directory.
        fs::hard_link(&pending, &video).map_err(|error| io(&video, error))?;
        fs::remove_file(&pending).map_err(|error| io(&pending, error))?;
        self.update(clip)
    }
    /// Atomically update only metadata; the original video is never rewritten.
    pub fn update(&self, clip: &Clip) -> Result<(), StorageError> {
        let temporary = self.path(&clip.id, "json.new")?;
        let destination = self.path(&clip.id, "json")?;
        let bytes = serde_json::to_vec_pretty(clip)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temporary)
            .map_err(|error| io(&temporary, error))?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| io(&temporary, error))?;
        fs::rename(&temporary, &destination).map_err(|error| io(&destination, error))
    }
    /// Newest first. A damaged entry is reported instead of erasing any videos.
    pub fn clips(&self) -> Result<(Vec<Clip>, Vec<String>), StorageError> {
        let mut clips = Vec::new();
        let mut warnings = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(|error| io(&self.root, error))? {
            let entry = entry.map_err(|error| io(&self.root, error))?;
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                continue;
            }
            let read = fs::read(&path)
                .map_err(|error| io(&path, error))
                .and_then(|bytes| {
                    serde_json::from_slice::<Clip>(&bytes).map_err(StorageError::from)
                });
            match read {
                Ok(clip) => match self.video(&clip.id) {
                    Ok(video) if video.is_file() => clips.push(clip),
                    _ => warnings.push(format!("{} has no matching video", path.display())),
                },
                Err(error) => warnings.push(error.to_string()),
            }
        }
        clips.sort_by_key(|clip| std::cmp::Reverse(clip.created));
        Ok((clips, warnings))
    }
    /// Discard only the specific clip the person confirmed, never a whole folder.
    pub fn remove(&self, id: &str) -> Result<(), StorageError> {
        for extension in ["webm", "json", "png"] {
            let path = self.path(id, extension)?;
            match fs::remove_file(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(io(&path, error)),
            }
        }
        Ok(())
    }
    fn path(&self, id: &str, extension: &str) -> Result<PathBuf, StorageError> {
        if id.is_empty()
            || id.len() > 100
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(StorageError::InvalidIdentity);
        }
        Ok(self.root.join(format!("{id}.{extension}")))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    fn library() -> Library {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Library::at(
            std::env::temp_dir().join(format!("alo-studio-storage-{}-{nonce}", std::process::id())),
        )
        .unwrap()
    }
    #[test]
    fn damaged_metadata_cannot_escape_the_recording_directory() {
        let library = library();
        for id in ["../outside", "/tmp/outside", "", "a/b", "a\\b"] {
            assert!(library.video(id).is_err());
        }
        fs::remove_dir_all(library.root).unwrap();
    }
    #[test]
    fn an_empty_or_unfinished_video_never_appears_as_saved() {
        let library = library();
        let clip = library.begin(1280, 720, false).unwrap();
        assert!(matches!(
            library.finish(&clip),
            Err(StorageError::EmptyVideo)
        ));
        assert!(library.clips().unwrap().0.is_empty());
        assert!(library.pending(&clip.id).unwrap().exists());
        fs::remove_dir_all(library.root).unwrap();
    }
    #[test]
    fn finishing_cannot_overwrite_a_saved_recording() {
        let library = library();
        let clip = library.begin(1280, 720, false).unwrap();
        fs::write(library.pending(&clip.id).unwrap(), b"encoder output").unwrap();
        library.finish(&clip).unwrap();
        fs::write(library.pending(&clip.id).unwrap(), b"different output").unwrap();
        assert!(library.finish(&clip).is_err());
        assert_eq!(
            fs::read(library.video(&clip.id).unwrap()).unwrap(),
            b"encoder output"
        );
        assert_eq!(library.clips().unwrap().0.len(), 1);
        fs::remove_dir_all(library.root).unwrap();
    }
}
