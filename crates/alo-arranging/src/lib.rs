//! **Where a person left their canvas, between one sign-in and the next.**
//!
//! Task 9 of `docs/autonomy/the-smallest-canvas-worth-showing.md`: *frame
//! positions and the camera survive a session ending and starting again.* Until
//! this crate nothing kept either, so every session began at the plane's origin at
//! life size with every window placed as if it had never been opened.
//!
//! One file, read by the shell when it stands the desktop up and written when it
//! comes down. That is the whole of it, and what it is **not** is most of the
//! design.
//!
//! # This crate decides nothing about a canvas
//!
//! What a place on the plane is, how far out a camera may look and how big a frame
//! may be are all `alo-canvas`'s. Every value read back off the disk is rebuilt by
//! [`alo_canvas::At::checked`], [`alo_canvas::Size::checked`] and
//! [`alo_canvas::Zoom::of`] — the same road a live canvas takes — so a file
//! hand-edited to put a window a million units off the plane, or to a zoom no
//! canvas has, is refused by the crate that owns that rule. There is no
//! `Deserialize` for a `Camera` or a `Frame` here and there must not be one: a
//! position read back without being checked is a position nothing validated.
//!
//! # A window is remembered by what it is, not by which one it was
//!
//! A `wl_surface` does not survive a session, so the key is the application's own
//! `app_id` — the thing that is the same when it opens again tomorrow. That has a
//! consequence worth stating rather than discovering: **two windows of one
//! application share one remembered place**, and the second to open gets it. A
//! per-window identity would need something the application does not give us, and
//! inventing one would mean remembering a window by a title a person can change.
//!
//! # An application that did not come back leaves nothing behind
//!
//! The acceptance says *a frame whose application did not come back is absent
//! rather than drawn empty*, and this is where that is kept: the file is a list of
//! places waiting to be claimed, and a place nothing claims is simply never used.
//! Nothing here draws, reserves room, or tells anybody that a window is missing.

use std::collections::BTreeMap;

use alo_canvas::{At, Camera, Size, Zoom};

/// Why an arrangement could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotArranged {
    /// The file is not the shape this crate writes.
    #[error("the arrangement could not be read: {0}")]
    Unreadable(String),
    /// A value in it is not one `alo-canvas` allows.
    ///
    /// Carries what was refused, because a hand-edited file is the case this
    /// exists for and *which line* is the only useful thing to say about it.
    #[error("the arrangement holds a place no canvas has: {0}")]
    NotOnThePlane(String),
}

/// One window's place, as it is written down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct Written {
    /// Plane units across.
    x: i32,
    /// Plane units down.
    y: i32,
    /// How wide it was.
    width: u32,
    /// How tall it was.
    height: u32,
}

/// The whole file, as it is written down.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
struct TheFile {
    /// Where the camera was looking, in plane units.
    looking_at: (i32, i32),
    /// How far in, in thousandths.
    zoom: u32,
    /// Each application's window, by `app_id`.
    ///
    /// A map rather than a list, so the file says plainly that one application has
    /// one remembered place and a second row for it cannot be written.
    windows: BTreeMap<String, Written>,
}

/// **Where a person left their canvas.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arrangement {
    /// Where they were looking.
    camera: Camera,
    /// Where each application's window was, and how big.
    windows: BTreeMap<String, (At, Size)>,
}

impl Default for Arrangement {
    fn default() -> Self {
        Self::fresh()
    }
}

impl Arrangement {
    /// A canvas nobody has arranged yet: the origin, at life size, nothing placed.
    #[must_use]
    pub fn fresh() -> Self {
        Self {
            camera: Camera::new(),
            windows: BTreeMap::new(),
        }
    }

    /// The camera this arrangement was left with.
    #[must_use]
    pub const fn camera(&self) -> Camera {
        self.camera
    }

    /// Remember where this application's window was.
    ///
    /// One place per `app_id`: writing a second replaces the first, which is the
    /// consequence of the key this crate has and is stated in its header rather
    /// than hidden here.
    pub fn window_was(&mut self, app_id: impl Into<String>, at: At, size: Size) {
        self.windows.insert(app_id.into(), (at, size));
    }

    /// Remember where the camera was.
    pub const fn looking(&mut self, camera: Camera) {
        self.camera = camera;
    }

    /// Where this application's window was, if it was here at all.
    #[must_use]
    pub fn where_it_was(&self, app_id: &str) -> Option<(At, Size)> {
        self.windows.get(app_id).copied()
    }

    /// How many windows are remembered.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.windows.len()
    }

    /// This arrangement, as the file says it.
    #[must_use]
    pub fn written(&self) -> String {
        let file = TheFile {
            looking_at: (self.camera.at().x, self.camera.at().y),
            zoom: self.camera.zoom().thousandths(),
            windows: self
                .windows
                .iter()
                .map(|(app_id, (at, size))| {
                    (
                        app_id.clone(),
                        Written {
                            x: at.x,
                            y: at.y,
                            width: size.width(),
                            height: size.height(),
                        },
                    )
                })
                .collect(),
        };
        // A file this crate wrote is always writable: every value in it came out
        // of a checked one. An unwritable one would be a bug here rather than
        // anything a person did, so it answers with nothing arranged rather than
        // taking the session down over a layout.
        toml::to_string_pretty(&file).unwrap_or_default()
    }

    /// An arrangement read back, with every value rebuilt by `alo-canvas`.
    ///
    /// # Errors
    /// [`NotArranged::Unreadable`] where the file is not this shape, and
    /// [`NotArranged::NotOnThePlane`] where a value in it is not one a canvas
    /// allows — a window a million units out, a zoom further in than any canvas
    /// goes, a window of no width. Refused rather than clamped: a person whose
    /// file was quietly corrected would have a canvas that disagreed with what
    /// they could read.
    pub fn read(written: &str) -> Result<Self, NotArranged> {
        let file: TheFile =
            toml::from_str(written).map_err(|why| NotArranged::Unreadable(why.to_string()))?;
        let camera = Camera::new()
            .looking_at(
                At::checked(file.looking_at.0, file.looking_at.1)
                    .ok_or_else(|| NotArranged::NotOnThePlane("looking-at".to_owned()))?,
            )
            .ok_or_else(|| NotArranged::NotOnThePlane("looking-at".to_owned()))?;
        let zoom = Zoom::of(file.zoom)
            .map_err(|why| NotArranged::NotOnThePlane(format!("zoom: {why}")))?;
        let camera = camera
            .zoomed_to(zoom, (0, 0))
            .ok_or_else(|| NotArranged::NotOnThePlane("zoom".to_owned()))?;
        let mut windows = BTreeMap::new();
        for (app_id, was) in file.windows {
            let at = At::checked(was.x, was.y)
                .ok_or_else(|| NotArranged::NotOnThePlane(format!("{app_id}: its place")))?;
            let size = Size::checked(was.width, was.height)
                .ok_or_else(|| NotArranged::NotOnThePlane(format!("{app_id}: its size")))?;
            windows.insert(app_id, (at, size));
        }
        Ok(Self { camera, windows })
    }
}

#[cfg(test)]
#[path = "arranging_tests.rs"]
mod tests;
