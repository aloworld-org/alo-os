//! The machines the recovery screen's tests are written against: one that can
//! go back, one with an update waiting, and the several that cannot.
//!
//! Every one of them is `alo_keeping_up::Deployments` as the base would report
//! it, and nothing here decides whether going back is possible — that is
//! `GoingBack::offered`'s answer about each of these.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use crate::Contrast;
use alo_appearance::{Scheme, TextScale};
use alo_keeping_up::{Changed, Deployments, Digest};

pub(crate) use crate::approval_testing::words;
use crate::recovery_raster::RecoveryLook;

/// A build, named by a digest a base would report.
pub(crate) fn a_build(pair: &str) -> Digest {
    Digest::read(&format!("sha256:{}", pair.repeat(32))).unwrap()
}

/// The build running now.
pub(crate) fn the_build_running() -> Digest {
    a_build("bb")
}

/// The build before it.
pub(crate) fn the_build_before() -> Digest {
    a_build("aa")
}

/// A machine whose base still keeps the build before: going back can be
/// offered.
pub(crate) fn a_machine_that_can_go_back() -> Deployments {
    Deployments::reported(Some(the_build_running()), None, Some(the_build_before()))
}

/// The same machine with an update already waiting for the next restart, which
/// going back would set aside.
pub(crate) fn a_machine_with_an_update_waiting() -> Deployments {
    Deployments::reported(
        Some(the_build_running()),
        Some(a_build("cc")),
        Some(the_build_before()),
    )
}

/// A machine with nothing before the build it is running.
pub(crate) fn a_machine_with_nothing_before() -> Deployments {
    Deployments::reported(Some(the_build_running()), None, None)
}

/// A machine whose record names the build before and whose base no longer
/// keeps it.
pub(crate) fn a_machine_that_no_longer_keeps_it() -> (Deployments, Changed) {
    (
        a_machine_with_nothing_before(),
        Changed::between(the_build_before(), the_build_running()),
    )
}

/// A machine whose base names no build it booted.
pub(crate) fn a_machine_running_nothing_it_can_name() -> Deployments {
    Deployments::reported(None, None, None)
}

/// A light screen at the ordinary text size.
pub(crate) fn a_light_look() -> RecoveryLook {
    RecoveryLook {
        contrast: Contrast::AsDesigned,
        scheme: Scheme::Light,
        scale: TextScale::percent(100).unwrap(),
    }
}
