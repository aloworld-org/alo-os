//! Reusable server and client integration fixtures.
mod application;
mod fixture;
mod the_canvas;
mod wm_capabilities;
pub use application::Application;
pub use fixture::Fixture;
pub use the_canvas::{THREE_ZOOMS, TWO_PANS, close_enough, looking, motion, on_the_screen};
pub use wm_capabilities::assert_window_capabilities;
