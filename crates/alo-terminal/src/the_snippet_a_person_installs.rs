//! **The shell integration alo OS ships**, which a person installs and alo
//! never installs for them.
//!
//! [ADR 0123](../../../docs/decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
//! term 6, accepted by the owner on 2026-10-18: *alo OS ships the snippet, the
//! sentence saying plainly what it reveals — every command you run, to alo, on
//! this machine — and a one-press way to add it. alo does not write to
//! anybody's shell startup unprompted.*
//!
//! This is the snippet and the sentence. **The one-press way is not built**, and
//! that is said here rather than discovered: what surface offers it is a
//! decision nobody has made, and a file that installed itself would break the
//! term it was written for.
//!
//! # Why it is `include_str!` and not a path
//!
//! A path is a promise about where a file will be on a machine, and this one
//! has to survive a container image, a developer's checkout and a test. The
//! bytes are the thing; where they end up is the installer's business, and
//! there is no installer yet.
//!
//! # What it is held to
//!
//! `the_snippet_a_person_installs_tests.rs` runs **a real interactive `bash`**
//! on a real pseudoterminal with this sourced, presses the key, and asserts
//! that [`crate::what_the_shell_says_about_the_prompt`] reads back what was
//! typed. A snippet tested by reading it would be a test of the reading.

/// The integration for `bash`, as a person would paste it into `.bashrc`.
///
/// Shipped as bytes rather than a path: see this module's header.
const FOR_BASH: &str = include_str!("../shell-integration/alo.bash");

/// The key the snippet binds, as a person would describe it.
///
/// `Ctrl`+`X` `Ctrl`+`A`. The owner chose the `Ctrl`+`X` family on 2026-10-18
/// and this is the free pair in it — measured the same day, `Ctrl`+`X`
/// `Ctrl`+`E` is `edit-and-execute-command` in the `bash` a person actually
/// gets, and `Ctrl`+`X` `Ctrl`+`A` is unbound.
///
/// **Not a user-facing string**: a person reads a key in their own language's
/// words for it, which is `alo-strings`' business and not this crate's. This is
/// the readline binding, which is the same in every language.
pub const THE_KEY_IT_BINDS: &str = r#""\C-x\C-a""#;

/// The snippet for a shell, or nothing for one alo does not ship integration
/// for.
///
/// **`zsh` is not here yet**, and answering `None` is how that is said. A
/// snippet that handed a `zsh` user the `bash` one would be worse than nothing:
/// `bind -x` is not a `zsh` builtin, and the error would arrive on every new
/// terminal they opened.
#[must_use]
pub fn for_the_shell_called(name: &str) -> Option<&'static str> {
    // The name a person's `SHELL` holds is a path on most machines, so the last
    // component is what identifies it. `/bin/bash`, `/usr/bin/bash` and `bash`
    // are the same shell and all three reach a person.
    match name.rsplit('/').next().unwrap_or(name) {
        "bash" => Some(FOR_BASH),
        _ => None,
    }
}

#[cfg(all(test, target_os = "linux"))]
#[path = "the_snippet_a_person_installs_tests.rs"]
mod tests;
