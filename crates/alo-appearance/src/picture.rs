//! One picture, and how it meets the edges of a screen.
//!
//! **A wallpaper that came with alo OS is named; a person's own picture is a
//! path.** They are two different things and this crate keeps them apart for two
//! reasons. A shipped wallpaper lives wherever the image puts it, and an image
//! that reorganised its own folders would otherwise silently blank the
//! background of every machine that never changed it. And a name that were
//! allowed to be a path would be a path chosen by whoever wrote the settings
//! file, pointed anywhere on the disk, dressed as something alo OS shipped —
//! so a name with a separator in it is refused here rather than resolved later.
//!
//! **A person's picture is named in full, from the top of the disk.** Where a
//! relative path leads depends on where the shell happened to be started from,
//! which is not something a person choosing a photograph is deciding.

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Why a picture cannot be used as a background.
///
/// Each says what to give instead, because the person is in the middle of
/// picking a picture.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PictureError {
    /// A shipped wallpaper with no name.
    #[error("name the wallpaper — it is how the image is asked for the picture it shipped")]
    Unnamed,
    /// A shipped wallpaper whose name is a path.
    #[error(
        "{0} is a path, and a wallpaper that came with alo OS is named rather than pathed — use your own picture instead if it is a file on this disk"
    )]
    NameIsAPath(String),
    /// A person's picture, given as a relative path.
    #[error(
        "give the whole path to {0}, starting from the top of the disk — where a relative path leads depends on where the shell was started from"
    )]
    NotAWholePath(PathBuf),
}

/// Which picture: one alo OS shipped, or one of the person's own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Of {
    /// A wallpaper that came with the image, by the name the image knows it by.
    Shipped(String),
    /// A file on this machine, named in full.
    File(PathBuf),
}

/// How a picture meets the edges of a screen it is not the shape of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Fitting {
    /// Cover the screen, cropping whatever does not fit. What a person means
    /// nine times out of ten, so it is what a picture arrives as.
    #[default]
    Fill,
    /// Show all of it, leaving a margin where the shape does not match.
    Fit,
    /// Cover the screen by distorting the picture.
    Stretch,
    /// Show it at its own size, in the middle.
    Centre,
    /// Show it at its own size, repeated.
    Tile,
}

/// One picture, and how it is shown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Picture {
    /// Which picture.
    of: Of,
    /// How it meets the edges.
    fitting: Fitting,
}

impl Picture {
    /// A wallpaper that came with the image.
    ///
    /// # Errors
    /// [`PictureError::Unnamed`] for an empty name, and
    /// [`PictureError::NameIsAPath`] for a name with a separator in it — which
    /// is the refusal that stops a settings file from dressing an arbitrary
    /// file up as something alo OS shipped.
    pub fn shipped(name: &str) -> Result<Self, PictureError> {
        if name.is_empty() {
            return Err(PictureError::Unnamed);
        }
        let mut components = Path::new(name).components();
        let one_ordinary_name =
            matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none();
        // The separator is checked as well as the components, because a
        // backslash is an ordinary character in a name on Linux and a separator
        // on the machine somebody edits the file from.
        if !one_ordinary_name || name.contains(['/', '\\']) {
            return Err(PictureError::NameIsAPath(name.to_owned()));
        }
        Ok(Self {
            of: Of::Shipped(name.to_owned()),
            fitting: Fitting::default(),
        })
    }

    /// A picture of the person's own.
    ///
    /// # Errors
    /// [`PictureError::NotAWholePath`] for a path that is not absolute.
    pub fn file(path: PathBuf) -> Result<Self, PictureError> {
        if !path.is_absolute() {
            return Err(PictureError::NotAWholePath(path));
        }
        Ok(Self {
            of: Of::File(path),
            fitting: Fitting::default(),
        })
    }

    /// A shipped wallpaper the compiler can build, for the one this crate
    /// ships.
    ///
    /// Unchecked, and the only caller is [`crate::shipped`] — which is held to
    /// the same rules by a test that puts the name it ships back through
    /// [`Picture::shipped`]. It exists so that a shell cannot fail to start over
    /// its own wallpaper, which would be a worse problem than a grey screen.
    pub(crate) fn unchecked(name: &str) -> Self {
        Self {
            of: Of::Shipped(name.to_owned()),
            fitting: Fitting::Fill,
        }
    }

    /// The same picture, meeting the edges differently.
    #[must_use]
    pub fn fitted(mut self, fitting: Fitting) -> Self {
        self.fitting = fitting;
        self
    }

    /// Which picture this is.
    #[must_use]
    pub fn of(&self) -> &Of {
        &self.of
    }

    /// How it meets the edges.
    #[must_use]
    pub fn fitting(&self) -> Fitting {
        self.fitting
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The absolute path a test uses, spelled for whichever machine is running
    /// it: this crate touches no disk, so nothing here has to exist.
    fn somewhere() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(r"C:\Users\a\Pictures\harbour.jpg")
        } else {
            PathBuf::from("/home/a/Pictures/harbour.jpg")
        }
    }

    /// Both kinds arrive, and a picture is filled unless somebody says
    /// otherwise.
    #[test]
    fn a_picture_is_a_name_or_a_whole_path() {
        let shipped = Picture::shipped("alo").unwrap();
        assert_eq!(shipped.of(), &Of::Shipped("alo".to_owned()));
        assert_eq!(shipped.fitting(), Fitting::Fill);

        let mine = Picture::file(somewhere()).unwrap();
        assert_eq!(mine.of(), &Of::File(somewhere()));
        assert_eq!(mine.fitted(Fitting::Centre).fitting(), Fitting::Centre);
    }

    /// **A name that could be a path would be a path.** The refusal is what
    /// stops a settings file from asking for a file elsewhere on the disk while
    /// claiming to ask for a wallpaper that came with the image.
    #[test]
    fn a_shipped_wallpaper_cannot_be_a_path() {
        for name in [
            "../../etc/shadow",
            "..",
            "sub/alo",
            r"sub\alo",
            "/etc/hostname",
        ] {
            assert_eq!(
                Picture::shipped(name),
                Err(PictureError::NameIsAPath(name.to_owned())),
                "{name} is a path and is refused as a wallpaper name"
            );
        }
        assert_eq!(Picture::shipped(""), Err(PictureError::Unnamed));
    }

    /// A relative path leads somewhere that depends on how the shell was
    /// started, so it is refused where it is given.
    #[test]
    fn a_persons_picture_is_named_in_full() {
        let relative = PathBuf::from("Pictures/harbour.jpg");
        assert_eq!(
            Picture::file(relative.clone()),
            Err(PictureError::NotAWholePath(relative))
        );
    }

    /// A refusal says what to give instead.
    #[test]
    fn a_refusal_says_what_to_give_instead() {
        assert_eq!(
            Picture::shipped("").unwrap_err().to_string(),
            "name the wallpaper — it is how the image is asked for the picture it shipped"
        );
        assert!(
            Picture::file(PathBuf::from("harbour.jpg"))
                .unwrap_err()
                .to_string()
                .contains("starting from the top of the disk")
        );
    }

    /// A picture survives a settings file unchanged.
    #[test]
    fn a_picture_survives_being_written_down() {
        let picture = Picture::file(somewhere()).unwrap().fitted(Fitting::Fit);
        let written = serde_json::to_string(&picture).unwrap();
        assert_eq!(serde_json::from_str::<Picture>(&written).unwrap(), picture);
    }
}
