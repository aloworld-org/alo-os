//! alo Studio's recording lifecycle, media engine, and local recording library.
//!
//! The installed app has no network permission. Screen access is requested
//! through the desktop portal for each recording; the microphone starts off.

mod capture_timing;
pub mod completed_media;
pub mod model;
mod parent_window;
pub mod performance;
mod permission_lifetime;
pub mod portal;
mod recorded_video;
pub mod recorder;
mod recording_branch;
pub mod storage;
pub mod ui;
pub mod words;

/// The same identity is used by the portal, desktop entry, and Flatpak.
pub const APPLICATION_ID: &str = "org.aloworld.Studio";
