//! alo Studio's recording lifecycle, media engine, and local recording library.
//!
//! The installed app has no network permission. Screen access is requested
//! through the desktop portal for each recording; the microphone starts off.

pub mod model;
pub mod performance;
pub mod portal;
pub mod recorder;
pub mod storage;
pub mod ui;
pub mod words;

/// The same identity is used by the portal, desktop entry, and Flatpak.
pub const APPLICATION_ID: &str = "org.aloworld.Studio";
