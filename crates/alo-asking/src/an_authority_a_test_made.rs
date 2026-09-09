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
//! # Why it is a scope and not a variable
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

use std::sync::{Arc, Mutex, PoisonError, RwLock};

use ureq::tls::{Certificate, RootCerts, TlsConfig};

/// The authority in force, which is `None` everywhere except inside
/// [`while_trusting_only`].
static IN_FORCE: RwLock<Option<TlsConfig>> = RwLock::new(None);

/// One test at a time, because two windows would disagree about the answer.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Run `doing` with this authority as **the only one trusted**.
///
/// Instead of the compiled-in roots rather than as well as them: a test proving
/// a server is verified should not have Mozilla's roots underneath it, or a
/// certificate that chained to a public authority by accident would pass and
/// nobody would know.
///
/// Tests calling this take turns, so one window never sees another's authority.
///
/// # Panics
/// When `pem` is not a certificate, which in a test is the failure being
/// reported — and **must** be. Carrying on with the real roots instead would
/// leave a test that meant to verify against its own authority quietly
/// verifying against Mozilla's, and it would pass or fail for a reason nobody
/// could see.
#[expect(
    clippy::expect_used,
    reason = "this function exists only under a test-only feature, and a certificate a test \
              could not make is that test's failure rather than something to work around"
)]
pub fn while_trusting_only<T>(pem: &[u8], doing: impl FnOnce() -> T) -> T {
    let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner);

    let authority = Certificate::from_pem(pem).expect("a test's authority is a certificate");
    let only_this_one = TlsConfig::builder()
        .root_certs(RootCerts::Specific(Arc::new(vec![authority])))
        .build();

    *IN_FORCE.write().unwrap_or_else(PoisonError::into_inner) = Some(only_this_one);
    let answered = doing();
    *IN_FORCE.write().unwrap_or_else(PoisonError::into_inner) = None;
    answered
}

/// The authority a test named, if a window is open.
///
/// `None` everywhere else, and then the caller changes nothing — so the default,
/// Mozilla's roots, still applies.
pub(crate) fn named_by_a_test() -> Option<TlsConfig> {
    IN_FORCE
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}
