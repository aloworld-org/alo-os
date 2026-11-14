//! **Walking a booted alo OS machine, unattended**: boot the written disk in
//! qemu, press keys at it the way a person does, wait for the screen to change
//! and settle, read what its journal says on the serial port, and write down a
//! pass or a fail for each check with the evidence for it.
//!
//! The walks this replaces were done by hand on a lane machine with no KVM,
//! where qemu emulated the processor: first-boot installs took about 75
//! minutes, keys arrived 30 to 100 seconds late, shifted keys arrived garbled
//! when sent quickly, and a screenshot taken after a key often showed the
//! screen before it. This crate is the instrument for a runner with KVM
//! (`.github/workflows/walk.yml`), and each of those faults has its cure in
//! one file:
//!
//! - garbled shifted keys — `strokes.rs` sends each edge of each key as its
//!   own QMP `input-send-event`, with the hold chosen here;
//! - a picture of the screen before the key — `settling.rs` waits for the
//!   screen to differ from the one the key was pressed at, then to hold still,
//!   and says which of three outcomes happened;
//! - a monitor command lost because the socket closed — `qmp.rs` holds the
//!   connection and reads each answer.
//!
//! **What a walk here does not prove**, said where the instrument is: it is
//! still qemu, with qemu's devices — a virtio display, a PS/2 keyboard, user
//! networking — and not a person's machine. CLAUDE.md's rule of an
//! integration test on real hardware stands, and `docs/walks/walks-on-github.md`
//! says what each check does and does not show.
//!
//! A test instrument, never shipped: nothing in the image depends on it.

pub mod booting;
pub mod frame;
pub mod keys;
pub mod machine;
pub mod qemu;
pub mod qmp;
pub mod redacting;
pub mod report;
pub mod settling;
pub mod step;
pub mod strokes;
pub mod the_first_walk;
pub mod the_log;
pub mod walking;

#[cfg(test)]
mod a_machine_of_pictures;

pub use booting::Booting;
pub use frame::Frame;
pub use keys::Chord;
pub use machine::{Machine, MachineError};
pub use report::{Check, Report, Verdict};
pub use settling::Settling;
pub use step::{Since, Step, Walk};
pub use strokes::Pace;
pub use the_first_walk::{THE_PROVIDER_KEY, the_first_walk};
pub use walking::walk;
