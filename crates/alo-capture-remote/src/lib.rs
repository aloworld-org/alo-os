//! Rented PipeWire access, narrowed before another process receives its socket.
//!
//! No selection or grant is made here. The portal has already identified the
//! application, obtained the person's fresh choice, and recorded permission.

#[cfg(target_os = "linux")]
mod restricted;
#[cfg(target_os = "linux")]
pub use restricted::restricted_remote;
