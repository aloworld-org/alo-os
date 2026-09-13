//! Which release of the model runtime this crate's requests were measured
//! against.
//!
//! ADR 0006 pins the runtime, and the image pins it in `image/Containerfile`'s
//! `ARG THE_RUNTIME`. What nothing pinned until 2026-09-13 was **the runtime this
//! crate's wire shapes are true of** — so the create request `bring` sent was
//! correct against every fixture here and refused by the release the image
//! installs. This constant is that release, written once, and
//! `tests/the_pinned_runtime_accepts_what_alo_os_sends.rs` fails the day the
//! image names another: a later runtime is then a deliberate change, walked
//! against the real program before this line moves, rather than an upgrade the
//! requests find out about on somebody's machine.

/// The model runtime release every request in [`crate::ollama`] and
/// `crate::handing_over` was put to, and what it answered is in their
/// documentation.
pub const THE_PINNED_RUNTIME: &str = "0.34.0";
