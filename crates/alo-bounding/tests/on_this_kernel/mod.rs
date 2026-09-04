//! The boundary the tests in this directory run under, imposed the way a
//! machine really gets one.
//!
//! Since
//! [ADR 0018](../../../../docs/decisions/0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)
//! there are two halves to that and a test needs both: `alo-boundaryd` loads the
//! programme and pins it, and `alo-agentd` opens one of those pins by path. So
//! this holds an [`Imposed`] and a [`Boundary`] over the same programme, which
//! is what the machine has after it has booted — and it is worth a shared module
//! rather than three copies, because a fixture that drifted between three files
//! would be three different machines being tested.
//!
//! # Every test imposes its own, and takes it away again
//!
//! The three files here used to share one programme through a `OnceLock`,
//! because a value dropped detached it and four attached programmes at once is
//! not what any of them is measuring. Pinning changed that in both directions: a
//! pinned programme **outlives the process**, so a `OnceLock` that is never
//! dropped would leave a boundary attached to `file_open` on whoever's machine
//! ran the tests, for as long as it stayed up.
//!
//! So each of these is made for one test and taken away at the end of it, and
//! [`Drop`] is the right shape here for the reason it is the wrong shape in
//! `alo-bounding` itself: this is a test tidying up after itself, not a machine
//! deciding to stop enforcing its grants.
//!
//! # It needs root, a BPF filesystem, and a kernel that started the BPF LSM
//!
//! It fails loudly on a machine without them rather than skipping itself, which
//! is ADR 0015's own rule: a test that quietly skipped would report green on
//! every machine in the world, including the ones where the boundary does
//! nothing at all.

#![allow(
    dead_code,
    reason = "three test binaries share this module and each uses the part of it its own \
              subject needs; what is unused in one of them is the fixture of another"
)]

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

use alo_bounding::{Boundary, Imposed, Pinned};

/// The boundary as a booted machine has it: loaded and pinned by the loader,
/// and opened by path the way the person's own daemon opens it.
pub struct AsAMachineHasIt {
    /// Where this one is pinned, which is never the real machine's place.
    pinned: Pinned,

    /// The loader's half: the programme, its maps, and what can be read of them.
    pub imposed: Imposed,

    /// The daemon's half: the one map a turn is written into.
    pub boundary: Boundary,
}

impl AsAMachineHasIt {
    /// Impose one, pinned somewhere this test run owns.
    ///
    /// `what` names the test, so a failure says which one left something behind
    /// if anything ever does.
    pub fn on_this_kernel(what: &str) -> Self {
        let pinned = Pinned::beneath(
            &PathBuf::from("/sys/fs/bpf").join(format!("alo-test-{}-{what}", std::process::id())),
        );
        // Whatever a run that was killed left here. Nothing else on the machine
        // uses this name, and `Pinned::nothing_is_there` would otherwise refuse
        // over the wreckage of something that is no longer running.
        pinned.taken_away();
        pinned.made().unwrap_or_else(|why| {
            panic!(
                "the boundary has nowhere to be pinned, so nothing below is being tested: \
                 {why}\n\
                 This needs a BPF filesystem: `mount -t bpf bpffs /sys/fs/bpf`, which on a real \
                 machine `systemd` has already done. `docs/hardware.md` asks the question."
            )
        });
        let imposed = Imposed::once(&pinned).unwrap_or_else(|why| {
            panic!(
                "the boundary could not be imposed on this kernel, so nothing below is being \
                 tested: {why}\n\
                 This needs root, `CONFIG_BPF_LSM=y`, and `bpf` in the list of security modules \
                 the kernel *started* — `cat /sys/kernel/security/lsm`, which is not the same \
                 question as how the kernel was built. `docs/hardware.md` has the three commands."
            )
        });
        let boundary = Boundary::opened(&pinned).unwrap_or_else(|why| {
            panic!(
                "the boundary was pinned and could not be opened again by path, which is the one \
                 thing the agent's daemon does with it: {why}"
            )
        });
        Self {
            pinned,
            imposed,
            boundary,
        }
    }
}

impl Drop for AsAMachineHasIt {
    /// Take the pins away, which detaches the programme.
    ///
    /// Before the two halves are dropped, because it is the pin that holds the
    /// programme on the hook: a test that removed nothing would leave the
    /// machine enforcing a boundary nobody asked it for.
    fn drop(&mut self) {
        self.pinned.taken_away();
    }
}

/// The lock that makes the tests in one file run one at a time.
///
/// They have to: one of them moves this whole process into a control group,
/// two of them measure counters the whole machine shares, and every one of them
/// attaches a programme to `file_open` for as long as it runs.
pub fn one_at_a_time() -> MutexGuard<'static, ()> {
    static ORDER: OnceLock<Mutex<()>> = OnceLock::new();
    match ORDER.get_or_init(|| Mutex::new(())).lock() {
        Ok(order) => order,
        // A test that panicked while holding it poisoned it, and what is left is
        // still a lock: the next test wants the exclusion rather than the value.
        Err(poisoned) => poisoned.into_inner(),
    }
}
