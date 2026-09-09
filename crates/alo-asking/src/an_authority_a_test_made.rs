//! The one thing a test may change about how this crate trusts a server, and
//! **it does not exist in anything a machine runs.**
//!
//! # Why there is a seam here at all
//!
//! `alo-asking` sets no root certificates, so `ureq`'s default applies:
//! `RootCerts::WebPki`, which is the Mozilla root programme compiled into the
//! binary. Not the machine's store, not `SSL_CERT_FILE`, not an environment
//! variable. That is the right default for a daemon carrying a person's
//! credential and **nothing here changes it.**
//!
//! It also means a certificate this repository can issue is not trustable, and
//! so the question *does the key actually arrive at the server the person
//! chose, over a verified connection* could not be asked at all. The choice was
//! between leaving that unproven and giving a test — and only a test — a way to
//! name an authority of its own.
//!
//! # What keeps it out of a machine
//!
//! Three things, and the last is a check rather than a promise:
//!
//! 1. **A feature that is off by default.** Without `trust-a-test-authority`
//!    this file is not compiled and `put` is byte-for-byte what it was.
//! 2. **Only a `dev-dependencies` entry turns it on.** `image/Containerfile`
//!    builds `--package alo-agentd --package alo-boundaryd`, and a `--package`
//!    release build resolves no dev-dependency, so the feature is off there.
//! 3. **`crates/alo-secrets/tests/nothing_ships_the_fixture.rs`** refuses any
//!    manifest that turns it on outside a `dev-dependencies` table.
//!
//! # Why it is a scope on one thread, and not a variable
//!
//! Thread-local, so a request on another thread cannot inherit it — see
//! `IN_FORCE`.
//!
//! An environment variable was the obvious way and is the wrong one twice over:
//! `std::env::set_var` is `unsafe` in this edition and `CLAUDE.md` allows no
//! `unsafe` outside `alo-bounding-kernel`, and a variable stays set for whatever
//! runs next. [`while_trusting_only`] is a window instead — the authority
//! applies for one closure, and outside it this crate trusts what it always
//! trusted.
//!
//! # What it does not do
//!
//! It does not disable verification, and there is no path here that can. The
//! certificate chain, its validity dates and the identity in it are checked
//! exactly as they always were — what changes is **which authority** is at the
//! root of the chain, and only inside that window.

use std::cell::RefCell;
use std::sync::Arc;

use ureq::tls::{Certificate, RootCerts, TlsConfig};

thread_local! {
    /// The authority in force **on this thread**, and `None` on every other.
    ///
    /// Thread-local rather than a global, and that is a correctness matter
    /// rather than tidiness. A request runs on the thread that made it —
    /// `ureq` is synchronous — so a window opened here must not change what
    /// anything else in the process trusts. Held in a global, a request on
    /// another thread would **inherit** this test's authority while the window
    /// happened to be open, and would then be verifying against something its
    /// own test never chose.
    static IN_FORCE: RefCell<Option<TlsConfig>> = const { RefCell::new(None) };
}

/// Puts the authority back the way it was, however the closure ends.
///
/// A `Drop` rather than a line after the call, so that a panicking test — which
/// is how a test reports failure — cannot leave its authority in force for
/// whatever this thread does next.
struct OnlyUntilThisIsDropped;

impl Drop for OnlyUntilThisIsDropped {
    fn drop(&mut self) {
        IN_FORCE.with_borrow_mut(|held| *held = None);
    }
}

/// Run `doing` with this authority as **the only one trusted, on this thread**.
///
/// Instead of the compiled-in roots rather than as well as them: a test proving
/// a server is verified should not have Mozilla's roots underneath it, or a
/// certificate that chained to a public authority by accident would pass and
/// nobody would know.
///
/// Nothing on any other thread is affected, and nothing after the closure is.
///
/// # Panics
/// When `pem` is not a certificate, which in a test is the failure being
/// reported — and **must** be. Carrying on with the real roots instead would
/// leave a test that meant to verify against its own authority quietly
/// verifying against Mozilla's, and it would pass or fail for a reason nobody
/// could see.
#[expect(
    clippy::expect_used,
    reason = "this function exists only under a test-only feature, and a certificate a test               could not make is that test's failure rather than something to work around"
)]
pub fn while_trusting_only<T>(pem: &[u8], doing: impl FnOnce() -> T) -> T {
    let authority = Certificate::from_pem(pem).expect("a test's authority is a certificate");
    let only_this_one = TlsConfig::builder()
        .root_certs(RootCerts::Specific(Arc::new(vec![authority])))
        .build();

    IN_FORCE.with_borrow_mut(|held| *held = Some(only_this_one));
    let _until_this_is_dropped = OnlyUntilThisIsDropped;
    doing()
}

/// The authority a test named, if a window is open **on this thread**.
///
/// `None` everywhere else, and then the caller changes nothing — so the default,
/// Mozilla's roots, still applies.
pub(crate) fn named_by_a_test() -> Option<TlsConfig> {
    IN_FORCE.with_borrow(Clone::clone)
}
