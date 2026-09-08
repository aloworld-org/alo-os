//! Reusable server and client integration fixtures.
mod application;
mod fixture;
mod wm_capabilities;
pub use application::Application;
pub use fixture::Fixture;
pub use wm_capabilities::assert_window_capabilities;
