//! Where a portal request's answer is sent, as the specification names it.
//!
//! Every method on an `org.freedesktop.portal.*` interface that answers later
//! returns a *handle*: the object path of an `org.freedesktop.portal.Request`,
//! whose `Response` signal carries the answer. The specification fixes the
//! path, so that an application can listen for the answer before it asks:
//!
//! ```text
//! /org/freedesktop/portal/desktop/request/SENDER/TOKEN
//! ```
//!
//! `SENDER` is the caller's unique bus name with the leading `:` removed and
//! every `.` made `_`; `TOKEN` is the `handle_token` it passed in its options,
//! or one the backend makes when it passed none.

use std::sync::atomic::{AtomicU64, Ordering};

/// The object every portal interface is served on.
pub const THE_PORTALS_OBJECT: &str = "/org/freedesktop/portal/desktop";

/// The most characters a token may have.
///
/// The specification sets none; an object path element this long is not a
/// token anybody wrote in good faith.
const LONGEST_TOKEN: usize = 64;

/// Tokens made for requests that passed none, counted so no two are the same.
static MADE: AtomicU64 = AtomicU64::new(0);

/// A token that is not letters, digits and underscores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotAToken;

/// The handle of a request from `sender`, named by `token` when it passed one.
///
/// # Errors
/// [`NotAToken`] for a token that is empty, too long, or has anything but
/// ASCII letters, digits and underscores in it — each of which would make a
/// path that is not one, or one that is somebody else's.
pub fn handle_for(sender: &str, token: Option<&str>) -> Result<String, NotAToken> {
    let token = match token {
        Some(token) if a_token(token) => token.to_owned(),
        Some(_) => return Err(NotAToken),
        None => format!("alo{}", MADE.fetch_add(1, Ordering::Relaxed)),
    };
    let sender: String = sender
        .trim_start_matches(':')
        .chars()
        .map(|letter| if letter == '.' { '_' } else { letter })
        .collect();
    Ok(format!("{THE_PORTALS_OBJECT}/request/{sender}/{token}"))
}

/// Whether `token` can be the last element of an object path.
fn a_token(token: &str) -> bool {
    !token.is_empty()
        && token.len() <= LONGEST_TOKEN
        && token
            .chars()
            .all(|letter| letter.is_ascii_alphanumeric() || letter == '_')
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **The path is the one the specification tells an application to expect.**
    #[test]
    fn the_handle_is_the_path_the_specification_names() {
        assert_eq!(
            handle_for(":1.42", Some("alo_secret_7")).unwrap(),
            "/org/freedesktop/portal/desktop/request/1_42/alo_secret_7"
        );
        let made = handle_for(":1.42", None).unwrap();
        let again = handle_for(":1.42", None).unwrap();
        assert!(made.starts_with("/org/freedesktop/portal/desktop/request/1_42/alo"));
        assert_ne!(made, again, "two requests with no token shared a handle");
    }

    /// **A token that would make another path is refused**, not cleaned up.
    #[test]
    fn a_token_that_is_not_one_is_refused() {
        for not in ["", "a/b", "../1_43/theirs", "a.b", "a b", "é"] {
            assert_eq!(handle_for(":1.42", Some(not)), Err(NotAToken), "{not:?}");
        }
        assert_eq!(
            handle_for(":1.42", Some(&"t".repeat(LONGEST_TOKEN + 1))),
            Err(NotAToken)
        );
        assert!(handle_for(":1.42", Some(&"t".repeat(LONGEST_TOKEN))).is_ok());
    }
}
