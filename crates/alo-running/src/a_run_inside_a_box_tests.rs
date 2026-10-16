//! What can be said about the lifecycle without a kernel holding a boundary.
//!
//! Which is not much, and saying so is the honest shape of this file. Every
//! road through [`super::ran_with_the_helper_at`] begins with
//! `Boundary::in_place`, so there is no way to reach step two on a machine
//! without a BPF LSM — and a test that mocked one would be asserting about a
//! boundary that decides nothing.
//!
//! **The run itself is tested in
//! `tests/the_kernel_refuses_what_a_run_was_not_given.rs`**, which needs root,
//! a BPF filesystem and a kernel that started the module, and fails loudly
//! rather than skipping on a machine without them.
//!
//! What is here is the one thing that is a fact about this crate rather than
//! about a machine: where the helper is, and that it is the program this crate
//! actually builds.

use std::path::Path;

use super::THE_HELPER;

/// **The helper is named by an absolute path.**
///
/// A relative one would be found against whatever directory this service
/// happens to be in, which is the same fault [`crate::ARun`] refuses for the
/// command — and it would be worse here, because this is the program that does
/// the confining rather than the one being confined.
#[test]
fn the_helper_is_an_absolute_path() {
    assert!(
        Path::new(THE_HELPER).is_absolute(),
        "the helper is started by this path and it is relative: {THE_HELPER}"
    );
}

/// **The helper's path ends in the name of the program this crate builds.**
///
/// A constant and a `[[bin]]` name are two places one fact is written, and this
/// is the test that keeps them one fact. Renaming the binary without this would
/// leave a crate that compiles, installs one name and starts another — and the
/// failure would appear at the first run on a real machine rather than in CI.
#[test]
fn the_helper_is_the_program_this_crate_builds() {
    assert_eq!(
        Path::new(THE_HELPER).file_name(),
        Some(std::ffi::OsStr::new("alo-enter-the-box")),
        "the installed path and the built program have different names"
    );
}
