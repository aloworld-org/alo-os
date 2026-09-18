//! Shared fixtures for the lock's input and pixel tests.
#![allow(clippy::panic, clippy::indexing_slicing)]
#![expect(clippy::unwrap_used, reason = "a failed fixture is a failed test")]
use crate::{LockPressed, LockSurface, SignInKey};
use alo_accounts::{Accounts, Session};
use alo_locking::Seat;
use alo_overlay::Summoning;
use alo_strings::Strings;

/// Words.
pub fn words() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}
/// Accounts.
pub fn accounts() -> Accounts {
    let mut accounts = Accounts::none().unwrap();
    accounts
        .created("ada", 1000, "correct horse battery staple")
        .unwrap();
    accounts
        .created("ben", 1001, "ben's correct password")
        .unwrap();
    accounts
}
/// Session.
pub fn session() -> Session {
    /// A real authenticated session, reused to avoid repeated password hashing.
    static SESSION: std::sync::OnceLock<Session> = std::sync::OnceLock::new();
    SESSION
        .get_or_init(|| {
            Session::opened(
                accounts()
                    .signs_in("ada", "correct horse battery staple")
                    .unwrap(),
                1000,
            )
            .unwrap()
        })
        .clone()
}
/// Surface.
pub fn surface() -> LockSurface<String> {
    LockSurface::of(
        Seat::opened(session()).locked(&mut Summoning::closed()),
        words(),
    )
    .unwrap()
}
/// Key.
pub fn key(surface: LockSurface<String>, key: SignInKey) -> LockSurface<String> {
    match surface.pressed(key, || panic!("accounts were read before submission")) {
        LockPressed::Still(surface) => *surface,
        LockPressed::Opened { .. } => panic!("opened without submission"),
    }
}
/// Typed.
pub fn typed(name: &str, password: &str) -> LockSurface<String> {
    let mut surface = key(surface(), SignInKey::Enter);
    for c in name.chars() {
        surface = key(surface, SignInKey::Letter(c));
    }
    surface = key(surface, SignInKey::Enter);
    for c in password.chars() {
        surface = key(surface, SignInKey::Letter(c));
    }
    surface
}
